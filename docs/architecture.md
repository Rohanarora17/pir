# Architecture and implementation boundary

## Design status

The current protocol schema is a tested foundation, but it remains provisional until the Phase 1
gate. In particular, the record representation, dataset commitment, PIR parameters, light client
adapter and complete failure vocabulary depend on evidence that has not yet been produced.

An architecture change that affects interfaces, guarantees, dependencies, data formats or snapshot
behaviour requires an [architecture decision record](decisions/README.md). The record must link to
the [research question](../research/questions.md) and [experiment evidence](../experiments/README.md)
that support it. Implementation convenience alone does not freeze an open design choice.

## Project question

Can a lightweight Ethereum client hide which supported WETH balance it reads from a state provider
while still checking the returned value against finalised Ethereum consensus?

The project combines three separate guarantees. PIR hides the selected record from the server.
Ethereum account and storage proofs authenticate the returned value under an execution state root.
A consensus light client authenticates that the root belongs to a finalised Ethereum block.

```text
trusted checkpoint
        |
        v
consensus light client -> finalised execution state root
                                      |
                                      v
private query -> PIR server -> proof record
                                |     |
                                |     +-> storage proof -> value
                                +--------> snapshot generation and requested key
```

## Component boundaries

### Protocol crate

`pvrpc-protocol` owns the formats and rules shared by the publisher, server and client. It must not
depend on an Ethereum node, light client or PIR library. The current manifest and proof record are
the first part of this boundary.

### Snapshot builder

The builder will read a declared address corpus at one pinned finalised block. It will derive each
WETH `balanceOf(address)` storage key, fetch the native account and storage proofs, verify them and
write fixed-size proof records. It will publish a generation only after every record and the PIR
preprocessing data are complete.

### PIR server

The server will load one immutable generation and answer a single-server PIR query over its indexed
records. It will not learn the selected record under the assumptions of the chosen PIR scheme. The
first benchmark will use CPU execution. GPU support will be considered only after profiling shows
that server computation is the limiting cost.

### Rust client and verifier

The client will derive the expected storage key, create the private query, decode one proof record
and enforce its generation and key binding. It will then verify the account proof and storage proof
against an independently authenticated state root. The client will return a balance only after all
checks pass.

### Consensus light client adapter

The adapter will provide the finalised execution block hash and state root accepted by the light
client. It will not fetch storage proofs. Its role is to stop the state provider from choosing a
fabricated root that also matches a fabricated proof.

### Evaluation harness

The harness will rebuild a declared corpus and measure snapshot construction time, stored bytes per
record, PIR preprocessing time, server latency, throughput, bandwidth, client decoding and proof
verification. It will also test stale generations, key substitution, malformed proofs and missing
records.

## Protocol rules fixed now

1. A snapshot is immutable after publication.
2. A generation number, block hash and state root identify the same database generation.
3. The contract, mapping slot, dataset dimensions and PIR parameters are public metadata.
4. A proof record carries its generation, record index, storage key, value and storage proof.
5. The client rejects records from another generation or for another key.
6. A zero value is valid only when accompanied by a proof. No proof means failure.
7. Unknown JSON fields and unsupported schema versions are rejected.
8. The initial workload uses Sepolia WETH9 at
   `0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14` and `balanceOf` mapping slot `3`.

## Next implementation slice

The next vertical slice should use a small real Sepolia corpus. It should pin one finalised block,
derive WETH balance storage keys, call `eth_getProof`, verify the account and storage proofs locally
and write records matching the protocol crate. This gives us a real verified read before PIR is
introduced.

## Decisions still open

- The Ethereum execution client or RPC endpoint used by the snapshot builder.
- The Rust Merkle Patricia proof library, or whether the minimal verifier should be implemented in
  the project.
- The consensus light client integration boundary, most likely Helios output or a small adapter.
- The PIR backend and its licence after a minimal server and client query is reproduced locally.
- The fixed record size after measuring proof sizes on the selected real corpus.
- The repository licence.
