#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

cbindgen --config cbindgen.toml --crate exocore-client-ios --output exocore.h

protoc -I"$CUR_DIR/../../protos/" \
  --swift_opt=Visibility=Public \
  --swift_out=xcode/exocore-client-ios/exocore-client-ios/proto/ \
  $CUR_DIR/../../protos/exocore/core/*.proto \
  $CUR_DIR/../../protos/exocore/index/*.proto \
  $CUR_DIR/../../protos/exocore/apps/*.proto \
  $CUR_DIR/../../protos/exocore/test/*.proto
