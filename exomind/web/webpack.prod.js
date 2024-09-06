const { merge } = require('webpack-merge');
const common = require('./webpack.common.js');
const HtmlWebPackPlugin = require('html-webpack-plugin');

module.exports = merge(common, {
  mode: 'production',

  entry: {
    web: ["./src/index.js"],
  },

  optimization: {
    splitChunks: {
      chunks: 'all',
    },
  },

  plugins: [
    new HtmlWebPackPlugin({
      inject: true,
      chunks: ['web'],
      filename: 'index.html',
      template: './src/index.html'
    })
  ]
});
