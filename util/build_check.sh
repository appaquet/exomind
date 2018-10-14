#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd $CUR_DIR/../

cargo test

# TODO: Proper message for missing ndk
export PATH="$PATH:/Users/appaquet/Work/ampme/ampme-ampplayer/NDK/arm/bin/"
cargo build --target "armv7-linux-androideabi"

cargo build --target "aarch64-apple-ios"
