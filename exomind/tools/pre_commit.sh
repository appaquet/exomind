#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

EXOMIND_ROOT="$CUR_DIR/.."

echo "Validating wasm app..."
$EXOMIND_ROOT/app/tools/build.sh

echo "Linting web..."
cd $EXOMIND_ROOT/web
yarn lint