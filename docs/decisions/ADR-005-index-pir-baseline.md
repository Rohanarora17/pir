# ADR-005: Use direct index PIR as the next immutable-snapshot baseline

- Status: `proposed`
- Date: `2026-10-15`
- Proposer: assistant; final adoption remains pending review and broader corpus evidence
- Authorization: Rohan approved direct index PIR implementation and comparison with full keyword IKPIR
- Questions: `RQ-005`, `RQ-006`; related representation question `RQ-004`
- Evidence: [EXP-006](../../experiments/EXP-006-pir-baseline/summary.md)

## Recommendation

Carry a direct index-PIR adapter into subsequent immutable-snapshot integration work. Use the
pinned IKPIR repository's SimplePIR engine as the initial development candidate, retaining FrodoPIR
and full keyword IKPIR as comparison paths. This is not final backend acceptance or a new privacy
claim. The adapter remains experimental and outside the protocol crate.

The client resolves a logical corpus index locally from a complete public map. Snapshot, map,
database geometry and preprocessing artifacts must eventually be committed to the same published
generation. No wallet-specific map lookup or ordinary RPC fallback is permitted. An independently
expected key and root remain required when verifying the decoded record.

## Evidence and rationale

All eight cases (two engines, two modes, two record sizes) recover and verify all 32 real records.
At 3 KiB, direct index request/response frames total 12,504 bytes, versus 98,720 for full keyword
lookup. SimplePIR's canonical minimum setup is 15,667,284 bytes for index mode and 125,664,173 for
keyword mode. FrodoPIR's setup is larger in this particular geometry. These are real local size
measurements with upstream setup estimates; they are not general engine-throughput rankings.

Direct indexing fits the current explicit logical indices and immutable generation model. Full
keyword IKPIR adds fingerprint stores, bucket/segment geometry and update-client machinery. Its
dynamic lookup/update features remain useful candidates, but moving to a new Ethereum root can
change native proofs and binding fields even when balances are unchanged. Incremental value-store
updates alone do not establish a correct or cheap snapshot transition.

The entire 3 KiB padded dataset plus manifest/map is only 109,390 bytes, downloadable once and
cacheable per immutable generation. Both PIR setups exceed that. The current corpus demonstrates
functionality, not a bandwidth advantage. Larger-corpus and repeated-generation costs must be
measured before claims of practical benefit or a final backend decision.

## Integration boundaries

Preserve default upstream LWE dimensions (Simple 1275, sigma 6.4; Frodo 1566). EXP-006 uses 8-bit
plaintext cells checked against the upstream geometry-dependent correctness bound. Final security
parameters require a recorded threat model and source review, not inference from passing tests.

One client state supports one outstanding query. Exact message sizes and preprocessing identifiers
are checked before backend calls. Current setup bundles remain in-process and require bounded,
versioned encoding before network deployment. The public tag prevents accidental mixing in the
local probe; it is not an authenticated generation-manifest implementation.

The source is an unaudited prototype with Apache-2.0 declared at the pinned revision. Preserved
licence/NOTICE and the active declared dependency inventory are in EXP-006. Repository licence,
complete distribution policy, network service and consensus-root authentication remain open.

## Acceptance and revisit conditions

Keep proposed until the integration choice and its limits are reviewed. Revisit for larger corpus
geometry, dominant client hint costs, public-map growth, snapshot rebuild costs, concurrency limits,
maintenance/security findings or a demonstrated benefit from keyword lookup. Day 8's root adapter
and Day 9's representation/baseline comparison can proceed without treating this as a final freeze.
