# [egui](https://github.com/emilk/egui) bindings for [miniquad](https://github.com/not-fl3/miniquad)

[![Latest version](https://img.shields.io/crates/v/egui-miniquad.svg)](https://crates.io/crates/egui-miniquad)
[![Documentation](https://docs.rs/egui-miniquad/badge.svg)](https://docs.rs/egui-miniquad)
[![Build Status](https://github.com/not-fl3/egui-miniquad/workflows/CI/badge.svg)](https://github.com/not-fl3/egui-miniquad/actions?workflow=CI)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

## native

On Linux you first must run `apt install libx11-dev libxi-dev libgl1-mesa-dev` (miniquad dependencies).

`cargo run --release --example demo`

## Compiling for the web

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page. For this you need to set up some tools. There are a few simple scripts that help you with this:

``` sh
./setup_web.sh
./build_web.sh
./start_server.sh
open http://127.0.0.1:8080/
```

* `setup_web.sh` installs the tools required to build for web
* `build_web.sh` compiles your code to wasm and puts it in the `docs/` folder (see below)
* `start_server.sh` starts a local HTTP server so you can test before you publish
* Open http://127.0.0.1:8080/ in a web browser to view

The finished web app is found in the `docs/` folder (this is so that you can easily share it with [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site)).

You can try the demo in this repository by visiting <https://not-fl3.github.io/egui-miniquad/>.
