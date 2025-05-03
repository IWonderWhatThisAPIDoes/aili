//! Resolution of [stylesheets](aili_style::stylesheet) in the context
//! of [state graphs](aili_model::state).

mod apply;
mod mapping_builder;

pub use apply::apply_stylesheet;
