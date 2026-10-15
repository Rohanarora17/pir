#!/bin/sh
set -eu

EXPERIMENT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_DIR=$(CDPATH= cd -- "$EXPERIMENT_DIR/../.." && pwd)
: "${OUTPUT_DIR:?Set OUTPUT_DIR to a new replay directory}"
PVRPC_RPC_URL=${PVRPC_RPC_URL:-https://ethereum-sepolia-rpc.publicnode.com}
export PVRPC_RPC_URL

python3 - "$PROJECT_DIR" "$EXPERIMENT_DIR" <<'PY'
import importlib.util
import json
from pathlib import Path
import sys

project, experiment = map(Path, sys.argv[1:])
spec = importlib.util.spec_from_file_location("collector", project / "experiments/scripts/collect_corpus.py")
collector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(collector)
logs = json.loads((experiment / "source/transfer-logs.response.json").read_text())["result"]
addresses = collector.addresses_from_logs(logs, 11917601, 11919648, 32)
if addresses != json.loads((experiment / "addresses.json").read_text()):
    raise SystemExit("Declared addresses differ from deterministic source-log selection")
print("Declared 32-address list reproduced from retained transfer logs")
PY

python3 "$PROJECT_DIR/experiments/scripts/collect_corpus.py" collect \
  --addresses "$EXPERIMENT_DIR/addresses.json" \
  --block-hash 0xb5a539c8976e51dd412a0e6d1e7fec573b260cfddcfa8ae13cf006df5f787a5e \
  --output "$OUTPUT_DIR"
python3 "$PROJECT_DIR/experiments/scripts/collect_corpus.py" audit "$OUTPUT_DIR"
