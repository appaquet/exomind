#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

echo "Formatting..."
./format.sh

echo "Cargo checking code, tests and benches"
cargo check --workspace --tests --benches --all-features

echo "Running tests..."
cargo test --workspace --all-features

echo "Running clippy..."
./clippy.sh