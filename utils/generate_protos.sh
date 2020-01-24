#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export GENERATE_PROTOS=1
cargo clean -p exocore-common
cargo build -p exocore-common

# Capnp files
for proto_path in `ls $CUR_DIR/../target/debug/build/exocore-common-*/out/protos/*_capnp.rs`; do
  proto_file="$(basename -- $proto_path)"
  dest_path="$CUR_DIR/../common/src/protos/generated/$proto_file"
  echo "Copying $proto_file to $dest_path"

  echo "#![allow(unknown_lints)]" > $dest_path
  echo "#![allow(clippy::all)]" >> $dest_path
  echo "" >> $dest_path
  cat $proto_path >> $dest_path
done

# Prost files
for proto_path in `ls $CUR_DIR/../target/debug/build/exocore-common-*/out/*.*.rs`; do
  proto_file="$(basename -- $proto_path)"
  dest_file=${proto_file/\./_}
  dest_path="$CUR_DIR/../common/src/protos/generated/$dest_file"
  echo "Copying $proto_file to $dest_path"

  cp $proto_path $dest_path
done

# Descriptors
protoc -I"$CUR_DIR/../common/protos/" $CUR_DIR/../common/protos/exocore/index/*.proto -o "$CUR_DIR/../common/src/protos/generated/exocore_index.fd"
protoc -I"$CUR_DIR/../common/protos/" $CUR_DIR/../common/protos/exocore/test/*.proto -o "$CUR_DIR/../common/src/protos/generated/exocore_test.fd"

cargo fmt --all
cargo test --all
