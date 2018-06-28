const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const UnusedWebpackPlugin = require('unused-webpack-plugin');

module.exports = {
    mode: "development",
    devtool: 'source-map',
    entry: './src/index.ts',
    output: {
        path: path.resolve('target'),
        filename: 'bundle.js'
    },
    resolve: {
        modules: [
            path.resolve(__dirname, './src'),
            path.resolve(__dirname, './node_modules')
        ],
        extensions: [".ts"]
    },
    module: {
        rules: [
            { test: /\.ts$/, loader: "ts-loader", exclude: /node_modules/ }
        ]
    },
    plugins: [
        new CopyWebpackPlugin([
            { from: './static/*', to: './', flatten: true }
        ]),
        new UnusedWebpackPlugin({
            directories : [path.join(__dirname, 'src')]
        }),
    ]
};