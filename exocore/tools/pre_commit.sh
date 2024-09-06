#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export EXOCORE_ROOT="$CUR_DIR/../"

echo "Formatting..."
$CUR_DIR/format.sh

echo "Cargo checking code, tests and benches"
cd $EXOCORE_ROOT
cargo check --workspace --tests --benches --all-features

echo "Running tests..."
cd $EXOCORE_ROOT
cargo test --workspace --all-features

echo "Running clippy..."
$CUR_DIR/clippy.sh

echo "Validating web compilation for exocore-client-web"
cd $EXOCORE_ROOT/clients/web
cargo clippy --target "wasm32-unknown-unknown"

echo "Validating exo compilation"
cd $EXOCORE_ROOT/exo
cargo check
