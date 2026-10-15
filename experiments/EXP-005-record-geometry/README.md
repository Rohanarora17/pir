# EXP-005: Proof size and candidate record geometry

Question: RQ-004. How much space does the retained real corpus need when each private retrieval
returns a complete storage proof, with the shared account proof in the public snapshot manifest?

Hypothesis: binary nodes and explicit lengths fit a smaller padded row than current hex JSON.
This is an isolated experimental codec, not a production format or an accepted representation ADR.
No PIR backend or final size is selected. Public metadata is fetched independently of the wallet.

Measure decoded node bytes, compact JSON record bytes, binary frame bytes, unique nodes, duplicated
bytes and padding at 2048/2560/3072/4096/8192 bytes. Independently derive requested mapping keys,
round-trip every supported row and verify with pvrpc-verifier. Record explicit rejection of rows
larger than a chosen capacity. Deduplicated storage is only a lower-bound comparison; a private
trie-node retrieval protocol, index, padding and query budget are not implemented.

Candidate: four magic bytes `PVR0`, a big-endian u32 payload length, then the existing record fields
in order: schema u32, snapshot (chain u64, block u64, block hash32, root32, generation u64), index
u64, storage key32, value32, proof count u16, then each node as u32 length plus bytes. All integers
are big-endian. Append zero bytes to the caller's exact record size. Decoder requires that exact
frame length, valid schema, bounded proof counts/lengths, complete payload and zero padding.
The metadata header is 174 bytes including the frame header; each proof node adds four length bytes.
The account proof remains in the manifest. Authentication is provided by the existing verifier,
not by frame parsing or zero padding. The frame is not a final wire protocol.

The real run uses EXP-003 only, without RPC. Synthetic malformed frames are unit-test controls,
not Ethereum measurements. The supplied execution root is not authenticated by consensus.

Run with a new output filename (existing evidence is never overwritten):

```bash
OUTPUT_FILE=/private/tmp/EXP-005-replay.json sh experiments/EXP-005-record-geometry/commands.sh
```

The runner also executes the codec tests. For the existing sandbox cache, set
`CARGO_HOME=/private/tmp/pvrpc-verifier-cargo CARGO_TARGET_DIR=/private/tmp/pvrpc-geometry-target`.
