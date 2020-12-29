#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../

cargo clean -p exocore-core
cargo clippy --tests --workspace --all-features -- -D clippy::all # deny all warning to return error
