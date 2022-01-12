const { merge } = require('webpack-merge');
const common = require('./webpack.common.js');
const HtmlWebPackPlugin = require('html-webpack-plugin');

module.exports = merge(common, {
  mode: 'development',

  entry: {
    web: ["./src/index.js"],
  },

  devtool: 'cheap-source-map',

  devServer: {
    disableHostCheck: true, // to accept localhost.exomind.io
    historyApiFallback: true, // allow history push on front-end
    clientLogLevel: 'debug',
    open: false,
    hot: true,
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
