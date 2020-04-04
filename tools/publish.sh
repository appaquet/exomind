#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd $CUR_DIR/../

CRATES=("core" "transport" "chain" "index"  "." "exo")

for CRATE in "${CRATES[@]}"; do
  pushd $CUR_DIR/../$CRATE
  cargo publish ${@:1}
  popd
done
