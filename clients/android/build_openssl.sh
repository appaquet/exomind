#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

OPENSSL_VERSION="1.1.1b"
TARGET_DIR=$CUR_DIR/openssl/target

mkdir -p $CUR_DIR/openssl
cd $CUR_DIR/openssl

if [[ ! -d "openssl-$OPENSSL_VERSION" ]]; then
    curl https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz | tar xz || exit 1
fi
cd $CUR_DIR/openssl/openssl-$OPENSSL_VERSION/

if [[ `uname` == "Darwin" ]]; then
    NDK_HOST="darwin-x86_64"
elif [[ `uname` == "Linux" ]]; then
    NDK_HOST="linux-x86_64"
else
    echo "Unsupported OS: $(uname)"
    exit 1
fi

#
# See https://github.com/openssl/openssl/blob/master/NOTES.ANDROID
#
export ANDROID_NDK_HOME=$NDK_HOME
INITIAL_PATH=$PATH

# Build for ARM
export PREFIX=$TARGET_DIR/arm
if [[ ! -d "$PREFIX" ]]; then
    export PATH=$NDK_HOME/toolchains/arm-linux-androideabi-4.9/prebuilt/$NDK_HOST/bin:$INITIAL_PATH
    ./Configure android-arm -D__ANDROID_API__=14 --prefix=$PREFIX
    make -j12 clean
    make -j12
    make -j12 install
fi

# Build for x86
export PREFIX=$TARGET_DIR/x86
if [[ ! -d "$PREFIX" ]]; then
    export PATH=$NDK_HOME/toolchains/x86-4.9/prebuilt/$NDK_HOST/bin:$INITIAL_PATH
    ./Configure android-x86 -D__ANDROID_API__=14 --prefix=$PREFIX
    make -j12 clean
    make -j12
    make -j12 install
fi

# Build for x86_64
export PREFIX=$TARGET_DIR/x86_64
if [[ ! -d "$PREFIX" ]]; then
    export PATH=$NDK_HOME/toolchains/x86_64-4.9/prebuilt/$NDK_HOST/bin:$INITIAL_PATH
    ./Configure android-x86_64 -D__ANDROID_API__=21 --prefix=$PREFIX
    make -j12 clean
    make -j12
    make -j12 install
fi

# Build for ARM64
export PREFIX=$TARGET_DIR/aarch64
if [[ ! -d "$PREFIX" ]]; then
    export PATH=$NDK_HOME/toolchains/aarch64-linux-android-4.9/prebuilt/$NDK_HOST/bin:$INITIAL_PATH
    ./Configure android-arm64 -D__ANDROID_API__=21 --prefix=$PREFIX
    make -j12 clean
    make -j12
    make -j12 install
fi
