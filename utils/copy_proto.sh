#!/usr/bin/env bash

set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cargo build -p exocore-common
cp $CUR_DIR/target/debug/build/exocore-common-*/out/proto/*_capnp.rs common/src/serialization/protos