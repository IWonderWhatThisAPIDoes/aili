//! Utilities for stylesheet resolution.

mod selector_resolver;
mod style;

pub use selector_resolver::{SelectionCaret, SelectorResolver};
pub use style::{CascadeSelector, CascadeStyle, CascadeStyleRule};
