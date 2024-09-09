#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../

cargo clean -p exocore-protos
cargo clean -p exocore-discovery
cargo clean -p exomind-protos
cargo clippy --tests --workspace --all-features -- -D clippy::all # deny all warning to return error
