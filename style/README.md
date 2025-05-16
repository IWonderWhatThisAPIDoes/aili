# Aili-Style

Generic stylesheets that assign properties to nodes
of the [Aili State model](../model) based on the structure
of their surroundings in the State graph.

The stylesheets are mainly used by [Aili-Translate](../translate/)
to apply visual styles to elements. A small subset is also used
by [Aili-GDBState] to provide the backend with hints
about the length of dynamically allocated arrays.

This module contains the definitions of the stylesheets,
as well as basic utilities for their evaluation.

See the [stylesheet authors' manual](../doc/stylesheets.md)
for more information about how stylesheets are used.

## Documentation

The following command generates documentation and saves it
in the target directory.

```sh
cargo doc --no-deps
```
