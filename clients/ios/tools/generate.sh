#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOCORE_IOS_ROOT="$CUR_DIR/../"
EXOCORE_ROOT="$EXOCORE_IOS_ROOT/../../"
EXOCORE_C_ROOT="$EXOCORE_ROOT/clients/c"

$EXOCORE_C_ROOT/tools/generate.sh
cp $EXOCORE_C_ROOT/exocore.h $EXOCORE_IOS_ROOT/swift/exocore.h

rm -rf $EXOCORE_IOS_ROOT/swift/protos
mkdir -p $EXOCORE_IOS_ROOT/swift/protos
protoc -I "$EXOCORE_ROOT/protos/protobuf/" \
  --swift_opt=Visibility=Public \
  --swift_out=$EXOCORE_IOS_ROOT/swift/protos/ \
  $EXOCORE_ROOT/protos/protobuf/exocore/core/*.proto \
  $EXOCORE_ROOT/protos/protobuf/exocore/apps/*.proto \
  $EXOCORE_ROOT/protos/protobuf/exocore/store/*.proto \
  $EXOCORE_ROOT/protos/protobuf/exocore/test/*.proto
