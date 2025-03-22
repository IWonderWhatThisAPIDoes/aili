//! Values assignable to properties.

use crate::selectable::Selectable;
use aili_model::state::{NodeId, NodeValue};
use derive_more::{Debug, Display, From};

/// Value that can be the result of a stylesheet expression
/// and can be assigned to cascade variables and properties.
#[derive(Clone, PartialEq, Eq, Debug, Display, Default, From)]
pub enum PropertyValue<T: NodeId> {
    /// The property was cleared or it was never assigned.
    #[default]
    #[debug("unset")]
    #[display("")]
    Unset,

    /// The property's value is a reference to a selectable entity.
    #[debug("@({_0:?})")]
    #[display("@({_0:?})")]
    Selection(Selectable<T>),

    /// The property's value is a program value extracted from state
    /// or calculated with arithmetics.
    #[debug("({_0:?})")]
    #[display("{_0:?}")]
    #[from(NodeValue, bool, u64, i64)]
    Value(NodeValue),

    /// The property's value is a string literal or compound string.
    #[debug("{_0:?}")]
    #[from]
    String(String),
}

impl<T: NodeId> PropertyValue<T> {
    /// Checks whether a property value is trurhy.
    ///
    /// The following values are falsy. All other values
    /// are truthy.
    /// - [`Unset`](PropertyValue::Unset)
    /// - Empty [`String`](PropertyValue::String)
    /// - False [`Bool`](NodeValue::Bool)
    /// - Zero [`Int`](NodeValue::Int) and [`Uint`](NodeValue::Uint)
    ///
    /// Note that, in particular, all [`Selection`](PropertyValue::Selection)s
    /// are truthy. `!!select(...)` is a shorthand for verifying
    /// that an entity that matches the given selector exists, even if it has
    /// no value, its value is false, or it is not a node.
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Unset => false,
            Self::String(s) => !s.is_empty(),
            Self::Selection(_) => true,
            Self::Value(NodeValue::Bool(b)) => *b,
            Self::Value(NodeValue::Int(i)) => *i != 0,
            Self::Value(NodeValue::Uint(u)) => *u != 0,
        }
    }
}

impl<T: NodeId> PartialOrd for PropertyValue<T> {
    /// Compare two property values.
    ///
    /// Comparison is subject to the following rules.
    /// - Two values of type [`Unset`](PropertyValue::Unset) are equal.
    /// - Two values of type [`Selection`](PropertyValue::Selection) are
    ///   equal if they refer to the same selectable entity.
    ///   Otherwise they are unordered.
    /// - Two values of type [`String`](PropertyValue::String) are
    ///   equal if they contain the same character sequence.
    ///   Otherwise they are unordered.
    /// - Two values of type [`Value`](PropertyValue::Value) are totally
    ///   ordered by their numeric values. `true == 1` and `false == 0`.
    /// - Any other pair of values is unordered.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Unset, Self::Unset) => Some(std::cmp::Ordering::Equal),
            (Self::String(left), Self::String(right)) => {
                if left == right {
                    Some(std::cmp::Ordering::Equal)
                } else {
                    None
                }
            }
            (Self::Value(left), Self::Value(right)) => left.partial_cmp(right),
            (Self::Selection(left), Self::Selection(right)) => {
                if left == right {
                    Some(std::cmp::Ordering::Equal)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
