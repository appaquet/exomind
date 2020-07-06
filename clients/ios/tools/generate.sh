#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOCORE_IOS_ROOT="$CUR_DIR/../"
EXOCORE_ROOT="$EXOCORE_IOS_ROOT/../../"

pushd $EXOCORE_IOS_ROOT
cbindgen --config cbindgen.toml --crate exocore-client-ios --output swift/exocore.h
popd

pushd $EXOCORE_IOS_ROOT
rm -rf swift/protos
mkdir -p swift/protos
protoc -I "$EXOCORE_ROOT/protos/" \
  --swift_opt=Visibility=Public \
  --swift_out=swift/protos/ \
  $EXOCORE_ROOT/protos/exocore/core/*.proto \
  $EXOCORE_ROOT/protos/exocore/apps/*.proto \
  $EXOCORE_ROOT/protos/exocore/index/*.proto \
  $EXOCORE_ROOT/protos/exocore/test/*.proto
popd
