# Aili-Parser

Parser for a human-readable and -writable representation
of the stylesheets used by [Aili-Translate](../translate).

## Using Aili-Parser

The module provides a single function for parsing a stylesheet.

```rust
use aili_parser::parse_stylesheet;

let source = ":: { display: graph; }";

let stylesheet = parse_stylesheet(source, |e| eprintln!("Parser has recovered from a syntax error: {e}"))
    .expect("Parser has encountered an irrecoverable error");
```

## Stylesheet Syntax

The stylesheets describe how individual entities in the Aili State graph
should be represented in the Aili Vis tree.
As the name suggests, the stylesheets are heavily inspired
by [CSS](https://developer.mozilla.org/docs/Web/CSS), so if you are familiar
with that, you may be able to draw some parallels. Nontheless,
there are notable differences arising from the fact that the State graph
is not a tree, and that in some regards, the stylesheet needs to be more flexible
than CSS.

A stylesheet is a series of rules of the form
```text
<selector> { <clause> ... }
```
where `selector` describes a pattern that determines whether the rule
applies to each entity in the State graph, and `clause`s specify
the visual properties that affect how the entity will be represented
in the Vis tree.

### Selectors

TODO

### Clauses

A clause has the following form.
```text
<lvalue> : <expression> ;
```

The semicolon after the last clause of a rule may be omited.

An `lvalue` is an identifier of a property that can be set by the stylesheets.
The following properties are currently available:

#### Property `display`

Affects whether the entity will be represented in the Vis tree,
and if so, what kind of visual entity will be used.

The value assigned to this property is coerced to a string.
If a value of `"connector"` is assigned, the entity will be represented
by a connector in the Vis tree. Otherwise, the entity will be represented
by an element with tag name matching the assigned string.

If a value of `none` or `unset` is assigned, the entity will not be rendered.
This is the default value.

```text
:: {
    display: graph;  // Will be displayed as an element with tag name "graph"
}

:: {
    display: connector;  // Will be displayed as a connector
}

:: {
    display: none;  // Will not be displayed
}
```

#### Properties `parent` and `target`

Specifies the placement of the entity's representation within the Vis tree.

By default, the structure of the Vis tree reflects that of the State graph[^1].
This is not always desirable, as some visualizations have different structure
than the program they represent. In those cases, these properties may be used
to override the placement of both elements and connectors.

These properties only affect visual placements. Stylesheet resolution
is not affected in any way.

Both properties accept references (see [Select Expressions](#select-expressions))
to entities with [`display`](#property-display)
set to a value other than `connector`. If anything else is assigned
(including references to other entities), the property is set to `none`,
which has the same effect as setting `display` to `none`.
It is also possible to use [variables](#variables) that contain the aforementioned
references.

If `display` is `connector`, `parent` and `target` specify the respective
endpoints of the connector that represents the entity.
Otherwise, `parent` specifies the parent element and `target` is ignored.

```text
:: {
    // Store the root node in a variable
    --root: @;
}

* {
    // All other nodes will be direct children of the root node,
    // regardless of the state graph structure
    parent: --root;
}

:ref {
    // Display reference nodes as connectors
    display: connector;
    // Parent is already implicitly set, we set the target
    // to the actual target that the reference node points to
    target: @(ref);
}
```

[^1]: Or, rather, a spanning tree thereof, since the State graph is not a tree.

#### Variables

Identifiers that start with a `--` (double dash) are interpreted as variable names.
Values assigned to these may be recalled by following clauses in the same rule or,
in some cases, by the selectors and clauses of other rules
(see [Variable Visibility](#variable-visibility) for a detailed explanation
of where variables may be recalled).

```text
:: {
    --a: red;    // Set a variable
    color: --a;  // ...and recall it later
}

:: main {
    color: --a;  // In some cases, variables may be recalled
                 // by other rules, although this is not always guaranteed
}
```

#### Attributes

If the `lvalue` is any identifier except one of the above, or any quoted string literal,
the right-hand side is coerced to a string and assigned to the matching attribute
of the Vis element. If a value of `none` or `unset` is used, the attribute is removed.

```text
:: {
    stroke: black;  // Value of "black" will be assigned to the "stroke" attribute
    fill: none;     // The "fill" attribute will be removed
}
```

#### Fragment Attributes

TODO

### Expressions

TODO

#### Select Expressions

TODO

### Variable Visibility

TODO

## Documentation

The following command generates documentation and saves it
in the target directory.

```sh
cargo doc --no-deps
```
