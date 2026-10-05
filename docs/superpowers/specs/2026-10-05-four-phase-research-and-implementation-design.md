# Four-Phase Research and Implementation Design

## Purpose

This document defines how the private and verifiable Ethereum state-read project will be carried
out over four phases of ten working days each. Research is part of the engineering process. Every
research activity must answer a design question, produce inspectable evidence and end in a recorded
decision or a clearly stated reason why the decision remains open.

The project will not spend a complete phase on literature review alone. Phase 1 combines technical
research with small executable probes so that the main system is built on evidence. Later phases
continue this method through corpus measurements, baselines, adversarial experiments and system
benchmarks.

## Project outcome

The final system will privately retrieve one indexed WETH balance record from one immutable
Ethereum snapshot. The client will verify the returned account and storage proof against an
execution state root authenticated by an Ethereum consensus light client.

The system must make three guarantees separately:

1. The PIR protocol hides the selected record from the state provider under the selected PIR threat
   model.
2. Ethereum account and storage proofs authenticate the returned value under the snapshot state
   root.
3. The consensus light client authenticates that the state root belongs to an accepted finalised
   Ethereum block.

The first version supports one declared ERC-20 balance dataset and one active finalised snapshot
generation. It does not support arbitrary `eth_call`, private writes or the complete Ethereum state.

## Existing foundation

The repository already contains a Rust workspace with a protocol crate and manifest validator. The
protocol crate defines versioned snapshot manifests, proof records, snapshot and storage-key binding
and strict Ethereum hex encoding. These types are an initial boundary, not a final design. Phase 1
will test them against real proof data before their format is frozen.

The existing synthetic manifest proves only that the format and validation rules work. It does not
authenticate Ethereum data, run PIR or verify a Merkle Patricia proof.

## Working method

Each research item will use the following record:

- **Question:** the design uncertainty being investigated.
- **Options:** the credible choices being compared.
- **Method:** the code, corpus, benchmark or source inspection used to compare them.
- **Evidence:** commands, raw inputs, outputs, software revisions and measured results.
- **Decision:** the chosen option and the reasons for choosing it.
- **Revisit condition:** evidence that would justify reopening the decision.

Research questions will use identifiers such as `RQ-001`. Reproducible experiments will use
identifiers such as `EXP-001`. Accepted architecture decisions will use identifiers such as
`ADR-001`. An architecture decision must link to the research questions and experiments that
support it.

Claims will be labelled as one of the following:

- **Hypothesis:** an expectation that has not been tested in this project.
- **Source finding:** a statement supported by an identified paper, specification or repository.
- **Local result:** a result reproduced on stated hardware and software.
- **Design decision:** a choice made from the available evidence.
- **Implemented behaviour:** a property enforced by code and tests.

This prevents an upstream performance claim, a local smoke test and an end-to-end system result
from being reported as if they were the same evidence.

## Planned documentation and experiment structure

The project will add the following structure after this design is approved:

```text
private-verifiable-rpc/
├── docs/
│   ├── architecture.md
│   ├── roadmap.md
│   └── decisions/
│       ├── README.md
│       └── ADR-NNN-short-title.md
├── research/
│   ├── README.md
│   ├── questions.md
│   ├── decision-matrix.md
│   └── sources.md
├── experiments/
│   ├── README.md
│   ├── manifests/
│   ├── scripts/
│   └── results/
└── crates/
```

Raw experiment output must not be edited by hand. Each committed result will include the command,
input manifest, machine information, software revision and a short interpretation. Large generated
corpora will be reproducible from manifests and scripts rather than committed directly.

## Phase 1: Foundation, research and design lock

### Objective

Resolve the choices that affect every later component. The phase finishes with a small real
Ethereum proof that verifies locally, a reproduced PIR query and documented decisions for the core
interfaces.

### Day-level work

| Day | Work and evidence |
|---:|---|
| 1 | Establish the research register, experiment format, decision template and four-phase roadmap. Audit the existing protocol types against the proposal. |
| 2 | Select the Sepolia WETH contract, confirm its storage layout and define the first address corpus. Record why this workload is valid and what it does not represent. |
| 3 | Implement a block-hash-pinned `eth_getProof` collection probe. Preserve the raw response, block header and RPC method parameters. |
| 4 | Build the first real proof corpus. Check proof completeness, block consistency and the difference between a zero value and authenticated absence. |
| 5 | Compare Rust Merkle Patricia proof verification options. Verify at least one account proof and storage proof locally against the selected state root. |
| 6 | Measure proof-node and complete-record sizes. Compare padded proof bundles with a content-addressed trie-node representation and propose the first record size. |
| 7 | Reproduce one server and client query using candidate PIR backends. Record build requirements, licences, maintenance status, CPU or GPU dependency and observed request shape. |
| 8 | Probe the consensus light client boundary. Identify the exact output needed by the client and demonstrate how a finalised execution block hash and state root will enter the verifier. |
| 9 | Compare bundled proof PIR, trie-node PIR and whole-shard download using the measured corpus. Complete the decision matrix and identify remaining risks. |
| 10 | Review the evidence, accept the necessary architecture decisions and freeze the interfaces required by Phase 2. Run the complete foundation test suite. |

### Research questions

- Which Sepolia WETH deployment and address corpus provide a reproducible balance workload?
- Does the selected RPC path support block-hash-pinned proof retrieval at a finalised block?
- Which Rust proof verifier correctly handles Ethereum account and storage proofs with acceptable
  integration cost?
- What proof-size distribution determines the first fixed record size?
- Is a padded proof bundle the most reliable first backend, or is trie-node retrieval already
  practical for the selected corpus?
- Can a candidate PIR backend execute one complete server and client query on available hardware?
- What are the backend's licence, maintenance and distribution constraints?
- What exact light client output authenticates the execution state root without trusting the state
  provider?
- At what corpus size does PIR provide a meaningful bandwidth advantage over downloading the whole
  declared dataset?

### Deliverables

- Research register, source register and decision matrix.
- Reproducible experiment harness and result format.
- Real block-pinned `eth_getProof` fixture set.
- One locally verified account and storage proof.
- Proof-size and candidate record-layout report.
- One reproduced PIR server and client query.
- Light client adapter interface and feasibility evidence.
- Accepted architecture decisions for the Phase 2 interfaces.

### Phase gate

Phase 1 passes only when all of the following are true:

1. A real WETH storage value is verified locally against the state root of a pinned finalised block.
2. The proof corpus can be regenerated using documented commands.
3. At least one usable PIR backend completes a server and client query on available hardware.
4. The PIR backend's licence and build requirements are recorded.
5. The light client adapter has a precise input and output contract.
6. The record layout and maximum supported proof size are derived from measured data.
7. The protocol format is updated to match the decisions and its tests pass.

If a gate fails, Phase 2 begins only with an explicit fallback decision. A fallback may retain the
bounded proof corpus, choose the simplest working proof-bundle representation or replace an
unusable backend. The failed result remains part of the project findings.

## Phase 2: Verified snapshot pipeline

### Objective

Build a reproducible pipeline that converts one declared address corpus and one pinned finalised
block into an immutable database of verified, fixed-shape proof records.

### Day-level work

| Day | Work and evidence |
|---:|---|
| 11 | Implement configuration, corpus loading and finalised block selection using the Phase 1 interfaces. |
| 12 | Implement canonical WETH `balanceOf(address)` storage-key derivation and known-vector tests. |
| 13 | Implement block-pinned account and storage proof ingestion with explicit RPC failure classes. |
| 14 | Integrate account-proof verification against `stateRoot`. Reject a proof for another account or root. |
| 15 | Integrate storage-proof verification against the account `storageRoot`. Distinguish membership, authenticated absence and unavailable data. |
| 16 | Encode verified values and proof material into the selected record format. Enforce size limits without truncation. |
| 17 | Build the snapshot manifest, index and database as one staged generation. |
| 18 | Implement atomic publication, active-generation selection and bounded retention of the previous generation. |
| 19 | Add malformed proof, mixed block, oversized record, interrupted build and partial publication tests. Reproduce the corpus from a clean directory. |
| 20 | Measure construction time, database size, proof amplification, peak memory and publication time. Review the Phase 2 gate. |

### Research component

Phase 2 studies the cost and correctness of transforming native Ethereum proofs into PIR-ready
records. Measurements will test whether the Phase 1 record size remains sufficient and whether
proof duplication makes the bundled representation impractical. These results may change
preprocessing or storage choices without changing the client trust model.

### Phase gate

Phase 2 passes when a clean command rebuilds a complete immutable generation for the declared
corpus, every published record was verified before publication, interrupted builds never become
active and all output files agree on the same block hash, state root and generation.

## Phase 3: Private retrieval and authenticated client

### Objective

Join the verified snapshot pipeline, PIR service, Rust client and consensus light client adapter
into one complete private read.

### Day-level work

| Day | Work and evidence |
|---:|---|
| 21 | Define the backend-independent PIR query and response boundary using the frozen proof-record format. |
| 22 | Implement PIR database preprocessing for one immutable generation. |
| 23 | Implement the server request path and generation selection. Record the visible request metadata. |
| 24 | Implement client query construction and response decoding. Test record selection without using Ethereum verification. |
| 25 | Integrate the consensus light client adapter and obtain an authenticated finalised execution root. |
| 26 | Bind the PIR response to the requested storage key, record index and snapshot generation. |
| 27 | Verify the returned account and storage proof against the authenticated root. Return a value only after every check passes. |
| 28 | Define and implement stale, unavailable, malformed, unsupported and verification-failed outcomes. Do not fall back to a revealing RPC call. |
| 29 | Measure an ordinary `eth_getProof` request, full-dataset download and CPU PIR query over the same corpus and root. |
| 30 | Run the complete end-to-end private read and review privacy assumptions, correctness evidence and the Phase 3 gate. |

### Research component

Phase 3 measures the privacy and systems cost of the first complete composition. It compares PIR
with simpler baselines and records which metadata remains public, including contract shard,
snapshot, connection identity, timing and traffic volume. CPU performance is the baseline. GPU
acceleration is not selected until these measurements identify server computation as the limiting
cost.

### Phase gate

Phase 3 passes when the client privately retrieves one indexed balance record, rejects a response
for a different key or snapshot, verifies the proof against an independently authenticated
finalised root and produces reproducible latency and bandwidth results for all three baselines.

## Phase 4: Adversarial evaluation and final system

### Objective

Establish the practical limits and failure behaviour of the integrated service, then prepare a
demonstration and report that can be reproduced by another person.

### Day-level work

| Day | Work and evidence |
|---:|---|
| 31 | Finalise the experiment runner, workload manifest, metric schema and machine-information capture. |
| 32 | Test altered values, wrong accounts, wrong slots, wrong roots, malformed RLP, bad hashes and invalid absence claims. |
| 33 | Test missing records, incomplete proof paths, timeouts, unsupported requests and attempts to trigger an ordinary RPC fallback. |
| 34 | Test generation publication, concurrent pinned reads, retention expiry, restart during publication and snapshot transition. |
| 35 | Measure preprocessing, database size, RAM, client work, proof verification, bandwidth and end-to-end p50 and p95 latency. |
| 36 | Measure throughput and saturation under controlled concurrency. Separate kernel time, network time and amortised setup cost. |
| 37 | Apply the GPU decision rule. If server computation is the measured bottleneck, run the GPU comparison. Otherwise document why CPU remains the correct baseline. |
| 38 | Reproduce the main results from a clean environment and check that plots and tables are generated from raw result files. |
| 39 | Build the live demonstration showing a verified read, corrupt-response rejection and a consistent snapshot transition. Record the tested build as a fallback. |
| 40 | Complete the findings, limitations, threat model and reproducibility instructions. Run the final acceptance suite. |

### Research component

Phase 4 is the main experimental analysis. It tests the integrated service rather than relying on
component benchmarks. Negative results are valid outcomes. For example, a finding that a whole
shard download beats PIR below a measured corpus size is useful if the comparison is reproducible
and the claim is kept within the tested conditions.

### Final gate

The project is complete when another person can rebuild the declared snapshot, reproduce one
private verified read, observe rejection of corrupted data, repeat the principal benchmarks and
understand which privacy, availability and freshness guarantees are outside the system's scope.

## Evaluation metrics

The evaluation will record:

- Snapshot construction time and publication time.
- Raw proof bytes, encoded record bytes and database amplification.
- PIR preprocessing time and output size.
- Peak host RAM and, where applicable, GPU memory.
- Client query-construction and response-decoding time.
- Account and storage proof-verification time.
- Request and response bytes.
- End-to-end p50 and p95 latency.
- Throughput and saturation point.
- Snapshot transition delay and retained-generation cost.
- Failure status and outbound request behaviour for every adversarial case.

Results must identify the corpus, chain ID, block hash, state root, software revision, machine,
backend parameters and number of repetitions. Synthetic datasets and Ethereum proof corpora will
be labelled separately.

## Testing strategy

Protocol and verification rules will be developed with test-first cycles. Unit tests will cover
encoding, versioning, storage-key derivation, proof verification and failure classification.
Integration tests will cover snapshot construction, atomic publication, PIR round trips and the
light client adapter. Adversarial tests will mutate one trust-chain element at a time. Benchmark
scripts will be deterministic and will save raw results before producing summaries.

The test suite must never obtain a passing result by silently contacting an ordinary revealing RPC
method. Network behaviour will be observable in the integration harness so this property can be
tested.

## Scope control

The following work remains outside the four phases unless a phase gate proves it necessary:

- Arbitrary contract execution or general private `eth_call`.
- Private transactions or private writes.
- Hiding the selected contract, snapshot, client IP, timing or traffic volume.
- A new PIR construction, zero-knowledge proof system or consensus protocol.
- Complete Ethereum state coverage.
- A production service-level availability guarantee.

The project may reuse established cryptography and client implementations. Its contribution is a
complete, measured and failure-aware composition for private Ethereum state reads.

## Decisions that remain open until Phase 1

- Sepolia WETH contract and initial address corpus.
- RPC provider or local execution client used for proof collection.
- Rust Merkle Patricia proof verifier.
- Bundled proof, trie-node or hybrid record representation.
- Fixed record size and maximum supported proof shape.
- PIR backend and parameter set.
- Consensus light client adapter implementation.
- Snapshot retention count.
- Repository licence.

These are intentionally open because Phase 1 is designed to produce the evidence needed to decide
them. Later implementation must not silently choose or change them outside the recorded decision
process.
