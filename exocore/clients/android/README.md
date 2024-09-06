# Android build

* Install Rust targets
    ```
    rustup target add i686-linux-android
    rustup target add x86_64-linux-android
    rustup target add armv7-linux-androideabi
    rustup target add aarch64-linux-android
    ```

* Install Android NDK 17C
    * Unfortunately, we need an older version of the NDK since newest don't include gcc
    * Download [Android NDK 17C](https://developer.android.com/ndk/downloads/older_releases.html#ndk-17c-downloads)
    * Unzip in `~/Android/` so that the NDK path is `~/Android/android-ndk-r17c`
    * Expose the `NDK_HOME` environment variable pointing to `~/Android/android-ndk-r17c`

* Install Android SDK
    * The easiest is to install [Android Studio](https://developer.android.com/studio/)
    * Expose the `ANDROID_HOME` environment variable pointing to the SDK
    * Install API 14 and API 21 tools:
        ```
        $ANDROID_HOME/tools/bin/sdkmanager "platform-tools"
        $ANDROID_HOME/tools/bin/sdkmanager "platforms;android-14"
        $ANDROID_HOME/tools/bin/sdkmanager "platforms;android-21"
        $ANDROID_HOME/tools/bin/sdkmanager "build-tools"
        ```
        
* Create a Standalone NDK
    ```
    $NDK_HOME/build/tools/make_standalone_toolchain.py --api 14 --arch arm --install-dir ~/Android/NDK/arm
    $NDK_HOME/build/tools/make_standalone_toolchain.py --api 14 --arch x86 --install-dir ~/Android/NDK/x86
    $NDK_HOME/build/tools/make_standalone_toolchain.py --api 21 --arch arm64 --install-dir ~/Android/NDK/arm64
    $NDK_HOME/build/tools/make_standalone_toolchain.py --api 21 --arch x86_64 --install-dir ~/Android/NDK/x86_64
    ```

* Configure your `~/.cargo/config` by adding:
    ```
    [target.aarch64-linux-android]
    ar = "<YOUR_HOME>/Android/NDK/arm64/bin/aarch64-linux-android-ar"
    linker = "<YOUR_HOME>/Android/NDK/arm64/bin/aarch64-linux-android-clang"

    [target.armv7-linux-androideabi]
    ar = "<YOUR_HOME>/Android/NDK/arm/bin/arm-linux-androideabi-ar"
    linker = "<YOUR_HOME>/Android/NDK/arm/bin/arm-linux-androideabi-clang"

    [target.arm-linux-androideabi]
    ar = "<YOUR_HOME>/Android/NDK/arm/bin/arm-linux-androideabi-ar"
    linker = "<YOUR_HOME>/Android/NDK/arm/bin/arm-linux-androideabi-clang"

    [target.i686-linux-android]
    ar = "<YOUR_HOME>/Android/NDK/x86/bin/i686-linux-android-ar"
    linker = "<YOUR_HOME>/Android/NDK/x86/bin/i686-linux-android-clang"
    
    [target.x86_64-linux-android]
    ar = "<YOUR_HOME>/Android/NDK/x86_64/bin/x86_64-linux-androidi-ar"
    linker = "<YOUR_HOME>/Android/NDK/x86_64/bin/x86_64-linux-android-clang"
    ```
    
* Build the clients
    * `./build.sh`
