//! Definitions of conversions from [`RawPropertyKey`] to [`PropertyKey`].

use super::{FragmentKey, PropertyKey};
use aili_style::stylesheet::RawPropertyKey;
use derive_more::{Display, Error};

/// Indicates an invalid conversion of property keys.
#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
pub enum InvalidSymbol {
    /// An uknown named fragment was used.
    #[display("invalid fragment name '{_0}'")]
    #[error(ignore)]
    InvalidFragment(String),
}

/// Maps [`PropertyKey`]s to their symbol names.
///
/// ## Symbol Names
/// | Symbol name                           | Associated clause key                 |
/// |---------------------------------------|---------------------------------------|
/// | `display`                             | [`Display`](PropertyKey::Display)     |
/// | `parent`                              | [`Parent`](PropertyKey::Parent)       |
/// | `target`                              | [`Target`](PropertyKey::Target)       |
/// | Other                                 | [`Attribute`](PropertyKey::Attribute) |
pub fn unquoted_style_key(key: &str) -> PropertyKey {
    match key {
        "display" => PropertyKey::Display,
        "parent" => PropertyKey::Parent,
        "target" => PropertyKey::Target,
        _ => PropertyKey::Attribute(key.to_owned()),
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
        _ => Err(InvalidSymbol::InvalidFragment(key.to_owned())),
    }
}

impl TryFrom<RawPropertyKey> for PropertyKey {
    type Error = InvalidSymbol;
    fn try_from(value: RawPropertyKey) -> Result<Self, Self::Error> {
        match value {
            RawPropertyKey::Property(p) => Ok(unquoted_style_key(&p)),
            RawPropertyKey::QuotedProperty(p) => Ok(PropertyKey::Attribute(p)),
            RawPropertyKey::FragmentProperty(f, p) => {
                Ok(PropertyKey::FragmentAttribute(fragment_key(&f)?, p))
            }
        }
    }
}
