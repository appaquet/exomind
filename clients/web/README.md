# Web client

## Dependencies

* Install Rust's WASM target
  * `rustup target add wasm32-unknown-unknown`

* Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

* Install Clang
  * On Ubuntu:
    * `apt install clang`
  * On MacOS:
    * Xcode 12.2+ (macOS Big Sur and later)
      * Make sure Xcode command line tools are installed.

* Install Node & Yarn
  * [Install Node](https://github.com/nodesource/distributions/blob/master/README.md#debinstall)
  * Install yarn: `npm install -g yarn`

## Building

* Build:
  * `./tools/build.sh`

## Usage

* See [Web example](../../examples/web/README.md)

## Known issues

* Connection can only be via localhost or https since WebCrypto used in libp2p's secio implementation only works over secure code.
  * See <https://stackoverflow.com/questions/46670556/how-to-enable-crypto-subtle-for-unsecure-origins-in-chrome> and
        <https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto>
