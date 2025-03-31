//! Definitions of symbol names used by semantic analysis

use aili_model::state::{EdgeLabel, NodeTypeClass};
use aili_translate::{
    property::{FragmentKey, PropertyKey},
    stylesheet::{
        StyleKey,
        expression::{Expression, UnaryOperator},
    },
};
use derive_more::{Display, Error};

/// Error type returned by symbol name matchers
/// if they cannot parse the provided symbol.
#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display("symbol {_0:?} is not valid in this context")]
#[error(ignore)]
pub struct InvalidSymbol(pub String);

/// Maps [`EdgeLabel`]s to their symbol names.
///
/// ## Symbol Names
/// | Symbol name | Associated edge label         |
/// |-------------|-------------------------------|
/// | `main`      | [`Main`](EdgeLabel::Main)     |
/// | `next`      | [`Next`](EdgeLabel::Next)     |
/// | `ret`       | [`Result`](EdgeLabel::Result) |
/// | `ref`       | [`Deref`](EdgeLabel::Deref)   |
/// | `len`       | [`Length`](EdgeLabel::Length) |
pub fn edge_label_from_name(name: &str) -> Result<EdgeLabel, InvalidSymbol> {
    match name {
        "main" => Ok(EdgeLabel::Main),
        "next" => Ok(EdgeLabel::Next),
        "ret" => Ok(EdgeLabel::Result),
        "ref" => Ok(EdgeLabel::Deref),
        "len" => Ok(EdgeLabel::Length),
        _ => Err(InvalidSymbol(name.to_owned())),
    }
}

/// Maps stylesheet clause keys ([`StyleKey`]) to their symbol names.
///
/// ## Symbol Names
/// | Symbol name                           | Associated clause key                 |
/// |---------------------------------------|---------------------------------------|
/// | `display`                             | [`Display`](PropertyKey::Display)     |
/// | `parent`                              | [`Parent`](PropertyKey::Parent)       |
/// | `target`                              | [`Target`](PropertyKey::Target)       |
/// | Symbols matching [`is_variable_name`] | [`Variable`](StyleKey::Variable)      |
/// | Other                                 | [`Attribute`](PropertyKey::Attribute) |
pub fn unquoted_style_key(key: &str) -> StyleKey {
    match key {
        "display" => StyleKey::Property(PropertyKey::Display),
        "parent" => StyleKey::Property(PropertyKey::Parent),
        "target" => StyleKey::Property(PropertyKey::Target),
        _ => {
            if is_variable_name(key) {
                StyleKey::Variable(key.to_owned())
            } else {
                StyleKey::Property(PropertyKey::Attribute(key.to_owned()))
            }
        }
    }
}

/// Determines whether a symbol is considered a variable name.
///
/// Symbols that start with `--` (double dash) are variable names.
pub fn is_variable_name(key: &str) -> bool {
    key.starts_with("--")
}

/// Maps function-like [`UnaryOperator`]s to their names.
///
/// ## Symbol Names
/// | Symbol name                                        | Associated operator                           |
/// |----------------------------------------------------|-----------------------------------------------|
/// | `isset`                                            | [`IsSet`](UnaryOperator::IsSet)               |
/// | `val`                                              | [`NodeValue`](UnaryOperator::NodeValue)       |
/// | `typename`                                         | [`NodeTypeName`](UnaryOperator::NodeTypeName) |
/// | `is-`[suffix matching [`node_type_class_by_name`]] | [`NodeIsA`](UnaryOperator::NodeIsA)           |
pub fn unary_function_by_name(name: &str) -> Result<UnaryOperator, InvalidSymbol> {
    match name {
        "isset" => Ok(UnaryOperator::IsSet),
        "val" => Ok(UnaryOperator::NodeValue),
        "typename" => Ok(UnaryOperator::NodeTypeName),
        _ => {
            let type_class_from_name = name
                .strip_prefix("is-")
                .map(node_type_class_by_name)
                .and_then(Result::ok);
            if let Some(type_class) = type_class_from_name {
                Ok(UnaryOperator::NodeIsA(type_class))
            } else {
                Err(InvalidSymbol(name.to_owned()))
            }
        }
    }
}

/// Maps [`NodeTypeClass`]es to their names.
///
/// ## Symbol Names
/// | Symbol name | Associated type class             |
/// |-------------|-----------------------------------|
/// | `root`      | [`Root`](NodeTypeClass::Root)     |
/// | `frame`     | [`Frame`](NodeTypeClass::Frame)   |
/// | `val`       | [`Atom`](NodeTypeClass::Atom)     |
/// | `struct`    | [`Struct`](NodeTypeClass::Struct) |
/// | `arr`       | [`Array`](NodeTypeClass::Array)   |
/// | `ref`       | [`Ref`](NodeTypeClass::Ref)       |
pub fn node_type_class_by_name(name: &str) -> Result<NodeTypeClass, InvalidSymbol> {
    match name {
        "root" => Ok(NodeTypeClass::Root),
        "frame" => Ok(NodeTypeClass::Frame),
        "val" => Ok(NodeTypeClass::Atom),
        "struct" => Ok(NodeTypeClass::Struct),
        "arr" => Ok(NodeTypeClass::Array),
        "ref" => Ok(NodeTypeClass::Ref),
        _ => Err(InvalidSymbol(name.to_owned())),
    }
}

/// Maps literal [`Expression`]s to their names.
///
/// ## Symbol Names
/// | Symbol name       | Associated literal                  |
/// |-------------------|-------------------------------------|
/// | `unset` or `none` | [`Unset`](Expression::Unset)        |
/// | `true`            | [`Bool`](Expression::Bool)`(true)`  |
/// | `false`           | [`Bool`](Expression::Bool)`(false)` |
pub fn literal_expression_by_name(name: &str) -> Result<Expression, InvalidSymbol> {
    match name {
        "unset" => Ok(Expression::Unset),
        "none" => Ok(Expression::Unset),
        "true" => Ok(Expression::Bool(true)),
        "false" => Ok(Expression::Bool(false)),
        _ => {
            if is_variable_name(name) {
                Ok(Expression::Variable(name.to_owned()))
            } else {
                Err(InvalidSymbol(name.to_owned()))
            }
        }
    }
}

/// Maps [`FragmentKey`]s to their names.
///
/// ## Symbol Names
/// | Symbol name | Associated fragment           |
/// |-------------|-------------------------------|
/// | `start`     | [`Start`](FragmentKey::Start) |
/// | `end`       | [`End`](FragmentKey::End)     |
pub fn fragment_key(key: &str) -> Result<FragmentKey, InvalidSymbol> {
    match key {
        "start" => Ok(FragmentKey::Start),
        "end" => Ok(FragmentKey::End),
        _ => Err(InvalidSymbol(key.to_owned())),
    }
}
