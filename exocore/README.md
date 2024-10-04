# Exocore

**Warning: Exocore/exomind a personal project. I daily drive it, but it may not be stable for your use case.**

Exocore is a distributed applications framework with private and encrypted data storage. It allows users to own their personal cloud, which is extensible via WebAssembly applications and accessible through Web, Mobile, and Backend SDKs. It is designed to be resilient to failures and will eventually support offline usage (e.g., on mobile).

Exocore is primarily built for [Exomind](../exomind/README.md), a personal knowledge management tool developed in parallel. Exocore serves as the application framework for Exomind.

The primary concept in Exocore is a Cell, a unique container for a user's applications and data.

A cell consists of:

* **Chain nodes**: Manage replication and storage using a blockchain data structure.
* **Store nodes**: Handle indexation, querying, and mutation of data (collocated with chain nodes).
* **Application host nodes**: Run applications written in WebAssembly (collocated with store nodes).

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
