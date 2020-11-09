
# Web client

## Dependencies
* Install Node & Yarn
    * [Install Node](https://github.com/nodesource/distributions/blob/master/README.md#debinstall)
    * Install yarn: `npm install -g yarn`

## Development
* `yarn start_dev`

* Configure a new Exocore node, add it to the cell and create an inlined node config:

  `exo -d ./node/path config print --inline --format json`

* Browse to [http://127.0.0.1:8080](http://127.0.0.1:8080), and use this config to bootstrap.


## Known issues
* Connection can only be via localhost or https since WebCrypto used in libp2p's secio implementation only works over secure code.
  * See [Exocore web client known issues](https://github.com/appaquet/exocore/tree/master/clients/web#notes)
