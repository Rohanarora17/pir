# EXP-002: Block-hash-pinned proof collection

## Research question

This experiment informs `RQ-002`. It checks whether a snapshot collector can request
`eth_getProof` for one exact Sepolia block hash and fail closed when the selected block is missing or
its state is no longer available.

## Hypothesis

At least one public Sepolia endpoint will accept the EIP-1898 block selector
`{"blockHash": "0x...", "requireCanonical": true}` for `eth_getProof`. The successful response
should match a block-number request for the same block. A nonexistent hash must not return a proof.

## Method

The experiment sends the same request set to ethPandaOps, PublicNode, 1RPC and Tenderly:

1. Record `web3_clientVersion`, chain ID and the provider-reported finalised head.
2. Fetch the selected header by hash.
3. Fetch the same WETH storage proof by block number.
4. Fetch it by block hash with `requireCanonical` set to `true`.
5. Repeat the hash request with `requireCanonical` set to `false`.
6. Request a nonexistent hash and retain the exact failure response.
7. Request the older block retained by `EXP-001` to separate selector support from state retention.

The selected positive block is Sepolia block `11849215` with hash
`0xae3702c08fffb7b4599d39c511311825b4123a78ef4aa8b1d397f373195cd5f7`.

## Expected observations

- A supported endpoint returns the same proof object for the number and hash selectors.
- A nonexistent hash returns an error or null result and never a proof object.
- Endpoints may support block-hash selectors but differ in historical-state retention.
- Provider error codes and messages may differ, so the collector should not depend on one exact
  error string.

## Reproduce

The retained evidence is immutable. Write a replay to a separate directory:

```bash
OUTPUT_DIR=/private/tmp/EXP-002-replay ./commands.sh
```

## Safety and claim boundary

The experiment sends read-only JSON-RPC calls and uses only public chain data. It tests selector
handling and provider availability. It does not verify Merkle Patricia proofs or authenticate the
state root through a consensus light client.
