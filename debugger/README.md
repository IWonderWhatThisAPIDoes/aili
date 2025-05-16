# Aili-Debugger

Demo Electron application that showcases the full Aili pipeline.

This application can be used to debug real C programs.
[Stylesheets](../style) can be edited in real time
with diagnostic messages, and applications can be stepped
through, like in a typical debugger.

## Build

The application can be packaged into distributable binary.
Make sure that [Aili-JSAPI](../jsapi/README.md#generate-node-package)
is built with the GDBState feature enabled.

```sh
npm run build
```

## Development Server

The application can also be run on a development server.
Again, [Aili-JSAPI](../jsapi/README.md#generate-node-package)
must be built with the GDBState feature.

```sh
npm run start
```

## Examples

The `examples` directory contains an assortment of small programs
that can be visualized with the Aili debugger.
Stylesheets and (if needed) memory hint sheets created for the specific
programs are also included.

The following examples are present:

- AVL tree insertion
- Breadth-first traversal of a graph structure
- Binary search over an array
- Bubble sort over an array
- Depth-first traversal of a graph structure
- Construction and traversal of a linked list
- Dynamically-sized array container

A general-purpose (non-semantic) stylesheet is also included
that can be used with all of the programs.

## Documentation

The following command generates documentation and saves it to the `doc` directory.
```sh
npm run doc
```
