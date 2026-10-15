# EXP-007: Consensus-only finalized execution context

Question: `RQ-007`. Day 8 feasibility experiment; outside the production workspace.

Hypothesis: Helios's consensus core can verify a bootstrap and signed light-client updates,
then export an execution block number/hash/state root from the accepted finalized header without
an execution RPC, a private address, a storage key, or a PIR query.

Method:

1. Retain unmodified Helios source at `2af9b41887e069144ee56cd710f3ddca195473f8` and its pinned
   `ethereum_hashing` patch at `7ee70944ed4fabe301551da8c447e4f4ae5e6c35`.
2. Replay Helios's checked-in `minimal/Deneb` `advance_finality_without_sync_committee` vectors.
   Use their `meta.yaml` checkpoint and genesis validators root as synthetic test inputs.
3. Verify the bootstrap's checkpoint/committee/execution branches, then verify each update before
   applying it. Compare finalized and optimistic beacon/execution roots with retained expectations.
4. Export a private-construction capability only after an update advances normal quorum-based
   finality. Enforce caller-supplied checkpoint recency and finalized lag, chain/genesis identity,
   and exact provider snapshot agreement. Never use `force_update`.
5. Replay five updates from the companion `light_client_sync` case without its force-update
   step; verify timeout does not promote the optimistic head.
6. Exercise corrupted signatures, execution/finality branches, wrong network/checkpoint, stale
   context, clock regression and provider root substitution. Retain unedited output and checksums.

Expected observation: synthetic finalized slots advance independently of optimistic slots; corrupt
updates do not advance the accepted state; provider roots cannot override the verified header.

Boundary: these are upstream synthetic minimal-preset fixtures (32 sync committee members), not
Sepolia/mainnet (512), live sync, or independent authentication of EXP-003. Fixture chain ID `0` is
an explicit local label. Fixture execution fields may be synthetic/default values. A trusted
checkpoint is an input, not something established by this test. Configured chain ID is associated
with a pinned genesis root and fork schedule; consensus headers do not encode an execution chain ID.
Production checkpoint sourcing, clock/slot conversion, fork/network configuration, persistence,
bounded network decoding, retry/sync, historical ancestry, and full proof/PIR composition remain
separate work. This experiment makes no fresh-chain or end-to-end verified Ethereum-read claim.

Run `sh experiments/EXP-007-consensus-root/commands.sh` from the repository root.
