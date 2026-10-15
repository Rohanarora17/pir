#!/bin/sh
set -eu
EXPERIMENT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_DIR=$(CDPATH= cd -- "$EXPERIMENT_DIR/../.." && pwd)
: "${OUTPUT_DIR:?Set OUTPUT_DIR to a new directory}"
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$PROJECT_DIR/target/pir-scale}
export CARGO_TARGET_DIR
python3 "$PROJECT_DIR/experiments/EXP-006-pir-baseline/prepare_source.py"
python3 -m unittest discover -s "$EXPERIMENT_DIR" -p 'test_*.py' -v
cargo +stable test --locked --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml"
cargo +stable fmt --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml" -- --check
cargo +stable clippy --locked --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml" --all-targets -- -D warnings
cargo +stable build --release --locked --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml"
python3 "$EXPERIMENT_DIR/run_cases.py" "$CARGO_TARGET_DIR/release/pvrpc-pir-scale" "$OUTPUT_DIR"
python3 "$EXPERIMENT_DIR/summarize.py" "$OUTPUT_DIR"
