# Aili-JSAPI

Provides a high-level interface between modules written in Rust and Javascript.

## Features

Some features are not included in the package by default.
These have to be enabled with their respective
[Cargo features](https://doc.rust-lang.org/cargo/reference/features.html).

### `gdbstate`

API of the [GDBState](../gdbstate) package is not exposed by default.
Enabling this feature adds it to the package.

## Generate Node Package

This module must be built using [wasm-pack](https://crates.io/crates/wasm-pack)
to create a Node package which can then be included in Javascript-based modules.

```sh
wasm-pack build --target bundler
```

To build with the GDBState feature, add it to the command line.

```sh
wasm-pack build --target bundler --features gdbstate
```

## Documentation

The following command generates documentation and saves it
in the target directory.

```sh
cargo doc --no-deps
```
