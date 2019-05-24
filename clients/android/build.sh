#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd $CUR_DIR

if [[ ! -d "$CUR_DIR/openssl" ]]; then
    echo "Building openssl"
    ./build_openssl.sh
fi

if [[ "$NDK_STANDALONE" == "" ]]; then
    NDK_STANDALONE=~/Android/NDK
fi

export OPENSSL_DIR=$CUR_DIR/openssl/target/arm
export PATH=$NDK_STANDALONE/arm/bin:$PATH
cargo build -p exocore-client-android --target arm-linux-androideabi

export OPENSSL_DIR=$CUR_DIR/openssl/target/aarch64
export PATH=$NDK_STANDALONE/arm64/bin:$PATH
cargo build -p exocore-client-android --target aarch64-linux-android

export OPENSSL_DIR=$CUR_DIR/openssl/target/x86
export PATH=$NDK_STANDALONE/x86/bin:$PATH
cargo build -p exocore-client-android --target i686-linux-android
