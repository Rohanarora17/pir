# Pinned source and licence evidence

Scope: the local EXP-006 proof-record probe, not a release or final project licensing decision.

- Repository: https://github.com/orochi-network/IKPIR
- Revision: `fcfb2b540c1cfa92c2de5ec818db3aa9fc817395`
- Retained source archive SHA-256: `c271a8be463a94d62467f9cae4dd17014aa2bb983bb3f6e0f0ed8877e41efe57`
- Upstream root LICENSE: Apache License 2.0; copied with NOTICE under `source/` and included in archive.
- Upstream crates `ikpir-common`, `ikpir-client`, `ikpir-server` and `segmented-cuckoo` inherit that
  declared licence. Their source is unmodified and is checked against the archive before building.
- The pinned upstream is a pre-1.0 paper artifact. Its SECURITY policy says it is unaudited, lacks
  deployment serialization and is not intended for production use with real user data. This is
  not a maintained-production-library or security-certification claim.

`raw/license-tree.txt` records the resolved **normal** graph's declared licence expressions using
the exact locked probe. It includes Apache-2.0/MIT alternatives, required Unicode-3.0 alongside
unicode-ident's alternative licence, Zlib, BSL-1.0 (Boost Software License for xxhash-rust), BSD
alternatives, Unlicense/MIT and CC0-1.0. BSL-1.0 here is not the Business Source License.

These are package metadata findings. No copyleft-only expression was observed in that active
normal graph. Dev-only and inactive target/optional packages, vulnerability advisories, licence-
file-by-file audit and formal legal compatibility review are outside this inventory. The upstream
`deny.toml` is its policy; it is not evidence that we ran cargo-deny locally.

Project-owned protocol/verifier/geometry/probe crates have no declared repository licence. Keep
RQ-006 investigating and preserve upstream attribution and notices. Do not describe the complete
project as distribution-cleared or silently choose a licence because this experimental source
dependency declares Apache-2.0. Production adoption, final repository licence and required notice
packaging remain explicit decisions before release.
