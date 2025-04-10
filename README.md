<img src="./assets/logo.png" width="400" alt="Aili" />

Prototype semantic visual debugger.

## Modules in This Repository

| Module                   | Description                                                           |
|--------------------------|-----------------------------------------------------------------------|
| [Demo](./demo)           | Demo application that showcases Vis and Translate modules.            |
| [JSAPI](./jsapi)         | Bindings between modules written in Rust and Javascript.              |
| [Model](./model)         | Definitions of Program State and Visualization models.                |
| [Parser](./parser)       | Parser for stylesheets of the Translate module.                       |
| [Translate](./translate) | Translator for converting between Program state and Visualization models based on a stylesheet that describes the mapping. |
| [Vis](./vis)             | Browser-based renderer of Visualization model, written in Typescript. |
