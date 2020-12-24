#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

VERSION=$1
if [[ -z $VERSION ]]; then
  echo "syntax: $0 <version>"
  exit 1
fi

ROOT_DIR="$CUR_DIR/.."

sed -i.bak "s/\(\"version\":\).*/\1 \"$VERSION\",/g" $ROOT_DIR/web/package.json

CRATES=( \
  "." \
  "integrations/gmail" \
  "core" \
  "server" \
)

for CRATE in "${CRATES[@]}"; do
  TOML_PATH="$ROOT_DIR/${CRATE}/Cargo.toml"
  sed -i.bak "s/^\(version = \).*/\1\"$VERSION\"/g" $TOML_PATH
  sed -i.bak -E "s/(exomind.*version.*\")([0-9]+\.[0-9]+\.[0-9]+)(\".*)/\1$VERSION\3/g" $TOML_PATH
done