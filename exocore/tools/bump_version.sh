#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

VERSION=$1
if [[ -z $VERSION ]]; then
  echo "syntax: $0 <version>"
  exit 1
fi

ROOT_DIR="$CUR_DIR/.."
VERSION_RE="[0-9]+\.[0-9]+\.[0-9]+"

sed -i.bak -E "s/^([[:space:]]+spec\.version.*=).*/\1 '$VERSION'/" $ROOT_DIR/Exocore.podspec
sed -i.bak "s/\(\"version\":\).*/\1 \"$VERSION\",/g" $ROOT_DIR/package.json

CRATES=( \
  "." \
  "clients/web" \
  "clients/android" \
  "clients/c" \
  "transport" \
  "protos" \
  "core" \
  "exo" \
  "discovery" \
  "chain" \
  "store" \
  "apps/host" \
  "apps/macros" \
  "apps/sdk" \
)

for CRATE in "${CRATES[@]}"; do
  TOML_PATH="$ROOT_DIR/${CRATE}/Cargo.toml"
  sed -i.bak "s/^\(version = \).*/\1\"${VERSION}\"/g" $TOML_PATH
  sed -i.bak -E "s/(exocore.*version.*\")(${VERSION_RE})(\".*)/\1${VERSION}\3/g" $TOML_PATH
done

cd $ROOT_DIR
cargo update