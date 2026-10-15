#!/bin/sh
set -eu
EXPERIMENT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
: "${OUTPUT_DIR:?Set OUTPUT_DIR to a new directory}"
: "${WORKER:?Set WORKER to the prepared EXP-008 release binary}"
python3 -m unittest discover -s "$EXPERIMENT_DIR" -p 'test_*.py' -v
python3 "$EXPERIMENT_DIR/guarded_run.py" "$WORKER" "$OUTPUT_DIR"
python3 "$EXPERIMENT_DIR/check_evidence.py" "$OUTPUT_DIR"
