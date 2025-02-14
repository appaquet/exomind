# iOS client

**Note: The iOS client doesn't contain any Rust code (yet) and uses the C client.**

## Dependencies

**You need to be on MacOS**

* Install Exocore C client dependencies. See [README](../c/README.md).

* Install Rust targets
  * `rustup target add aarch64-apple-ios`
  * `rustup target add x86_64-apple-ios`

* Install [cargo lipo](https://github.com/TimNN/cargo-lipo) to generate [universal binaries](https://en.wikipedia.org/wiki/Universal_binary).
  * `cargo install cargo-lipo`

* Install Cocoapods
  * `sudo gem install cocoapods`

* Install [Swift Protobuf Plugin](https://github.com/apple/swift-protobuf/)
  * `brew install swift-protobuf`

## Building

* Generate headers & protobuf
  * `./tools/generate.sh`

* Build the universal lib:
  * `./tools/build.sh`

## Usage

* See [iOS example](../../examples/ios/README.md)

## Known issues

* App gets terminated due to signal 13 when resuming from background.
  * Mio doesn't handle `SIGPIPE` signal when Tokio tries to write to a closed connection. See <https://github.com/tokio-rs/mio/issues/949>
  * You can ignore the the signal by adding this in your app (ex: in `didFinishLaunchingWithOptions`):

    ```swift
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        signal(SIGPIPE, SIG_IGN)
        ...
        return true
    }
    ```
