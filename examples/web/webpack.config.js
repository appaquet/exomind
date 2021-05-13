const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: 'development',

  entry: "./src/index.tsx",

  experiments: {
    // WebAssembly is disabled in webpack 5 by default
    syncWebAssembly: true
  },

  output: {
    path: dist,
    filename: "bundle.js"
  },

  devServer: {
    contentBase: dist,
  },

  module: {
    rules: [
      {
        test: /\.(tsx?|js?)$/,
        use: 'ts-loader',
        exclude: /node_modules/
      },
    ]
  },

  resolve: {
    extensions: ['.js', '.jsx', '.ts', '.tsx']
  },

  plugins: [
    new HtmlWebpackPlugin({
      template: 'index.html'
    })
  ]
};
