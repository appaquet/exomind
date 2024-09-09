#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export EXOCORE_ROOT="$CUR_DIR/../"
export REPO_ROOT="$EXOCORE_ROOT/../"

echo "Validating web compilation for exocore-client-web"
cd $EXOCORE_ROOT/clients/web
cargo clippy --target "wasm32-unknown-unknown"

echo "Validating exo compilation"
cd $EXOCORE_ROOT/exo
cargo check
