#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

echo "Formatting..."
./format.sh

echo "Cargo checking code, tests and benches"
cargo check --all --all-features
cargo check --tests --all --all-features
cargo check --benches --all --all-features

echo "Running tests..."
cargo test --all --all-features

echo "Running clippy..."
./clippy.sh

echo "Validating wasm compilation for exocore-client-wasm"
cargo clippy -p exocore-client-wasm --target "wasm32-unknown-unknown"

echo "Validating cli compilation"
cargo check -p exocore-cli

echo "Validating release build..."
cargo build --release
