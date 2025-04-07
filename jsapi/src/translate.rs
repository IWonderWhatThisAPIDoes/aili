//! Stylesheet resolution and updating of the visualization tree.

use crate::{
    log::{Logger, Severity},
    state::StateGraph,
    stylesheet::Stylesheet,
    vis::VisTree,
};
use aili_model::state::RootedProgramStateGraph as _;
use aili_translate::{
    forward::{Renderer, RendererWarning},
    property::Selectable,
};
use wasm_bindgen::prelude::*;

/// Program state renderer that renders into a given [`VisTree`].
///
/// Use with [˙apply_stylesheet˙].
#[wasm_bindgen]
pub struct VisTreeRenderer(Renderer<'static, usize, VisTree>);

#[wasm_bindgen]
impl VisTreeRenderer {
    /// Constructs a new renderer that renders into the provided [`VisTree`].
    #[wasm_bindgen(constructor)]
    pub fn new(tree: VisTree) -> Self {
        Self(Renderer::new(tree))
    }

    /// Sets the logger to which log messages from the renderer should be sent.
    #[wasm_bindgen(setter, js_name = "logger")]
    pub fn set_logger(&mut self, logger: Option<Logger>) {
        self.0.set_warning_handler(logger.map(|logger| {
            let handler = move |w| logger.log(Severity::Warning, &format!("{w}"));
            let boxed: Box<dyn FnMut(RendererWarning<usize>)> = Box::new(handler);
            boxed
        }));
    }

    /// Returns a human-readable representation of the current
    /// resolved style that the renderer has applied to the [`VisTree`].
    #[wasm_bindgen(js_name = "prettyPrint")]
    pub fn pretty_print(&self) -> String {
        format!("{:#?}", self.0)
    }
}

/// Resolves a [`Stylesheet`] over a [`StateGraph`] and renders
/// the result into a [`VisTreeRenderer`].
#[wasm_bindgen(js_name = "applyStylesheet")]
pub fn apply_stylesheet(
    stylesheet: &Stylesheet,
    graph: &StateGraph,
    renderer: &mut VisTreeRenderer,
) {
    let mapping = aili_translate::cascade::apply_stylesheet(&stylesheet.0, graph);
    renderer.0.update_root(Some(Selectable::node(graph.root())));
    renderer.0.update(mapping);
}
