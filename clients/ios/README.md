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
