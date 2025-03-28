//! Resolution of [stylesheets](crate::stylesheet) in the context
//! of [state graphs](aili_model::state).

mod apply;
pub mod eval;
mod select;
mod style;
mod test_graph;

pub use apply::*;
pub use style::CascadeStyle;
