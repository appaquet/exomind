# C client

## Dependencies

* Install [cbindgen](https://github.com/eqrion/cbindgen)
  * `cargo install cbindgen`

## Building

* Generate headers: `./tools/generate.sh`
  * Will generate `exocore.h` header

* Build: `cargo build`
  * Will generate dynamic & static libs in workspace's target directory.
