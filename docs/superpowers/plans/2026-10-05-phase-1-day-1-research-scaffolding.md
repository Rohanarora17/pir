# Phase 1 Day 1 Research Scaffolding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the research, experiment and decision scaffolding for the project and audit the existing protocol types against the approved proposal.

**Architecture:** Research artefacts will live beside the Rust workspace and use stable identifiers that connect questions, experiments and architecture decisions. Day 1 changes documentation and project structure only. It does not choose a Sepolia contract, proof library, PIR backend or light client implementation.

**Tech Stack:** Markdown, JSON examples, Rust workspace documentation and shell verification with `rg`.

**Spec:** `docs/superpowers/specs/2026-10-05-four-phase-research-and-implementation-design.md`

## Global Constraints

- Preserve the approved scope of one declared ERC-20 balance dataset and one finalised snapshot generation at a time.
- Keep PIR privacy, Ethereum proof verification and consensus root authentication as separate guarantees.
- Label every claim as a hypothesis, source finding, local result, design decision or implemented behaviour.
- Do not select a backend or freeze a protocol field without recorded evidence.
- Do not present the synthetic development manifest as authenticated Ethereum data.
- The parent assignment directory is not a Git repository. Use verified file checkpoints instead of commit steps until a repository is initialised.
- Use British and Indian English in project documentation.

## Review Focus

- A research question without a measurable decision criterion must be rejected by the register instructions.
- An experiment result without its command, input, machine and software revision must not support an architecture decision.
- An architecture decision must link to the research questions and experiments that support it.
- The protocol audit must distinguish implemented validation from planned Ethereum proof verification.
- Day 1 must not silently decide any item intentionally left open for Phase 1.

---

### Task 1: Add the four-phase roadmap and current status

**Files:**
- Create: `docs/roadmap.md`

**Interfaces:**
- Consumes: the approved four-phase design specification.
- Produces: the canonical phase status, ten-day boundaries, deliverables and phase gates used by the README and later research records.

- [x] **Step 1: Write the roadmap**

Create a concise roadmap containing:

```markdown
# Project roadmap

## Current status

- Active phase: Phase 1
- Active day: Day 1
- Current gate: not evaluated
- Completed evidence: protocol format unit tests and manifest CLI tests
- Next decision: Sepolia WETH workload and address corpus
```

Add a table for Phases 1 to 4 with exact day ranges `1–10`, `11–20`, `21–30` and `31–40`.
For each phase, copy the approved objective, principal research component, deliverable and gate from
the specification without introducing new scope.

- [x] **Step 2: Add status update rules**

State that the roadmap is updated only when evidence is saved, a decision is accepted or a gate is
evaluated. A day number records project progress and is not advanced merely because a calendar day
has passed.

- [x] **Step 3: Verify roadmap coverage**

Run:

```bash
rg -n "Phase 1|Phase 2|Phase 3|Phase 4|Active day|Phase gate" docs/roadmap.md
```

Expected: all four phases, the active day and gate language are present.

### Task 2: Create the research register and evidence policy

**Files:**
- Create: `research/README.md`
- Create: `research/questions.md`
- Create: `research/sources.md`
- Create: `research/decision-matrix.md`

**Interfaces:**
- Consumes: Phase 1 research questions from the specification.
- Produces: stable `RQ-NNN` question records and a comparison format used by experiments and ADRs.

- [x] **Step 1: Define the research record contract**

In `research/README.md`, define the required fields:

```markdown
## Research record

- Question
- Why it changes the design
- Options
- Method
- Evidence required
- Decision criterion
- Status
- Linked experiments
- Linked decisions
- Revisit condition
```

Define the valid statuses as `open`, `investigating`, `answered` and `superseded`. Define claim
labels for hypothesis, source finding, local result, design decision and implemented behaviour.

- [x] **Step 2: Register the Phase 1 questions**

Create `research/questions.md` with these identifiers:

- `RQ-001`: Sepolia WETH contract and address corpus.
- `RQ-002`: block-hash-pinned proof retrieval.
- `RQ-003`: Rust account and storage proof verifier.
- `RQ-004`: proof size and record representation.
- `RQ-005`: usable PIR backend and hardware requirements.
- `RQ-006`: PIR licence, maintenance and distribution constraints.
- `RQ-007`: consensus light client adapter contract.
- `RQ-008`: PIR crossover against full dataset download.

For each record, state why it affects the design, the planned Phase 1 method, the required evidence
and the decision criterion. Mark every question `open`. Link later-dependent questions explicitly,
such as `RQ-008` depending on measurements from `RQ-004` and `RQ-005`.

Use the same level-three headings under every question: `Why it changes the design`, `Options`,
`Method`, `Evidence required`, `Decision criterion`, `Status`, `Linked experiments`, `Linked
decisions` and `Revisit condition`.

- [x] **Step 3: Define source provenance**

In `research/sources.md`, define one entry format with source title, type, URL or repository revision,
date inspected, relevant claim, limitations and linked research questions. Seed the register with
the existing Ethereum specification, EIP-1186, EIP-1898, Helios, IKPIR and insPIRe sources already
used by the project research. Mark older repository inspections as needing revision checks before a
design decision relies on them.

Use these exact starting sources and inspected revisions where available:

- Ethereum Merkle Patricia Trie documentation: `https://ethereum.org/developers/docs/data-structures-and-encoding/patricia-merkle-trie/`
- EIP-1186: `https://eips.ethereum.org/EIPS/eip-1186`
- EIP-1898: `https://eips.ethereum.org/EIPS/eip-1898`
- Helios revision `43a8c9f3cdda41a6f383c4db41d9a83f102638b1`
- IKPIR revision `fcfb2b540c1cfa92c2de5ec818db3aa9fc817395`
- insPIRe GPU revision `c14d1d84a425cdaa9f86ed09465b09c9c9802f13`
- insPIRe serving revision `486678c82d3aa25b2c394048309a030077b0b61c`

- [x] **Step 4: Define the decision matrix**

In `research/decision-matrix.md`, create separate empty comparison tables for proof verifier, record
representation, PIR backend and light client adapter. Use these common columns:

```markdown
| Option | Correctness evidence | Integration cost | Runtime requirements | Licence | Maintenance | Measured result | Decision status |
```

Use `not measured` and `not reviewed` as explicit states. Do not rank an option before evidence is
recorded.

- [x] **Step 5: Verify stable identifiers and statuses**

Run:

```bash
rg -n '^## RQ-00[1-8]:' research/questions.md
rg -c '^### Decision criterion$' research/questions.md
rg -n "open|investigating|answered|superseded" research/README.md
rg -n "Proof verifier|Record representation|PIR backend|Light client adapter" research/decision-matrix.md
```

Expected: eight question headings, eight decision-criterion headings, four statuses and four
comparison areas.

### Task 3: Create architecture decision records

**Files:**
- Create: `docs/decisions/README.md`
- Create: `docs/decisions/ADR-000-template.md`

**Interfaces:**
- Consumes: research question and experiment identifiers.
- Produces: a permanent record format for accepted, rejected and superseded architecture choices.

- [x] **Step 1: Define decision states and numbering**

In `docs/decisions/README.md`, define sequential identifiers `ADR-001` onward and the states
`proposed`, `accepted`, `rejected` and `superseded`. State that an accepted decision must cite at
least one research question and the evidence that satisfies its decision criterion.

- [x] **Step 2: Write the ADR template**

Create `ADR-000-template.md` with these sections:

```markdown
# ADR-NNN: Decision title

- Status
- Date
- Deciders
- Linked research questions
- Linked experiments

## Context
## Evidence
## Decision
## Consequences
## Rejected alternatives
## Revisit condition
```

The template instructions must require concrete evidence paths and prohibit upstream benchmark
claims from being presented as local measurements.

- [x] **Step 3: Verify the decision-evidence links**

Run:

```bash
rg -n "Linked research questions|Linked experiments|Evidence|Revisit condition" docs/decisions/ADR-000-template.md
```

Expected: all four fields are present.

### Task 4: Define reproducible experiment artefacts

**Files:**
- Create: `experiments/README.md`
- Create: `experiments/manifests/README.md`
- Create: `experiments/results/README.md`

**Interfaces:**
- Consumes: `RQ-NNN` records.
- Produces: the `EXP-NNN` directory and metadata contract used by all four phases.

- [x] **Step 1: Define the experiment directory contract**

In `experiments/README.md`, require every experiment to have one directory named
`EXP-NNN-short-name` containing:

```text
README.md
manifest.json
commands.sh
raw/
summary.md
```

Define `raw/` as immutable command output, `summary.md` as interpretation and `commands.sh` as the
exact reproduction sequence. State that generated secrets and RPC credentials must never be stored.

- [x] **Step 2: Define the manifest fields**

In `experiments/manifests/README.md`, require these JSON fields:

```json
{
  "experiment_id": "EXP-001",
  "research_questions": ["RQ-002"],
  "claim_type": "local result",
  "software_revisions": {},
  "machine": {},
  "inputs": {},
  "parameters": {},
  "metrics": [],
  "started_at": "RFC 3339 timestamp",
  "completed_at": "RFC 3339 timestamp"
}
```

Explain that a real manifest replaces each descriptive example with recorded data. Require chain ID,
block hash and state root whenever Ethereum state is an input.

- [x] **Step 3: Define result handling**

In `experiments/results/README.md`, require raw data to remain separate from derived tables and
plots. Define failure as a recordable result with the command, exit status and diagnostic output.
State that synthetic and real Ethereum datasets must never be combined without labels.

- [x] **Step 4: Verify experiment requirements**

Run:

```bash
rg -n "EXP-NNN|manifest.json|commands.sh|raw/|summary.md|chain ID|block hash|state root" experiments
```

Expected: the complete experiment contract and Ethereum input requirements are present.

### Task 5: Audit protocol types against the proposal

**Files:**
- Create: `research/protocol-audit.md`
- Inspect: `crates/protocol/src/lib.rs`
- Inspect: `crates/protocol/tests/*.rs`
- Inspect: `docs/architecture.md`

**Interfaces:**
- Consumes: the approved project scope and existing Rust protocol types.
- Produces: an evidence-based list of implemented rules, provisional fields and missing boundaries
  that directs Phase 1 research without changing code prematurely.

- [x] **Step 1: Record the audit boundary**

State that the audit is a Day 1 source review. It is not proof verification, a security audit or a
decision to keep the current schema.

- [x] **Step 2: Compare requirements with current types**

Create a table with columns:

```markdown
| Requirement | Current representation | Evidence | Status | Phase 1 action |
```

Cover at least:

- Chain ID, block number, block hash, state root and generation binding.
- Declared contract and mapping slot.
- Dataset identity and address-corpus commitment.
- Record index and requested storage-key binding.
- Account proof and storage proof presence.
- Local Merkle Patricia proof verification.
- Consensus finality authentication and light client provenance.
- PIR scheme and preprocessing-generation binding.
- Fixed record length and oversized-proof handling.
- Database digest and atomic generation publication.
- Authenticated zero, authenticated absence and unavailable data.
- Stale, malformed, unsupported and verification-failed outcomes.

Use only `implemented and tested`, `represented but not authenticated`, `provisional` and `missing`
as status values.

- [x] **Step 3: Record the initial findings**

The audit must state these current facts:

- Snapshot equality and storage-key equality are implemented and tested.
- Proof byte presence is checked, but proof correctness is not verified.
- The state root is represented, but no consensus light client authenticates it yet.
- The current schema does not commit to the address corpus, database artefacts or PIR preprocessing.
- The current proof-bundle representation and `record_size` are provisional until real proof data is
  measured.
- The current error types do not yet express the complete online failure vocabulary.

- [x] **Step 4: Link gaps to research questions**

Map every provisional or missing item to `RQ-001` through `RQ-008`, or identify it as a Phase 2
implementation requirement. Do not create a design choice during the audit.

- [x] **Step 5: Verify audit completeness**

Run:

```bash
rg -n "implemented and tested|represented but not authenticated|provisional|missing" research/protocol-audit.md
rg -n "RQ-00[1-8]|Phase 2" research/protocol-audit.md
```

Expected: every status class appears and every gap has an owner.

### Task 6: Connect the scaffolding and verify Day 1

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture.md`

**Interfaces:**
- Consumes: the roadmap, research register, experiment contract, ADR format and protocol audit.
- Produces: one navigable project entry point and the verified Day 1 checkpoint.

- [x] **Step 1: Update project navigation**

Add links from `README.md` to the roadmap, approved design, research register, experiment contract,
decision records and protocol audit. Add a status statement that Phase 1 Day 1 is complete only after
the verification commands pass.

- [x] **Step 2: Update the architecture document**

Add a short design-status section explaining that the present protocol schema is provisional until
the Phase 1 gate and that architecture changes require an ADR linked to evidence.

- [x] **Step 3: Run documentation checks**

Run:

```bash
find docs research experiments -type f -print | sort
rg -n "RQ-00[1-8]" research/questions.md
rg -n "ADR-NNN|EXP-NNN" docs/decisions experiments
rg -n "roadmap|research|experiments|protocol audit" README.md
```

Expected: all Day 1 files are present and reachable from the README.

- [x] **Step 4: Run the existing Rust verification**

Run:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: formatting passes, all existing tests pass and Clippy reports no warnings.

- [x] **Step 5: Update the roadmap checkpoint**

After all checks pass, change the roadmap to record Phase 1 Day 1 as completed and Day 2 as next.
List the files created, the protocol-audit result and the unresolved decisions that Day 2 and later
research must address.
