#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

APP_DIR="$CUR_DIR/../"
EXOMIND_ROOT="$APP_DIR/../"

$EXOMIND_ROOT/tools/generate_protos.sh

pushd $APP_DIR
cargo build --target wasm32-unknown-unknown --release
cp $EXOMIND_ROOT/target/wasm32-unknown-unknown/release/exomind_app.wasm $EXOMIND_ROOT/app.wasm
popd

pushd $EXOMIND_ROOT
exo app package
popd