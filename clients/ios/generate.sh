#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

cbindgen --config cbindgen.toml --crate exocore-client-ios --output exocore.h

protoc -I"$CUR_DIR/../../common/protos/" \
  --swift_opt=Visibility=Public \
  --swift_out=xcode/exocore-client-ios/exocore-client-ios/proto/ \
  $CUR_DIR/../../common/protos/exocore/index/*.proto \
  $CUR_DIR/../../common/protos/exocore/test/*.proto
