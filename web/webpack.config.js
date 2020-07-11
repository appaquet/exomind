const HtmlWebPackPlugin = require('html-webpack-plugin');

module.exports = {
  entry: {
    web: ["./src/index.js"],
    hybrid: ["./src/ios/hybrid.js"],
    store: ["./src/ios/store.js"]
  },

  devtool: 'eval-cheap-module-source-map', // TODO: disable prod https://webpack.js.org/configuration/devtool/ and https://webpack.js.org/guides/production/

  devServer: {
    disableHostCheck: true, // to accept localhost.exomind.io
    historyApiFallback: true, // allow history push on front-end
    clientLogLevel: 'debug',
    open: false,
    proxy: {
      '/v1': {
        target: 'https://exomind.io',
        secure: false,
        changeOrigin: true,
        ws: true
      }
    }
  },

  module: {
    rules: [
      {
        test: /\.(js|jsx)$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader'
        }
      },

      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/
      },

      {
        test: /\.html$/,
        use: [
          {
            loader: 'html-loader'
          }
        ]
      },

      {
        test: /\.css$/i,
        use: ['style-loader', 'css-loader'],
      },

      {
        test: /\.less$/,
        use: ['style-loader', 'css-loader', 'less-loader'],
      },

      { test: /\.gif/, loader: 'url-loader?limit=10000&mimetype=image/gif' },
      { test: /\.jpg/, loader: 'url-loader?limit=10000&mimetype=image/jpg' },
      { test: /\.png/, loader: 'url-loader?limit=10000&mimetype=image/png' },
      { test: /\.svg/, loader: 'url-loader?limit=10000&mimetype=image/svg+xml' },
      { test: /\.woff(\?.*$|$)/, loader: "url-loader?limit=10000&mimetype=application/font-woff" },
      { test: /\.woff2(\?.*$|$)/, loader: "url-loader?limit=10000&minetype=application/font-woff" },
      { test: /\.ttf(\?.*$|$)/, loader: "file-loader" },
      { test: /\.eot(\?.*$|$)/, loader: "file-loader" },

    ]
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
    },
  },
  resolve: {
    extensions: ['.js', '.jsx', '.ts', '.tsx']
  },
  plugins: [
    new HtmlWebPackPlugin({
      inject: true,
      chunks: ['web'],
      filename: 'index.html',
      template: './src/index.html'
    }),
    new HtmlWebPackPlugin({
      inject: true,
      chunks: ['hybrid'],
      filename: 'hybrid.html',
      template: './src/ios/hybrid.html'
    })
  ]
};
