#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR/../"

cargo test --all

# TODO: Proper message for missing ndk
export PATH="$PATH:/Users/appaquet/Work/ampme/ampme-ampplayer/NDK/arm/bin/"
cargo check --target "armv7-linux-androideabi"

cargo check --target "aarch64-apple-ios"

cd "$CUR_DIR/../common/"
cargo check --target "wasm32-unknown-unknown"
