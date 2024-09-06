#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd $CUR_DIR/../

CRATES=("protos" "app" "integrations/gmail" "." "exm")

echo "Checking crates..."
for CRATE in "${CRATES[@]}"; do
  pushd $CUR_DIR/../$CRATE
  cargo check
  popd
done

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