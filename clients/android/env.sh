#!/usr/bin/env bash

if [[ "$ANDROID_NDK" == "" ]]; then
    export ANDROID_NDK=$NDK_HOME
fi
if [[ "$ANDROID_NDK" == "" ]]; then
    export ANDROID_NDK=$ANDROID_NDK_HOME
fi
if [[ ! -d $ANDROID_NDK ]]; then
    echo "Environment ANDROID_NDK or NDK_HOME should point to valid a Android NDK"
    exit 1
fi
export ANDROID_NDK_HOME=$ANDROID_NDK
export ANDROID_NDK=$ANDROID_NDK
export NDK_HOME=$ANDROID_NDK

if [[ "$ANDROID_SDK" == "" ]]; then
    export ANDROID_SDK=$SDK_HOME
fi
if [[ "$ANDROID_SDK" == "" ]]; then
    export ANDROID_SDK=$ANDROID_SDK_HOME
fi
if [[ "$ANDROID_SDK" == "" ]]; then
    export ANDROID_SDK=$ANDROID_HOME
fi
if [[ ! -d $ANDROID_SDK ]]; then
    echo "Environment ANDROID_SDK or SDK_HOME should point to valid a Android SDK"
    exit 1
fi
export ANDROID_SDK_HOME=$ANDROID_SDK
export ANDROID_SDK=$ANDROID_SDK
export SDK_HOME=$ANDROID_SDK
