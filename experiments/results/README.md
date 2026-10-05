# Experiment result handling

Raw observations and their interpretation remain separate.

## Raw evidence

Place direct stdout, stderr, machine-readable measurements, response fixtures and exit-status records
under the experiment's `raw/` directory. Do not correct, reorder or remove observations after a run.
If a command fails, retain its command, exit status and diagnostic output. A failed feasibility probe
is evidence and may rule out an option.

If a result must be redacted because a tool printed a credential, discard the compromised file,
rotate the credential when necessary and rerun after configuring safe output. Do not commit an
edited file and call it raw.

## Derived results

Generated tables and plots identify their input raw files and the script that produced them. The
summary distinguishes individual observations, aggregates and inferred explanations. It reports the
number of repetitions and does not convert an author-reported upstream benchmark into a local result.

Synthetic datasets and real Ethereum datasets are stored and reported separately. Results from one
must not be averaged with the other. A fixture verifies compatibility only for its recorded proof
shape. It does not establish representative Ethereum performance or complete state coverage.

## Ethereum result minimum

A retained real-state result records chain ID, block hash, state root and the mechanism used to
establish finality. If the root came only from the same RPC provider that supplied the proof, label
the result provider-consistent rather than consensus-authenticated.
