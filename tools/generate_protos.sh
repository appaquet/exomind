#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "$EXOCORE_REPO" == "" ]]; then
  echo "EXOCORE_REPO envionment variable needs to be defined"
  exit 1
fi

protoc \
  -I"$CUR_DIR/../protos/" \
  -I"$EXOCORE_REPO/protos/" \
  $CUR_DIR/../protos/exomind/*.proto \
  -o "$CUR_DIR/../exomind.fd"
