#!/bin/sh
set -eu

EXPERIMENT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_DIR=$(CDPATH= cd -- "$EXPERIMENT_DIR/../.." && pwd)
: "${OUTPUT_FILE:?Set OUTPUT_FILE to a new output path}"
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$PROJECT_DIR/target/record-geometry}
export CARGO_TARGET_DIR

python3 "$PROJECT_DIR/experiments/scripts/collect_corpus.py" audit \
  "$PROJECT_DIR/experiments/EXP-003-real-proof-corpus/retained"
cargo +stable test --locked --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml"
set -C
cargo +stable run --release --locked --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml" -- \
  "$PROJECT_DIR/experiments/EXP-003-real-proof-corpus/retained" > "$OUTPUT_FILE"
cat "$OUTPUT_FILE"
