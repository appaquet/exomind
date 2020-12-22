# Exomind server

Note: This server is a temporary solution to run Exomind's logic. Ultimately, Exomind's logic will run in WASM in Exocore.

## To run

* Bootstrap a node & make it join the a cell.
  (see [Exocore's quick start](https://github.com/appaquet/exocore#quick-start)) 

* Copy `examples/server.yaml` to the directory of the node. 
  Adjust the configuration.

  For Gmail integration, see the [README](../integrations/gmail/README.md).

* Start the server `cargo run -p exomind-server -- -c path/to/server.yaml`.