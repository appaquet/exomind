#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR/../"

echo "Cargo checking code, tests and benches"
cargo check --all
cargo check --tests --all
cargo check --benches --all

echo "Checking wasm compilation for exocore-client-wasm"
cargo check -p exocore-client-wasm --target "wasm32-unknown-unknown"
