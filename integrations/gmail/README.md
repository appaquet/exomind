# Gmail integration

## To run

* Visit [Google API Console](https://console.developers.google.com/) to obtain a OAuth 2.0 Client ID and 
  download `client_secret.json` credentials file. Move file in the node folder.

* Copy `../../examples/gmail.conf` and edit to your liking.

* Boot and join a node.
  * `exo node init`
  * `exo cell join` (and `exo cell node add` on the master node)

* Login to your Gmail account (at repo root) 
  `cargo run -p exomind-gmail -- -c path/to/gmail.yaml -n path/to/node.yaml gmail login <youremail@gmail.com>` 
  and follow console instruction for authentication with Gmail.

* Start server: `cargo run -p exomind-gmail -- -c path/to/gmail.yaml -n path/to/node.yaml daemon`