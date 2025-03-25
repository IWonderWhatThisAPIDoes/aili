//! Stylesheets that define what a program state representation
//! should look like.

pub mod expression;
pub mod selector;

use crate::property::PropertyKey;
use derive_more::Debug;
use expression::Expression;
use selector::Selector;

/// Single stylesheet rule that assignes a series
/// of property and variable values to a selector.
#[derive(PartialEq, Eq)]
pub struct StyleRule {
    /// Selector that determines what entities the rule applies to.
    pub selector: Selector,

    /// Properties assigned to each entity that matches.
    pub properties: Vec<StyleRuleItem>,
}

impl std::fmt::Debug for StyleRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {{ ", self.selector)?;
        for clause in &self.properties {
            write!(f, "{clause:?}; ")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

/// Single property or variable assignment entry.
#[derive(PartialEq, Eq, Debug)]
#[debug("{key:?}: ({value:?})")]
pub struct StyleRuleItem {
    /// Name of the property or variable to assign.
    ///
    /// Multiple entries of a rule may have the same key.
    /// They are then evaluated in declaration order.
    /// This is only relevant for variables, where the value
    /// assigned to a variable holds until it is overwritten.
    pub key: StyleKey,

    /// Expression that evaluates to the value that should
    /// be assigned to the property.
    pub value: Box<Expression>,
}

/// A key that values can be assigned to in a style rule.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum StyleKey {
    /// Assigns value to a property of the selected entity.
    Property(PropertyKey),

    /// Assigns values to a cascade variable.
    Variable(String),
}

/// Full stylesheet, a sequence of style rules.
#[derive(PartialEq, Eq, Debug, Default)]
pub struct Stylesheet(pub Vec<StyleRule>);
