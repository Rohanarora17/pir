#!/usr/bin/env python3
"""Read-only retained-evidence audit; does not establish Ethereum checkpoint trust."""
import hashlib
import json
import pathlib
ROOT = pathlib.Path(__file__).resolve().parent
REPO = ROOT.parent.parent
checksums = json.loads((ROOT / 'checksums.json').read_text())
for name, digest in checksums['files'].items():
    path = (REPO / name).resolve()
    if not path.is_relative_to(REPO):
        raise SystemExit('checksum path leaves repository')
    if hashlib.sha256(path.read_bytes()).hexdigest() != digest:
        raise SystemExit(f'checksum mismatch: {name}')
d = json.loads((ROOT / 'raw/fixture-run.json').read_text())
assert d['claim_type'] == 'synthetic result' and d['updates_verified'] == 5
for key in ['live_sync', 'force_update_used', 'execution_rpc_used', 'storage_query_sent', 'old_sepolia_corpus_authenticated']:
    assert d[key] is False, key
assert [v['heads']['finalized_slot'] for v in d['trace']] == [24, 72, 72, 80, 80]
assert [v['heads']['optimistic_slot'] for v in d['trace']] == [40, 88, 96, 96, 96]
assert [v['admitted_context']['execution_block_number'] for v in d['trace']] == [2, 20, 20, 28, 28]
for step in d['trace']:
    context = step['admitted_context']
    assert context['network']['chain_id'] == 0
    assert context['execution_state_root'] == '0x' + '20' * 32
    assert context['beacon_slot'] == step['heads']['finalized_slot']
    assert context['checked_at_slot'] == step['current_slot']
    assert step['participants'] == step['committee_size'] == 32
log = (ROOT / 'raw/reproduction.log').read_text()
assert 'test result: ok. 10 passed; 0 failed' in log
assert 'Ran 4 tests' in log and '\nOK\n' in log
manifest = json.loads((ROOT / 'manifest.json').read_text())
assert manifest['claim_type'] == 'synthetic result'
assert manifest['validation']['rust_boundary_tests_passed'] == 10
print(f"verified {len(checksums['files'])} retained file digests and five synthetic consensus transitions")
