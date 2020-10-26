#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOCORE_C_ROOT="$CUR_DIR/../"
EXOCORE_ROOT="$EXOCORE_C_ROOT/../../"

cd $EXOCORE_C_ROOT
cbindgen --config $EXOCORE_C_ROOT/cbindgen.toml --crate exocore-client-c --output $EXOCORE_C_ROOT/exocore.h
