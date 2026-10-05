# EXP-002 result

## Result

EIP-1898 block-hash selection is usable for the project's Sepolia `eth_getProof` collector. Every
tested endpoint returned the same proof when called with the selected block hash and
`requireCanonical` set to `false`. The normalised proof digest was
`69b7f100d43aca520a6873d14d16eff03761b7c4c850144b02664a858aafb658` in every successful
case.

The selected block is `11849215` with hash
`0xae3702c08fffb7b4599d39c511311825b4123a78ef4aa8b1d397f373195cd5f7` and state root
`0x67edd541eefa8852c61e417316acdd74442b3a37af755096f804e2b15124e098`.

## Provider matrix

| Provider | Client version | Number | Hash, canonical required | Hash, external canonicality | Missing hash | Older EXP-001 hash |
|---|---|---|---|---|---|---|
| ethPandaOps | reth 2.7.0 | Proof | Proof | Proof | Null | Proof |
| PublicNode | Geth 1.17.1 | Proof | Proof | Proof | Error `-32000` | State unavailable |
| 1RPC | Tenderly 1.0 | Proof | Proof-window error | Proof | Error `-32000` | State unavailable |
| Tenderly | Tenderly 1.0 | Proof-window error | Proof | Proof | Error `-32001` | Proof-window error |

`Hash, external canonicality` means the request identifies the exact block hash and sets
`requireCanonical` to `false`. Canonicality is then established by the project's separate consensus
light client rather than by the proof provider.

All requests completed at the transport layer with HTTP status `200`. The differences in the table
are JSON-RPC response behaviour rather than network failures.

## Findings

1. **Exact block selection works.** All four endpoints returned an identical proof for the explicit
   hash selector with external canonicality.
2. **Block number is not a safe compatibility fallback.** Tenderly rejected the number selector but
   returned the hash selector. Falling back would not consistently improve availability and would
   reintroduce block-number ambiguity.
3. **Provider canonicality is not portable.** 1RPC rejected `requireCanonical: true` for the same
   hash that it served with `requireCanonical: false`. The final system already assigns canonicality
   and finality to the consensus light client.
4. **Failures do not share one shape.** A missing hash produced null from ethPandaOps and different
   JSON-RPC errors from the other providers. The collector must accept only a complete proof object
   and reject null, transport failures, RPC errors and malformed objects.
5. **Selector support and historical retention are separate.** Every endpoint supported a fresh
   hash request. Only ethPandaOps returned the older EXP-001 proof during this run.
6. **The proof response does not repeat the block hash.** The later verifier must check the proof
   against the expected state root associated with the selected hash. Merely receiving a proof
   object does not establish this binding.

## Answer to RQ-002

The snapshot collector will require the EIP-1898 selector
`{"blockHash": selected_hash, "requireCanonical": false}` for every `eth_getProof` request. The
selected hash and expected state root come from the snapshot context. In the complete system, the
consensus light client authenticates both.

The collector will never fall back to a block number, `latest` or `finalized`. It will fail the
generation when the provider returns null, a transport failure, a JSON-RPC error, a malformed proof
or unavailable historical state. Provider capability and retention are checked before a generation
build starts.

## Limitations

- The result establishes public provider behaviour at one Sepolia block. It does not guarantee
  future availability or service limits.
- The selected state root is still provider-reported in this experiment. Consensus authentication
  is a later Phase 1 gate.
- The proof objects have matching digests but have not yet been verified locally against the state
  root.
- No non-canonical but known block was available for testing the provider's canonicality error path.
