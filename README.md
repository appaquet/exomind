# Exocore
[![codecov](https://codecov.io/gh/appaquet/exocore/branch/master/graph/badge.svg?token=OKZAHfPlaP)](https://codecov.io/gh/appaquet/exocore)
![Build](https://github.com/appaquet/exocore/workflows/Push%20tester/badge.svg)

**Warning: Exocore is at a very early development stage, hence incomplete, unstable and probably totally unsafe. Use at your own risk.**

Exocore is a distributed applications framework with private and encrypted data storage. Think of like an infrastructure that allows
a user to own his own personal cloud that is extensible via WebAssembly applications and accessible via Web/Mobile SDKs. It is designed 
to be resilient to failures, allow offline usage (ex: on mobile). 

Exocore is primarily built for [Exomind](https://github.com/appaquet/exomind), a personal knowledge management tool built in parallel
of this project. Exocore is the application framework for Exomind.

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

### v0.2
* **Cell management**
* **Applications**  (WebAssembly)
* **Encryption**

### v0.3 and beyond
* **Android SDK**
* **Blob storage**  (IPFS)
* **Offline support**


## Dependencies
* Build dependencies
    * On MacOS: Install Xcode and command lines tools
    * On Ubuntu: `apt install build-essential pkg-config libssl-dev`

* [Rust](https://www.rust-lang.org/learn/get-started)
  * Install using [rustup](https://www.rust-lang.org/learn/get-started)
  * Install `clippy` and `rustfmt`: `rustup component add clippy rustfmt`
  
* [Cap'n Proto](https://capnproto.org/install.html)
    * On MacOS: `brew install capnp` 
    * On Ubuntu: `apt install capnproto` 

* [Protobuf](https://developers.google.com/protocol-buffers/)
    * On MacOS: `brew install protobuf swift-protobuf` 
    * On Ubuntu: `apt install protobuf-compiler` 
    

## Usage & configuration
* CLI:
  * `./tools/install.sh` or `cd exo && cargo install --path .` or grab latest released binary.

* Configuration
    * Most commands requires a node configuration file, for which an example can be found in here: [`./examples/node.yaml`].
      `exo` can also generate and manage configurations. See [Quick start](#quick-start).
    * At minimum, the config requires 2 keypair: one for the node, one for the cell.
    * The node keypair is unique per server/node, while the cell keypair is shared among servers that host the cell.
    * See [Quick start](#quick-start) section for example 2 nodes setup.
    
## Quick start

### Create a Cell hosted on 2 nodes
* On node 1
  * Generate configuration: 

    `exo --dir ./node1 node init --name node1`

  * Edit configuration to include unique and accessible addresses:

    `exo -d ./node1 config edit`

  * Generate a cell:

    `exo -d ./node1 cell init --name my_cell`

* On node 2
  * Generate configuration: 

    `exo --dir ./node2 node init --name node1`

  * Edit configuration to include unique and accessible addresses. 
    If both nodes are running on the same machine, make sure they have unique ports.

    `exo -d ./node2 config edit`

  * Request to join the cell. 
    This will use exocore's discovery server (`disco.exocore.io`), but this can overriden:

    `exo -d ./node2 cell join`

    and copy the displayed discovery PIN.

* On node 1:
  * Add node 2 to cell:

    `exo -d ./node1 cell node add --chain --store` 

    Paste node 2's discovery PIN.

* Start both nodes:
  * Node 1: `exo -d ./node1 daemon`
  * Node 2: `exo -d ./node2 daemon`

### Join the example web client
* See [Web example README](./examples/web/README.md#Running)

### Install & run Exomind
* See [Exomind README](https://github.com/appaquet/exomind)

## Clients
#### Web
* See [Web WASM client README](./clients/web/README.md)

#### C
* See [C client README](./clients/c/README.md)

#### iOS
* See [iOS client README](./clients/ios/README.md)
  
## Documentation
* [Replication](chain/replication.md)
