#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

# Generate protos with a specific 'root' to prevent clashing with generated users' protos 
PROTOC_GEN_TS_PATH="$CUR_DIR/../../node_modules/.bin/protoc-gen-ts"
$CUR_DIR/../../node_modules/.bin/pbjs \
    -t static-module \
    -w corejs \
    -o $CUR_DIR/js/protos.js \
    -p "$CUR_DIR/../../protos/" \
    --es6 \
    -r 'exocore-root' \
    $CUR_DIR/../../protos/exocore/index/*.proto \
    $CUR_DIR/../../protos/exocore/test/*.proto

# Generate typescript definition for protos
$CUR_DIR/../../node_modules/.bin/pbts $CUR_DIR/js/protos.js -o $CUR_DIR/js/protos.d.ts
