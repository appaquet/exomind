# Gmail integration

## To run

* Follow [server README](../../server/README.md).

* Visit [Google API Console](https://console.developers.google.com/) to obtain a OAuth 2.0 Client ID and 
  download `client_secret.json` credentials file. Move file in the node folder.

* Edit the gmail config in the server configuration.

* Login to your Gmail account (at repo root) 
  `cargo run -p exomind-server -- -c path/to/server.yaml login <youremail@gmail.com>` 
  and follow console instruction for authentication with Gmail.

* Start server.