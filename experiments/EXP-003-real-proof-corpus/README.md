# EXP-003: First declared real WETH proof corpus

## Research questions

This extends the RQ-001 workload and RQ-002 collection evidence and supplies real inputs for
RQ-003's verifier and RQ-004's proof-shape measurements. It does not answer either later question.

## Hypothesis and method

A collector can retain exactly one structurally complete proof response per declared address at
one execution block hash, detect inconsistent contract-account data and reject incomplete builds.

1. Record PublicNode's provider-reported finalized header and the WETH Transfer logs in the
   2,048-block window ending at that header. Save the unedited source responses in `source/`.
2. Start with the two EXP-001 public seed addresses. Append lexicographically sorted distinct
   Transfer sender/recipient addresses, excluding the seeds, until the declared list has 32 entries.
   Reject removed, malformed, wrong-contract and out-of-range logs.
3. Derive slot-3 keys with the same `cast index address` operation used by EXP-001.
4. Before collecting, request the selected header by hash, check chain ID 11155111 and check that
   the selected header does not exceed or conflict with the provider's current finalized head.
5. Request every proof with `{blockHash: selected_hash, requireCanonical: false}`. The first proof
   is also the capability/retention probe. No selector/provider fallback or automatic retry occurs.
6. Require the expected contract and key, all account fields, nonempty whole-byte proof nodes,
   exactly one storage proof and a uint256 value. Require identical normalized account fields and
   account proof across the complete corpus.
7. Retain requests/responses, per-record results, provenance, counts and checksums. A failed build
   has a separate `.failed-*` directory and no completed manifest. Existing evidence is not replaced.
8. Audit the corpus offline by checking its complete file set and hashes and replaying all response
   checks against the recorded requests. Audit does not contact Ethereum.

## Reproduce

From this directory:

```bash
OUTPUT_DIR=/private/tmp/EXP-003-replay sh commands.sh
```

Use a fresh output path for each replay. The configured endpoint must retain the pinned state and
support the exact selector. Failure on expired historical state is evidence, not permission to
change the selected block. `PVRPC_RPC_URL` can select a public endpoint explicitly.
The first PublicNode replay already demonstrated expiry. The separate successful archive replay
used `PVRPC_RPC_URL=https://rpc.sepolia.ethpandaops.io`; set that variable for historical replays.

Offline audit of the retained result, from the project root:

```bash
python3 experiments/scripts/collect_corpus.py audit experiments/EXP-003-real-proof-corpus/retained
python3 experiments/scripts/collect_corpus.py audit experiments/EXP-003-real-proof-corpus/archive-replay
python3 -m unittest discover -s experiments/scripts/tests -v
```

## Claim boundary

Coverage means all 32 addresses in the declared list, not all WETH holders or all Ethereum state.
The event list's completeness is not authenticated. Provider fields and matching account proofs
are consistency checks, not a cryptographic binding to `stateRoot`.

A returned zero is recorded as `reported_zero_unverified`. The collector does not decide whether
it is stored zero, authenticated absence or a forged response. Day 5 must verify the account and
storage paths, including the zero case. The root is provider-reported; consensus authentication
and PIR remain separate future gates. The corpus is an experimental input, not a published
production snapshot and not a frozen PIR record format.
