# EXP-004: Rust proof-verifier comparison

## Question and method

RQ-003 asks which reusable Rust verifier should authenticate the account and storage paths.
Compare published `alloy-trie = 0.9.8` and `eth_trie = 0.6.1` on exactly the same retained EXP-003
inputs. This is an isolated executable experiment, not a production dependency selection.

The probe independently derives each slot-3 key and compares it with the retained corpus. It binds
the expected contract and key, RLP-encodes the reported account fields, verifies the account path
against the pinned state root, then verifies the storage path against the account's now-checked
storage root. Nonzero values require inclusion of the RLP integer; zero requires exclusion.

For each of the 32 real records, require rejection of wrong state root, wrong expected account,
forged account storage root, altered value, substituted RPC key, truncated account path, truncated
storage path, empty storage proof and malformed storage node. For each of the 24 nonzero records,
also claim zero and truncate its path: this must not pass as authenticated absence. Finally, test
invalid RLP with its matching root hash so parsing is actually exercised. A failed assertion or
panic fails the executable. These are 313 rejection checks per candidate, including wrapper-binding
checks as well as library-verification checks.

Ten additional passes over the 32 valid records provide illustrative local verification timings.
They exclude JSON file reading but include parsing of already loaded JSON fields, RLP encoding,
both proof checks and the candidate's per-call allocation. They are not production benchmarks.

A second executable, `recover_account`, tests integration with the account-proof-only manifest:
recover account bytes, decode their checked storage root, then recover values or absence from
native nodes and independently derived keys. RPC account fields and value hints are excluded from
verification. It runs with original and reversed node order; results are in `raw/account-recovery.jsonl`.

## Run

From this directory:

```bash
OUTPUT_FILE=/private/tmp/EXP-004-replay.jsonl sh commands.sh
```

Use a fresh result path. `Cargo.lock` pins the whole experiment graph. The installed compiler must
support both candidates: this comparison was run on Rust 1.89 nightly, and `eth_trie` declares a
1.87 minimum. The production workspace's Rust 1.85 declaration is unchanged. The full dependency
graph has not been compiled with Rust 1.85, so declared MSRV is not presented as a tested guarantee.

## Boundary

Both proofs are checked against the retained **provider-reported** state root. This does not
authenticate that root to Ethereum consensus, establish finality, or implement PIR. The eight
zero cases can now be classified as authenticated exclusions relative to that root. The root's
independent authentication remains RQ-007.

The account/storage wrappers are experiment code and intentionally use fixture assertions. The
future reusable crate needs strict fallible parsing, typed errors and resource bounds. Inline-node
coverage was inspected in upstream tests; it has not been independently reproduced in this probe.
