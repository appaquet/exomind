#!/usr/bin/env bash

cd ../web/
yarn install
yarn build_ios

cd ../ios/
./tools/switch_release.sh