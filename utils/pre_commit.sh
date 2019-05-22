#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

echo "Formatting..."
./format.sh

echo "Checking code..."
./check.sh

echo "Running clippy..."
./clippy.sh

echo "Running tests..."
cargo test --all --all-features

echo "Validating release build..."
cargo build --release
