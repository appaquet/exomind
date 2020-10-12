#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOMIND_ROOT="$CUR_DIR/.."

if [[ "$EXOCORE_REPO" == "" ]]; then
  if [[ -d "$EXOMIND_ROOT/../exocore" ]]; then
    EXOCORE_REPO="$EXOMIND_ROOT/../exocore"
  fi
fi

# Update exocore protos to latest
if [[ "$EXOCORE_REPO" != "" ]]; then
  rm -rf $EXOMIND_ROOT/protos/exocore
  cp -r $EXOCORE_REPO/protos/exocore $EXOMIND_ROOT/protos/
fi

# Generate exomind app file descriptor
protoc \
  -I"$EXOMIND_ROOT/protos/" \
  -I"$EXOCORE_REPO/protos/" \
  $EXOMIND_ROOT/protos/exomind/*.proto \
  -o "$EXOMIND_ROOT/exomind.fd"

# Validate Prost protos
cd $EXOMIND_ROOT
cargo clean -p exomind
cargo test --all

# Generate web protos
$EXOMIND_ROOT/web/tools/generate_protos.sh
$EXOMIND_ROOT/browsers/tools/generate_protos.sh
