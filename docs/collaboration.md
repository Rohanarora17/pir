# Collaboration and learning agreement

Recorded on 2026-10-07 at Rohan's request.

## Purpose and ownership

Rohan wants to be an active learner and participant who understands what is happening, which
decisions were made and why. The assistant can write roughly 80% of the code. Learning is a
requirement alongside working implementation: Rohan should be able to explain the system end to
end, defend its decisions, interpret experiments and investigate failures.

The percentage describes the intended division of effort, not a line-count quota. Rohan's
contributions should affect the work through reasoning, design, experiments and selected code.

## Work cycle

1. Before each substantial task, explain the objective, relevant concepts, accepted constraints,
   credible options, tradeoffs and recommendation. Make reasoning visible before implementation.
2. Discuss new consequential decisions together. Prepare evidence and concrete options, keep open
   choices open until agreed, and record actual participation. Routine coding can proceed
   autonomously within accepted decisions and existing authorization.
3. Implement small, understandable slices. Walk through the important code with a real input,
   tracing transformations, checks, outputs and failure paths. Identify missing guarantees.
4. Offer Rohan a meaningful small contribution for each milestone, such as predicting an
   experiment, choosing a failure case, modifying a check, interpreting command output or
   explaining why a shortcut breaks a guarantee. Adapt the contribution to his pace.
5. Check understanding conversationally through occasional explain-back or prediction prompts.
   Use simpler examples to address gaps. These are learning opportunities, not mandatory exams;
   do not mistake silence or approval of a change for demonstrated understanding.
6. Leave a written trail that supports resuming the work and explaining it independently.

## Learning sequence

Follow one real WETH balance read and connect each topic to project code and evidence.

| Topic | Learning outcome |
|---|---|
| Ethereum state | Explain accounts, contract storage, WETH's balance mapping and storage-key derivation. |
| Account and storage proofs | Explain the path from a state root to a checked value, including zero and absence. |
| Consensus light client | Explain how the root is authenticated, finality and checkpoint assumptions, and remaining trust. |
| PIR | Work through a tiny database, then explain the chosen scheme's query, response, decoding and assumptions. |
| System composition | Explain key and generation binding, immutable snapshots, publication and explicit failures. |
| Evaluation | Interpret baselines, measurements, adversarial cases, claim limits and operating costs. |

Explain PIR privacy, proof authenticity and root authentication separately. A library call or a
passing format check alone is not an explanation of its mechanism or security guarantee. Use
small examples and diagrams when they clarify the mechanism, followed by the actual artifacts.

## Decision and evidence records

Use the existing research questions, experiments and ADRs. Record the problem, options, evidence,
decision, consequences and revisit condition. Describe whether Rohan discussed, selected or
reviewed the choice, and distinguish assistant recommendations from user acceptance. Preserve
historical records; correct inaccurate attribution transparently instead of inventing participation.

Separate source findings, hypotheses, design decisions, implemented behaviour, local tests and
end-to-end verification. Keep a record of topics covered and explanations or experiments Rohan
has actually completed. Do not mark a topic as understood solely because it was explained.

## Initial catch-up session

Pending: trace one existing WETH read from the wallet address through mapping-slot derivation,
the RPC proof response and expected root. Review ADR-001's workload rationale and ADR-002's
block-hash collection rationale, including alternatives and evidence. Inspect which rules the
current protocol code enforces and which proof, PIR and light-client guarantees remain ahead.
Offer a small prediction or failure-case exercise as part of the walkthrough.

This is a proposed starting session, not a completed learning milestone. It does not change the
existing Day 4 checkpoint or mark any implementation gate complete.

## Milestone handoff

At the end of substantial work, provide:

- What changed and why, with links to important code and decisions.
- What was checked, the results and the limits of that evidence.
- A learning takeaway and a concrete participation opportunity.
- Pending work and unresolved decisions.
- Exact commands needed to reproduce the result or continue.

Keep this agreement discoverable from `AGENTS.md` and the README. Update learning records only
from actual sessions and user participation.

## Session checkpoint: 2026-10-15

Rohan explained that a provider can recompute an address's storage key using the same public
derivation, and that downloading the whole dataset hides selection at the cost of transferring
the dataset. These are demonstrated reasoning steps; homomorphic selection was introduced but
has not yet been checked through an exercise.

Rohan then requested implementation progress. Continue learning alongside concrete engineering,
with brief explanations tied to changes and evidence. Do not require a chain of tutorial answers
before carrying out already authorized roadmap work. EXP-003 implements the Day 4 collection
milestone. A useful next walkthrough is one collector rejection case and its matching test.

## Delegated verifier decision: 2026-10-15

Rohan excluded compiler upgrades as a selection factor, specified integration/features/flexibility,
and then explicitly delegated the final choice based on future integrations. The assistant selected
eth_trie 0.6.1 in accepted ADR-003. Selection is complete; reusable integration, PIR and consensus
authentication remain separate pending work. Do not ask Rohan to reapprove this same dependency
choice unless a documented revisit condition is met.

## Helios selection: 2026-10-15

After the assistant compared Helios with Nimbus, Lodestar, Snowbridge and SP1 Helios, Rohan said,
"okay let's go with helios only". Accepted ADR-007 records that implementation choice. This does
not demonstrate understanding of checkpoint/finality mechanics or accept the core-only integration
proposal in ADR-006. Do not ask Rohan to reapprove Helios selection unless a documented revisit
condition is met. Integration mode, live sync, recovery and root authentication remain pending.

## Evaluation correction: 2026-10-15

Rohan challenged using the tiny 32-record dataset to judge PIR's practical value, pointing out that
its overhead is expected to dominate whole-dataset download at that scale. Treat this as a correction
to evaluation emphasis: the existing corpus is a functional fixture. RQ-008 now separates scale
tests, real versus synthetic corpus evidence, setup amortization and generation-update costs.
Do not present the small-corpus bandwidth result as the architectural verdict. This records Rohan's
reasoning contribution; it does not claim that larger-scale performance has been demonstrated.
