#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

pushd $CUR_DIR/../exo
cargo install ${@:1} --path . --locked
popd
