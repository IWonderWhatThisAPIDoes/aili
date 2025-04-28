//! Resolution of [stylesheets](crate::stylesheet) in the context
//! of [state graphs](aili_model::state).

mod apply;
pub mod eval;
mod mapping_builder;
mod select;
mod selector_resolver;
mod style;
mod test_graph;

pub use apply::apply_stylesheet;
pub use style::CascadeStyle;
