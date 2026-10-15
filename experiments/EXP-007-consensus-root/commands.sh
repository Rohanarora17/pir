#!/bin/sh
set -eu
cd "$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
export CARGO_HOME="${PVRPC_CARGO_HOME:-/private/tmp/pvrpc-verifier-cargo}"
export CARGO_TARGET_DIR="${PVRPC_CONSENSUS_TARGET:-/private/tmp/pvrpc-consensus-target}"
python3 -m unittest discover -s experiments/EXP-007-consensus-root -p 'test_prepare_source.py' -v
python3 experiments/EXP-007-consensus-root/prepare_source.py
cargo +stable test --locked --manifest-path experiments/EXP-007-consensus-root/probe/Cargo.toml
cargo +stable fmt --manifest-path experiments/EXP-007-consensus-root/probe/Cargo.toml -- --check
cargo +stable clippy --locked --manifest-path experiments/EXP-007-consensus-root/probe/Cargo.toml --all-targets -- -D warnings
cargo +stable run --locked --manifest-path experiments/EXP-007-consensus-root/probe/Cargo.toml --quiet
