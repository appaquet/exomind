#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

EXOMIND_ROOT=$CUR_DIR/../

if [[ ! -d "$EXOCORE_REPO/protos" ]]; then
  echo "EXOCORE_REPO environment variable needs to be defined"
  exit 1
fi

# Generate protos
PROTOC_GEN_TS_PATH="./node_modules/.bin/protoc-gen-ts"
OUT_DIR="./proto"
./node_modules/.bin/pbjs \
    -t static-module \
    -w corejs \
    -o $CUR_DIR/js/proto.js \
    -p "$EXOCORE_REPO/protos/" \
    -p "$EXOMIND_ROOT/protos/" \
    --es6 \
    $EXOMIND_ROOT/protos/exomind/*.proto

# Generate typescript definition for protos
./node_modules/.bin/pbts $CUR_DIR/js/proto.js -o $CUR_DIR/js/proto.d.ts
