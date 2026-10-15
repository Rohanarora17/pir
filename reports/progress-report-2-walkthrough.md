# Progress Report 2: explanation and discussion notes

Prepared on 15 October 2026. These notes support a walkthrough of the report. They do not record
that Rohan has already studied or demonstrated understanding of every topic. The report is the
short update. Experiment summaries and code retain the implementation detail.

## Start with the project question

Can a client obtain a WETH token balance without revealing which address it selected, and still
check that the answer agrees with Ethereum?

Three different checks are needed:

| Question | Component | Current evidence |
|---|---|---|
| Can the provider learn which record was selected? | Private information retrieval (PIR) | Existing schemes are used in local experiments. Passing correctness tests does not prove privacy. |
| Does the returned balance agree with the specified state root? | Account and storage proof verifier | Implemented and tested on 32 real records. |
| Does that root belong to Ethereum's accepted chain? | Helios light client | Tested separately on synthetic data. Live Sepolia authentication is pending. |

A **state root** is a hash committing to Ethereum's state at a particular block. A **proof** contains
the pieces needed to check one small part of that state against the root, without downloading the
whole state. Accepting a root supplied by the same provider still leaves the question of whether it
is the right root. That is why proof verification and light-client verification are separate jobs.

Source: [verifier walkthrough](../crates/verifier/README.md),
[PIR results](../experiments/EXP-006-pir-baseline/summary.md),
[Helios results](../experiments/EXP-007-consensus-root/summary.md).

## What did “five signed Deneb updates” mean?

This described the input to a small Helios test, not five live Ethereum blocks verified by our
complete application.

- **Update:** a message carrying information for the light client to check and update its view of
  the chain.
- **Signed:** the test data includes committee signatures that Helios checks. Our wrapper also
  checks the required proof branches before applying an update.
- **Deneb:** the Ethereum consensus version whose data format these particular test inputs use.
  The version name does not describe a feature we invented.
- **Five:** simply the number of retained updates replayed in that test. It is not a security
  threshold or a measure of live network coverage.

The test uses synthetic data with a small test configuration. It checks that our connection to
Helios can export the expected finalised block and root and reject bad input. It does not show
that we have synchronised with Sepolia or authenticated the real balance dataset. The report now
states this result without requiring the reader to interpret the fixture's internal labels.

**Finalised** means accepted as settled under the consensus rules being checked. A newer observed
block is not automatically finalised. In the test, a newer observed head must not replace the
finalised root until a verified update supports that change. The wrapper must not promote it merely
because time passes. This is not a claim that finality has no underlying security assumptions.

A **checkpoint** is an independently trusted starting reference for light-client verification.
Obtaining a suitable recent checkpoint and testing live synchronisation remain pending.

Exact inputs, checks and limitations: [EXP-007](../experiments/EXP-007-consensus-root/summary.md).

## Follow one actual balance read

The retained example is record 0, address
`0xce54cf5a0de3843011cf20389c1b6a4aac442d6a`, at Sepolia block `11919648`.
The current test root is still provider-supplied.

1. **Choose the request locally.** WETH stores balances in a mapping at storage slot `3`. The
   balance's storage key is calculated as `keccak256(pad32(address) || pad32(3))`. Here, `pad32`
   means adding leading zero bytes to make a 32-byte input, and `||` means joining the inputs.
   The key is a storage location, not a password. The provider can calculate it too.
2. **Retrieve the record privately.** In the direct-index experiment, the client finds the record
   position in a public list and requests that position through PIR. Merely sending its hash or
   storage key in an ordinary request would not hide the choice.
3. **Check which request the answer belongs to.** The record must match the expected contract,
   storage key, record position and snapshot. These expectations must be selected independently of
   the response. Otherwise a provider could return a valid proof for a different request.
4. **Check the account proof.** This links the WETH contract account to the expected state root and
   recovers the root of that contract's storage.
5. **Check the storage proof.** This links the requested storage key to the checked contract
   storage root. For this record, the proof establishes that the entry is absent, which means a
   zero balance under Ethereum's storage rules.
6. **Reject inconsistent claims.** Changing the claimed balance to a nonzero value produces
   `ValueMismatch`. Removing a required proof node produces a failure, not a zero balance.

The detailed example includes the full calculated key and recovered storage root. There is no need
to memorise those hashes. The useful skill is explaining which input determines them, what each
check establishes and what would cause rejection.

Read [the example](../crates/verifier/README.md#real-input-walkthrough) alongside
[`Verifier::verify`](../crates/verifier/src/lib.rs). The real-data tests are offline and make no
live RPC requests.

## Explain the numbers without memorising the table

**32 records, 24 nonzero and 8 absent:** real examples for checking success, zero and rejection
behaviour. They are not a representative sample of all Ethereum proof shapes or a scale benchmark.

**313 rejection checks per library:** both candidates rejected the tested bad cases. This does
not establish that all possible malformed inputs are safe. The reusable verifier has additional
project checks and regression tests.

**3,072 versus 4,096 bytes:** proof sizes vary, but the tested PIR arrangement uses equal-sized
records. Padding fills unused space. Smaller records waste less space on this dataset. A proof
that does not fit must fail, not be silently shortened. Neither size is a universal Ethereum limit.

**256 successful reads:** 32 records multiplied by two PIR implementations, two lookup methods
and two record sizes. These are repeated tests of the same dataset, not 256 distinct addresses or
256 live, consensus-authenticated reads.

**12,504 bytes per direct query and reply:** communication for a read in the tested 3,072-byte
configuration. It excludes initial setup. The smaller SimplePIR direct setup estimate is
15,667,284 bytes, while the entire small dataset plus shared information is only 109,390 bytes.
Those setup objects have not yet been serialised and downloaded over a network.

**Setup information, sometimes called a hint:** information prepared from the database that the
client needs to recover the selected answer from a PIR reply. It is an upfront cost as well as a
client-storage cost. Its reuse and replacement policy matter when comparing workloads.

**35.84% possible saving:** repeated storage-proof parts could be stored once. This counts raw
proof bytes only. The index, padding and extra private requests could reduce or outweigh that
saving. The optimised representation has not yet been integrated.

Sources: [verifier comparison](../experiments/EXP-004-verifier-comparison/summary.md),
[record sizes](../experiments/EXP-005-record-geometry/summary.md),
[PIR measurements](../experiments/EXP-006-pir-baseline/summary.md).

## Questions worth discussing before presenting

**Why use eth_trie?** Both candidates passed the comparison. eth_trie fitted the actual record
format because it recovers the checked account bytes, including the storage root, from the proof.
The adapter keeps library-specific types inside the verifier. Compiler upgrades were explicitly
excluded as a deciding factor. [Decision and tradeoffs](../docs/decisions/ADR-003-eth-trie-proof-verifier.md).

**Why Helios?** It fits the Rust project and the separate job of authenticating a public root.
The saved-data experiment supports that connection. It was not a performance comparison or a
security audit against every alternative. Helios is selected, while the precise integration method
and live behaviour remain open. [Accepted choice](../docs/decisions/ADR-007-helios-light-client.md).

**Why not download everything?** Downloading a complete snapshot once also hides later selections.
For the present tiny dataset it is cheaper in bandwidth. The research question is which larger
workloads, storage limits and update rates justify PIR's setup and server costs. A larger database
alone does not settle that question. This evaluation correction came directly from Rohan's
observation, recorded in [the collaboration notes](../docs/collaboration.md#evaluation-correction-2026-10-15).

**What have we added rather than invented?** The work so far connects existing PIR and proof
libraries, adds request/snapshot checks and failure handling, retains reproducible experiments,
and measures proof storage and retrieval costs. The planned value is an evidence-based comparison
of when this composition is practical. It does not claim a new PIR cryptographic scheme or an
already demonstrated performance improvement at scale.

**What did you personally do?** Distinguish using libraries, assistant-written implementation and
personal reasoning accurately. Recorded contributions include explaining that the provider can
recompute public storage keys, the full-download privacy tradeoff, and challenging the tiny-dataset
performance conclusion. Do not claim to have independently implemented Helios or PIR, or to have
mastered a mechanism before working through it. Decisions and code ownership can be explained
honestly without understating the reasoning contribution.

**What is not working end to end yet?** Live root authentication combined with the real proof/PIR
path, a network service, a complete client, snapshot publication and larger-scale evaluation.
The final PIR implementation and record size remain open. A fresh corpus at a Helios-checked block
is the proposed way to test the full path. The old corpus is not authenticated by merely obtaining
a newer root.

## A useful first learning step

Work through the real absent-balance example above and predict these two outcomes: changing only
the claimed balance, then removing part of the proof. The distinction is a checked zero versus a
failed verification. This exercise is proposed, not recorded as completed.

From the repository root, run the existing real-corpus verification test:

```sh
cargo +stable test -p pvrpc-verifier recovers_24_inclusions --locked -- --nocapture
```

On this workspace the existing temporary cache can be selected with
`CARGO_HOME=/private/tmp/pvrpc-verifier-cargo` and
`CARGO_TARGET_DIR=/private/tmp/pvrpc-verifier-target`. A clean machine may download dependencies.

Then follow the PIR setup/query/reply/decode path using a small example. The conceptual description
here explains the component's purpose. It is not a substitute for understanding the selected
scheme's mathematical privacy assumptions, and that deeper walkthrough remains to be done.
