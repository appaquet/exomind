
# Web client

## Development
* `yarn start_dev`

* Configure a new Exocore node, add it to the cell and create a standalone node config:

  `exo config standalone path/to/web/config.yaml --exclude-app-schemas json`

* Browse to [http://127.0.0.1:8080](http://127.0.0.1:8080), and use this config to bootstrap.


## Known issues
* Connection can only be via localhost or https since WebCrypto used in libp2p's secio implementation only works over secure code.
  * See [Exocore web client known issues](https://github.com/appaquet/exocore/tree/master/clients/web#notes)
