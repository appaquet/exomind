
## Dependencies
* [Rust](https://www.rust-lang.org/learn/get-started)
* [Cap'n Proto](https://capnproto.org/install.html)
    * On MacOS: `brew install capnp` 
    * On Ubuntu: `apt install capnproto` 

## Rust
* Install components:
  * `rustup component add clippy rustfmt`
  
* Install targets:
  * `rustup target add armv7-linux-androideabi`
  * `rustup target add aarch64-apple-ios`
  * `rustup target add wasm32-unknown-unknown`
