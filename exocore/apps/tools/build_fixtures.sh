#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

APPS_DIR="$CUR_DIR/../"
REPO_ROOT="$APPS_DIR/../../"

pushd $APPS_DIR/example/
cargo build --target wasm32-wasip1 --release
cp $REPO_ROOT/target/wasm32-wasip1/release/exocore_apps_example.wasm $APPS_DIR/host/fixtures/example.wasm
