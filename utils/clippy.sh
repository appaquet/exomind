#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../

# Force check on whole workspace to prevent clippy run on dependencies
cargo check --tests --all

# Then clean common, which will force clippy to run on all its dependencies
cargo clean -p exocore-common
cargo clippy --tests --all
