#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOMIND_IOS_ROOT="$CUR_DIR/../"
EXOMIND_ROOT="$EXOMIND_IOS_ROOT/../"

rm -rf $EXOMIND_IOS_ROOT/Exomind/protos
mkdir -p $EXOMIND_IOS_ROOT/Exomind/protos
protoc -I "$EXOMIND_ROOT/protos/protobuf/" \
  --swift_opt=ProtoPathModuleMappings=$EXOMIND_IOS_ROOT/protobuf_mapping.asciipb \
  --swift_out=$EXOMIND_IOS_ROOT/Exomind/protos/ \
  $EXOMIND_ROOT/protos/protobuf/exomind/*.proto 
