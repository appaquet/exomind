/* eslint-env commonjs */

const webpack = require("webpack");
const packageJson = require('./package.json');

// from https://gist.github.com/earksiinni/053470a04defc6d7dfaacd5e5a073b15 
module.exports = {
  devtool: "inline-source-map",

  output: {
    libraryTarget: "window",
  },

  target: "web",

  plugins: [
    // keep in sync with webpack.electron.extra.js
    new webpack.DefinePlugin({
      '_EXOMIND_VERSION': JSON.stringify(packageJson.version),
      '_EXOMIND_BUILD_TIME': JSON.stringify(new Date().getTime())
    })
  ]
};