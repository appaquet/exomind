#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd $CUR_DIR/../

if [[ "$OSTYPE" != "darwin"* ]]; then
  echo "Publishing should be done on macOS because of cocoapod..."
  exit 1
fi

CRATES=("core" "transport" "chain" "store"  "." "exo" "discovery")

echo "Checking crates..."
for CRATE in "${CRATES[@]}"; do
  pushd $CUR_DIR/../$CRATE
  cargo check
  popd
done

echo "Checking npm..."
npm run build
npm publish --dry-run

echo "Checking pod..."
pod spec lint Exocore.podspec

####

read -p "Do you want to publish now? (y/n) " CONT
if [ "$CONT" != "y" ]; then
  echo "Cancelled"
  exit 1
fi

echo "Publishing to crates.io..."
for CRATE in "${CRATES[@]}"; do
  pushd $CUR_DIR/../$CRATE
  cargo publish
  popd
done

echo "Publishing to npm..."
npm publish

echo "Publishing to cocoapod..."
pod trunk push Exocore.podspec