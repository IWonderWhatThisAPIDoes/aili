//! Stylesheet resolution and updating of the visualization tree.

use crate::{
    log::{Logger, Severity},
    state::StateGraph,
    stylesheet::Stylesheet,
    vis::VisTree,
};
use aili_model::state::{ProgramStateGraph, RootedProgramStateGraph};
use aili_translate::{
    forward::{Renderer, RendererWarning},
    property::Selectable,
};
use wasm_bindgen::prelude::*;

/// Declares a renderer type for a given target.
///
/// State graphs are not dyn-polymorphic,
/// so the bindings need to distinguish different types.
macro_rules! declare_renderer {
    ( $name:ident ( $state:ty ) ) => {
        /// Program state renderer that renders into a given [`VisTree`].
        #[wasm_bindgen]
        pub struct $name(Renderer<'static, <$state as ProgramStateGraph>::NodeId, VisTree>);

        #[wasm_bindgen]
        impl $name {
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
                    let boxed: Box<
                        dyn FnMut(RendererWarning<<$state as ProgramStateGraph>::NodeId>),
                    > = Box::new(handler);
                    boxed
                }));
            }

            /// Returns a human-readable representation of the current
            /// resolved style that the renderer has applied to the [`VisTree`].
            #[wasm_bindgen(js_name = "prettyPrint")]
            pub fn pretty_print(&self) -> String {
                format!("{:#?}", self.0)
            }

            /// Resolves a [`Stylesheet`] over a state graph and renders the result.
            #[wasm_bindgen(js_name = "applyStylesheet")]
            pub fn apply_stylesheet(&mut self, stylesheet: &Stylesheet, graph: &$state) {
                let mapping = aili_translate::cascade::apply_stylesheet(&stylesheet.0, graph);
                self.0.update_root(Some(Selectable::node(graph.root())));
                self.0.update(mapping);
            }
        }
    };
}

declare_renderer!(VisTreeRenderer(StateGraph));
#[cfg(feature = "gdbstate")]
declare_renderer!(GdbVisTreeRenderer(crate::gdbstate::GdbStateGraph));

/// Resolves a [`Stylesheet`] over a [`StateGraph`] and renders
/// the result into a [`VisTreeRenderer`].
#[wasm_bindgen(js_name = "applyStylesheet")]
pub fn apply_stylesheet(
    stylesheet: &Stylesheet,
    graph: &StateGraph,
    renderer: &mut VisTreeRenderer,
) {
    renderer.apply_stylesheet(stylesheet, graph);
}
