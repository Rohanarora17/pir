# EXP-001: Sepolia WETH workload selection

## Research question

This experiment informs `RQ-001`. It checks whether one public Sepolia WETH deployment can provide
a reproducible ERC-20 balance workload with both a non-zero record and an authenticated zero or
absent record at the same pinned block.

## Hypothesis

The WETH9 deployment at `0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14` uses storage slot `3`
for `balanceOf`. For each chosen address, a direct read of
`keccak256(pad32(address) || pad32(3))` should equal the result returned by `balanceOf(address)` at
the pinned block.

## Method

1. Pin the Uniswap Sepolia token list and WETH9 source to named Git revisions.
2. Record one block first observed through the provider's `finalized` tag.
3. Confirm the chain ID, block hash, state root and deployed contract code at that block.
4. Use the user-declared public address as the expected zero or absent case.
5. Select the recipient of the last WETH `Transfer` event in the final 50-block window as the
   non-zero candidate.
6. Derive both slot keys from mapping slot `3`.
7. Compare `balanceOf`, `eth_getStorageAt` and the value returned inside `eth_getProof`.

The address selection rule is deterministic. The retained transfer logs, source revisions and pinned
block allow another person to reconstruct both records without private information.

## Reproduce

The retained `raw/` directory is immutable. Write a replay to a separate directory:

```bash
OUTPUT_DIR=/private/tmp/EXP-001-replay ./commands.sh
```

The script creates `raw/` and `SHA256SUMS` inside that directory. It refuses to overwrite files from
an earlier run.

## Expected observations

- The Uniswap token list identifies the selected address as Sepolia WETH.
- The verified WETH9 layout places `balanceOf` at mapping slot `3`.
- The user-declared address returns zero through both the contract call and direct storage read.
- The event-derived address returns the same non-zero value through both paths.
- Each `eth_getProof` response carries the same storage value as the corresponding direct read.

## Safety and claim boundary

All inputs are public chain data. This experiment does not request a signature, transaction or
private key. It checks provider-consistent state and proof collection. It does not yet verify the
proof locally and it does not authenticate the state root with a consensus light client. Those are
separate Phase 1 experiments.
