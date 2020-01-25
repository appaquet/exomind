# iOS build

**You need to be on MacOS**

* Install Rust targets
    * `rustup target add aarch64-apple-ios`
    * `rustup target add x86_64-apple-ios`
    
* Install tools
    * `cargo install cargo-lipo`
    * `cargo install cbindgen` (optional, only if changing the API)

* Install [Swift Protobuf Plugin](https://github.com/apple/swift-protobuf/)
    * `brew install swift-protobuf`

* Generate headers & protobufs
    * `./generate.sh`

* Build the universal lib: `cargo lipo`

* Open the [`xcode`](./xcode/exocore-client-ios) project in XCode