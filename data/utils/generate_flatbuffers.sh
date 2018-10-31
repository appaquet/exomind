#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd $CUR_DIR/../

flatc --size-prefixed --gen-mutable -o src/chain -r fbs/chain_schema.fbs
