#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export GENERATE_PROTOS=1
cargo clean
cargo build -p exocore-common

for proto_path in `ls $CUR_DIR/../target/debug/build/exocore-common-*/out/proto/*_capnp.rs`; do
  proto_file="$(basename -- $proto_path)"
  dest_path="$CUR_DIR/../common/src/serialization/protos/$proto_file"
  echo "Copying $proto_file to $dest_path"

  echo "#![allow(unknown_lints)]" > $dest_path
  echo "#![allow(clippy::all)]" >> $dest_path
  echo "" >> $dest_path
  cat $proto_path >> $dest_path
done

cargo fmt --all
cargo test --all
