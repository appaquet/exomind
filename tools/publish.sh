#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd $CUR_DIR/../

if [[ "$OSTYPE" != "darwin"* ]]; then
  echo "Publishing should be done on macOS because of cocoapod..."
  exit 1
fi

GIT_CHANGES=$(git status | grep "nothing to commit" | wc -l | sed -e 's/^[ \t]*//')
if [[ $GIT_CHANGES != "1" ]]; then
  echo "Changes need to be commited first..."
  exit 1
fi

VERSION=$(cat package.json | grep version | sed -nE 's/.*([0-9]+\.[0-9]+\.[0-9]+).*/\1/p')
echo "Preparing publishing for version $VERSION"

CRATES=("protos" "core" "transport" "chain" "store" "apps/host" "apps/macros" "apps/sdk" "." "discovery" "exo")

echo "Checking crates..."
for CRATE in "${CRATES[@]}"; do
  pushd $CUR_DIR/../$CRATE
  cargo check
  popd
done

echo "Checking npm..."
npm publish "https://github.com/appaquet/exocore/releases/download/v${VERSION}/exocore-web.tar.gz" --dry-run

echo "Checking pod..."
pod spec lint Exocore.podspec --allow-warnings

####

read -p "Do you want to publish now? (y/n) " CONT
if [ "$CONT" != "y" ]; then
  echo "Cancelled"
  exit 1
fi

echo "Publishing to crates.io..."
for CRATE in "${CRATES[@]}"; do
  pushd $CUR_DIR/../$CRATE
  cargo publish --no-verify # no verify since we check build before

  echo "Waiting 30 seconds for crates.io to publish before next crate..."
  sleep 30
  popd
done

echo "Publishing to npm..."
npm publish "https://github.com/appaquet/exocore/releases/download/v${VERSION}/exocore-web.tar.gz"

echo "Publishing to cocoapod..."
pod trunk push Exocore.podspec --allow-warnings
