# Exocore
[![Build Status](https://dev.azure.com/appaquet/exocore/_apis/build/status/appaquet.exocore?branchName=master)](https://dev.azure.com/appaquet/exocore/_build/latest?definitionId=1&branchName=master)
[![codecov](https://codecov.io/gh/appaquet/exocore/branch/master/graph/badge.svg?token=OKZAHfPlaP)](https://codecov.io/gh/appaquet/exocore)

## Dependencies
* [Rust](https://www.rust-lang.org/learn/get-started)
* [Cap'n Proto](https://capnproto.org/install.html)
    * On MacOS: `brew install capnp` 
    * On Ubuntu: `apt install capnproto` 

## Setup
* Install components & default targets:
  * `rustup component add clippy rustfmt`
  * `rustup target add wasm32-unknown-unknown`

* iOS build (optional):
  * On MacOS: `rustup target add aarch64-apple-ios`

* Android build (optional)
  * Follow instructions [here](https://github.com/kennytm/rust-ios-android) to setup Rust with Android targets & expose the Standalone NDF folder to `ANDROID_NDK_STANDALONE` environment variable.
  * Install Android target: `rustup target add armv7-linux-androideabi`

## Development
* Ideally, use [CLion](https://www.jetbrains.com/clion/) with the [Rust plugin](https://github.com/intellij-rust/intellij-rust). 
  You can also use IntelliJ, only CLion has debugger support.

## Documentation
* [Replication](data/replication.md)
