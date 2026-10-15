#!/usr/bin/env python3
"""Day 4 collection evidence. This does NOT verify MPT proofs or consensus finality."""

import argparse
from datetime import datetime, timezone
import hashlib
from http.client import HTTPException, IncompleteRead
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tempfile
import time
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import Request, urlopen


CONTRACT = "0xfff9976782d46cc05630d1f6ebab18b2324d6b14"
CHAIN_ID = 11155111
PROJECT = Path(__file__).resolve().parents[2]


class CorpusError(Exception):
    def __init__(self, message, kind="invalid_corpus"):
        super().__init__(message)
        self.kind = kind


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def read_json(path):
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError) as error:
        raise CorpusError(f"Cannot read JSON artifact: {path.name}") from error


def hex_bytes(value, size=None):
    if not isinstance(value, str) or not re.fullmatch(r"0x(?:[0-9a-fA-F]{2})+", value):
        raise CorpusError("Expected non-empty, whole-byte 0x hex data", "malformed_response")
    if size is not None and len(value) != 2 + size * 2:
        raise CorpusError(f"Expected {size} bytes", "malformed_response")
    return value.lower()


def quantity(value):
    if not isinstance(value, str) or not re.fullmatch(r"0x(?:0|[1-9a-fA-F][0-9a-fA-F]*)", value):
        raise CorpusError("Expected canonical unsigned RPC quantity", "malformed_response")
    number = int(value, 16)
    if number >= 2**256:
        raise CorpusError("RPC quantity exceeds 256 bits", "malformed_response")
    return number


def storage_position(value):
    # Providers encode proof keys either as fixed DATA32 or a minimal hex quantity.
    if isinstance(value, str) and len(value) == 66:
        return hex_bytes(value, 32)
    return "0x" + format(quantity(value), "064x")


def validate_addresses(addresses):
    if not isinstance(addresses, list) or not addresses:
        raise CorpusError("Address corpus must be a non-empty list")
    normalized = [hex_bytes(address, 20) for address in addresses]
    if len(set(normalized)) != len(normalized):
        raise CorpusError("Duplicate address in declared corpus")
    return normalized


def addresses_from_logs(logs, start, end, count):
    """Explicit bounded corpus; this does not enumerate or prove all token holders."""
    seeds = ["0xce54cf5a0de3843011cf20389c1b6a4aac442d6a",
             "0x3289680dd4d6c10bb19b899729cda5eef58aeff1"]
    if not isinstance(logs, list) or not (0 <= start <= end) or count < len(seeds):
        raise CorpusError("Invalid transfer-log corpus inputs")
    candidates = set()
    topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
    for log in logs:
        try:
            if (hex_bytes(log["address"], 20) != CONTRACT or log["removed"] is not False
                    or not start <= quantity(log["blockNumber"]) <= end
                    or len(log["topics"]) != 3 or log["topics"][0].lower() != topic):
                raise CorpusError("Invalid, removed or out-of-range transfer log")
            for item in log["topics"][1:]:
                padded = hex_bytes(item, 32)
                if padded[2:26] != "0" * 24:
                    raise CorpusError("Transfer address topic is not padded correctly")
                candidates.add("0x" + padded[-40:])
        except (KeyError, TypeError, AttributeError) as error:
            raise CorpusError("Incomplete transfer log") from error
    addresses = seeds + sorted(candidates.difference(seeds))[:count - len(seeds)]
    if len(addresses) != count:
        raise CorpusError("Not enough distinct addresses for declared corpus size")
    return validate_addresses(addresses)


def storage_key(address):
    address = hex_bytes(address, 20)
    try:
        result = subprocess.run(
            ["cast", "index", "address", address, "3"],
            check=True, capture_output=True, text=True, timeout=15,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise CorpusError("cast storage-key derivation failed", "local_dependency") from error
    return hex_bytes(result.stdout.strip(), 32)


def validate_proof(proof, expected_key):
    if not isinstance(proof, dict):
        raise CorpusError("Proof response is missing", "missing_result")
    try:
        if hex_bytes(proof["address"], 20) != CONTRACT:
            raise CorpusError("Proof names another contract", "account_mismatch")
        account = {
            "address": CONTRACT,
            "balance": hex(quantity(proof["balance"])),
            "nonce": hex(quantity(proof["nonce"])),
            "codeHash": hex_bytes(proof["codeHash"], 32),
            "storageHash": hex_bytes(proof["storageHash"], 32),
        }
        nodes = proof["accountProof"]
        storage = proof["storageProof"]
        if not isinstance(nodes, list) or not nodes:
            raise CorpusError("Missing account proof nodes", "malformed_response")
        account["accountProof"] = [hex_bytes(node) for node in nodes]
        if not isinstance(storage, list) or len(storage) != 1 or not isinstance(storage[0], dict):
            raise CorpusError("Expected exactly one storage proof", "malformed_response")
        entry = storage[0]
        if storage_position(entry["key"]) != hex_bytes(expected_key, 32):
            raise CorpusError("Proof names another storage key", "key_mismatch")
        value = quantity(entry["value"])
        nodes = entry["proof"]
        if not isinstance(nodes, list) or not nodes:
            raise CorpusError("Missing storage proof nodes", "malformed_response")
        nodes = [hex_bytes(node) for node in nodes]
    except (KeyError, TypeError) as error:
        raise CorpusError("Incomplete proof object", "malformed_response") from error
    return {
        "storage_key": expected_key,
        "value": "0x" + format(value, "064x"),
        "value_status": "reported_zero_unverified" if value == 0 else "reported_nonzero_unverified",
        "storage_proof_nodes": len(nodes),
        "storage_proof_bytes": sum((len(node) - 2) // 2 for node in nodes),
        "account": account,
    }


def unwrap(response, request_id):
    if not isinstance(response, dict) or response.get("jsonrpc") != "2.0":
        raise CorpusError("Invalid JSON-RPC envelope", "malformed_response")
    if type(response.get("id")) is not int or response["id"] != request_id:
        raise CorpusError("JSON-RPC response ID mismatch", "malformed_response")
    if "error" in response:
        error = response["error"]
        code = error.get("code") if isinstance(error, dict) else None
        message = str(error.get("message", "")) if isinstance(error, dict) else ""
        kind = "rpc_error"
        if code == -32602 or "unsupported" in message.lower():
            kind = "unsupported_selector"
        elif any(word in message.lower() for word in ("historical", "pruned", "state unavailable")):
            kind = "unavailable_state"
        elif "block" in message.lower() and any(word in message.lower() for word in ("missing", "not found")):
            kind = "missing_block"
        raise CorpusError(f"JSON-RPC failure ({code}): {message}", kind)
    if response.get("result") is None:
        raise CorpusError("RPC returned no result", "missing_result")
    return response["result"]


class RPC:
    def __init__(self, endpoint, raw):
        parsed = urlsplit(endpoint)
        if parsed.scheme not in ("http", "https") or not parsed.hostname:
            raise CorpusError("Expected an HTTP(S) RPC endpoint")
        if parsed.username or parsed.password or parsed.query or parsed.fragment:
            raise CorpusError("Credentialed/query URLs are not supported by this public experiment")
        self.endpoint = endpoint
        self.origin = f"{parsed.scheme}://{parsed.netloc}"
        self.raw = raw
        self.next_id = 1

    def read_body(self, response, label):
        try:
            return response.read()
        except IncompleteRead as error:
            (self.raw / f"{label}.response.json").write_bytes(error.partial)
            raise CorpusError("Truncated RPC response body", "transport_error") from error
        except (HTTPException, OSError) as error:
            raise CorpusError("Failed reading RPC response body", "transport_error") from error

    def call(self, label, method, params):
        request_id = self.next_id
        self.next_id += 1
        payload = {"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}
        write_json(self.raw / f"{label}.request.json", payload)
        request = Request(self.endpoint, data=json.dumps(payload).encode(),
                          headers={"Content-Type": "application/json", "User-Agent": "pvrpc-experiment/0.1"})
        try:
            with urlopen(request, timeout=30) as response:
                body = self.read_body(response, label)
        except HTTPError as error:
            with error:
                (self.raw / f"{label}.response.json").write_bytes(self.read_body(error, label))
            raise CorpusError(f"RPC HTTP status {error.code}", "http_error") from error
        except (URLError, TimeoutError, OSError, HTTPException) as error:
            raise CorpusError("RPC transport failed", "transport_error") from error
        (self.raw / f"{label}.response.json").write_bytes(body)
        try:
            response = json.loads(body)
        except ValueError as error:
            raise CorpusError("RPC returned invalid JSON", "malformed_response") from error
        return unwrap(response, request_id)


class Replay:
    """Re-run collection checks using retained request/response bytes, without RPC access."""
    def __init__(self, raw):
        self.raw = raw
        self.next_id = 1

    def call(self, label, method, params):
        request = read_json(self.raw / f"{label}.request.json")
        expected = {"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params}
        if request != expected:
            raise CorpusError(f"Retained request mismatch: {label}")
        self.next_id += 1
        return unwrap(read_json(self.raw / f"{label}.response.json"), request["id"])


def collect_records(rpc, addresses, block_hash):
    if quantity(rpc.call("chain-id", "eth_chainId", [])) != CHAIN_ID:
        raise CorpusError("Wrong chain ID", "chain_mismatch")
    client_version = rpc.call("client-version", "web3_clientVersion", [])
    if not isinstance(client_version, str) or not client_version:
        raise CorpusError("Missing provider client version", "malformed_response")
    header = rpc.call("header", "eth_getBlockByHash", [block_hash, False])
    finalized = rpc.call("finalized", "eth_getBlockByNumber", ["finalized", False])
    try:
        if hex_bytes(header["hash"], 32) != block_hash:
            raise CorpusError("Header does not match requested block hash", "snapshot_mismatch")
        root = hex_bytes(header["stateRoot"], 32)
        number = quantity(header["number"])
        final_number = quantity(finalized["number"])
        final_hash = hex_bytes(finalized["hash"], 32)
        final_root = hex_bytes(finalized["stateRoot"], 32)
        if final_hash == block_hash and (final_number != number or final_root != root):
            raise CorpusError("Conflicting fields for the same block hash", "snapshot_mismatch")
        if number > final_number or (number == final_number and final_hash != block_hash):
            raise CorpusError("Selected block exceeds or conflicts with provider finalized head")
    except (KeyError, TypeError) as error:
        raise CorpusError("Incomplete block header", "malformed_response") from error
    snapshot = {"chain_id": CHAIN_ID, "block_number": number, "block_hash": block_hash, "state_root": root}
    account = None
    records = []
    keys = set()
    for index, address in enumerate(addresses):
        key = storage_key(address)
        if key in keys:
            raise CorpusError("Duplicate derived key in address corpus")
        keys.add(key)
        label = f"proof-{index:04d}"
        proof = rpc.call(label, "eth_getProof", [CONTRACT, [key],
                         {"blockHash": block_hash, "requireCanonical": False}])
        record = validate_proof(proof, key)
        current_account = record.pop("account")
        if account is not None and current_account != account:
            raise CorpusError("Contract account changed within one snapshot", "account_mismatch")
        account = current_account
        records.append({"record_index": index, "address": address,
                        "proof_file": f"raw/{label}.response.json", **record})
    return {"snapshot": snapshot, "contract": CONTRACT, "mapping_slot": 3,
            "proof_verification": "not_performed", "consensus_authentication": "not_performed",
            "account": account, "records": records}, client_version


def command_version(command):
    try:
        return subprocess.check_output(command, text=True, stderr=subprocess.DEVNULL).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unavailable"


def collect(output, endpoint, addresses, block_hash):
    output = Path(output).absolute()
    if output.exists() or output.is_symlink():
        raise CorpusError("Refusing to overwrite an existing output path")
    addresses = validate_addresses(addresses)
    block_hash = hex_bytes(block_hash, 32)
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=f".{output.name}.building-", dir=output.parent))
    started = datetime.now(timezone.utc).isoformat()
    timer = time.monotonic()
    try:
        (stage / "raw").mkdir()
        write_json(stage / "addresses.json", addresses)
        rpc = RPC(endpoint, stage / "raw")
        corpus, client_version = collect_records(rpc, addresses, block_hash)
        write_json(stage / "corpus.json", corpus)
        zeros = sum(r["value_status"] == "reported_zero_unverified" for r in corpus["records"])
        manifest = {
            "experiment_id": "EXP-003", "research_questions": ["RQ-001", "RQ-002", "RQ-003", "RQ-004"],
            "claim_type": "local result",
            "software_revisions": {
                "project_base_revision": command_version(["git", "-C", str(PROJECT), "rev-parse", "HEAD"]),
                "project_tree_state": "dirty" if command_version([
                    "git", "-C", str(PROJECT), "status", "--porcelain"]) else "clean",
                "collector_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                "python": platform.python_version(), "cast": command_version(["cast", "--version"]),
                "provider_client_version": client_version, "operating_system": platform.platform(),
            },
            "machine": {"architecture": platform.machine(), "logical_cores": os.cpu_count(),
                        "cpu": command_version(["sysctl", "-n", "machdep.cpu.brand_string"]),
                        "memory_bytes": command_version(["sysctl", "-n", "hw.memsize"]), "gpu_used": False},
            "inputs": {**corpus["snapshot"], "contract_address": CONTRACT, "mapping_slot": 3,
                       "address_corpus_identifier": "EXP-003-declared-addresses-v1",
                       "address_corpus_file": "addresses.json", "rpc_endpoint_origin": rpc.origin,
                       "rpc_endpoint_environment_variable": "PVRPC_RPC_URL",
                       "finality_source": "provider-reported finalized head; unauthenticated"},
            "parameters": {"record_count": len(addresses), "proof_verification": "not_performed",
                           "consensus_authentication": "not_performed", "selector": "block_hash",
                           "require_canonical": False, "reported_zero_count": zeros,
                           "reported_nonzero_count": len(addresses) - zeros,
                           "collection_seconds": round(time.monotonic() - timer, 6)},
            "metrics": [{"name": "reported_value_class_and_proof_size", "unit": "records/nodes/bytes",
                         "raw_result_file": "corpus.json"}],
            "started_at": started, "completed_at": datetime.now(timezone.utc).isoformat(),
        }
        write_json(stage / "manifest.json", manifest)
        hashes = {str(p.relative_to(stage)): hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in sorted(stage.rglob("*")) if p.is_file()}
        write_json(stage / "checksums.json", hashes)
        audit(stage)
        # os.rename cannot replace a nonempty corpus. Fail also for an existing empty path.
        if output.exists() or output.is_symlink():
            raise CorpusError("Output appeared during collection; refusing overwrite")
        stage.rename(output)
        return manifest
    except Exception as error:
        write_json(stage / "failure.json", {"kind": getattr(error, "kind", "local_error"),
                   "message": str(error), "completed": False})
        # A failed result has no completed manifest, including an audit or publication failure.
        (stage / "manifest.json").unlink(missing_ok=True)
        failure = stage.with_name(stage.name.replace(".building-", ".failed-", 1))
        stage.rename(failure)
        if isinstance(error, CorpusError):
            raise
        raise CorpusError("Collection failed; inspect retained failure evidence", "local_error") from error


def audit(directory):
    directory = Path(directory)
    hashes = read_json(directory / "checksums.json")
    actual = {str(p.relative_to(directory)) for p in directory.rglob("*")
              if p.is_file() and p != directory / "checksums.json"}
    if not isinstance(hashes, dict) or set(hashes) != actual:
        raise CorpusError("Artifact set is incomplete or contains unexpected files")
    for name, expected in hashes.items():
        path = directory / name
        if path.is_symlink() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise CorpusError(f"Artifact checksum mismatch: {name}")
    manifest = read_json(directory / "manifest.json")
    addresses = validate_addresses(read_json(directory / "addresses.json"))
    try:
        block_hash = hex_bytes(manifest["inputs"]["block_hash"], 32)
        rebuilt, _ = collect_records(Replay(directory / "raw"), addresses, block_hash)
        if rebuilt != read_json(directory / "corpus.json"):
            raise CorpusError("Corpus summary differs from retained RPC evidence")
        if any(manifest["inputs"][k] != v for k, v in rebuilt["snapshot"].items()):
            raise CorpusError("Manifest snapshot differs from retained header")
        params = manifest["parameters"]
        zeros = sum(r["value_status"] == "reported_zero_unverified" for r in rebuilt["records"])
        if (params["record_count"] != len(addresses) or params["reported_zero_count"] != zeros
                or params["reported_nonzero_count"] != len(addresses) - zeros):
            raise CorpusError("Manifest counts differ from declared corpus")
        if (manifest["inputs"]["contract_address"] != CONTRACT
                or manifest["inputs"]["mapping_slot"] != 3
                or params["selector"] != "block_hash" or params["require_canonical"] is not False):
            raise CorpusError("Manifest disagrees with collection rules")
        if params["proof_verification"] != "not_performed" or params["consensus_authentication"] != "not_performed":
            raise CorpusError("Collection evidence cannot claim proof or consensus verification")
    except (KeyError, TypeError) as error:
        raise CorpusError("Incomplete corpus manifest") from error
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    collect_parser = commands.add_parser("collect")
    collect_parser.add_argument("--addresses", required=True, type=Path)
    collect_parser.add_argument("--block-hash", required=True)
    collect_parser.add_argument("--output", required=True, type=Path)
    audit_parser = commands.add_parser("audit")
    audit_parser.add_argument("directory", type=Path)
    address_parser = commands.add_parser("select-addresses")
    address_parser.add_argument("--logs", required=True, type=Path)
    address_parser.add_argument("--from-block", required=True, type=int)
    address_parser.add_argument("--to-block", required=True, type=int)
    address_parser.add_argument("--count", type=int, default=32)
    address_parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.command == "select-addresses":
            logs = read_json(args.logs)
            addresses = addresses_from_logs(unwrap(logs, 1), args.from_block, args.to_block, args.count)
            with args.output.open("x") as handle:
                handle.write(json.dumps(addresses, indent=2) + "\n")
            print(f"Selected {len(addresses)} declared addresses")
            return 0
        elif args.command == "audit":
            manifest = audit(args.directory)
        else:
            endpoint = os.environ.get("PVRPC_RPC_URL")
            if not endpoint:
                raise CorpusError("Set PVRPC_RPC_URL to a public RPC endpoint")
            manifest = collect(args.output, endpoint, read_json(args.addresses), args.block_hash)
        print(json.dumps({"command": args.command, "records": manifest["parameters"]["record_count"],
                          "block_hash": manifest["inputs"]["block_hash"],
                          "proof_verification": "not_performed", "consensus_authentication": "not_performed"}))
    except (CorpusError, OSError) as error:
        print(f"{getattr(error, 'kind', 'local_error')}: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
