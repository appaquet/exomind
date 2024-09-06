#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

EXOMIND_ROOT="$CUR_DIR/.."

echo "Formatting..."
$CUR_DIR/format.sh

echo "Cargo checking code, tests and benches"
cd $EXOMIND_ROOT
cargo check --workspace --tests --benches --all-features

echo "Running tests..."
cd $EXOMIND_ROOT
cargo test --workspace --all-features

echo "Validating wasm app..."
$EXOMIND_ROOT/app/tools/build.sh

echo "Running clippy..."
$CUR_DIR/clippy.sh

echo "Linting web..."
cd $EXOMIND_ROOT/web
yarn lint