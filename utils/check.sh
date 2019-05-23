#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR/../"

echo "Cargo checking code, tests and benches"
cargo check --all --all-features
cargo check --tests --all --all-features
cargo check --benches --all --all-features

echo "Checking wasm compilation for exocore-client-wasm"
cargo check -p exocore-client-wasm --target "wasm32-unknown-unknown"
