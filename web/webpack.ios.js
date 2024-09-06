const { merge } = require('webpack-merge');
const common = require('./webpack.common.js');
const HtmlWebPackPlugin = require('html-webpack-plugin');

module.exports = merge(common, {
  mode: 'production',

  entry: {
    store: ["./src/ios/store.js"],
    hybrid: ["./src/ios/hybrid.js"]
  },
  devServer: {
    writeToDisk: true
  },
  plugins: [
    new HtmlWebPackPlugin({
      inject: true,
      chunks: ['hybrid'],
      filename: 'hybrid.html',
      template: './src/ios/hybrid.html'
    })
  ]
});