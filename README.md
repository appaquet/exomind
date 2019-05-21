# Exocore
[![Build Status](https://dev.azure.com/appaquet/exocore/_apis/build/status/appaquet.exocore?branchName=master)](https://dev.azure.com/appaquet/exocore/_build/latest?definitionId=1&branchName=master)
[![codecov](https://codecov.io/gh/appaquet/exocore/branch/master/graph/badge.svg?token=OKZAHfPlaP)](https://codecov.io/gh/appaquet/exocore)

## Dependencies
* Build essentials
    * On MacOS: Install Xcode and command lines tools
    * On Ubuntu: `apt install build-essential pkg-config libssl-dev`
* [Rust](https://www.rust-lang.org/learn/get-started)
* [Cap'n Proto](https://capnproto.org/install.html)
    * On MacOS: `brew install capnp` 
    * On Ubuntu: `apt install capnproto` 
* Clang (for WASM compilation)
    * On MacOS: 
        * Unfortunately, clang installed by Xcode isn't recent enough to compile to WASM. Follow instructions on 
          [this page](https://00f.net/2019/04/07/compiling-to-webassembly-with-llvm-and-clang/)
          to instal LLVM 8 from HomeBrew.
        * `brew install llvm`
        * Use LLVM from HomeBrew:
            * Bash `export PATH=/usr/local/opt/llvm/bin:$PATH`
            * Fish `set -g fish_user_paths "/usr/local/opt/llvm/bin" $fish_user_paths`
    * On Ubuntu: `apt install clang`
* Node & NPM for WASM example

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
* For CLion's profile, see [install profiler](https://www.jetbrains.com/help/clion/cpu-profiler.html)

## CLI
* To run the CLI: 
  * Via Cargo: `cargo run --package exocore-cli -- <cli option>`
  * Via utils: `./utils/cli.sh <cli options>`
* Configuration
    * Most command requires a `config.yaml` file, for which an example can be found in here: [`./examples/config.yaml`]
    * At minimum, the config requires 2 keypair: one for the node, one for the cell.
    * The node keypair is unique per server/node, while the cell keypair is shared among servers that host the cell.
    
### Examples
#### Create a Cell hosted on 2 nodes
* `cp ./examples/config.yaml node1.yaml`
* `cp ./examples/config.yaml node2.yaml`
* For each node's config:
    * Generate keypair for the node: `./utils/cli.sh keys generate`
    * Change the `node_keypair` and `node_public_key` config with this keypair.
    * Change `listen_addresses` with unique port per node.
    * Change `data_dir` with unique data directory per node.
    * Put the other node `public_key` and `addresses` in the `cells/nodes` section.
* Generate keypair for the cell: `./utils/cli.sh keys generate` 
* Add this keypair in both `node1.yaml` and `node2.yaml` in the `cells` section.
* Initialize chain one first node: `./utils/cli.sh cell --config node1.yaml --public_key <cell_public_key> create_genesis_block`
* Start both nodes:
    * Node 1: `./utils/cli.sh server --config ./node1.yaml start`
    * Node 2: `./utils/cli.sh server --config ./node2.yaml start`
    

## WASM web client
* Install [`wasm-pack`](https://github.com/rustwasm/wasm-pack) to build and package WASM as NPM package.
* Run the [web example](./examples/web):
  * Start development server which will watch files and rebuild automatically:
    * `cd ./examples/web && npm run start`
  * Open browser to [http://127.0.0.1:8080]
* Or build manually: 
    * `cd ./clients/wasm && wasm-pack build`
    * This will create a NPM module at [`./clients/wasm/pkg`]


## Documentation
* [Replication](data/replication.md)
