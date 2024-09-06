#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

IOS_DIR="$CUR_DIR/.."

sed -i.bak "s/ExomindDev/Exomind/g" $IOS_DIR/Exomind/Info.plist
sed -i.bak "s/ExomindDev/Exomind/g" $IOS_DIR/ExomindActionExt/Info.plist
sed -i.bak "s/ExomindDev/Exomind/g" $IOS_DIR/Exomind.xcodeproj/project.pbxproj