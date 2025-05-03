# Aili-Translate

Bridges the gap between Aili State model and Aili Visualization model
by statefully converting between the two based on a stylesheet
that describes the mapping between the models.

## Using Aili-Translate

Aili-Translate stands in the middle of an application, so it is usualy
not used on its own. Here is a small example that statelessly
resolves a [stylesheet](../style) and displays the result in a Vis tree.

Both State graph and Vis tree would be provided by the back end
and front end, respectively. A stylesheet may be constructed
manually, although it would most likely be done using
[Aili-Parser](../parser).

```rust
use aili_model::{state::RootedProgramStateGraph, vis::VisTree};
use aili_style::{
    cascade::style::CascadeStyle,
    selectable::Selectable,
    stylesheet::Stylesheet,
};
use aili_translate::{
    cascade::apply_stylesheet,
    forward::Renderer,
    property::PropertyKey,
};

fn translate(
    state: impl RootedProgramStateGraph,
    vis: impl VisTree,
    stylesheet: Stylesheet<PropertyKey>,
) {
    // Compile the stylesheet so that Translate can use it
    let compiled_stylesheet = CascadeStyle::from(stylesheet);

    // Evaluate the stylesheet on a State graph
    // to determine the desired appearence of the Vis tree
    let mapping = apply_stylesheet(&compiled_stylesheet, &state);

    // Create a renderer that will paste the desired properties
    // into the Vis tree
    let mut renderer = Renderer::new(vis);

    // First let the renderer know which node maps
    // to the root of the Vis tree
    renderer.update_root(Some(Selectable::node(state.root())));

    // Fill in the Vis tree and style it according to the stylesheet
    //
    // This creates new Vis entities that correspond to visible
    // State entities. The renderer is stateful so this can be called
    // multiple times without recreating the entire Vis tree.
    renderer.update(mapping);
}
```

## Documentation

The following command generates documentation and saves it
in the target directory.

```sh
cargo doc --no-deps
```
