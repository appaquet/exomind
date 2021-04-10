# Gmail integration

## To run

1) Boot and join a new node to your cell.
    * `exo -d ./gmail_node node init`
    * `exo -d ./gmail_node cell join` (and `exo cell node add` on the main node, with no roles)

2) Copy gmail daemon config to your node directory.
    * `cp ../../examples/gmail.conf ./gmail_node/`

3) Visit [Google API Console](https://console.developers.google.com/) to obtain a OAuth 2.0 Client ID and 
  download `client_secret.json` credentials file. Move file in the node folder (ex: `./gmail_node`)

4) Login to your Gmail account (at repo root) and follow console instructions to authenticate with Gmail.
    * `exomind-gmail ./gmail_node login <youremail@gmail.com>` 

5) Start the deaemon.
    * `exomind-gmail ./gmail_node daemon`