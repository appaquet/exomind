#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR/../"

cargo check --all
cargo check --tests --all
cargo check --benches --all

if [[ "$ANDROID_NDK_STANDALONE" != "" ]]; then
    export PATH="$PATH:$ANDROID_NDK_STANDALONE/arm/bin/"
    cargo check --target "armv7-linux-androideabi"
else
    echo "The ANDROID_NDK_STANDALONE path is not set. You can create a standalone NDK using script from: https://github.com/kennytm/rust-ios-android. Not testing Android build."
    sleep 2
fi

if [[ `uname -s` == "Darwin" ]]; then
    cargo check --target "aarch64-apple-ios"
else
    echo "Not currently on MacOS. Not testing iOS build."
    sleep 2
fi

cd "$CUR_DIR/../common/"
cargo check --target "wasm32-unknown-unknown"
