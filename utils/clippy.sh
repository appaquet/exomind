#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../

cargo clean -p exocore-common
RUSTFLAGS="-A dead_code -A unused_variables" cargo clippy --all -- -A clippy::new_ret_no_self
