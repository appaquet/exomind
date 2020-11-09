# Gmail integration

## To run

* Create a `local_conf` directory in this README's directory.

* Visit [Google API Console](https://console.developers.google.com/) to obtain a OAuth 2.0 Client ID and download `client_secret.json` credentials file. Place the `client_secret.json` file in `local_conf`.

* Create a new node your Exocore's cell (see [Exocore's quick start](https://github.com/appaquet/exocore#quick-start)) for the Gmail integration with no roles. Place this node's config (along the cell's config if it isn't a inlined config) in `local_conf/node.yaml`.

* Copy `examples/gmail.yaml` config to `local_conf` and configure it (should be good as-is).

* Login to your Gmail account `cargo run -- -c local_conf/gmail.yaml login <youremail@gmail.com>` and follow console instruction for authentication with Gmail.

* Start server `cargo run -- -c local_conf/gmail.yaml start`.