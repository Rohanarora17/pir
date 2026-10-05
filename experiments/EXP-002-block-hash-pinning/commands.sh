#!/bin/sh
set -eu

EXPERIMENT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
OUTPUT_DIR=${OUTPUT_DIR:-$EXPERIMENT_DIR}
RAW_DIR="$OUTPUT_DIR/raw"

CHAIN_ID_HEX=0xaa36a7
BLOCK_NUMBER=11849215
BLOCK_NUMBER_HEX=0xb4cdff
BLOCK_HASH=0xae3702c08fffb7b4599d39c511311825b4123a78ef4aa8b1d397f373195cd5f7
STATE_ROOT=0x67edd541eefa8852c61e417316acdd74442b3a37af755096f804e2b15124e098
HISTORICAL_BLOCK_HASH=0x9d11a05a418308f20ee690d5700d04bb23893b168a089389b89975cef3eb14e2
MISSING_BLOCK_HASH=0x1111111111111111111111111111111111111111111111111111111111111111
WETH=0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14
STORAGE_KEY=0x88080a0f7453a7e6f91b5d6e4f0c47e15d7639174219416b4e29a716c14f0857

for tool in curl jq openssl cmp awk; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'missing required command: %s\n' "$tool" >&2
    exit 1
  }
done

mkdir -p "$RAW_DIR"
if find "$RAW_DIR" -type f -mindepth 1 -print -quit | grep -q .; then
  printf 'refusing to overwrite retained raw evidence in %s\n' "$RAW_DIR" >&2
  exit 1
fi

request() {
  provider=$1
  endpoint=$2
  label=$3
  method=$4
  params=$5
  provider_dir="$RAW_DIR/$provider"
  mkdir -p "$provider_dir"

  jq -n --arg method "$method" --argjson params "$params" \
    '{jsonrpc:"2.0",id:1,method:$method,params:$params}' \
    > "$provider_dir/$label.request.json"

  set +e
  http_code=$(curl -sS --retry 2 --retry-all-errors --retry-delay 1 --max-time 30 \
    -H 'content-type: application/json' \
    --data-binary "@$provider_dir/$label.request.json" \
    -o "$provider_dir/$label.response.json" \
    -w '%{http_code}' \
    "$endpoint")
  curl_exit=$?
  set -e

  jq -n \
    --arg endpoint "$endpoint" \
    --argjson curl_exit "$curl_exit" \
    --arg http_code "$http_code" \
    '{endpoint:$endpoint,curlExit:$curl_exit,httpCode:$http_code}' \
    > "$provider_dir/$label.transport.json"
}

collect_provider() {
  provider=$1
  endpoint=$2

  request "$provider" "$endpoint" client_version web3_clientVersion '[]'
  request "$provider" "$endpoint" chain_id eth_chainId '[]'
  request "$provider" "$endpoint" finalised_head eth_getBlockByNumber '["finalized",false]'
  request "$provider" "$endpoint" header_by_hash eth_getBlockByHash "[\"$BLOCK_HASH\",false]"
  request "$provider" "$endpoint" proof_by_number eth_getProof \
    "[\"$WETH\",[\"$STORAGE_KEY\"],\"$BLOCK_NUMBER_HEX\"]"
  request "$provider" "$endpoint" proof_by_hash eth_getProof \
    "[\"$WETH\",[\"$STORAGE_KEY\"],{\"blockHash\":\"$BLOCK_HASH\",\"requireCanonical\":true}]"
  request "$provider" "$endpoint" proof_by_hash_noncanonical_allowed eth_getProof \
    "[\"$WETH\",[\"$STORAGE_KEY\"],{\"blockHash\":\"$BLOCK_HASH\",\"requireCanonical\":false}]"
  request "$provider" "$endpoint" proof_missing_hash eth_getProof \
    "[\"$WETH\",[\"$STORAGE_KEY\"],{\"blockHash\":\"$MISSING_BLOCK_HASH\",\"requireCanonical\":true}]"
  request "$provider" "$endpoint" proof_historical_hash eth_getProof \
    "[\"$WETH\",[\"$STORAGE_KEY\"],{\"blockHash\":\"$HISTORICAL_BLOCK_HASH\",\"requireCanonical\":true}]"
}

collect_provider ethpandaops https://rpc.sepolia.ethpandaops.io
collect_provider publicnode https://ethereum-sepolia-rpc.publicnode.com
collect_provider one_rpc https://1rpc.io/sepolia
collect_provider tenderly https://sepolia.gateway.tenderly.co

classify() {
  response=$1
  transport=$2
  curl_exit=$(jq -r .curlExit "$transport")
  if test "$curl_exit" != 0; then
    printf 'transport_error'
  elif test ! -s "$response"; then
    printf 'empty_response'
  else
    jq -r '
      if (.result | type) == "object" and (.result.accountProof | type) == "array" then "proof"
      elif (.result | type) == "object" then "object"
      elif .result == null and .error == null then "null"
      elif .error != null then "rpc_error"
      else "other"
      end
    ' "$response"
  fi
}

printf 'provider\tcase\tclassification\terror_code\terror_message\n' > "$OUTPUT_DIR/matrix.tsv"
for provider in ethpandaops publicnode one_rpc tenderly; do
  for label in proof_by_number proof_by_hash proof_by_hash_noncanonical_allowed proof_missing_hash proof_historical_hash; do
    response="$RAW_DIR/$provider/$label.response.json"
    transport="$RAW_DIR/$provider/$label.transport.json"
    classification=$(classify "$response" "$transport")
    error_code=$(jq -r '.error.code // "-"' "$response" 2>/dev/null || printf '-')
    error_message=$(jq -r '.error.message // "-" | gsub("[\\t\\r\\n]"; " ")' "$response" 2>/dev/null || printf '-')
    printf '%s\t%s\t%s\t%s\t%s\n' \
      "$provider" "$label" "$classification" "$error_code" "$error_message" \
      >> "$OUTPUT_DIR/matrix.tsv"
  done
done

printf 'provider\tcase\tproof_sha256\n' > "$OUTPUT_DIR/proof-digests.tsv"
for provider in ethpandaops publicnode one_rpc tenderly; do
  for label in proof_by_number proof_by_hash proof_by_hash_noncanonical_allowed; do
    response="$RAW_DIR/$provider/$label.response.json"
    transport="$RAW_DIR/$provider/$label.transport.json"
    if test "$(classify "$response" "$transport")" = proof; then
      digest=$(jq -S .result "$response" | openssl dgst -sha256 | awk '{print $2}')
      printf '%s\t%s\t%s\n' "$provider" "$label" "$digest" \
        >> "$OUTPUT_DIR/proof-digests.tsv"
    fi
  done
done

test "$(jq -r .result "$RAW_DIR/ethpandaops/chain_id.response.json")" = "$CHAIN_ID_HEX"
test "$(jq -r .result.hash "$RAW_DIR/ethpandaops/header_by_hash.response.json")" = "$BLOCK_HASH"
test "$(jq -r .result.stateRoot "$RAW_DIR/ethpandaops/header_by_hash.response.json")" = "$STATE_ROOT"
test "$(jq -r .result.number "$RAW_DIR/ethpandaops/header_by_hash.response.json")" = "$BLOCK_NUMBER_HEX"
test "$(( $(printf '%d' "$(jq -r .result.number "$RAW_DIR/ethpandaops/finalised_head.response.json")") ))" -ge "$BLOCK_NUMBER"

jq -S .result "$RAW_DIR/ethpandaops/proof_by_number.response.json" \
  > "$OUTPUT_DIR/ethpandaops-proof-by-number.json"
jq -S .result "$RAW_DIR/ethpandaops/proof_by_hash.response.json" \
  > "$OUTPUT_DIR/ethpandaops-proof-by-hash.json"
cmp "$OUTPUT_DIR/ethpandaops-proof-by-number.json" "$OUTPUT_DIR/ethpandaops-proof-by-hash.json"

test "$(classify "$RAW_DIR/ethpandaops/proof_by_hash.response.json" "$RAW_DIR/ethpandaops/proof_by_hash.transport.json")" = proof
for provider in ethpandaops publicnode one_rpc tenderly; do
  test "$(classify "$RAW_DIR/$provider/proof_missing_hash.response.json" "$RAW_DIR/$provider/proof_missing_hash.transport.json")" != proof
done

(
  cd "$RAW_DIR"
  find . -type f -print0 | sort -z | xargs -0 openssl dgst -sha256
) > "$OUTPUT_DIR/SHA256SUMS"

printf 'EXP-002 checks passed\n'
