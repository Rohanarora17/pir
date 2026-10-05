# Phase 1 research questions

## RQ-001: Which Sepolia WETH workload should the project declare?

### Why it changes the design

The contract, storage layout and address corpus determine whether direct `balanceOf(address)` storage
reads are valid, how storage keys are derived and what coverage the service can claim.

### Options

- A canonical Sepolia WETH deployment with a verified source and direct balance mapping.
- Another well-documented ERC-20 deployment if no suitable WETH corpus is reproducible.
- A locally deployed reference token only as a synthetic control, not as the main Ethereum corpus.

### Method

Inspect verified contract source and deployment history, identify the balance mapping slot, derive
known storage keys and compare direct storage values with ordinary contract calls at one pinned
finalised block. Construct a deterministic address list from declared public data.

### Evidence required

Contract address, chain ID, source revision, mapping-slot derivation, pinned block hash, address-list
construction rule and matching storage and contract-call values.

### Decision criterion

Select the deployment only if its storage layout is independently checkable, at least one non-zero
and one zero or absent balance can be reproduced at the same block and the corpus can be regenerated
without private data.

### Status

`answered`

### Linked experiments

[`EXP-001: Sepolia WETH workload selection`](../experiments/EXP-001-sepolia-weth-workload/)

### Linked decisions

[`ADR-001: Select the Sepolia WETH9 balance workload`](../docs/decisions/ADR-001-sepolia-weth-workload.md)

### Revisit condition

Reopen if the contract is upgraded, its verified layout is inconsistent with observed state or the
corpus cannot supply representative membership and absence cases.

## RQ-002: How will proofs be pinned to one finalised block?

### Why it changes the design

If proof requests can silently resolve against different heads, the snapshot can combine values and
proofs from inconsistent state roots even when each response looks valid in isolation.

### Options

- `eth_getProof` using an EIP-1898 block-hash selector where supported.
- A local execution client interface that reads from an explicitly selected block hash.
- A block-number request guarded by before-and-after header checks as a labelled fallback only.

### Method

Probe candidate providers and clients with a known block hash, record the exact JSON-RPC request and
response, fetch the corresponding header independently and reject any response that cannot be tied
to the requested block hash and state root.

### Evidence required

Raw request, raw response, provider or client version, execution header, finality source and a
negative test using an unsupported or mismatched selector.

### Decision criterion

Choose a path only if a caller can identify the exact block hash used for every proof and fail closed
when that block is unavailable. A provider-supplied root without independent authentication does not
satisfy the criterion.

### Status

`answered`

### Linked experiments

[`EXP-002: Block-hash-pinned proof collection`](../experiments/EXP-002-block-hash-pinning/)

### Linked decisions

[`ADR-002: Require block-hash-pinned proof collection`](../docs/decisions/ADR-002-block-hash-pinned-proof-collection.md)

### Revisit condition

Reopen if the chosen provider changes selector support or the light client cannot authenticate the
same execution context.

## RQ-003: Which Rust verifier should authenticate account and storage proofs?

### Why it changes the design

The verifier determines proof compatibility, error semantics, dependency size and how the client
connects an account proof, `storageRoot` and storage proof to the authenticated state root.

### Options

- Reuse the Alloy proof verification path used by Helios.
- Use another maintained Rust Merkle Patricia Trie verifier with Ethereum proof compatibility.
- Implement the minimal verifier locally only if reusable libraries cannot satisfy the corpus.

### Method

Run each viable verifier against the same real account and storage proofs, Ethereum client fixtures
and deliberately corrupted nodes. Inspect support for inline nodes, RLP errors, membership and
authenticated absence.

### Evidence required

Pinned dependency revision, licence, build command, successful real-proof verification, named
negative cases, public API shape and measured verification time.

### Decision criterion

Prefer the maintained reusable verifier that accepts the real corpus, rejects every required corrupt
case, exposes the roots and failure information needed by the client and does not force unrelated
execution-client functionality into the protocol crate.

### Status

`open`

### Linked experiments

None yet. Day 5 will register verifier comparisons.

### Linked decisions

None yet.

### Revisit condition

Reopen if later corpus records expose unsupported proof shapes, the dependency becomes unavailable or
the verifier cannot express authenticated absence correctly.

## RQ-004: What proof representation and record size should PIR retrieve?

### Why it changes the design

Padded complete proofs simplify retrieval but duplicate common nodes. Content-addressed nodes reduce
duplication but require multiple private reads and a fixed traversal budget. Record geometry directly
changes storage, bandwidth and PIR parameters.

### Options

- One padded proof bundle for every supported address.
- Content-addressed trie nodes with fixed-budget private traversal.
- A hybrid with public common nodes and private query-specific proof material.

### Method

Measure node sizes, proof lengths, duplication and encoded record sizes on the real corpus. Estimate
the number of PIR operations, padding cost and failure behaviour for every option. Retain whole-shard
download as a baseline.

### Evidence required

Corpus manifest, raw size distribution, maximum observed proof shape, padding rule, storage
amplification, expected query count and explicit behaviour when the declared limit is exceeded.

### Decision criterion

Select the simplest representation that supports the Phase 3 end-to-end read within measured
storage and bandwidth bounds. Limits must come from observed data with an explicit unsupported case,
not from two hand-selected fixtures.

### Status

`open`

### Linked experiments

None yet. Day 6 will register the record-shape experiment.

### Linked decisions

None yet.

### Revisit condition

Reopen if a larger corpus exceeds the declared proof shape, duplication becomes the dominant cost or
the selected PIR backend imposes incompatible record geometry.

## RQ-005: Which PIR backend is usable for the first complete system?

### Why it changes the design

The backend determines preprocessing, client state, database geometry, server hardware and whether a
CPU baseline can be established before GPU optimisation.

### Options

- IKPIR with a CPU-first development configuration.
- insPIRe where compatible GPU hardware and its serving layer are reproducible.
- Another maintained single-server PIR implementation that satisfies the same query-privacy scope.

### Method

Build pinned revisions, run one real server and client query, record parameters and visible request
shape and measure preprocessing, query, response, memory and failure behaviour on available hardware.

### Evidence required

Repository revision, dependency versions, complete commands, hardware description, parameter set,
successful round trip, raw timings, bytes transferred and a failure trace for an invalid query.

### Decision criterion

The first backend must complete a reproducible round trip, support the selected record geometry and
fit available development hardware. Prefer a CPU baseline unless measurement shows that the backend
requires GPU serving or CPU service is already the blocking cost.

### Status

`open`

### Linked experiments

None yet. Day 7 will register backend smoke tests.

### Linked decisions

None yet.

### Revisit condition

Reopen if the chosen backend cannot fit measured records, fails under the intended corpus size or a
better-maintained compatible backend becomes available before Phase 3 freezes integration.

## RQ-006: Can the selected PIR backend be legally and practically distributed?

### Why it changes the design

A locally runnable repository without a compatible licence, maintained dependency chain or clear
redistribution terms cannot be the foundation of a public reproducible project.

### Options

- Adopt a backend with an explicit compatible open-source licence.
- Keep a backend as a non-distributed experimental comparison.
- Select a different backend when licence or maintenance evidence is insufficient.

### Method

Inspect the repository licence at the pinned revision, dependency licences, build documentation,
recent maintenance and release state. Record facts without inferring permission from public source
availability.

### Evidence required

Licence file and revision, dependency review, build status, last maintained revision and a written
distribution boundary.

### Decision criterion

Only a backend with explicit compatible terms and reproducible dependencies may be integrated as a
distributed project dependency. An unresolved licence excludes distribution even if a local smoke
test succeeds.

### Status

`open`

### Linked experiments

Shares the Day 7 backend records with `RQ-005`.

### Linked decisions

None yet.

### Revisit condition

Reopen if upstream adds or changes a licence, dependency terms change or distribution scope changes.

## RQ-007: What is the smallest correct consensus light client adapter?

### Why it changes the design

The client must authenticate the execution state root independently of the state provider without
placing consensus code inside the protocol crate or trusting a provider-selected header.

### Options

- Embed a narrow Helios library interface.
- Consume a locally running Helios process through a versioned adapter.
- Use a fixture adapter only for deterministic tests while keeping a real Helios path mandatory for
  the final demonstration.

### Method

Inspect the pinned Helios interfaces and run a probe that obtains a finalised beacon header,
execution block hash and execution `stateRoot`. Define the checkpoint, chain and freshness inputs
that the adapter must expose.

### Evidence required

Pinned Helios revision, exact output fields, sync or fixture command, trusted checkpoint source,
finality and freshness semantics and one mismatch test against a provider-supplied root.

### Decision criterion

Choose the smallest interface that supplies an independently authenticated finalised execution block
hash and state root with explicit chain and freshness information. The adapter must not fetch the
private storage key.

### Status

`open`

### Linked experiments

None yet. Day 8 will register the adapter probe.

### Linked decisions

None yet.

### Revisit condition

Reopen if the chosen Helios interface changes, cannot supply the selected execution context or adds a
trust dependency inconsistent with the threat model.

## RQ-008: When is PIR preferable to downloading the declared dataset?

### Why it changes the design

For a small public shard, downloading the complete dataset may be simpler, cheaper and more private
than running PIR. The project needs a measured crossover rather than assuming PIR is always useful.

### Options

- PIR for single and repeated reads.
- Whole-dataset download followed by local lookup.
- Ordinary `eth_getProof` as the revealing correctness baseline.

### Method

Using identical roots and logical queries, sweep corpus sizes and measure upstream and downstream
bytes, preprocessing, client state, latency and repeated-read cost for all three paths.

### Evidence required

Results linked to the same corpus generator, block root, hardware, network conditions, backend
parameters and repetition count. Report single-read and amortised repeated-read results separately.

### Decision criterion

Report the measured corpus and reuse region where PIR provides a useful bandwidth or latency tradeoff.
If no tested region favours PIR, retain the engineering prototype and report that negative result
without claiming practical superiority.

### Status

`open`

### Linked experiments

Depends on the record measurements from `RQ-004` and backend measurements from `RQ-005`. The main
comparison runs on Days 9 and 29.

### Linked decisions

None yet.

### Revisit condition

Reopen when the record representation, PIR parameters, corpus scale or network model changes.
