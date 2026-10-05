# ADR-002: Require block-hash-pinned proof collection

- Status: `accepted`
- Date: `2026-10-05`
- Decider: Rohan Arora
- Linked research question: `RQ-002`
- Linked experiment: [`EXP-002`](../../experiments/EXP-002-block-hash-pinning/)

## Context

A snapshot may require several proof requests. A moving tag can cross a head change while a block
number does not identify a unique block during a reorganisation. Every proof must therefore be tied
to one exact block hash and later verified against the state root authenticated for that hash.

EIP-1898 permits `eth_getProof` to receive a block selector containing `blockHash` and
`requireCanonical`. The proof provider is not the project's source of canonicality. That role belongs
to the consensus light client.

## Evidence

`EXP-002` sent number selectors, hash selectors and missing hashes to four public Sepolia endpoints.
All four returned an identical proof when given the selected hash with `requireCanonical` set to
`false`. Successful proof responses had the same normalised SHA-256 digest.

The endpoint behaviour was not otherwise uniform. One endpoint returned null for a missing hash.
Other endpoints returned different error codes. One endpoint rejected the selected number while
accepting its hash. Another rejected `requireCanonical: true` while serving the same hash when
canonicality was left to the caller. Only one endpoint retained the older EXP-001 state.

## Decision

Every snapshot proof request must use this EIP-1898 selector:

```json
{
  "blockHash": "0x<selected block hash>",
  "requireCanonical": false
}
```

The snapshot context contains the selected block hash and expected state root. In the complete
system these values come from the consensus light client. The local proof verifier binds every
account proof to that state root.

The collector accepts only a complete proof object. Null, transport failures, JSON-RPC errors,
malformed objects and unavailable state fail the complete snapshot generation. It never retries
with a block number, `latest` or `finalized`.

Before building a generation, the collector probes whether the configured endpoint supports the
exact hash selector and still retains the selected state. Provider selection remains configuration
rather than a protocol rule.

## Consequences

Every proof request has an unambiguous execution-state identity. Provider canonicality checks do not
become part of the trust model. An endpoint with insufficient retention can stop a generation, so
the builder may need an archive-capable endpoint when collection is delayed.

The collector must preserve distinct diagnostics for unsupported selectors, missing blocks,
unavailable historical state, malformed responses and transport failure. These diagnostics do not
change the fail-closed acceptance rule.

## Rejected alternatives

- Moving tags can resolve to different blocks across a multi-request build.
- Block numbers can become ambiguous across a reorganisation and are not uniformly more compatible
  across the tested endpoints.
- `requireCanonical: true` delegates a trust decision to the provider and reduced compatibility in
  the observed matrix. The independently authenticated light-client root provides the required
  canonicality guarantee.
- Silently switching providers or selectors during a build could mix state sources and hide a
  capability failure.

## Revisit condition

Reopen this decision if the selected execution client cannot accept EIP-1898 for `eth_getProof`, if
the light-client adapter cannot supply the same block hash and state root or if a later standard
provides a stronger proof response that includes an explicit block binding.
