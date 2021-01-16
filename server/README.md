# Exomind server

Note: This server is a temporary solution to run Exomind's logic. Ultimately, Exomind's logic will run in WASM in Exocore.

## To run

* Bootstrap a node & make it join the cell.
  (see [Exocore's quick start](https://github.com/appaquet/exocore#quick-start)) 

* Install Exomind in the created cell.
  * Download the [release](https://github.com/appaquet/exomind/releases) of the app package.
  * Install it `exo cell app install --path exomind-app.zip`
  * Or you can install via the URL  `exo cell app install --url https://github.com/appaquet/exomind/releases/download/<VERSION>/exomind-app.zip`

* Copy `examples/server.yaml` to the directory of the node. 
  Adjust the configuration.

  For Gmail integration, see the [README](../integrations/gmail/README.md).

* Start the server `cargo run -p exomind-server -- -c path/to/server.yaml start`.