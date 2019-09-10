# Exocore
[![codecov](https://codecov.io/gh/appaquet/exocore/branch/master/graph/badge.svg?token=OKZAHfPlaP)](https://codecov.io/gh/appaquet/exocore)
[![dependency status](https://deps.rs/repo/github/appaquet/exocore/status.svg)](https://deps.rs/repo/github/appaquet/exocore)

**Warning: Exocore is at a very early development stage, hence very unstable and probably totally unsafe. Use at your own risk.**

Exocore is a distributed applications framework with private and encrypted data storage. 
It runs applications and their data in an encrypted [blockchain](https://en.wikipedia.org/wiki/Blockchain) 
which is unique per user and distributed on trusted or semi-trusted nodes. It is designed to be resilient to 
failures, allow offline usage (ex: on mobile) and can be used as a backend for applications requiring user data 
encryption. Exocore exposes SDKs for web/webassembly, mobile (Android/iOS) and Rust.

The primary concept in Exocore is a Cell, which is a unique container for a user's applications and data. 
A cell consists of:
* Data nodes managing replication and storage of the blockchain
* Index nodes managing indexation, querying and mutation of the data (collocated with data node)
* Applications nodes run applications written in WebAssembly (could be collocated with index  nodes)
* Clients (fat or thin) that can also act as index, data and applications nodes

## Development status
* **Data storage and replication layer**: Proof of concept
* **Transport layer**: Proof of concept
* **Index layer:** In development
* **SDKs**: In development
* **Security**: In development
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
  * Install using [rustup](https://www.rust-lang.org/learn/get-started)
  * Clippy and Rustfmt: `rustup component add clippy rustfmt`
* [Cap'n Proto](https://capnproto.org/install.html)
    * On MacOS: `brew install capnp` 
    * On Ubuntu: `apt install capnproto` 

### WASM (optional)
* See [WASM client README](./clients/wasm/README.md)

### Android (optional)
* See [Android client README](./clients/android/README.md)

### iOS (optional)
* See [iOS client README](./clients/ios/README.md)

## Usage & configuration
* CLI:
  * Using Cargo: `cargo run --package exocore-cli -- <cli option>`
                 or `./utils/cli.sh <cli options>`
  * Using the latest Docker image:
    `docker run --rm -it -v "$PWD:/volume" docker.pkg.github.com/appaquet/exocore/cli exocore-cli <cli options>`

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
  * Open browser to [http://127.0.0.1:8080](http://127.0.0.1:8080)
* Or build manually: 
    * `cd ./clients/wasm && wasm-pack build`
    * This will create a NPM module at ./clients/wasm/pkg

## Documentation
* [Replication](data/replication.md)
