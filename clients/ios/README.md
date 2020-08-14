# iOS client

## Dependencies
**You need to be on MacOS**

* Install Rust targets
    * `rustup target add aarch64-apple-ios`
    * `rustup target add x86_64-apple-ios`
    
* Install tools
    * `cargo install cargo-lipo`
    * `cargo install cbindgen`

* Install Cocoapods
    * `sudo gem install cocoapods`

* Install [Swift Protobuf Plugin](https://github.com/apple/swift-protobuf/)
    * `brew install swift-protobuf`


## Building
* Generate headers & protobuf
    * `./tools/generate.sh`

* Build the universal lib: `./tools/build.sh`


## Example project

* `cd ../../examples/ios`
* `pod install`
* Open the project in XCode / AppCode
* Generate a node configuration and copy it to the app:

  `exo config standalone path/to/ios/node/config.yaml --exclude-app-schemas yaml`


## Known issues
* App gets terminated due to signal 13 when resuming from background.
  * Mio doesn't handle `SIGPIPE` signal when Tokio tries to write to a closed connection. See (here)[https://github.com/tokio-rs/mio/issues/949].
  * You can ignore the the signal by adding this in your app (ex: in `didFinishLaunchingWithOptions`):
    ```swift
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        signal(SIGPIPE, SIG_IGN)
        ...
        return true
    }
    ```