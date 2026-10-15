# EXP-007 result: a narrow consensus-root boundary is feasible

Claim type: **synthetic result**. This is a reproducible Helios consensus-core fixture replay and
root-admission experiment. It does not authenticate Sepolia or EXP-003's provider-reported root.
Recommendation is recorded in **proposed** [ADR-006](../../docs/decisions/ADR-006-consensus-root-boundary.md).

## What ran

Unmodified Helios revision `2af9b41887e069144ee56cd710f3ddca195473f8` and its exact
`ethereum_hashing` patch are retained as checksummed archives. The source preparer verifies bytes,
file types and the exact extracted inventory, rejecting extra build scripts and symlinks. No
upstream code or manifests are patched. The top-level probe repeats the upstream hashing override
as a retained local path because Cargo ignores patches in dependency workspaces.

The isolated Rust build links only the required consensus core and dependencies, not Helios's full
Ethereum client. Its lockfile resolves upstream's Alloy range to 2.5.0. Rust 1.95.0 built it locally.
Helios emits one upstream deprecated generic-array `as_slice` warning; the probe passes Clippy with
warnings denied for its own targets. No dependency warning suppression or source edit was made.

The retained minimal Deneb bootstrap and five updates supply a fixture checkpoint, genesis root,
fork schedule, signatures and SSZ branches. The wrapper verifies before applying; its store is
private. All five signatures have 32/32 participants. Expected beacon and execution-payload SSZ
roots are checked after each transition, including the optimistic head.

| Current slot | Accepted finalized slot | Optimistic slot | Execution block number |
|---:|---:|---:|---:|
| 41 | 24 | 40 | 2 |
| 89 | 72 | 88 | 20 |
| 97 | 72 | 96 | 20 |
| 97 | 80 | 96 | 28 |
| 97 | 80 | 96 | 28 |

Final accepted fixture execution block hash is
`0xfe530efb8eb3ef8c0911d9d4fed422ab2c5ae9f7171bb73cd99c28152e6773f8`.
The execution state root is `0x2020202020202020202020202020202020202020202020202020202020202020`.
These deliberately synthetic values belong to a local chain label `0`. The finalized payload SSZ
root and beacon root are different hashes; neither is a substitute for the execution trie root.
Full unedited trace: [raw/fixture-run.json](raw/fixture-run.json).

## Verification

- Ten Rust tests cover fixture head/root agreement; private finality export after bootstrap;
  checkpoint age/provenance; network/genesis identity; clock/lag admission; exact provider
  chain/number/hash/root matching; signature/finality/execution-branch corruption; unchanged state
  after failed updates; root-type distinction; and no timeout promotion of an optimistic head.
- A companion vector would use upstream `force_update` to promote slot 130 after timeout. Our
  wrapper retains finalized slot 96 and returns stale. It exposes no forced-update path.
- Four Python tests cover clean extraction reuse and rejection of modified files, extra build.rs
  and matching-byte symlinks. An independent reviewer identified the inventory gap; the regression
  failed before the fix, and the reviewer independently confirmed the fix and passing tests.
- All 27 existing main-workspace tests pass. Production code and dependency manifests are unchanged.
- The exact reproduction script passes, including tests, format check, Clippy and fixture replay.

Raw logs preserve both expected initial red tests and a corrected fixture assumption. The companion
fixture's first five updates do **not** exercise low participation; the failed coverage assertion is
retained in `raw/fixture-assumption-failure.log`. It was replaced with a timeout-promotion test.
No valid sub-quorum signature test or production security audit is claimed.

## Interface and learning walkthrough

At current slot 97, the third update proves optimistic head 96 but leaves finality at 72. The wrapper
exports execution block 20 from finalized 72. A subsequent verified update advances finality to 80,
so it exports execution block 28. A provider claiming a different block hash or root is rejected
before native proof verification. At slot 113, the exported slot-80 context exceeds this fixture's
32-slot lag policy and returns `Stale`; it does not silently accept an optimistic root.

`finalized()` returns a private-construction context; arbitrary JSON cannot create one. Its
`check_snapshot(snapshot, now_slot)` must precede proof verification, with the expected key/index
and generation obtained independently. Generation is not consensus-authenticated: the test accepts
a changed generation at this boundary intentionally, leaving publication binding to the client.

Rohan authorized this investigation; final adapter acceptance and a production freshness policy are
not attributed to him. A useful participation task is to rerun the third-update case and explain why
slot 96's newer-looking header cannot replace finalized slot 72, then identify why slot 113 fails.
This is offered, not recorded as a completed learning exercise.

## Pending work and unresolved choices

Day 8's **fixture feasibility and exact interface** are established. RQ-007 remains investigating
until target-network behavior and the integration choice are reviewed. Live Sepolia needs an
independently trusted recent checkpoint, pinned network/fork config, trusted slot clock, bounded
network decoding, sync/persistence/recovery and testing on the mainnet consensus preset. Synthetic
128-slot checkpoint age and 32-slot finalized lag are experiment inputs, not production policy.

A recent finalized root cannot authenticate our old corpus block without retained authenticated
history or an ancestry proof. A final composed read should first collect a new corpus at the
independently selected root, or implement and test that history path. Hash-only headers such as the
pinned source's Gloas representation need hash-checked execution-header retrieval; export currently
returns `UnsupportedHeader`. Setup/networking, generation commitments, final PIR/backend geometry
and repository licence remain open. Day 9 can compare representations without pretending these
root-authentication requirements have been completed.

## Reproduce

From the repository root:

```sh
sh experiments/EXP-007-consensus-root/commands.sh
python3 experiments/EXP-007-consensus-root/check_evidence.py
```

The script uses writable temporary Cargo caches and retained sources. Locked crates may need a
network download on a clean machine. The fixture executable performs no network access or storage
query. JSON output is a diagnostic artifact; reloading it cannot mint the in-memory capability.
