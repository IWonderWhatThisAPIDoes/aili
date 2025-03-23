//! Stylesheet expressions that evaluate to property values.

use aili_model::state::{EdgeLabel, NodeTypeClass};
use derive_more::Debug;

/// Stylesheet expression.
///
/// The following kinds expressions exist:
/// - Literal values
/// - Variable invocations
/// - [`LimitedSelector`] queries
/// - Compound operator expressions
///
/// All expressions are without side effects.
/// Variable invocation and selector query expressions
/// are however stateful.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    /// Variable invoked by its name.
    #[debug("{_0}")]
    Variable(String),

    /// The `unset` (null) literal.
    #[debug("unset")]
    Unset,

    /// Boolean literal.
    #[debug("{}", if *_0 { "true" } else { "false" })]
    Bool(bool),

    /// String literal.
    #[debug("\"{_0}\"")]
    String(String),

    /// Integer literal.
    #[debug("{_0}")]
    Int(u64),

    /// Selectable element refered to by its selector.
    #[debug("@[{_0:?}]")]
    Select(Box<LimitedSelector>),

    /// Unary operator expression.
    #[debug("{_0:?}({_1:?})")]
    UnaryOperator(UnaryOperator, Box<Expression>),

    /// Binary operator expression.
    #[debug("({_0:?} {_1:?} {_2:?})")]
    BinaryOperator(Box<Expression>, BinaryOperator, Box<Expression>),

    /// Conditional ternary operator expression.
    ///
    /// If its first argument is truthy, resolves to its second argument.
    /// Otherwise resolves to its third argument.
    #[debug("({_0:?} ? {_1:?} : {_2:?})")]
    Conditional(Box<Expression>, Box<Expression>, Box<Expression>),
}

/// Identifier of the operator in a [`UnaryOperator`](Expression::UnaryOperator) expression.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnaryOperator {
    /// Coerces a value to a number if possible.
    ///
    /// ## Return Values
    /// | Argument                                                 | Return value                                         |
    /// |----------------------------------------------------------|------------------------------------------------------|
    /// | [`Unset`](crate::property::PropertyValue::Unset)         | [`Unset`](crate::property::PropertyValue::Unset)     |
    /// | [`String`](crate::property::PropertyValue::String)       | Argument is unchanged                                |
    /// | [`Int`](aili_model::state::NodeValue::Int)               | Argument is unchanged                                |
    /// | [`Uint`](aili_model::state::NodeValue::Uint)             | Argument is unchanged                                |
    /// | [`Bool`](aili_model::state::NodeValue::Bool)             | [`Uint`](aili_model::state::NodeValue::Uint), 0 or 1 |
    /// | [`Selection`](crate::property::PropertyValue::Selection) | Equivalent to `+val(x)`                              |
    #[debug("+")]
    Plus,

    /// Coerces a value to a number and negates it if possible.
    ///
    /// ## Return Values
    /// | Argument                                                 | Return value                                                                                                       |
    /// |----------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------|
    /// | [`Unset`](crate::property::PropertyValue::Unset)         | [`Unset`](crate::property::PropertyValue::Unset)                                                                   |
    /// | [`String`](crate::property::PropertyValue::String)       | [`Unset`](crate::property::PropertyValue::Unset),                                                                  |
    /// | [`Int`](aili_model::state::NodeValue::Int)               | [`Int`](aili_model::state::NodeValue::Int) or [`Unset`](crate::property::PropertyValue::Unset) in case of overflow |
    /// | [`Uint`](aili_model::state::NodeValue::Uint)             | [`Int`](aili_model::state::NodeValue::Int) or [`Unset`](crate::property::PropertyValue::Unset) in case of overflow |
    /// | [`Bool`](aili_model::state::NodeValue::Bool)             | [`Int`](aili_model::state::NodeValue::Int), 0 or -1                                                                |
    /// | [`Selection`](crate::property::PropertyValue::Selection) | Equivalent to `-val(x)`                                                                                            |
    #[debug("-")]
    Minus,

    /// Coerces a value to a boolean and negates it.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool).
    /// Negation of [`is_truthy`](crate::property::PropertyValue::is_truthy).
    #[debug("!")]
    Not,

    /// Extracts a value from a selected node.
    ///
    /// ## Return Values
    /// [`NodeValue`](aili_model::state::NodeValue) of the node referred to by the argument.
    ///
    /// [`Unset`](crate::property::PropertyValue::Unset) if the argument is not
    /// a [`Selection`](crate::property::PropertyValue::Selection),
    /// the selected entity is not a node, or it is a node with no value.
    #[debug("val")]
    NodeValue,

    /// Checks whether a selected node is of a given type class.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool). True if the argument is a selection of a node,
    /// and its type class matches this operator. False otherwise.
    #[debug("is:{_0:?}")]
    NodeIsA(NodeTypeClass),

    /// Gets the name of state node's type.
    ///
    /// ## Return Values
    /// [`String`](crate::property::PropertyValue::String) containing the name of the type of the argument
    /// if it is a selection of a node and it has one of types [`Frame`](aili_model::state::NodeType::Frame),
    /// [`Atom`](aili_model::state::NodeType::Atom), or [`Struct`](aili_model::state::NodeType::Struct).
    /// [`Unset`](crate::property::PropertyValue::Unset) otherwise.
    #[debug("nameof")]
    NodeTypeName,

    /// Checks whether a value is defined.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool). False if the argument
    /// is [`Unset`](crate::property::PropertyValue::Unset), true otherwise.
    #[debug("isset")]
    IsSet,
}

/// Identifier of the operator in a [`BinaryOperator`](Expression::BinaryOperator) expression.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinaryOperator {
    /// Arithmetic addition or string concatenation.
    ///
    /// ## Return Values
    /// - If either argument is [`Selection`](crate::property::PropertyValue::Selection), it is first evaluated
    ///   (equivalent to using the [`NodeValue`](UnaryOperator::NodeValue) operator).
    /// - Then, if either argument is [`String`](crate::property::PropertyValue::String), the other is converted
    ///   to string and the arguments are concatenated. [`Unset`](crate::property::PropertyValue::Unset) converts
    ///   to empty string and [`Bool`](aili_model::state::NodeValue::Bool) values convert to `"true"` and `"false"`.
    /// - Otwerwise, if both arguments can be coerced to numeric types, they are added.
    ///   [`Unset`](crate::property::PropertyValue::Unset) is returned in case of overflow.
    /// - Otherwise, [`Unset`](crate::property::PropertyValue::Unset) is returned.
    #[debug("+")]
    Plus,

    /// Arithmetic subtraction.
    ///
    /// ## Return Values
    /// - If either argument is [`Selection`](crate::property::PropertyValue::Selection), it is first evaluated
    ///   (equivalent to using the [`NodeValue`](UnaryOperator::NodeValue) operator).
    /// - Then, if both arguments can be coerced to numeric types, they are subtracted.
    ///   [`Unset`](crate::property::PropertyValue::Unset) is returned in case of overflow.
    /// - Otherwise, [`Unset`](crate::property::PropertyValue::Unset) is returned.
    #[debug("-")]
    Minus,

    /// Arithmetic multiplication.
    ///
    /// ## Return Values
    /// - If either argument is [`Selection`](crate::property::PropertyValue::Selection), it is first evaluated
    ///   (equivalent to using the [`NodeValue`](UnaryOperator::NodeValue) operator).
    /// - Then, if both arguments can be coerced to numeric types, they are multiplied.
    ///   [`Unset`](crate::property::PropertyValue::Unset) is returned in case of overflow.
    /// - Otherwise, [`Unset`](crate::property::PropertyValue::Unset) is returned.
    #[debug("*")]
    Mul,

    /// Euclidean integer arithmetic division.
    ///
    /// ## Return Values
    /// - If either argument is [`Selection`](crate::property::PropertyValue::Selection), it is first evaluated
    ///   (equivalent to using the [`NodeValue`](UnaryOperator::NodeValue) operator).
    /// - Then, if both arguments can be coerced to numeric types, they are divided.
    ///   [`Unset`](crate::property::PropertyValue::Unset) is returned in case of overflow.
    /// - Otherwise, [`Unset`](crate::property::PropertyValue::Unset) is returned.
    #[debug("/")]
    Div,

    /// Euclidean integer arithmetic remainder.
    ///
    /// ## Return Values
    /// - If either argument is [`Selection`](crate::property::PropertyValue::Selection), it is first evaluated
    ///   (equivalent to using the [`NodeValue`](UnaryOperator::NodeValue) operator).
    /// - Then, if both arguments can be coerced to numeric types, they are divided.
    ///   [`Unset`](crate::property::PropertyValue::Unset) is returned in case of overflow.
    /// - Otherwise, [`Unset`](crate::property::PropertyValue::Unset) is returned.
    #[debug("%")]
    Mod,

    /// Tests values for equality.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool). True if arguments are equal, false otherwise.
    ///
    /// If either argument is [`Selection`](crate::property::PropertyValue::Selection), it is first evaluated
    /// (equivalent to using the [`NodeValue`](UnaryOperator::NodeValue) operator). Then, the following pairs of values are equal.
    /// All other pairs of values are not equal.
    /// - [`Unset`](crate::property::PropertyValue::Unset) is equal to itself.
    /// - Two [`String`](crate::property::PropertyValue::String)s are equal if they contain identical characters.
    /// - Two numeric values ([`Int`](aili_model::state::NodeValue::Int) or [`Uint`](aili_model::state::NodeValue::Uint))
    ///   are equal if they have the same value arithmetically.
    /// - True is equal to one and itself. False is equal to zero and itself.
    #[debug("==")]
    Eq,

    /// Tests values for inequality.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool). False if arguments are equal, true otherwise.
    /// See [`BinaryOperator::Eq`] for definition of equality.
    #[debug("!=")]
    Ne,

    /// Compares values.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool). True if left operand is ordered before right operand.
    /// False otherwise.
    ///
    /// - If either argument is [`Selection`](crate::property::PropertyValue::Selection), it is first evaluated
    ///   (equivalent to using the [`NodeValue`](UnaryOperator::NodeValue) operator).
    /// - Then, if both operands can be coerced to non-equal numeric values,
    ///   the one with smaller value is ordered before the other.
    /// - Otherwise, the values are not ordered.
    #[debug("<")]
    Lt,

    /// Compares values.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool). True if left operand is ordered before or is equal to right operand.
    /// False otherwise.
    /// See [`BinaryOperator::Lt`] for definition of ordering.
    #[debug("<=")]
    Le,

    /// Compares values.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool). True if left operand is ordered after right operand.
    /// False otherwise.
    /// See [`BinaryOperator::Lt`] for definition of ordering.
    #[debug(">")]
    Gt,

    /// Compares values.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool). True if left operand is ordered after or is equal to right operand.
    /// False otherwise.
    /// See [`BinaryOperator::Lt`] for definition of ordering.
    #[debug(">=")]
    Ge,

    /// Logical conjunction.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool). True if both arguments
    /// are [truthy](crate::property::PropertyValue::is_truthy), false otherwise.
    #[debug("&&")]
    And,

    /// Logical disjunction.
    ///
    /// ## Return Values
    /// [`Bool`](aili_model::state::NodeValue::Bool). True if either argument
    /// is [truthy](crate::property::PropertyValue::is_truthy), false otherwise.
    #[debug("||")]
    Or,
}

/// Selector that is limited to a single path
/// and exact matches for edges (matchers other than
/// [`Exact`](super::selector::EdgeMatcher::Exact) are not allowed).
///
/// These selectors can always unambiguously select at most one entity.
#[derive(Clone, PartialEq, Eq)]
pub struct LimitedSelector {
    /// Path that must be matched in order to select something.
    pub path: Vec<EdgeLabel>,

    /// Specifies whether the selector selects an extra element
    /// attached to the matched node or edge, instead of the node
    /// or edge directly.
    pub extra_label: Option<String>,
}

impl LimitedSelector {
    /// Shorthand for constructing a limited selector that matches a node.
    pub fn from_path(path: impl IntoIterator<Item = EdgeLabel>) -> Self {
        Self {
            path: Vec::from_iter(path),
            extra_label: None,
        }
    }
}

impl std::fmt::Debug for LimitedSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.path.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{segment:?}")?;
        }
        if let Some(extra) = &self.extra_label {
            write!(f, "::extra({extra})")?;
        }
        Ok(())
    }
}
