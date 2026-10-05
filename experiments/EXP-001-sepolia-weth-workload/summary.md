# EXP-001 result

## Result

The experiment supports selecting the Uniswap-listed Sepolia WETH9 deployment at
`0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14` as the declared Phase 1 workload.

The pinned block is `11848974` with hash
`0x9d11a05a418308f20ee690d5700d04bb23893b168a089389b89975cef3eb14e2` and state
root `0x7443069b0b51afdd2c7e2d8e54723a3b4fb66e8e200cdc3e96ff314589b0d5e2`.
The block timestamp is `2026-10-05T13:02:24Z`.

## Storage layout

The pinned WETH9 source declares `name`, `symbol`, `decimals`, `balanceOf` and `allowance` in that
order. Solidity assigns the first three declarations to slots `0`, `1` and `2`. A mapping starts at
a fresh slot, so `balanceOf` uses mapping slot `3`. For an address `a`, its record is stored at
`keccak256(pad32(a) || pad32(3))`.

The experiment did not rely on this source inference alone. It derived both storage keys and checked
the resulting storage values against ordinary `balanceOf` calls at the same block.

## Address cases

| Case | Address | Derived storage key | Value |
|---|---|---|---:|
| User-declared zero case | `0xCE54cF5a0dE3843011cF20389C1b6a4AaC442d6A` | `0x88080a0f7453a7e6f91b5d6e4f0c47e15d7639174219416b4e29a716c14f0857` | `0` |
| Event-derived non-zero case | `0x3289680dd4d6c10bb19b899729cda5eef58aeff1` | `0x621ed32f145b61c99f199a418095340a8de179d06aead00e8739fc7ce9e27b2d` | `98711243076711886151` wei |

The non-zero value is `98.711243076711886151 WETH`. The address was selected as the recipient of
the last WETH `Transfer` event in the 50-block window ending at the pinned block. The retained log
response contains four events and makes this selection reproducible.

For both cases, the ordinary contract call, direct storage read and value inside the provider's
`eth_getProof` response were equal. The user case carried 9 account proof nodes and 5 storage proof
nodes. The non-zero case carried 9 account proof nodes and 6 storage proof nodes.

## Answer to RQ-001

Use the Uniswap-listed Sepolia WETH9 deployment, mapping slot `3` and a deterministic public address
corpus. The first two-record corpus contains the user-declared zero case and the last transfer
recipient from a pinned 50-block event window. Later corpus expansion must keep the same public
selection rule and pinned-block discipline.

Native Sepolia ETH is not the main workload because its balance is part of an externally owned
account leaf. It would test an account proof but would remove the contract storage root, derived
mapping key and storage proof that this project intends to compose with PIR.

Sepolia USDC remains a possible compatibility workload. Circle's implementation uses an upgradeable
proxy, so selecting it would also require pinning and checking the implementation and inherited
storage layout active at the snapshot block. That work is useful after the WETH path is complete but
is outside this initial one-dataset boundary.

## Limitations

- The proof responses are retained but have not yet been verified by local Rust code.
- Finality is recorded from an RPC provider's `finalized` view. A consensus light client has not yet
  authenticated the state root.
- The two-record corpus establishes membership and zero or absence cases. It is not representative
  enough for proof-size or performance claims.
- A zero returned by the contract and provider proof is not yet classified locally as an explicit
  zero leaf or authenticated absence. The proof verifier experiment will make that distinction.
