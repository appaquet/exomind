#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../

# TODO: Remove this when we have foundation done
export RUSTFLAGS="-A dead_code -A unused_variables"

# Force check on whole workspace to prevent clippy run on dependencies
cargo check --all

# Then clean common, which will force clippy to run on all its dependencies
cargo clean -p exocore-common
cargo clippy --all -- -A clippy::new_ret_no_self
