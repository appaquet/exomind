#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOCORE_ROOT="$CUR_DIR/.."
REPO_ROOT="$EXOCORE_ROOT/.."

# Protobuf descriptors
protoc -I"$EXOCORE_ROOT/protos/protobuf/" $EXOCORE_ROOT/protos/protobuf/exocore/store/*.proto -o "$EXOCORE_ROOT/protos/src/generated/exocore_store.fd"
protoc -I"$EXOCORE_ROOT/protos/protobuf/" $EXOCORE_ROOT/protos/protobuf/exocore/test/*.proto -o "$EXOCORE_ROOT/protos/src/generated/exocore_test.fd"

# Prost & capnp generation
export GENERATE_PROTOS=1
cargo clean -p exocore-protos
cargo build -p exocore-protos || true # we only care about build.rs being run
$REPO_ROOT/tools/format.sh

# Generate web protos if possible
echo "Generating web protos..."
$EXOCORE_ROOT/clients/web/tools/generate_protos.sh

# Generate iOS protos if possible
if [[ "$OSTYPE" == "darwin"* ]]; then
  echo "Generating iOS protos..."
  $EXOCORE_ROOT/clients/ios/tools/generate.sh
fi
