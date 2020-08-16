#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export EXOCORE_ROOT="$CUR_DIR/.."
export GENERATE_PROTOS=1
cargo clean -p exocore-core
cargo build -p exocore-core

# Capnp files
for proto_path in `ls $EXOCORE_ROOT/target/debug/build/exocore-core-*/out/protos/*_capnp.rs`; do
  proto_file="$(basename -- $proto_path)"
  dest_path="$EXOCORE_ROOT/core/src/protos/generated/$proto_file"
  echo "Copying $proto_file to $dest_path"

  echo "#![allow(unknown_lints)]" > $dest_path
  echo "#![allow(clippy::all)]" >> $dest_path
  echo "" >> $dest_path
  cat $proto_path >> $dest_path
done

# Prost files
for proto_path in `ls $EXOCORE_ROOT/target/debug/build/exocore-core-*/out/*.*.rs`; do
  proto_file="$(basename -- $proto_path)"
  dest_file=${proto_file/\./_}
  dest_path="$EXOCORE_ROOT/core/src/protos/generated/$dest_file"
  echo "Copying $proto_file to $dest_path"

  cp $proto_path $dest_path
done

cargo fmt --all

# Descriptors
protoc -I"$EXOCORE_ROOT/protos/" $EXOCORE_ROOT/protos/exocore/index/*.proto -o "$EXOCORE_ROOT/core/src/protos/generated/exocore_index.fd"
protoc -I"$EXOCORE_ROOT/protos/" $EXOCORE_ROOT/protos/exocore/test/*.proto -o "$EXOCORE_ROOT/core/src/protos/generated/exocore_test.fd"

# Generate web protos if possible
if [[ -d "$EXOCORE_ROOT/node_modules" ]]; then
  echo "Generating web protos..."
  ./clients/web/tools/generate_protos.sh
fi

# Generate iOS protos if possible
if [[ "$OSTYPE" == "darwin"* ]]; then
  echo "Generating iOS protos..."
  ./clients/ios/tools/generate.sh
fi
