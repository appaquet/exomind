#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

EXOCORE_IOS_ROOT="$CUR_DIR/../"
EXOCORE_ROOT="$EXOCORE_IOS_ROOT/../../"
EXOCORE_C_ROOT="$EXOCORE_ROOT/clients/c"

MODE=${1:-debug}
if [[ "$MODE" == "release" ]]; then
    CARGO_ARGS="--release"
elif [[ "$MODE" == "debug" ]]; then
    CARGO_ARGS=""
elif [[ "$MODE" == "production" ]]; then
    CARGO_ARGS="--profile production"
else
    echo "syntax: $0 [release|debug|production]"
    exit 1
fi

EXOCORE_IOS_LIB_DIR="$EXOCORE_IOS_ROOT/lib"
rm -rf $EXOCORE_IOS_LIB_DIR
mkdir -p $EXOCORE_IOS_LIB_DIR/libs
mkdir -p $EXOCORE_IOS_LIB_DIR/header

# Build static libs
pushd $EXOCORE_C_ROOT
SIM_TARGETS="aarch64-apple-ios-sim"
cargo lipo $CARGO_ARGS --targets $SIM_TARGETS 
mkdir -p $EXOCORE_IOS_LIB_DIR/libs/sim
cp $EXOCORE_ROOT/target/universal/$MODE/libexocore.a $EXOCORE_IOS_LIB_DIR/libs/sim

IOS_TARGETS="aarch64-apple-ios"
cargo lipo $CARGO_ARGS --targets $IOS_TARGETS 
mkdir -p $EXOCORE_IOS_LIB_DIR/libs/ios
cp $EXOCORE_ROOT/target/universal/$MODE/libexocore.a $EXOCORE_IOS_LIB_DIR/libs/ios
popd

# Build framework
xcodebuild \
    -create-xcframework \
    -library $EXOCORE_IOS_LIB_DIR/libs/sim/libexocore.a \
    -headers $EXOCORE_IOS_LIB_DIR/header/ \
    -library $EXOCORE_IOS_LIB_DIR/libs/ios/libexocore.a \
    -headers $EXOCORE_IOS_LIB_DIR/header/ \
    -output $EXOCORE_IOS_LIB_DIR/ExocoreLibs.xcframework

# Prevent duplicate libs in release archive
rm -rf $EXOCORE_IOS_LIB_DIR/libs 
rm -rf $EXOCORE_IOS_LIB_DIR/header