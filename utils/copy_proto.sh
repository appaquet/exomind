#!/usr/bin/env bash

set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cargo clean
cargo build -p exocore-common
cp $CUR_DIR/../target/debug/build/exocore-common-*/out/proto/*_capnp.rs $CUR_DIR/../common/src/serialization/protos