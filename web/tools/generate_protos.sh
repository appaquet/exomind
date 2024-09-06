#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOMIND_ROOT=$CUR_DIR/../../
EXOMIND_WEB_ROOT="$EXOMIND_ROOT/web"

rm -rf $EXOMIND_WEB_ROOT/protos
mkdir -p $EXOMIND_WEB_ROOT/protos

# Generate protos
PROTOC_GEN_TS_PATH="$EXOMIND_WEB_ROOT/node_modules/.bin/protoc-gen-ts"
$EXOMIND_WEB_ROOT/node_modules/.bin/pbjs \
    -t static-module \
    -w es6 \
    --es6 \
    --sparse \
    -o $EXOMIND_WEB_ROOT/src/protos/index.js \
    -p "$EXOMIND_ROOT/protos/protobuf/" \
    --root 'exomind-root' \
    $EXOMIND_ROOT/protos/protobuf/exomind/*.proto

# Generate typescript definition for protos
$EXOMIND_WEB_ROOT/node_modules/.bin/pbts $EXOMIND_WEB_ROOT/src/protos/index.js -o $EXOMIND_WEB_ROOT/src/protos/index.d.ts
