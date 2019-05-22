#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd $CUR_DIR

. ./env.sh

if [[ ! -d "$CUR_DIR/openssl" ]]; then
    echo "Building openssl"
    ./build_openssl.sh
fi

if [[ ! -x "$(command -v cargo-apk)" ]]; then
    echo "cargo-apk needs to be installed"
    exit 1
fi

export OPENSSL_DIR=$CUR_DIR/openssl/target/arm
cargo-apk build --lib -p exocore-client-android
