# Browsers extension

## Chrome

* Follow README of [web client](../web/README.md) to build web dependencies.

* Download `protobufjs` lib:

  `wget -O chrome/protobuf.min.js https://raw.githubusercontent.com/protobufjs/protobuf.js/master/dist/protobuf.min.js`

* Generate protobuf protos:

  `./tools/generate_protos.js`

* [Load the unpacked extension in Chrome](https://developer.chrome.com/docs/extensions/get-started)

* Configure the extension by triggering it and filling the following fields:
  * `Endpoint` is the URL to a `store` node exposing HTTP transport.

  * `Auth token` is a token that authenticate the extension's calls gainst the endpoint.
      Generate a token using Web client's configuration:

      `exo cell generate-auth-token`
