
module.exports = {
  experiments: {
    syncWebAssembly: true
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

      // { test: /\.gif/, loader: 'url-loader?limit=10000&mimetype=image/gif' },
      // { test: /\.jpg/, loader: 'url-loader?limit=10000&mimetype=image/jpg' },
      // { test: /\.png/, loader: 'url-loader?limit=10000&mimetype=image/png' },
      // { test: /\.svg/, loader: 'url-loader?limit=10000&mimetype=image/svg+xml' },
      // { test: /\.woff(\?.*$|$)/, loader: "url-loader?limit=10000&mimetype=application/font-woff" },
      // { test: /\.woff2(\?.*$|$)/, loader: "url-loader?limit=10000&minetype=application/font-woff" },
      {
        test: /\.(png|jpg|gif|svg|woff|woff2)$/i,
        use: [
          {
            loader: 'url-loader',
            options: {
              limit: 8192,
            },
          },
        ],
      },
      { test: /\.ttf(\?.*$|$)/, loader: "file-loader" },
      { test: /\.eot(\?.*$|$)/, loader: "file-loader" },
    ]
  },
  resolve: {
    extensions: ['.js', '.jsx', '.ts', '.tsx']
  },
};
