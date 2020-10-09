#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

pushd $CUR_DIR/../exo
cargo install ${@:1} --path . --locked
popd
