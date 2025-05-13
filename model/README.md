# Aili-Model

Defines the abstract models that form the interface between
the main modules of Aili.

## State Graph

The State graph model represents the internal state of a debuggee
with a language-independent graph structure.

![State graph of a C implementation of vector](../assets/state.png)

## Visualization Tree

The Visualization tree model represents a scene that should be presented
to a user.

## Documentation

The following command generates documentation and saves it
in the target directory.

```sh
cargo doc --no-deps
```
