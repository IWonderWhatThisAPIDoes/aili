//! Resolution of [stylesheets](crate::stylesheet) in the context
//! of [state graphs](aili_model::state).

pub mod apply;
pub mod eval;
pub mod flat_selector;
pub mod flat_stylesheet;
pub mod select;
pub mod test_graph;
