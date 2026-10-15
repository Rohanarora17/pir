# Implementation review and verification

An independent read-only code review identified two important consistency/evidence gaps:

1. The selected and provider-finalized headers could name the same hash but disagree on root or
   number. Both mutations were reproduced with failing tests. The collector now rejects either
   inconsistency before proof collection; offline replay enforces the same rule.
2. Truncated HTTP success/error bodies could escape as a generic local error and lose received
   bytes. Tests reproduced both statuses. The collector now retains `IncompleteRead.partial`,
   classifies the failure as a transport error and closes HTTP error responses explicitly.

Final verification: 18 Python tests and 14 Rust tests pass. Rust formatting and clippy with warnings
denied pass. The original retained 32-record corpus passes the stricter offline audit.

The following exclusions are explicit milestone boundaries, not undisclosed review findings:

- Cryptographic MPT verification, authenticated roots and ancestry verification remain future work.
- The generic collector accepts explicit nonempty address lists; EXP-003 declares exactly 32.
- Historical-state expiry is an expected fail-closed provider capability limit.
- Checksums provide local artifact consistency, not authenticity against coordinated rewriting.

No critical/important review finding remains open. No minor finding was deferred.
