#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

# Generate protos
PROTOC_GEN_TS_PATH="./node_modules/.bin/protoc-gen-ts"
OUT_DIR="./proto"
./node_modules/.bin/pbjs \
    -t static-module \
    -w corejs \
    -o $CUR_DIR/js/proto.js \
    -p "$CUR_DIR/../../protos/" \
    --es6 \
    $CUR_DIR/../../protos/exocore/index/*.proto \
    $CUR_DIR/../../protos/exocore/test/*.proto

# Generate typescript definition for protos
./node_modules/.bin/pbts $CUR_DIR/js/proto.js -o $CUR_DIR/js/proto.d.ts
