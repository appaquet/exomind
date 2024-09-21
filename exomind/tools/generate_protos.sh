#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOMIND_ROOT="$CUR_DIR/.."
REPO_ROOT="$EXOMIND_ROOT/.."
EXOCORE_ROOT="$REPO_ROOT/exocore"

# Update exocore protos to latest
rm -rf $EXOMIND_ROOT/protos/protobuf/exocore
cp -r $EXOCORE_ROOT/protos/protobuf/exocore $EXOMIND_ROOT/protos/protobuf/

# Generate exomind app file descriptor
protoc \
  -I"$EXOMIND_ROOT/protos/protobuf/" \
  -I"$EXOCORE_ROOT/protos/protobuf/" \
  $EXOMIND_ROOT/protos/protobuf/exomind/*.proto \
  -o "$EXOMIND_ROOT/exomind.fd"

# Validate Prost protos
cd $EXOMIND_ROOT
export GENERATE_PROTOS=1
cargo clean -p exomind
cargo test --all

# Generate web protos
$EXOMIND_ROOT/web/tools/generate_protos.sh
$EXOMIND_ROOT/browsers/tools/generate_protos.sh
