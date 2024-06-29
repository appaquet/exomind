# Web client example

## Dependencies
* Install dependencies for [Web client](../../clients/web/README.md#Dependencies)

## Running
* Build WASM web client
  
  `../../clients/web/tools/build.sh`

* Launch example server which will watch files and rebuild automatically:

  `npm install && npm run start`

  Note: `npm` is used here instead of `yarn` because we depend on `exocore` via its path and that `yarn` has the bad habit of copying all files in the `exocore` directory, including its massive target directory, because it bypasses ignored files. See [this issue](https://github.com/yarnpkg/yarn/issues/2822).

* Open browser to [http://127.0.0.1:8080](http://127.0.0.1:8080)
    * Copy the displayed discovery PIN.
    * On the main node, add node `exo cell node add` and enter paste discovery PIN.

## End to end testing

* Install Playwright dependencies `npx playwright install --with-deps chromium`

* Run test server in background `exo -d tests/node daemon &`

* Run tests `./tools/e2e.sh`
