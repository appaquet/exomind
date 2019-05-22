# Exocore
[![Build Status](https://dev.azure.com/appaquet/exocore/_apis/build/status/appaquet.exocore?branchName=master)](https://dev.azure.com/appaquet/exocore/_build/latest?definitionId=1&branchName=master)
[![codecov](https://codecov.io/gh/appaquet/exocore/branch/master/graph/badge.svg?token=OKZAHfPlaP)](https://codecov.io/gh/appaquet/exocore)

**Warning: Exocore is at a very early development stage, hence very unstable and probably totally unsafe. Use at your own risk.**

Exocore is a distributed applications framework with private and encrypted data storage. 
It runs applications and their data in an encrypted [blockchain](https://en.wikipedia.org/wiki/Blockchain) 
which is unique per user and distributed on trusted or semi-trusted nodes. It is designed to be resilient to 
failures, allow offline usage (ex: on mobile) and can be used as a backend for applications requiring user data 
encryption. Exocore exposes SDKs for web/WebAssembly, mobile (Android/iOS) and Rust.

The primary concept in Exocore is a Cell, which is a unique container for a user's applications and data. 
A cell consists of:
* Data nodes managing replication and storage of the blockchain
* Index nodes managing indexation, querying and mutation of the data (could be collocated with data node)
* Applications nodes run applications written in WebAssembly (could be collocated with data and index nodes)
* Clients (fat or thin) that can also act as index, data and applications nodes

## Development status
* **Data storage and replication**: Proof of concept
* **Transport**: Proof of concept
* **SDKs**: In development
* **Security**: In development
* **Index layer:** Not yet started
* **Applications layer**: Not yet started
* **Cell management layer**: Not yet started
* **Nodes discovery**: Not yet started
* **Blob storage (IPFS)**: Not yet started

## Dependencies
### General
* Build dependencies
    * On MacOS: Install Xcode and command lines tools
    * On Ubuntu: `apt install build-essential pkg-config libssl-dev`
* [Rust](https://www.rust-lang.org/learn/get-started)
* [Cap'n Proto](https://capnproto.org/install.html)
    * On MacOS: `brew install capnp` 
    * On Ubuntu: `apt install capnproto` 

### WASM
* Clang
    * On MacOS: 
        * Unfortunately, clang installed by Xcode isn't recent enough to compile to WASM. Follow instructions on 
          [this page](https://00f.net/2019/04/07/compiling-to-webassembly-with-llvm-and-clang/)
          to instal LLVM 8 from HomeBrew.
            * `brew install llvm`
            * Use LLVM from HomeBrew:
                * Bash `export PATH=/usr/local/opt/llvm/bin:$PATH`
                * Fish `set -g fish_user_paths "/usr/local/opt/llvm/bin" $fish_user_paths`
    * On Ubuntu: `apt install clang`
* [Node & NPM](https://github.com/nodesource/distributions/blob/master/README.md#debinstall)
* [`wasm-pack`](https://github.com/rustwasm/wasm-pack) to build and package WASM as NPM package
    * `cargo install wasm-pack`

## Setup
* Install components & default targets:
  * `rustup component add clippy rustfmt`
  * `rustup target add wasm32-unknown-unknown`

* iOS build on MacOS (optional):
  * `rustup target add aarch64-apple-ios`

* Android build (optional):
  * Using Docker
    * `docker run --rm -v "$pwd:/root/src" -w /root/src appaquet/cargo-apk cargo-apk build -p exocore-client-android --bin exocore-client-android`
    
  * Using host machine
      * Install Rust targets:
        * `rustup target add i686-linux-android`
        * `rustup target add arm-linux-androideabi`
      * Follow instructions from [`android-rs-glue`](https://github.com/rust-windowing/android-rs-glue#setting-up-your-environment).
        * You need to install Android NDK 17 since GCC isn't packaged since NDK 17 (now clang). See this [issue](https://github.com/rust-windowing/android-rs-glue/issues/208).
        * If installation of cargo-apk doesn't work, you can build locally using
         [appaquet's repo](https://github.com/rust-windowing/android-rs-glue/tree/883c01df16710c98011fcbcc7659db89451877a8) 
         and install cargo-apk by doing `cd cargo-apk && cargo install --path .`
      * To build: `cargo-apk build -p exocore-client-android --bin exocore-client-android`

## CLI
* To run the CLI: 
  * `cargo run --package exocore-cli -- <cli option>`
    or `./utils/cli.sh <cli options>`
* Configuration
    * Most command requires a `config.yaml` file, for which an example can be found in here: [`./examples/config.yaml`]
    * At minimum, the config requires 2 keypair: one for the node, one for the cell.
    * The node keypair is unique per server/node, while the cell keypair is shared among servers that host the cell.
    
## Quick start
### Create a Cell hosted on 2 nodes
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
* Run the [web example](./examples/web):
  * Start development server which will watch files and rebuild automatically:
    * `cd ./examples/web && npm run start`
  * Open browser to [http://127.0.0.1:8080]
* Or build manually: 
    * `cd ./clients/wasm && wasm-pack build`
    * This will create a NPM module at [`./clients/wasm/pkg`]

## Documentation
* [Replication](data/replication.md)
