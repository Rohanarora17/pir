#!/bin/sh
set -eu

EXPERIMENT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
RAW_DIR="$EXPERIMENT_DIR/raw"
RPC_URL=${RPC_URL:-https://rpc.sepolia.ethpandaops.io}

CHAIN_ID=11155111
CHAIN_ID_HEX=0xaa36a7
BLOCK_NUMBER=11848974
BLOCK_NUMBER_HEX=0xb4cd0e
BLOCK_HASH=0x9d11a05a418308f20ee690d5700d04bb23893b168a089389b89975cef3eb14e2
STATE_ROOT=0x7443069b0b51afdd2c7e2d8e54723a3b4fb66e8e200cdc3e96ff314589b0d5e2
WINDOW_START_HEX=0xb4ccdd
WETH=0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14
USER_ADDRESS=0xCE54cF5a0dE3843011cF20389C1b6a4AaC442d6A
EXPECTED_EVENT_HOLDER=0x3289680dd4d6c10bb19b899729cda5eef58aeff1
MAPPING_SLOT=3
TRANSFER_TOPIC=0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
TOKEN_LIST_REVISION=b41e2b93ef284c4acc897d100e77686e531fa249
WETH_SOURCE_REVISION=ed24991304291297c3b4a52818d02f46a17aa9a2

for tool in cast curl jq openssl; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'missing required command: %s\n' "$tool" >&2
    exit 1
  }
done

if find "$RAW_DIR" -type f -mindepth 1 -print -quit | grep -q .; then
  printf 'refusing to overwrite retained raw evidence in %s\n' "$RAW_DIR" >&2
  exit 1
fi

rpc() {
  method=$1
  params=$2
  output=$3
  jq -n --arg method "$method" --argjson params "$params" \
    '{jsonrpc:"2.0",id:1,method:$method,params:$params}' \
    | curl -fsS --retry 4 --retry-all-errors --retry-delay 2 --max-time 60 \
      -H 'content-type: application/json' --data-binary @- "$RPC_URL" > "$output"
  jq -e 'has("result") and (.error | not)' "$output" >/dev/null
}

curl -fsS --retry 4 --retry-all-errors --retry-delay 2 --max-time 60 \
  "https://raw.githubusercontent.com/Uniswap/default-token-list/$TOKEN_LIST_REVISION/src/tokens/sepolia.json" \
  > "$RAW_DIR/uniswap-sepolia-token-list.json"
curl -fsS --retry 4 --retry-all-errors --retry-delay 2 --max-time 60 \
  "https://raw.githubusercontent.com/Uniswap/v2-periphery/$WETH_SOURCE_REVISION/contracts/test/WETH9.sol" \
  > "$RAW_DIR/WETH9.sol"

jq -e --arg address "${WETH}" \
  --argjson chain_id "$CHAIN_ID" \
  'any(.[]; (.address | ascii_downcase) == ($address | ascii_downcase) and .chainId == $chain_id and .symbol == "WETH")' \
  "$RAW_DIR/uniswap-sepolia-token-list.json" >/dev/null
grep -Eq 'mapping[[:space:]]*\(address[[:space:]]*=>[[:space:]]*uint\)[[:space:]]*public[[:space:]]+balanceOf;' \
  "$RAW_DIR/WETH9.sol"

rpc eth_chainId '[]' "$RAW_DIR/chain-id.json"
rpc eth_getBlockByNumber '["finalized",false]' "$RAW_DIR/finalized-observation.json"
rpc eth_getBlockByNumber "[\"$BLOCK_NUMBER_HEX\",false]" "$RAW_DIR/pinned-block.json"
rpc eth_getCode "[\"$WETH\",\"$BLOCK_NUMBER_HEX\"]" "$RAW_DIR/contract-code.json"

test "$(jq -r .result "$RAW_DIR/chain-id.json")" = "$CHAIN_ID_HEX"
test "$(jq -r .result.hash "$RAW_DIR/pinned-block.json")" = "$BLOCK_HASH"
test "$(jq -r .result.stateRoot "$RAW_DIR/pinned-block.json")" = "$STATE_ROOT"
test "$(jq -r .result.number "$RAW_DIR/finalized-observation.json" | cast to-dec)" -ge "$BLOCK_NUMBER"
test "$(jq -r .result "$RAW_DIR/contract-code.json")" != "0x"

rpc eth_getLogs \
  "[{\"address\":\"$WETH\",\"fromBlock\":\"$WINDOW_START_HEX\",\"toBlock\":\"$BLOCK_NUMBER_HEX\",\"topics\":[\"$TRANSFER_TOPIC\"]}]" \
  "$RAW_DIR/transfer-logs.json"

EVENT_HOLDER=$(jq -r '.result[-1].topics[2] | "0x" + .[-40:]' "$RAW_DIR/transfer-logs.json")
test "$(printf '%s' "$EVENT_HOLDER" | tr '[:upper:]' '[:lower:]')" = "$EXPECTED_EVENT_HOLDER"

USER_KEY=$(cast index address "$USER_ADDRESS" "$MAPPING_SLOT")
HOLDER_KEY=$(cast index address "$EVENT_HOLDER" "$MAPPING_SLOT")
USER_CALLDATA=$(cast calldata 'balanceOf(address)' "$USER_ADDRESS")
HOLDER_CALLDATA=$(cast calldata 'balanceOf(address)' "$EVENT_HOLDER")

jq -n \
  --arg contract "$WETH" \
  --arg block_number "$BLOCK_NUMBER_HEX" \
  --arg block_hash "$BLOCK_HASH" \
  --arg state_root "$STATE_ROOT" \
  --arg user_address "$USER_ADDRESS" \
  --arg user_key "$USER_KEY" \
  --arg event_holder "$EVENT_HOLDER" \
  --arg holder_key "$HOLDER_KEY" \
  --argjson mapping_slot "$MAPPING_SLOT" \
  '{contract:$contract,blockNumber:$block_number,blockHash:$block_hash,stateRoot:$state_root,mappingSlot:$mapping_slot,userAddress:$user_address,userStorageKey:$user_key,eventDerivedHolder:$event_holder,holderStorageKey:$holder_key}' \
  > "$RAW_DIR/derived-values.json"

rpc eth_call \
  "[{\"to\":\"$WETH\",\"data\":\"$USER_CALLDATA\"},\"$BLOCK_NUMBER_HEX\"]" \
  "$RAW_DIR/user-balance-call.json"
rpc eth_getStorageAt \
  "[\"$WETH\",\"$USER_KEY\",\"$BLOCK_NUMBER_HEX\"]" \
  "$RAW_DIR/user-storage.json"
rpc eth_getProof \
  "[\"$WETH\",[\"$USER_KEY\"],\"$BLOCK_NUMBER_HEX\"]" \
  "$RAW_DIR/user-proof.json"

rpc eth_call \
  "[{\"to\":\"$WETH\",\"data\":\"$HOLDER_CALLDATA\"},\"$BLOCK_NUMBER_HEX\"]" \
  "$RAW_DIR/holder-balance-call.json"
rpc eth_getStorageAt \
  "[\"$WETH\",\"$HOLDER_KEY\",\"$BLOCK_NUMBER_HEX\"]" \
  "$RAW_DIR/holder-storage.json"
rpc eth_getProof \
  "[\"$WETH\",[\"$HOLDER_KEY\"],\"$BLOCK_NUMBER_HEX\"]" \
  "$RAW_DIR/holder-proof.json"

USER_CALL=$(jq -r .result "$RAW_DIR/user-balance-call.json")
USER_STORAGE=$(jq -r .result "$RAW_DIR/user-storage.json")
USER_PROOF_VALUE=$(jq -r .result.storageProof[0].value "$RAW_DIR/user-proof.json")
HOLDER_CALL=$(jq -r .result "$RAW_DIR/holder-balance-call.json")
HOLDER_STORAGE=$(jq -r .result "$RAW_DIR/holder-storage.json")
HOLDER_PROOF_VALUE=$(jq -r .result.storageProof[0].value "$RAW_DIR/holder-proof.json")

test "$(cast to-dec "$USER_CALL")" = "$(cast to-dec "$USER_STORAGE")"
test "$(cast to-dec "$USER_STORAGE")" = "$(cast to-dec "$USER_PROOF_VALUE")"
test "$(cast to-dec "$USER_CALL")" = 0
test "$(cast to-dec "$HOLDER_CALL")" = "$(cast to-dec "$HOLDER_STORAGE")"
test "$(cast to-dec "$HOLDER_STORAGE")" = "$(cast to-dec "$HOLDER_PROOF_VALUE")"
test "$(cast to-dec "$HOLDER_CALL")" != 0

CODE_HASH=$(cast keccak "$(jq -r .result "$RAW_DIR/contract-code.json")")
PROOF_CODE_HASH=$(jq -r .result.codeHash "$RAW_DIR/user-proof.json")
test "$CODE_HASH" = "$PROOF_CODE_HASH"

jq -n \
  --arg user_value "$USER_CALL" \
  --arg holder_value "$HOLDER_CALL" \
  --arg holder_value_decimal "$(cast to-dec "$HOLDER_CALL")" \
  --arg code_hash "$CODE_HASH" \
  --arg user_account_nodes "$(jq -r '.result.accountProof | length' "$RAW_DIR/user-proof.json")" \
  --arg user_storage_nodes "$(jq -r '.result.storageProof[0].proof | length' "$RAW_DIR/user-proof.json")" \
  --arg holder_account_nodes "$(jq -r '.result.accountProof | length' "$RAW_DIR/holder-proof.json")" \
  --arg holder_storage_nodes "$(jq -r '.result.storageProof[0].proof | length' "$RAW_DIR/holder-proof.json")" \
  '{userValueHex:$user_value,holderValueHex:$holder_value,holderValueDecimal:$holder_value_decimal,codeHash:$code_hash,userAccountProofNodes:($user_account_nodes|tonumber),userStorageProofNodes:($user_storage_nodes|tonumber),holderAccountProofNodes:($holder_account_nodes|tonumber),holderStorageProofNodes:($holder_storage_nodes|tonumber)}' \
  > "$RAW_DIR/checks.json"

(
  cd "$RAW_DIR"
  find . -type f -maxdepth 1 -print0 \
    | sort -z \
    | xargs -0 openssl dgst -sha256
) > "$EXPERIMENT_DIR/SHA256SUMS"

printf 'EXP-001 checks passed\n'
