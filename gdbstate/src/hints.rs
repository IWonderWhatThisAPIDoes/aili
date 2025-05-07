//! User-provided hints to help deduce whether each pointer
//! points to an array or a single object.

use aili_style::stylesheet::RawPropertyKey;
use derive_more::{Debug, Display, Error};

/// [`aili_style::stylesheet::PropertyKey`] to a length hint sheet.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PointerLengthHintKey {
    /// Indicates the length of an array pointed to by a pointer.
    #[debug("length")]
    Length,
}

/// Error type emited when an unrecognized key is passed
/// to [`PointerLengthHintKey`].
#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
pub enum BadHintKey {
    /// Unrecognized key was passed.
    #[display("unrecognized key: {_0}")]
    #[error(ignore)]
    InvalidKey(String),

    /// Quoted key was passed.
    ///
    /// Quoted keys cannot be used in length hint sheets.
    #[display("quoted keys are not allowed: {_0:?}")]
    #[error(ignore)]
    Quoted(String),

    /// Fragment key was passed.
    ///
    /// Fragment keys cannot be used in length hint sheets.
    #[display("fragment keys are not allowed: {_0}/{_1:?}")]
    #[error(ignore)]
    Fragment(String, String),
}

impl TryFrom<RawPropertyKey> for PointerLengthHintKey {
    type Error = BadHintKey;
    fn try_from(value: RawPropertyKey) -> Result<Self, Self::Error> {
        match value {
            RawPropertyKey::Property(p) => match p.as_str() {
                "length" => Ok(Self::Length),
                _ => Err(BadHintKey::InvalidKey(p)),
            },
            RawPropertyKey::QuotedProperty(p) => Err(BadHintKey::Quoted(p)),
            RawPropertyKey::FragmentProperty(f, p) => Err(BadHintKey::Fragment(f, p)),
        }
    }
}
