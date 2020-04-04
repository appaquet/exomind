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

echo "Validating web compilation for exocore-client-web"
pushd $CUR_DIR/../clients/web
cargo clippy --target "wasm32-unknown-unknown"
popd

echo "Validating exo compilation"
pushd $CUR_DIR/../exo
cargo check
popd

echo "Validating release build..."
cargo build --release
