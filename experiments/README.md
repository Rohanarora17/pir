# Reproducible experiments

Experiments produce local evidence for registered research questions. Each experiment has an
immutable identifier and its own directory:

```text
experiments/EXP-NNN-short-name/
├── README.md
├── manifest.json
├── commands.sh
├── raw/
└── summary.md
```

## Required files

- `README.md` states the question, hypothesis, method, expected observations and safety boundary.
- `manifest.json` records machine, software, inputs, parameters, metrics and timestamps as described
  in [`manifests/README.md`](manifests/README.md).
- `commands.sh` contains the exact reproduction sequence. It starts with `set -eu`, uses explicit
  paths and versions where practical and never contains an RPC key, wallet key or other secret.
- `raw/` contains unedited program output, response fixtures and measurements. A failed command's
  output and exit status are valid raw evidence.
- `summary.md` interprets the raw results, answers only the linked question and states limitations.

## Lifecycle

1. Allocate the next unused `EXP-NNN` identifier.
2. Link at least one open or investigating `RQ-NNN` question.
3. Write the method, manifest and commands before interpreting the result.
4. Run the commands and save unedited output under `raw/`.
5. Complete the summary and change the linked research status only if its decision criterion is met.
6. Link the experiment from any ADR that relies on it.

Generated corpora that are too large to commit must be reproducible from a committed manifest and
script. The manifest records checksums for retained inputs and outputs. Synthetic results and real
Ethereum results use separate experiment identifiers.

## Registered experiments

- [`EXP-001`](EXP-001-sepolia-weth-workload/): Sepolia WETH9 workload, mapping slot and first zero
  and non-zero address cases.
- [`EXP-002`](EXP-002-block-hash-pinning/): EIP-1898 provider compatibility, historical retention
  and fail-closed block-hash proof collection.
- [`EXP-003`](EXP-003-real-proof-corpus/): declared 32-address real proof corpus, structural
  completeness, consistent account data, failure retention and offline replay audit.
- [`EXP-004`](EXP-004-verifier-comparison/): two pinned Rust verifiers on the same 32 real records,
  zero/exclusion handling, 313 negative checks per candidate and the accepted eth_trie choice.
- [`EXP-005`](EXP-005-record-geometry/): proof/JSON/binary sizes, node duplication, padding costs,
  explicit oversized-row failures and real encode/pad/decode/verify round trips. Candidate only.
- [`EXP-006`](EXP-006-pir-baseline/): pinned CPU index/keyword PIR client/server computation, 256
  verified real reads, setup/online byte and memory costs, admission failures and licence evidence.
  No network-service or consensus-authentication claim.

- [`EXP-007`](EXP-007-consensus-root/): pinned Helios consensus-core synthetic replay, exact finalized
  execution context and root/freshness admission tests. No live Sepolia authentication.

- [`EXP-008`](EXP-008-synthetic-pir-scale/): resource-bounded synthetic SimplePIR scale sweep,
  with fair once-per-snapshot caching, setup/online/client-memory accounting and explicit skipped
  sizes. First Day 9 slice, not a real-proof representation or live-network comparison.

- [`EXP-009`](EXP-009-guarded-local-scale/): guarded laptop extension at 131,072 records,
  with live pressure/RSS/swap monitoring, 64 correct sampled reads and two larger cases deferred
  for remote-VM testing. Fixed security parameters, synthetic data only.

## Security and privacy

Never store provider API keys, credentials, signing keys, private wallet data or unredacted personal
network identifiers. Reference credentials through documented environment variable names. Before
committing a raw RPC response, check that it contains only public chain data and the declared public
query corpus.
