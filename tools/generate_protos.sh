#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export EXOCORE_ROOT="$CUR_DIR/.."

# Protobuf descriptors
protoc -I"$EXOCORE_ROOT/protos/protobuf/" $EXOCORE_ROOT/protos/protobuf/exocore/store/*.proto -o "$EXOCORE_ROOT/protos/src/generated/exocore_store.fd"
protoc -I"$EXOCORE_ROOT/protos/protobuf/" $EXOCORE_ROOT/protos/protobuf/exocore/test/*.proto -o "$EXOCORE_ROOT/protos/src/generated/exocore_test.fd"

# Prost & capnp generation
export GENERATE_PROTOS=1
cargo clean -p exocore-protos
cargo build -p exocore-protos || true # we only care about build.rs being run
cargo +nightly fmt -- --config-path ./rustfmt-nightly.toml

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
