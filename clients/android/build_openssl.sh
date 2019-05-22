#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

. ./env.sh

rm -rf $CUR_DIR/openssl
mkdir $CUR_DIR/openssl
cd $CUR_DIR/openssl
OPENSSL_VERSION="1.1.1b"
curl https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz | tar xz || exit 1

TARGET_DIR=$CUR_DIR/openssl/target
rm -rf $TARGET_DIR
mkdir $TARGET_DIR

cd $CUR_DIR/openssl/openssl-$OPENSSL_VERSION/

# Build for ARM
export PREFIX=$TARGET_DIR/arm
mkdir $PREFIX
# See https://github.com/openssl/openssl/blob/master/NOTES.ANDROID
export PATH=$ANDROID_NDK/toolchains/arm-linux-androideabi-4.9/prebuilt/linux-x86_64/bin:$PATH
./Configure android-arm -D__ANDROID_API__=14 --prefix=$PREFIX
make
make install
