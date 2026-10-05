# Research process

Research in this project exists to resolve an engineering decision. Reading a paper or repository
is useful evidence, but it does not complete a research question by itself. A question is answered
only when its stated decision criterion is satisfied and the result is connected to an architecture
decision or a finding that no option is currently usable.

## Research record

Every entry in [`questions.md`](questions.md) contains:

- **Question:** the uncertainty being investigated.
- **Why it changes the design:** the interface, guarantee or resource decision affected by the answer.
- **Options:** credible choices that remain open.
- **Method:** the source review, executable probe, corpus or benchmark used to compare them.
- **Evidence required:** the artefacts that must exist before answering the question.
- **Decision criterion:** the condition used to select an option or reject all options.
- **Status:** one of the four states below.
- **Linked experiments:** reproducible `EXP-NNN` records.
- **Linked decisions:** `ADR-NNN` records supported by the answer.
- **Revisit condition:** evidence that would justify reopening an answered question.

## Status values

- `open`: registered, but evidence collection has not started.
- `investigating`: at least one source or local experiment is being evaluated.
- `answered`: the decision criterion has been satisfied and the result is recorded.
- `superseded`: a later question or changed project boundary has replaced this question.

## Claim labels

- **Hypothesis:** an expectation that has not been tested in this project.
- **Source finding:** a claim supported by an identified specification, paper or repository revision.
- **Local result:** a result reproduced using recorded inputs, commands, revisions and hardware.
- **Design decision:** a choice accepted through an architecture decision record.
- **Implemented behaviour:** a property enforced by the current code and tests.

A source finding must not be presented as a local result. A local component smoke test must not be
presented as an end-to-end service result. A design decision may rely on negative evidence, but it
must state the resulting limitation or fallback clearly.

## Identifier rules

- Research questions use `RQ-NNN`.
- Sources use `SRC-NNN`.
- Experiments use `EXP-NNN`.
- Architecture decisions use `ADR-NNN`.

Identifiers are never reused after an entry is superseded or rejected. Links use repository-relative
paths so that the evidence can be followed without relying on conversation history.
