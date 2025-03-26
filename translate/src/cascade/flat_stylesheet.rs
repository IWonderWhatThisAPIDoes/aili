//! Preprocessing of [`Stylesheet`]s to simplify matching.

use super::flat_selector::FlatSelector;
use crate::stylesheet::{StyleClause, StyleRule, Stylesheet};

pub struct FlatStylesheet(pub Vec<FlatStyleRule>);

impl From<Stylesheet> for FlatStylesheet {
    fn from(value: Stylesheet) -> Self {
        Self(value.0.into_iter().map(FlatStyleRule::from).collect())
    }
}

pub struct FlatStyleRule {
    pub machine: FlatSelector,
    pub properties: Vec<StyleClause>,
}

impl From<StyleRule> for FlatStyleRule {
    fn from(value: StyleRule) -> Self {
        Self {
            machine: value.selector.into(),
            properties: value.properties,
        }
    }
}
