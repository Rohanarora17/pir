#!/bin/sh
set -eu

EXPERIMENT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_DIR=$(CDPATH= cd -- "$EXPERIMENT_DIR/../.." && pwd)
: "${OUTPUT_FILE:?Set OUTPUT_FILE to a new results file}"
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$PROJECT_DIR/target/verifier-comparison}
export CARGO_TARGET_DIR

python3 "$PROJECT_DIR/experiments/scripts/collect_corpus.py" audit \
  "$PROJECT_DIR/experiments/EXP-003-real-proof-corpus/retained"

# noclobber also refuses an existing result path, including an empty file.
set -C
cargo run --release --locked --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml" -- \
  "$PROJECT_DIR/experiments/EXP-003-real-proof-corpus/retained" > "$OUTPUT_FILE"
cat "$OUTPUT_FILE"

RECOVERY_OUTPUT_FILE=${RECOVERY_OUTPUT_FILE:-$OUTPUT_FILE.recovery.jsonl}
cargo run --release --locked --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml" --bin recover_account -- \
  "$PROJECT_DIR/experiments/EXP-003-real-proof-corpus/retained" > "$RECOVERY_OUTPUT_FILE"
cargo run --release --locked --manifest-path "$EXPERIMENT_DIR/probe/Cargo.toml" --bin recover_account -- \
  "$PROJECT_DIR/experiments/EXP-003-real-proof-corpus/retained" --reverse-proof-order >> "$RECOVERY_OUTPUT_FILE"
cat "$RECOVERY_OUTPUT_FILE"
