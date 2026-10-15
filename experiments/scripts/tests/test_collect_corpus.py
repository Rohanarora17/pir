"""Collector contract tests. Real RPC transport is exercised against a local server."""

import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from threading import Thread


PROJECT = Path(__file__).resolve().parents[3]
SCRIPT = PROJECT / "experiments/scripts/collect_corpus.py"
FIXTURES = PROJECT / "experiments/EXP-001-sepolia-weth-workload/raw"
CONTRACT = "0xfff9976782d46cc05630d1f6ebab18b2324d6b14"
BLOCK = "0x9d11a05a418308f20ee690d5700d04bb23893b168a089389b89975cef3eb14e2"
ROOT = "0x7443069b0b51afdd2c7e2d8e54723a3b4fb66e8e200cdc3e96ff314589b0d5e2"
USER = "0xce54cf5a0de3843011cf20389c1b6a4aac442d6a"
HOLDER = "0x3289680dd4d6c10bb19b899729cda5eef58aeff1"
USER_KEY = "0x88080a0f7453a7e6f91b5d6e4f0c47e15d7639174219416b4e29a716c14f0857"
HOLDER_KEY = "0x621ed32f145b61c99f199a418095340a8de179d06aead00e8739fc7ce9e27b2d"


def fixture(name):
    return json.loads((FIXTURES / name).read_text())["result"]


class CollectorTests(unittest.TestCase):
    def setUp(self):
        if not SCRIPT.exists():
            self.fail("Day 4 collector is not implemented")
        spec = importlib.util.spec_from_file_location("collect_corpus", SCRIPT)
        self.collector = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.collector)
        self.user_proof = fixture("user-proof.json")
        self.holder_proof = fixture("holder-proof.json")

    def test_derives_the_actual_weth_mapping_keys(self):
        self.assertEqual(self.collector.storage_key(USER), USER_KEY)
        self.assertEqual(self.collector.storage_key(HOLDER), HOLDER_KEY)

    def test_accepts_real_zero_as_unverified_and_never_as_authenticated_absence(self):
        result = self.collector.validate_proof(self.user_proof, USER_KEY)
        self.assertEqual(result["value"], "0x" + "00" * 32)
        self.assertEqual(result["value_status"], "reported_zero_unverified")

    def test_accepts_real_nonzero_proof_and_normalizes_quantity(self):
        result = self.collector.validate_proof(self.holder_proof, HOLDER_KEY)
        self.assertEqual(result["value"], "0x00000000000000000000000000000000000000000000000559e4c8956ca62d47")
        self.assertEqual(result["storage_proof_nodes"], 6)

    def test_rejects_incomplete_or_substituted_proof_objects(self):
        # Every mutation removes a check needed to stop a bad response entering the corpus.
        mutations = [
            ("missing_account", lambda p: p.pop("accountProof")),
            ("empty_account", lambda p: p.update(accountProof=[])),
            ("empty_storage", lambda p: p["storageProof"][0].update(proof=[])),
            ("odd_hex_node", lambda p: p["storageProof"][0].update(proof=["0xabc"])),
            ("wrong_account", lambda p: p.update(address=USER)),
            ("wrong_key", lambda p: p["storageProof"][0].update(key=USER_KEY)),
            ("missing_storage", lambda p: p.update(storageProof=[])),
            ("extra_storage", lambda p: p["storageProof"].append(copy.deepcopy(p["storageProof"][0]))),
            ("oversized_value", lambda p: p["storageProof"][0].update(value="0x1" + "00" * 32)),
            ("negative_value", lambda p: p["storageProof"][0].update(value="-0x1")),
            ("bad_storage_root", lambda p: p.update(storageHash="0x12")),
            ("missing_nonce", lambda p: p.pop("nonce")),
        ]
        for name, change in mutations:
            with self.subTest(name=name):
                proof = copy.deepcopy(self.holder_proof)
                change(proof)
                with self.assertRaises(self.collector.CorpusError):
                    self.collector.validate_proof(proof, HOLDER_KEY)

    def test_rejects_empty_duplicate_or_invalid_address_lists(self):
        for addresses in [[], [USER, USER.upper().replace("0X", "0x")], ["0x1234"], [False]]:
            with self.subTest(addresses=addresses):
                with self.assertRaises(self.collector.CorpusError):
                    self.collector.validate_addresses(addresses)

    def test_selects_a_deterministic_declared_corpus_from_real_transfer_logs(self):
        logs = fixture("transfer-logs.json")
        selected = self.collector.addresses_from_logs(logs, 11848925, 11848974, 4)
        self.assertEqual(selected, [USER, HOLDER,
            "0x3bfa4769fb09eefc5a80d6e87c3b9c650f7ae48e",
            "0x3e48a6b1f3b7bc7e0d7b4d10bb9224aefca86aba"])
        self.assertEqual(selected, self.collector.addresses_from_logs(list(reversed(logs)), 11848925, 11848974, 4))

    def test_rejects_removed_out_of_range_or_wrong_contract_logs(self):
        for field, value in [("removed", True), ("blockNumber", "0x1"), ("address", USER)]:
            with self.subTest(field=field):
                logs = fixture("transfer-logs.json")
                logs[0][field] = value
                with self.assertRaises(self.collector.CorpusError):
                    self.collector.addresses_from_logs(logs, 11848925, 11848974, 4)

    def run_collection(self, directory, mutation=None, reject_default_agent=False, truncate_status=None):
        requests = []
        proofs = {USER_KEY: self.user_proof, HOLDER_KEY: self.holder_proof}
        header = fixture("pinned-block.json")

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_POST(self):
                incoming = self.rfile.read(int(self.headers["Content-Length"]))
                if truncate_status is not None:
                    self.send_response(truncate_status)
                    self.send_header("Content-Length", "100")
                    self.end_headers()
                    self.wfile.write(b"partial-rpc-body")
                    self.close_connection = True
                    return
                if reject_default_agent and self.headers.get("User-Agent", "").startswith("Python-urllib/"):
                    self.send_error(403, "Default urllib identity rejected")
                    return
                request = json.loads(incoming)
                requests.append(request)
                method = request["method"]
                if method == "eth_chainId":
                    result = "0xaa36a7"
                elif method == "web3_clientVersion":
                    result = "fixture-server"
                elif method in ["eth_getBlockByHash", "eth_getBlockByNumber"]:
                    result = copy.deepcopy(header)
                elif method == "eth_getProof":
                    result = copy.deepcopy(proofs[request["params"][1][0]])
                else:
                    raise AssertionError(f"Unexpected RPC method: {method}")
                response = {"jsonrpc": "2.0", "id": request["id"], "result": result}
                if mutation:
                    mutation(request, response)
                body = json.dumps(response).encode()
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            output = Path(directory) / "corpus"
            self.collector.collect(
                output, f"http://127.0.0.1:{server.server_port}", [USER, HOLDER], BLOCK
            )
            return output, requests
        finally:
            server.shutdown()
            server.server_close()
            thread.join()

    def test_collects_exactly_the_declared_corpus_and_audits_it_offline(self):
        with tempfile.TemporaryDirectory() as directory:
            output, requests = self.run_collection(directory)
            manifest = self.collector.audit(output)
            self.assertEqual(manifest["inputs"]["state_root"], ROOT)
            self.assertEqual(manifest["parameters"]["record_count"], 2)
            self.assertEqual(manifest["parameters"]["proof_verification"], "not_performed")
            records = json.loads((output / "corpus.json").read_text())["records"]
            self.assertEqual(len(records), 2)
            self.assertEqual({r["storage_key"] for r in records}, {USER_KEY, HOLDER_KEY})
            proof_requests = [r for r in requests if r["method"] == "eth_getProof"]
            self.assertEqual(len(proof_requests), 2)
            self.assertTrue(all(r["params"][2] == {"blockHash": BLOCK, "requireCanonical": False}
                                for r in proof_requests))

    def test_uses_an_explicit_client_identity_accepted_by_the_rpc_endpoint(self):
        with tempfile.TemporaryDirectory() as directory:
            output, _ = self.run_collection(directory, reject_default_agent=True)
            self.assertEqual(self.collector.audit(output)["parameters"]["record_count"], 2)

    def test_failed_second_record_never_produces_a_complete_corpus_or_fallback(self):
        for failure in ["null", "rpc_error", "wrong_id", "wrong_key", "different_account"]:
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as directory:
                seen = []

                def mutate(request, response):
                    seen.append(request)
                    if request["method"] != "eth_getProof" or request["params"][1] != [HOLDER_KEY]:
                        return
                    if failure == "null":
                        response["result"] = None
                    elif failure == "rpc_error":
                        response.pop("result")
                        response["error"] = {"code": -32000, "message": "historical state unavailable"}
                    elif failure == "wrong_id":
                        response["id"] = 99
                    elif failure == "wrong_key":
                        response["result"]["storageProof"][0]["key"] = USER_KEY
                    else:
                        response["result"]["storageHash"] = "0x" + "11" * 32

                with self.assertRaises(self.collector.CorpusError):
                    self.run_collection(directory, mutate)
                self.assertFalse((Path(directory) / "corpus").exists())
                failures = list(Path(directory).glob(".corpus.failed-*"))
                self.assertEqual(len(failures), 1)
                self.assertTrue((failures[0] / "failure.json").exists())
                self.assertFalse((failures[0] / "manifest.json").exists())
                self.assertEqual(len([r for r in seen if r["method"] == "eth_getProof"]), 2)

    def test_rejects_header_mismatch_before_collecting_proofs(self):
        def mutate(request, response):
            if request["method"] == "eth_getBlockByHash":
                response["result"]["hash"] = "0x" + "22" * 32

        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(self.collector.CorpusError):
                self.run_collection(directory, mutate)

    def test_rejects_wrong_chain_and_unfinalized_or_conflicting_header(self):
        for failure in ["chain", "future", "conflict"]:
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as directory:
                def mutate(request, response):
                    if failure == "chain" and request["method"] == "eth_chainId":
                        response["result"] = "0x1"
                    elif request["method"] == "eth_getBlockByNumber":
                        if failure == "future":
                            response["result"]["number"] = "0x1"
                        elif failure == "conflict":
                            response["result"]["hash"] = "0x" + "11" * 32
                with self.assertRaises(self.collector.CorpusError):
                    self.run_collection(directory, mutate)

    def test_rejects_different_fields_for_the_same_header_hash(self):
        for field, value in [("stateRoot", "0x" + "11" * 32), ("number", "0xb4cd0f")]:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                def mutate(request, response):
                    if request["method"] == "eth_getBlockByNumber":
                        response["result"][field] = value
                with self.assertRaises(self.collector.CorpusError):
                    self.run_collection(directory, mutate)

    def test_retains_partial_success_and_error_bodies_as_transport_failures(self):
        for status in [200, 500]:
            with self.subTest(status=status), tempfile.TemporaryDirectory() as directory:
                with self.assertRaises(self.collector.CorpusError):
                    self.run_collection(directory, truncate_status=status)
                failed = next(Path(directory).glob(".corpus.failed-*"))
                self.assertEqual(json.loads((failed / "failure.json").read_text())["kind"], "transport_error")
                self.assertEqual((failed / "raw/chain-id.response.json").read_bytes(), b"partial-rpc-body")
                self.assertFalse((Path(directory) / "corpus").exists())

    def test_rpc_rejects_errors_null_and_invalid_json_envelopes(self):
        for response in [None, [], {}, {"jsonrpc":"2.0", "id":True, "result":1},
                         {"jsonrpc":"2.0", "id":1, "result":None},
                         {"jsonrpc":"2.0", "id":1, "result":1, "error":{}}]:
            with self.subTest(response=response):
                with self.assertRaises(self.collector.CorpusError):
                    self.collector.unwrap(response, 1)

    def test_offline_audit_rejects_forged_counts_even_with_updated_file_hashes(self):
        with tempfile.TemporaryDirectory() as directory:
            output, _ = self.run_collection(directory)
            manifest_path = output / "manifest.json"
            manifest = json.loads(manifest_path.read_text())
            manifest["parameters"]["reported_nonzero_count"] = 999
            manifest_path.write_text(json.dumps(manifest))
            checksums_path = output / "checksums.json"
            checksums = json.loads(checksums_path.read_text())
            checksums["manifest.json"] = hashlib.sha256(manifest_path.read_bytes()).hexdigest()
            checksums_path.write_text(json.dumps(checksums))
            with self.assertRaises(self.collector.CorpusError):
                self.collector.audit(output)

    def test_refuses_overwriting_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            output, _ = self.run_collection(directory)
            before = (output / "manifest.json").read_bytes()
            with self.assertRaises(self.collector.CorpusError):
                self.collector.collect(output, "http://127.0.0.1:1", [USER], BLOCK)
            self.assertEqual((output / "manifest.json").read_bytes(), before)

    def test_offline_audit_rejects_missing_or_modified_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            output, _ = self.run_collection(directory)
            proof = next((output / "raw").glob("proof-*.response.json"))
            proof.write_text("{}")
            with self.assertRaises(self.collector.CorpusError):
                self.collector.audit(output)


if __name__ == "__main__":
    unittest.main()
