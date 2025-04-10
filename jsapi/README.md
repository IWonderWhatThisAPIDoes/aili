# Aili-JSAPI

Provides a high-level interface between modules written in Rust and Javascript.

## Generate Node Package

This module must be built using [wasm-pack](https://crates.io/crates/wasm-pack)
to create a Node package which can then be included in Javascript-based modules.

```sh
wasm-pack build --target bundler
```

## Documentation

The following command generates documentation and saves it
in the target directory.

```sh
cargo doc --no-deps
```
