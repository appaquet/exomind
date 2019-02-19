#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR/../"

cargo clean

cargo test --all

if [[ "$ANDROID_NDK_STANDALONE" != "" ]]; then
    export PATH="$PATH:$ANDROID_NDK_STANDALONE/arm/bin/"
    cargo check --target "armv7-linux-androideabi"
else
    echo "The ANDROID_NDK_STANDALONE path is not set. You can create a standalone NDK using script from: https://github.com/kennytm/rust-ios-android"
    echo "Not testing Android build."
    sleep 5
fi

cargo check --target "aarch64-apple-ios"

cd "$CUR_DIR/../common/"
cargo check --target "wasm32-unknown-unknown"
