#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR/../"

cargo clean

cargo test --all

if [[ "$ANDROID_NDK" != "" ]]; then
    export PATH="$PATH:$ANDROID_NDK/arm/bin/"
    cargo check --target "armv7-linux-androideabi"
else
    echo "The ANDROID_NDK path is not set. Not testing Android build"
    sleep 5
fi

cargo check --target "aarch64-apple-ios"

cd "$CUR_DIR/../common/"
cargo check --target "wasm32-unknown-unknown"
