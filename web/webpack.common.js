
module.exports = {
  experiments: {
    // WebAssembly is disabled in webpack 5 by default
    syncWebAssembly: true
  },

  module: {
    rules: [
      {
        test: /\.js?$/,
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
        test: /\.css$/i,
        use: ['style-loader', 'css-loader'],
      },

      {
        test: /\.less$/,
        use: ['style-loader', 'css-loader', 'less-loader'],
      },

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

      { test: /\.(ttf|eot)(\?.*$|$)/, loader: "file-loader" },
    ]
  },

  resolve: {
    extensions: ['.js', '.jsx', '.ts', '.tsx']
  },
};
