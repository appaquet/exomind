# Exocore

[![codecov](https://codecov.io/gh/appaquet/exocore/branch/master/graph/badge.svg?token=OKZAHfPlaP)](https://codecov.io/gh/appaquet/exocore)
![Build](https://github.com/appaquet/exocore/workflows/Push%20tester/badge.svg)

**Warning: Exocore is at a very early development stage, hence incomplete, unstable, and probably totally unsafe. Use at your own risk.**

Exocore is a distributed applications framework with private and encrypted data storage. Think of it as an infrastructure that allows
a user to own his own personal cloud that is extensible via WebAssembly applications and accessible via Web/Mobile/Backend SDKs. It is
designed to be resilient to failures and will eventually allow offline usage (ex: on mobile).

Exocore is primarily built for [Exomind](https://github.com/appaquet/exomind), a personal knowledge management tool built in parallel
to this project. Exocore is the application framework for Exomind.

The primary concept in Exocore is a Cell, which is a unique container for a user's applications and data.

A cell consists of:

* Chain nodes manage replication and storage by using a blockchain data structure.
* Store nodes manage indexation, querying, and mutation of the data (collocated with chain node).
* Application host nodes run applications written in WebAssembly (collocated with store nodes)

## Roadmap

### v0.1 (in progress)

* **Chain storage and replication**: Proof of concept
* **Transport**: Proof of concept
* **Entity store**: Proof of concept
* **Applications (WASM host)**:  Proof of concept

### v0.2

* **Cell management** (Configuration replication)
* **Enhanced security** (Chain encryption, configuration signatures, etc.)

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

* [Protobuf](https://protobuf.dev/)
  * On MacOS: `brew install protobuf swift-protobuf`
  * On Ubuntu: `apt install protobuf-compiler`

## Usage & configuration

* CLI:
  * `./tools/install.sh` or `cd exo && cargo install --path .` or grab latest released binary.

* Configuration
  * Most commands require a node configuration file, for which an example can be found here: [`./examples/node.yaml`].
      `exo` can also generate and manage configurations. See [Quick start](#quick-start).
  * At a minimum, the config requires 2 keypairs: one for the node, one for the cell.
  * The node keypair is unique per node, while the cell keypair is shared among servers that host the cell.
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

  * Request to join the cell as a chain and store node.
    This will use exocore's discovery server (`disco.exocore.io`) to exchange configurations:

    `exo -d ./node2 cell join --chain --store`

    and copy the displayed discovery PIN.

* On node 1:
  * Add node 2 to cell:

    `exo -d ./node1 cell node add`

    Paste node 2's discovery PIN and accept its join request.

* Start both nodes:
  * Node 1: `exo -d ./node1 daemon`
  * Node 2: `exo -d ./node2 daemon`

### Join the example web client

* See [Web example README](./examples/web/README.md#Running)

### Install & run Exomind

* See [Exomind README](https://github.com/appaquet/exomind)

## Clients

### Web

* See [Web WASM client README](./clients/web/README.md)

### C

* See [C client README](./clients/c/README.md)

### iOS

* See [iOS client README](./clients/ios/README.md)
  
## Documentation

* [Replication](chain/replication.md)
