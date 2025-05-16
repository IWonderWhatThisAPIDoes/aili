# Aili-Parser

Parser for a human-readable and -writable representation
of the stylesheets defined by [Aili-Style](../style).

See the [stylesheet authors' manual](../doc/stylesheets.md)
for more information about how stylesheets are used.

## Using Aili-Parser

The module provides a single function for parsing a stylesheet.

```rust
use aili_parser::parse_stylesheet;

let source = ":: { display: graph; }";

let stylesheet = parse_stylesheet(source, |e| eprintln!("Parser has recovered from a syntax error: {e}"))
    .expect("Parser has encountered an irrecoverable error");
```

## Documentation

The following command generates documentation and saves it
in the target directory.

```sh
cargo doc --no-deps
```
