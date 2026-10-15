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

`pvrpc-verifier` now implements the proof portion with eth_trie 0.6.1. Its project-owned
`ReadContext` binds the expected snapshot, contract, key and record index independently of the
response. It recovers the account first, decodes its storage root, then recovers storage with
typed presence/absence/errors. Local byte limits and structural admission precede backend
decoding. See the [API contract and real-input walkthrough](../crates/verifier/README.md).
The protocol has only read-only byte accessors added; it has no trie dependencies.

### Consensus light client adapter

Helios is the accepted implementation choice in [ADR-007](decisions/ADR-007-helios-light-client.md).
The integration mode within Helios remains open; ADR-006 proposes the core-only option.

The adapter will provide the finalised execution block hash and state root accepted by the light
client. It will not fetch storage proofs. Its role is to stop the state provider from choosing a
fabricated root that also matches a fabricated proof.

EXP-007 prototypes this boundary outside the production workspace using pinned Helios consensus
core and synthetic Deneb vectors. The store is private, updates are verified before application,
forced updates are excluded, and only a normal post-bootstrap finality advance enables export.
The private-construction context admits an exact chain/number/hash/state-root snapshot and checks
caller-defined freshness. It does not authenticate publisher generation or other manifest metadata.
See [proposed ADR-006](decisions/ADR-006-consensus-root-boundary.md) for exact inputs/errors, checkpoint
assumptions and unsupported hash-only header handling. Live Sepolia sync and proof/PIR composition
remain pending; the existing real corpus root is still provider-reported.

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
9. Every proof request uses the selected block hash with EIP-1898. The collector never falls back to
   a block number or moving tag.

## Next implementation slice

EXP-003 now provides 32 real proof responses collected with the hash selector and checked for
structural completeness and consistent account fields. Its provider-reported state root is not
independently authenticated, and its zeros have not been established as membership or absence.

EXP-004 verifies the account/storage paths with two Rust candidates and tests corrupt-proof
rejection. ADR-003 selects eth_trie's value-recovery verifier, now implemented in `pvrpc-verifier`
and tested against all 32 retained records. EXP-005 now measures proof/record geometry and checks
experimental padded binary rows through the verifier. EXP-006 now retrieves those rows using
index and keyword PIR with SimplePIR/FrodoPIR, with 256 verified local reads and bounded numeric-
vector framing. Proposed ADR-005 recommends direct indexing for subsequent snapshot integration.
EXP-007 establishes a synthetic consensus-root interface feasibility result; ADR-006 remains
proposed. Day 9 is in progress: EXP-008 adds a resource-bounded synthetic SimplePIR scale baseline and
once-per-snapshot cache accounting. Its estimated communication windows do not establish a client
storage or live-network benefit. The real proof-representation comparison remains pending.
Setup serialization, network
transport, live root authentication, authenticated generation commitments, final backend selection and layout remain open. Experimental codecs and
probes do not freeze the snapshot-builder API or introduce a production wire format.

## Decisions still open

- The Ethereum execution client or RPC endpoint used by the snapshot builder.
- The integration mode within the accepted Helios choice: consensus core, fuller embedded client
  or locally controlled Helios process.
- Final PIR backend acceptance, larger-corpus costs and distribution policy after the local probe.
- The fixed record size after measuring proof sizes on the selected real corpus.
- The repository licence.
