#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOMIND_ROOT=$CUR_DIR/../../
EXOMIND_WEB_ROOT="$EXOMIND_ROOT/web"
EXOMIND_BROWSER_ROOT="$EXOMIND_ROOT/browsers"

# Generate protos
PROTOC_GEN_TS_PATH="$EXOMIND_WEB_ROOT/node_modules/.bin/protoc-gen-ts"
$EXOMIND_WEB_ROOT/node_modules/.bin/pbjs \
    -t static-module \
    -w closure \
    -o $EXOMIND_BROWSER_ROOT/chrome/protos.js \
    -p "$EXOMIND_ROOT/protos/protobuf/" \
    -r 'exomind-root' \
    $EXOMIND_ROOT/protos/protobuf/exomind/base.proto \
    $EXOMIND_ROOT/protos/protobuf/exocore/store/mutation.proto