#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "$OSTYPE" != "darwin"* ]]; then
  echo "Publishing should be done on macOS because of cocoapod..."
  exit 1
fi

VERSION=$1
if [[ -z $VERSION ]]; then
  echo "syntax: $0 <version>"
  exit 1
fi

SED="sed"
if [[ "$OSTYPE" == "darwin"* ]]; then
  if ! command -v gsed &>/dev/null; then
    echo "on macos, you need to install gnused: brew install gnu-sed"
    exit
  fi
  SED="gsed"
fi

REPO_DIR="$CUR_DIR/.."
EXOCORE_DIR="$REPO_DIR/exocore"
EXOMIND_DIR="$REPO_DIR/exomind"
VERSION_RE="[0-9]+\.[0-9]+\.[0-9]+(|\-dev|\-pre[0-9]+)"

# Exocore
$SED -i.bak -E "s/^([[:space:]]+spec\.version.*=).*/\1 '$VERSION'/" $EXOCORE_DIR/Exocore.podspec
$SED -i.bak "s/\(\"version\":\).*/\1 \"$VERSION\",/g" $EXOCORE_DIR/package.json

# Exomind
$SED -i.bak "s/\(\"version\":\).*/\1 \"$VERSION\",/g" $EXOMIND_DIR/web/package.json
$SED -i.bak "s/\(\"version\":\).*/\1 \"$VERSION\",/g" $EXOMIND_DIR/browsers/chrome/manifest.json
$SED -i.bak "s/\(version:\).*/\1 $VERSION/g" $EXOMIND_DIR/app.yaml
$SED -i.bak "s/\(MARKETING_VERSION =\).*/\1 $VERSION;/g" $EXOMIND_DIR/ios/Exomind.xcodeproj/project.pbxproj
$SED -i.bak "s/\(CURRENT_PROJECT_VERSION =\).*/\1 $VERSION;/g" $EXOMIND_DIR/ios/Exomind.xcodeproj/project.pbxproj

# All crates
CRATES=$(find . -name Cargo.toml -not -path '*/node_modules/*' -not -path '**/3rd/**')
for TOML_PATH in "${CRATES[@]}"; do
  $SED -i.bak "s/^\(version = \).*/\1\"${VERSION}\"/g" $TOML_PATH
  $SED -i.bak -E "s/(exocore.*version.*\")(${VERSION_RE})(\".*)/\1${VERSION}\4/g" $TOML_PATH
  $SED -i.bak -E "s/(exomind.*version.*\")(${VERSION_RE})(\".*)/\1${VERSION}\4/g" $TOML_PATH
done

cd "$REPO_DIR"
cargo update

cd "$EXOCORE_DIR/examples/ios"
pod install

cd "$EXOMIND_DIR/ios"
pod install

cd "$EXOMIND_DIR/web"
yarn install