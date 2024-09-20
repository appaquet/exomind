#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../


for i in $(seq 0 3); do
  cargo test --workspace --all-features
  RET=$?

  if [ $RET -eq 1 ]; then
    exit $RET
  fi
done
