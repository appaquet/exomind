# Web WASM build

* Install Rust's WASM target
    * `rustup target add wasm32-unknown-unknown`
    
* Install Clang
    * On Ubuntu: 
        * `apt install clang`
    * On MacOS: 
        * Unfortunately, clang installed by Xcode isn't recent enough to compile to WASM. Follow instructions on 
          [this page](https://00f.net/2019/04/07/compiling-to-webassembly-with-llvm-and-clang/)
          to install LLVM 9 from HomeBrew.
            * `brew install llvm`
            * Use LLVM from HomeBrew:
                * Bash `export PATH=/usr/local/opt/llvm/bin:$PATH`
                * Fish `set -g fish_user_paths "/usr/local/opt/llvm/bin" $fish_user_paths`

* Install [Node & NPM](https://github.com/nodesource/distributions/blob/master/README.md#debinstall)

* To generate proto: 
    * Make sure `protoc` is installed. See root's [README](../../README.md).
    * Then `./generate.sh`

* To package WASM:
    * Install [`wasm-pack`](https://github.com/rustwasm/wasm-pack) to build and package WASM as NPM package
        * `cargo install wasm-pack`
    * Then `wasm-pack build --release`

* Launch the [sample project](../../examples/web) 
    * `cd ../../examples/web && npm install && npm run start`
    * Navigate to [http://localhost:8080](http://localhost:8080)
