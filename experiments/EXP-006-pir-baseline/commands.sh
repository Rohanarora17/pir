#!/bin/sh
set -eu
EXPERIMENT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_DIR=$(CDPATH= cd -- "$EXPERIMENT_DIR/../.." && pwd)
: "${OUTPUT_DIR:?Set OUTPUT_DIR to a new directory}"
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$PROJECT_DIR/target/pir-baseline}
export CARGO_TARGET_DIR
python3 -m unittest discover -s "$EXPERIMENT_DIR" -p 'test_prepare_source.py' -v
python3 "$EXPERIMENT_DIR/prepare_source.py"
python3 "$PROJECT_DIR/experiments/scripts/collect_corpus.py" audit \
  "$PROJECT_DIR/experiments/EXP-003-real-proof-corpus/retained"
cargo +stable test --locked --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml"
cargo +stable build --release --locked --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml"
python3 "$EXPERIMENT_DIR/run_cases.py" "$CARGO_TARGET_DIR/release/pvrpc-pir-baseline" \
  "$PROJECT_DIR/experiments/EXP-003-real-proof-corpus/retained" "$OUTPUT_DIR"
