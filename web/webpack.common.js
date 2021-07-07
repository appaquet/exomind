const webpack = require("webpack");
const packageJson = require('./package.json');

module.exports = {
  module: {
    rules: [
      {
        test: /\.(tsx?|js?)$/,
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
        test: /\.css$/,
        use: ['style-loader', 'css-loader'],
      },

      {
        test: /\.less$/,
        use: ['style-loader', 'css-loader', 'less-loader'],
      },

      { test: /\.gif$/, loader: 'url-loader?limit=10000&mimetype=image/gif' },
      { test: /\.jpg$/, loader: 'url-loader?limit=10000&mimetype=image/jpg' },
      { test: /\.png$/, loader: 'url-loader?limit=10000&mimetype=image/png' },
      { test: /\.svg$/, loader: 'url-loader?limit=10000&mimetype=image/svg+xml' },
      { test: /\.woff(\?.*$|$)/, loader: "url-loader?limit=10000&mimetype=application/font-woff" },
      { test: /\.woff2(\?.*$|$)/, loader: "url-loader?limit=10000&minetype=application/font-woff" },
      { test: /\.ttf(\?.*$|$)/, loader: "file-loader" },
      { test: /\.eot(\?.*$|$)/, loader: "file-loader" },
    ]
  },

  resolve: {
    extensions: ['.js', '.jsx', '.ts', '.tsx']
  },


  plugins: [
    // keep in sync with webpack.electron.extra.js
    new webpack.DefinePlugin({
      '_EXOMIND_VERSION': JSON.stringify(packageJson.version),
      '_EXOMIND_BUILD_TIME': JSON.stringify(new Date().getTime())
    })
  ]
};
