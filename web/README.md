
# Web client

## Dependencies
* Install Node & Yarn
    * [Install Node](https://github.com/nodesource/distributions/blob/master/README.md#debinstall)
    * Install yarn: `npm install -g yarn`

## Development
### Web client
* Start in developer mode: `yarn start_dev` 
* Browse to [http://127.0.0.1:8080](http://127.0.0.1:8080) and copy the displayed discovery code.
* On the main node, add the web node with `exo cell node add`, paste the discovery code. Restart your main node.

#### Note
When using the web client, connections can only be via localhost or https since WebCrypto used 
in libp2p's secio implementation only works over secure code. See [Exocore web client known issues](https://github.com/appaquet/exocore/tree/master/clients/web#notes).

### Electron client
* Build an Electron client: `yarn electron`, watch output for path to created binary (depends on the platform).
* Launch the created binary.
* Note the discovery code shown in the Electron app.
* On the main node, add the web node with `exo cell node add`, paste the discovery code. Restart your main node.