# iOS build

**You need to be on MacOS**

* Install Rust targets
    * `rustup target add aarch64-apple-ios`
    * `rustup target add x86_64-apple-ios`
    
* Install tools
    * `cargo install cargo-lipo`
    * `cargo install cbindgen` (optional, only if changing the API)

* Generate headers
    * If you change the API, you need to generate new C headers
    * `cbindgen --config cbindgen.toml --crate exocore_client_ios --output exocore.h`

* Build the universal lib: `cargo lipo`

* Open the [`xcode`](./xcode/exocore-client-ios) project in XCode