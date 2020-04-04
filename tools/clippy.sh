#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../

# Force check on whole workspace to prevent clippy run on dependencies
cargo check --tests --all

# Which will force clippy to run on all its dependencies
cargo clean  \
    -p exo \
    -p exocore-index \
    -p exocore-data \
    -p exocore-core \
    -p exocore-transport \
    -p exocore-client-web \
    -p exocore-client-android \
    -p exocore-client-ios

cargo clippy --tests --all -- -D clippy::all # deny all warning to return error
