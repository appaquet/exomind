# Exocore
[![codecov](https://codecov.io/gh/appaquet/exocore/branch/master/graph/badge.svg?token=OKZAHfPlaP)](https://codecov.io/gh/appaquet/exocore)
![Build](https://github.com/appaquet/exocore/workflows/Push%20tester/badge.svg)

**Warning: Exocore is at a very early development stage, hence incomplete, unstable and probably totally unsafe. Use at your own risk.**

Exocore is a distributed applications framework with private and encrypted data storage. Think of like an infrastructure that allows
a user to own his own personal cloud that is extensible via WebAssembly applications and accessible via Web/Mobile SDKs. It is designed 
to be resilient to failures, allow offline usage (ex: on mobile). 

The primary concept in Exocore is a Cell, which is a unique container for a user's applications and data.

A cell consists of:
* Chain nodes managing replication and storage by using a blockchain data structure.
* Store nodes managing indexation, querying and mutation of the data (collocated with chain node).
* Applications nodes run applications written in WebAssembly (that can be collocated with store nodes)
* Clients (fat or thin) that can also act as index, data and partially run applications' WebAssembly.

## Roadmap
### v0.1 (in progress)
* **Chain storage and replication layer**: Proof of concept
* **Transport layer**: Proof of concept
* **Store layer:** Proof of concept
* **Encryption**: In development

### v0.2
* **Cell management**
* **Applications**  (WebAssembly)
* **Android SDK**

### v0.3 and beyond
* **Nodes discovery**
* **Blob storage**  (IPFS)
* **Offline support**


## Dependencies
* Build dependencies
    * On MacOS: Install Xcode and command lines tools
    * On Ubuntu: `apt install build-essential pkg-config libssl-dev`

* This repository uses [git-lfs](https://git-lfs.github.com/) to store binaries.
  * On MacOS: `brew install git-lfs`
  * On Ubuntu: `apt install git-lfs`
    
* [Rust](https://www.rust-lang.org/learn/get-started)
  * Install using [rustup](https://www.rust-lang.org/learn/get-started)
  * Clippy and Rustfmt: `rustup component add clippy rustfmt`
  
* [Cap'n Proto](https://capnproto.org/install.html)
    * On MacOS: `brew install capnp` 
    * On Ubuntu: `apt install capnproto` 

* [Protobuf](https://developers.google.com/protocol-buffers/)
    * On MacOS: `brew install protobuf` 
    * On Ubuntu: `apt install protobuf-compiler` 
    

## Usage & configuration
* CLI:
  * `./tools/install.sh` or `cd exo && cargo install --path .`

* Configuration
    * Most commands requires a node configuration file, for which an example can be found in here: [`./examples/node.yaml`].
    * At minimum, the config requires 2 keypair: one for the node, one for the cell.
    * The node keypair is unique per server/node, while the cell keypair is shared among servers that host the cell.
    * See [Quick start](#quick-start) section for example 2 nodes setup.
    
## Quick start

### Create a Cell hosted on 2 nodes
* On node 1
  * Generate node's configuration: 
    `exo --dir ./node1 node init --name node1`
  * Edit configuration to include accessible addresses:
    `exo -d ./node1 config edit`
  * Generate a cell:
    `exo -d ./node1 cell init --name my_cell`

* On node 2
  * Generate node's configuration: 
    `exo --dir ./node2 node init --name node1`
  * Edit configuration to include accessible addresses. 
    If both nodes are running on same machine, make sure they have unique ports.
    `exo -d ./node2 config edit`
  * Copy node's public info:
    `exo -d ./node2 config print --cell`

* On node 1:
  * Add node 2:
    `exo -d ./node1 cell node add --chain --store` 
    and then copy node 2's public info in editor.
  * Copy cell's config:
    `exo -d ./node1 cell print --inline` 

* On node 2:
  * Join the just created cell:
    `exo -d ./node2 cell join`
    and then copy cell's config in editor.

* Start both nodes:
  * Node 1: `exo -d ./node1 daemon`
  * Node 2: `exo -d ./node2 daemon`

### Launch sample web project
* Run the [web example](./examples/web):
  * Build WASM client
    * `./clients/web/tools/build.sh`
  * Start development server which will watch files and rebuild automatically:
    * `cd ./examples/web && npm install && npm run start`
  * Generate cell configuration for web:
    * Follow [Quick start](#quick-start) as if web was another node, without a `chain` and `store` role.
    * Then convert config to JSON: `exo -d ./web/node config print --inline --format json`
  * Open browser to [http://127.0.0.1:8080](http://127.0.0.1:8080)
    * Paste JSON config
    * Remove listen addresses
    * Save

## Clients
#### Web
* See [Web WASM client README](./clients/web/README.md)

#### C
* See [C client README](./clients/c/README.md)

#### iOS
* See [iOS client README](./clients/ios/README.md)
  
## Documentation
* [Replication](chain/replication.md)
