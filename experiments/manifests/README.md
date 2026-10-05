# Experiment manifest contract

Every `EXP-NNN` directory contains a `manifest.json` with these required fields:

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

The example describes field shape. A real manifest replaces every descriptive value with the exact
recorded value. Empty objects are invalid when that category affects the result.

## Field requirements

- `experiment_id` matches the experiment directory.
- `research_questions` contains every `RQ-NNN` the experiment is intended to inform.
- `claim_type` is normally `local result`. Synthetic controls are labelled `synthetic result`.
- `software_revisions` records the project revision when available, dependency revisions, compiler,
  operating system and relevant driver or CUDA versions.
- `machine` records CPU model, logical cores used, host RAM and any GPU model and memory used.
- `inputs` records file paths, generation method and SHA-256 checksums.
- `parameters` records every protocol and benchmark parameter needed to reproduce the command.
- `metrics` lists metric names, units and the raw result files that contain them.
- `started_at` and `completed_at` use full RFC 3339 timestamps with an offset.

When Ethereum state is an input, `inputs` additionally records the chain ID, block number, block
hash, state root, contract address, mapping slot and address-corpus identifier. A block tag such as
`latest` is not an acceptable replacement for the block hash in a retained result.

When PIR is measured, `parameters` additionally records the scheme, security parameters, database
dimensions, record size, query count, client state size and preprocessing generation.

Secrets are represented only by the required environment variable name. Their values never appear
in the manifest, commands or raw results.
