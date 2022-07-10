#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOCORE_ROOT="$CUR_DIR/../../../"
EXOCORE_WEB_ROOT="$CUR_DIR/../"

rm -rf $EXOCORE_WEB_ROOT/protos
mkdir -p $EXOCORE_WEB_ROOT/protos

# Generate protos with a specific 'root' to prevent clashing with generated users' protos 
PROTOC_GEN_TS_PATH="$EXOCORE_WEB_ROOT/node_modules/.bin/protoc-gen-ts"
$EXOCORE_ROOT/node_modules/.bin/pbjs \
    -t static-module \
    --wrap es6 \
    --es6 \
    -o $EXOCORE_WEB_ROOT/protos/index.js \
    -p $EXOCORE_ROOT/protos/protobuf/ \
    --root 'exocore-root' \
    $EXOCORE_ROOT/protos/protobuf/exocore/core/*.proto \
    $EXOCORE_ROOT/protos/protobuf/exocore/store/*.proto \
    $EXOCORE_ROOT/protos/protobuf/exocore/test/*.proto

# Generate typescript definition for protos
$EXOCORE_ROOT/node_modules/.bin/pbts $EXOCORE_WEB_ROOT/protos/index.js -o $EXOCORE_WEB_ROOT/protos/index.d.ts
