#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

EXOCORE_ROOT="$CUR_DIR/../../../"
EXOCORE_WEB_ROOT="$CUR_DIR/../"

pushd $EXOCORE_WEB_ROOT
wasm-pack build --out-dir=wasm $1
rm -f wasm/README.md wasm/.gitignore wasm/package.json
popd

if [[ ! -d $EXOCORE_ROOT/nodes_module ]]; then
    pushd $EXOCORE_ROOT
    yarn install
    popd
fi

$CUR_DIR/generate_protos.sh

pushd $EXOCORE_ROOT
yarn tsc
popd
