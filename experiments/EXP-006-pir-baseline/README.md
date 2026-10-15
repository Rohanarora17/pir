# EXP-006: Direct index PIR versus keyword IKPIR

Questions: RQ-005 and RQ-006. Reproduce CPU client/server reads of the same real proof records
through direct index PIR and full two-segment keyword IKPIR. Both use pinned upstream engines,
FrodoPIR and SimplePIR, with default LWE dimensions and conservative 8-bit plaintext cells.
No backend is selected by creating this probe. Production protocol crates remain unchanged.

Hypothesis: direct indexing avoids keyword buckets/segments and may reduce setup and response
costs for the current immutable, publicly indexed corpus. Full keyword lookup may still be useful
for future datasets. We measure both before accepting a dependency or update model.

Run the four combinations of mode/engine at 3072 and 4096 record bytes in separate processes.
Each process imports and verifies EXP-003 records, prepares the actual database and hints, runs
all 32 private reads, decodes EXP-005 frames and verifies the recovered native proofs. Each
session has one outstanding query. No precomputed warm-query or decode queue is used.

Keyword mode uses 16 buckets, four slots per bucket (64 slots, 50% load), 64-bit fingerprints,
two segments and HintPatchClient frozen at its initial epoch. Every request queries both segments;
no key-dependent early stop is allowed. No live mutation or hint-delta optimization is measured.
Index mode uses the 32-row array and the locally resolved logical corpus index. Its map is public
and downloaded in full, not looked up using a wallet-specific server request.

Public setup is an in-process object at this stage. Setup bytes are upstream minimum canonical
size estimates, not a tested network serializer. Query/response vectors use our bounded experimental
framing and are actually encoded and decoded before the receiver invokes the backend. There is
no TCP/HTTP transport or real network latency. The adapter tag binds preprocessing parameters
and the snapshot. Data authenticity still depends on the verifier and a caller-supplied root;
the retained fixture root has no consensus authentication.

Measure server/client setup, each cold query construction, server answering, client PIR decoding,
frame/proof verification, request/response bytes, setup hint/public-state bytes, and process peak
RSS. Whole-dataset download includes the public manifest and index-map size. Byte and timing
measurements are not a proof of cryptographic query privacy.

The pinned source archive, Apache-2.0 LICENSE, NOTICE and upstream SECURITY policy are retained
under source/. Extraction is generated under ignored upstream/. Security parameters are not
reduced to make the experiment faster. The upstream prototype is unaudited and has unversioned
in-process bundles; this probe is not a production service.
