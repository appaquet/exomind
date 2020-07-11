#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

EXOMIND_ROOT=$CUR_DIR/../../
EXOMIND_WEB_ROOT="$EXOMIND_ROOT/web"

# Generate protos
PROTOC_GEN_TS_PATH="$EXOMIND_WEB_ROOT/node_modules/.bin/protoc-gen-ts"
OUT_DIR="./proto"
$EXOMIND_WEB_ROOT/node_modules/.bin/pbjs \
    -t static-module \
    -w corejs \
    --sparse \
    -o $EXOMIND_WEB_ROOT/src/protos/index.js \
    -p "$EXOMIND_ROOT/protos/" \
    -r 'exomind-root' \
    --es6 \
    $EXOMIND_ROOT/protos/exomind/*.proto

# Generate typescript definition for protos
$EXOMIND_WEB_ROOT/node_modules/.bin/pbts $EXOMIND_WEB_ROOT/src/protos/index.js -o $EXOMIND_WEB_ROOT/src/protos/index.d.ts
