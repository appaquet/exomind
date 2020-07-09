#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

pushd $CUR_DIR/..
cargo install ${@:1} --path exo --locked
popd
