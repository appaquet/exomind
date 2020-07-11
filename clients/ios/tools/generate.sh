#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOCORE_IOS_ROOT="$CUR_DIR/../"
EXOCORE_ROOT="$EXOCORE_IOS_ROOT/../../"

cbindgen --config $EXOCORE_IOS_ROOT/cbindgen.toml --crate exocore-client-ios --output $EXOCORE_IOS_ROOT/swift/exocore.h

rm -rf $EXOCORE_IOS_ROOT/swift/protos
mkdir -p $EXOCORE_IOS_ROOT/swift/protos
protoc -I "$EXOCORE_ROOT/protos/" \
  --swift_opt=Visibility=Public \
  --swift_out=$EXOCORE_IOS_ROOT/swift/protos/ \
  $EXOCORE_ROOT/protos/exocore/core/*.proto \
  $EXOCORE_ROOT/protos/exocore/apps/*.proto \
  $EXOCORE_ROOT/protos/exocore/index/*.proto \
  $EXOCORE_ROOT/protos/exocore/test/*.proto
