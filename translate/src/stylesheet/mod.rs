//! Stylesheets that define what a program state representation
//! should look like.

pub mod expression;
pub mod selector;

use expression::Expression;
use selector::Selector;

/// Single stylesheet rule that assignes a series
/// of property and variable values to a selector.
pub struct StyleRule {
    /// Selector that determines what entities the rule applies to.
    pub selector: Selector,

    /// Properties assigned to each entity that matches.
    pub properties: Vec<StyleRuleProperty>,
}

/// Single property assignment entry.
pub struct StyleRuleProperty {
    /// Name of the property or variable to assign.
    ///
    /// Multiple entries of a rule may have the same key.
    /// They are then evaluated in declaration order.
    /// This is only relevant for variables, where the value
    /// assigned to a variable holds until it is overwritten.
    pub key: StylePropertyKey,

    /// Expression that evaluates to the value that should
    /// be assigned to the property.
    pub value: Box<Expression>,
}

/// A key that values can be assigned to in a style rule.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum StylePropertyKey {
    /// Assigns value to a property of the selected entity.
    Property(String),

    /// Assigns values to a cascade variable.
    Variable(String),
}

/// Full stylesheet, a sequence of style rules.
pub struct Stylesheet(pub Vec<StyleRule>);
