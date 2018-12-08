#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../

# TODO: Put back unused warnings once we have everything layed out
RUSTFLAGS="-A dead_code -A unused_variables" cargo +nightly clippy --all
