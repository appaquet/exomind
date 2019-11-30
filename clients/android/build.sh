#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd $CUR_DIR

if [[ "$NDK_STANDALONE" == "" ]]; then
    NDK_STANDALONE=~/Android/NDK
fi

export PATH=$NDK_STANDALONE/arm/bin:$PATH
cargo build -p exocore-client-android --target armv7-linux-androideabi

export PATH=$NDK_STANDALONE/arm64/bin:$PATH
cargo build -p exocore-client-android --target aarch64-linux-android

export PATH=$NDK_STANDALONE/x86/bin:$PATH
cargo build -p exocore-client-android --target i686-linux-android

export PATH=$NDK_STANDALONE/x86_64/bin:$PATH
cargo build -p exocore-client-android --target x86_64-linux-android
