# Architecture decisions

Architecture decision records explain choices that change interfaces, guarantees, dependencies,
data formats or operational behaviour. They preserve the evidence available when the choice was
made so a later result can challenge the decision without reconstructing conversation history.

## Numbering

Decision identifiers start at `ADR-001` and increase sequentially. An identifier is never reused,
including when a decision is rejected or superseded. Copy [`ADR-000-template.md`](ADR-000-template.md)
and replace `000` with the next unused number.

## States

- `proposed`: written for review but not binding.
- `accepted`: supported by the required evidence and currently binding.
- `rejected`: evaluated but not selected.
- `superseded`: replaced by a later ADR that links back to this record.

An accepted decision must cite at least one `RQ-NNN` research question and the evidence that
satisfies its decision criterion. Executable evidence uses `EXP-NNN`. A source-only decision must
explain why a local experiment is unnecessary. Upstream benchmark claims remain source findings
until reproduced locally.

## When an ADR is required

Create an ADR before changing:

- The proof verifier or its trust boundary.
- The proof-record representation or version.
- The PIR backend or security parameters.
- The consensus light client adapter.
- Snapshot publication, retention or freshness policy.
- A previously accepted privacy or correctness claim.

Small implementation details that preserve accepted interfaces and guarantees do not need an ADR.
When new evidence invalidates an accepted decision, write a new record and mark the earlier one
`superseded`. Do not rewrite the old evidence.
