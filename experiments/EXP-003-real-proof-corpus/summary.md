# EXP-003 result

## Result

Collected all **32 declared addresses** from PublicNode at Sepolia execution block **11919648**:

- Block hash: `0xb5a539c8976e51dd412a0e6d1e7fec573b260cfddcfa8ae13cf006df5f787a5e`
- Provider-reported state root: `0xcd9f7d6a28dee33eaa3190be42a94656883f3043699dc408cdbbcc379a352e3d`
- Contract: `0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14`, balance mapping slot `3`.
- Reported balances: **24 nonzero, 8 zero**, all explicitly unverified.
- Storage proof shapes: **4–7 nodes**, **2064–2529 decoded bytes**, median **2327 bytes**.
- Contract account proof: **9 nodes**, identical normalized account data and account proof for every record.
- Initial successful collection: **9.616368 seconds**, excluding input discovery, offline audit and tests.

All 32 proof requests name the same explicit hash with `requireCanonical: false`. There is exactly
one response per declared address. Offline audit checks all retained file hashes and reconstructs
the corpus from the saved requests/responses without contacting a provider.

The deterministic input list starts with the two EXP-001 seeds and adds sorted public senders and
recipients from **282 WETH Transfer logs**, containing **80 distinct addresses**, in blocks
**11917601–11919648**. Selecting 32 addresses declares coverage; it does not enumerate all holders.

## Failure discovered and corrected

The first attempt received HTTP 403 with body `error code: 1010` on `eth_chainId`. A direct
comparison using the same endpoint and payload returned 403 with `Python-urllib/3.14` and 200 with
the explicit `pvrpc-experiment/0.1` identity. The collector now sends that project identity.
An HTTP integration regression test reproduces rejection of the default identity. The original
failed evidence is retained separately, without a completed manifest. No provider or block fallback
was introduced to obtain the successful result.

A later clean replay against PublicNode failed with `-32000`, explicitly reporting that the
selected historical state was unavailable. The collector preserved this failed build and did not
substitute a newer block. A separate run explicitly configured with
`PVRPC_RPC_URL=https://rpc.sepolia.ethpandaops.io` successfully reproduced all 32 records at the
same block hash. The normalized `corpus.json` is identical across both providers. Raw replay
evidence and its manifest are retained in `archive-replay/`; the expiry failure is retained in
`failed-retention-replay/`. Offline audit of both successful corpora passes.

The [implementation review](review.md) records the additional negative tests and fixed findings.

## What this establishes

- RQ-001's workload can supply a declared 32-address corpus with zero and nonzero reported cases.
- RQ-002's hash-selector rule is exercised over a complete multi-record collection.
- Day 5 now has real account/storage proof fixtures and explicit zero cases to verify.
- The corpus supplies preliminary proof sizes for RQ-004. Record geometry remains open.

## Limits and next step

Proof nodes are structurally complete objects, not yet proven complete trie paths. The collector
does not cryptographically tie proofs to the state root. An adversarial provider could return
consistent invented fields or repeat an old proof, and these structural checks alone would not
establish authenticity. SHA-256 file checksums detect changes relative to the saved checksums;
they do not authenticate Ethereum data or protect against a party rewriting the whole artifact set.

The eight zeros are **reported zeros**, not authenticated absence results. The next milestone is
RQ-003: compare native Rust MPT verifier options and verify these account/storage proofs against
the pinned root, including corrupted-value/root/key cases and proper zero/absence handling.

The header's finality is provider-reported and unauthenticated. Consensus light-client integration,
PIR privacy, fixed-size records and production snapshot publication are not established here.
The collection duration is a single run, not a throughput or latency benchmark. Provider retention
can expire, so a later network replay may fail even while the retained offline audit remains valid.

## Continue

From the project root, inspect and audit the corpus without network access:

```bash
python3 experiments/scripts/collect_corpus.py audit experiments/EXP-003-real-proof-corpus/retained
python3 experiments/scripts/collect_corpus.py audit experiments/EXP-003-real-proof-corpus/archive-replay
python3 -m unittest discover -s experiments/scripts/tests -v
cargo test --workspace --locked --offline
```

For another live replay, choose a fresh output path and an explicitly configured endpoint that
retains the pinned state:

```bash
PVRPC_RPC_URL=https://rpc.sepolia.ethpandaops.io OUTPUT_DIR=/private/tmp/EXP-003-next-replay \
  sh experiments/EXP-003-real-proof-corpus/commands.sh
```
