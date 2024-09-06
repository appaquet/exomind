#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

VERSION=$1
if [[ -z $VERSION ]]; then
  echo "syntax: $0 <version>"
  exit 1
fi

SEDBIN="sed"
if [[ "$OSTYPE" == "darwin"* ]]; then
  if ! command -v gsed &> /dev/null; then
      echo "on macos, you need to install gnused: brew install gnu-sed"
      exit
  fi
  SEDBIN="gsed"
fi

ROOT_DIR="$CUR_DIR/.."
VERSION_RE="[0-9]+\.[0-9]+\.[0-9]+(|\-dev|\-pre[0-9]+)"

$SEDBIN -i.bak "s/\(\"version\":\).*/\1 \"$VERSION\",/g" $ROOT_DIR/web/package.json
$SEDBIN -i.bak "s/\(\"version\":\).*/\1 \"$VERSION\",/g" $ROOT_DIR/browsers/chrome/manifest.json
$SEDBIN -i.bak "s/\(version:\).*/\1 $VERSION/g" $ROOT_DIR/app.yaml
$SEDBIN -i.bak "s/\(MARKETING_VERSION =\).*/\1 $VERSION;/g" $ROOT_DIR/ios/Exomind.xcodeproj/project.pbxproj
$SEDBIN -i.bak "s/\(CURRENT_PROJECT_VERSION =\).*/\1 $VERSION;/g" $ROOT_DIR/ios/Exomind.xcodeproj/project.pbxproj

CRATES=("app" "protos" "integrations/gmail" "." "exm")

for CRATE in "${CRATES[@]}"; do
  TOML_PATH="$ROOT_DIR/${CRATE}/Cargo.toml"
  $SEDBIN -i.bak "s/^\(version = \).*/\1\"$VERSION\"/g" $TOML_PATH
  $SEDBIN -i.bak -E "s/(exomind.*version.*\")(${VERSION_RE})(\".*)/\1${VERSION}\4/g" $TOML_PATH
done

cd $ROOT_DIR
cargo update