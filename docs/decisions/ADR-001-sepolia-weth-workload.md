# ADR-001: Select the Sepolia WETH9 balance workload

- Status: `accepted`
- Date: `2026-10-05`
- Decider: Rohan Arora
- Linked research question: `RQ-001`
- Linked experiment: [`EXP-001`](../../experiments/EXP-001-sepolia-weth-workload/)

## Context

The first snapshot pipeline needs one public ERC-20 workload with a checkable storage layout, a
deterministic address corpus and both non-zero and zero or absent records at one block. The choice
must preserve the project's contract storage proof path without adding unrelated token behaviour.

## Evidence

The Uniswap token list at revision `b41e2b93ef284c4acc897d100e77686e531fa249` identifies
`0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14` as WETH on chain `11155111`. The WETH9
source at Uniswap v2 periphery revision `ed24991304291297c3b4a52818d02f46a17aa9a2` exposes a
direct `balanceOf` mapping whose declared position gives mapping slot `3`.

`EXP-001` derived two keys at block `11848974`. For both addresses, the contract call, direct
storage read and provider proof value were equal. One record was zero and one was non-zero. The
non-zero address came from a deterministic public transfer-log rule.

## Decision

Use the Uniswap-listed Sepolia WETH9 deployment at
`0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14`. Use mapping slot `3` and derive every record key as
`keccak256(pad32(address) || pad32(3))`.

The initial corpus starts with the two `EXP-001` cases. A larger corpus may be built from the same
public event-derived rule. Every retained dataset must name one exact block hash and state root.

This decision selects the workload only. It does not select the proof verifier, proof record shape,
PIR backend or finality adapter.

## Consequences

The direct mapping keeps snapshot construction auditable while still exercising the complete
contract account proof and storage proof path. The zero user record gives a required negative case.
The event-derived holder gives a non-zero membership case without requiring a transaction or
private input from the user.

## Rejected alternatives

- Native Sepolia ETH does not satisfy the declared ERC-20 workload and removes the contract storage
  proof from the main path.
- Sepolia USDC is a useful later compatibility test, but its upgradeable proxy adds implementation
  and storage-layout decisions that are not needed to establish the first complete pipeline.
- The earlier `0x7b79995e5f793a07bc00c21412e50ecae098e7f9` candidate has verified mock WETH
  source. It is not the WETH address in the pinned Uniswap Sepolia token list.
- A locally deployed token remains a synthetic control only because it would not provide a public
  real-state corpus.

## Revisit condition

Reopen this decision if the pinned source and observed layout disagree, if local proof verification
rejects the retained proof shapes or if the event-derived corpus cannot support later record-size
and PIR experiments.
