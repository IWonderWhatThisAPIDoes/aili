<img src="./assets/logo.png" width="400" alt="Aili" />

Prototype semantic visual debugger.

## Modules in This Repository

![Overview of modules in the repository](./assets/overview.png)

| Module                          | Description                                                     |
|---------------------------------|-----------------------------------------------------------------|
| [:yellow_square: Debugger](./debugger) | Demo application that showcases the whole debugger pipeline. |
| [:yellow_square: Demo](./demo)  | Demo application that showcases Vis and Translate modules.      |
| [:crab: GDBState](./gdbstate)   | Implementation of the Program State model for C that uses the [GNU Project Debugger](https://www.sourceware.org/gdb). |
| [:yellow_square: Hooligan](./hooligan) | Hooking and logging utilities.                           |
| [:yellow_square::crab: JSAPI](./jsapi) | Bindings between modules written in Rust and Javascript. |
| [:crab: Model](./model)         | Definitions of Program State and Visualization models.          |
| [:crab: Parser](./parser)       | Parser for stylesheets of the Style module.                     |
| [:crab: Style](./style)         | Stylesheets that assign properties to State entities            |
| [:crab: Translate](./translate) | Translator for converting between Program state and Visualization models based on a stylesheet that describes the mapping. |
| [:yellow_square: Vis](./vis)    | Browser-based renderer of Visualization model, written in Typescript. |
