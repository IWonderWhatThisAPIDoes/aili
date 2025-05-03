//! Stylesheets that assign properties to state entities
//! based on the structure of their surroundings.

pub mod expression;
pub mod selector;

use derive_more::Debug;
use expression::Expression;
use selector::Selector;

/// Types that can be used as a key for style properties
pub trait PropertyKey: Clone + Eq + std::fmt::Debug + std::hash::Hash {}

impl<T> PropertyKey for T where T: Clone + Eq + std::fmt::Debug + std::hash::Hash {}

/// [`PropertyKey`] that consists of raw string keys
/// for properties and fragment properties.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum RawPropertyKey {
    /// Property identified by a name.
    Property(String),

    /// Property identified by a name with quoted name.
    QuotedProperty(String),

    /// Property of a fragment.
    FragmentProperty(String, String),
}

/// Single stylesheet rule that assignes a series
/// of property and variable values to a selector.
#[derive(PartialEq, Eq)]
pub struct StyleRule<K: PropertyKey = RawPropertyKey> {
    /// Selector that determines what entities the rule applies to.
    pub selector: Selector,

    /// Properties assigned to each entity that matches.
    pub properties: Vec<StyleClause<K>>,
}

impl<K: PropertyKey> StyleRule<K> {
    /// Converts a style rule to a different [`PropertyKey`].
    ///
    /// All properties whose keys cannot be converted are removed.
    pub fn map_key<L>(self) -> StyleRule<L>
    where
        L: PropertyKey,
        K: TryInto<L>,
    {
        StyleRule {
            selector: self.selector,
            properties: self
                .properties
                .into_iter()
                .map(StyleClause::try_map_key)
                .filter_map(Result::ok)
                .collect(),
        }
    }
}

impl<K: PropertyKey> Default for StyleRule<K> {
    fn default() -> Self {
        Self {
            selector: Selector::default(),
            properties: Vec::new(),
        }
    }
}

impl<K: PropertyKey> std::fmt::Debug for StyleRule<K> {
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
#[derive(Clone, PartialEq, Eq, Debug)]
#[debug("{key:?}: ({value:?})")]
pub struct StyleClause<K: PropertyKey = RawPropertyKey> {
    /// Name of the property or variable to assign.
    ///
    /// Multiple entries of a rule may have the same key.
    /// They are then evaluated in declaration order.
    /// This is only relevant for variables, where the value
    /// assigned to a variable holds until it is overwritten.
    pub key: StyleKey<K>,

    /// Expression that evaluates to the value that should
    /// be assigned to the property.
    pub value: Expression,
}

impl<K: PropertyKey> StyleClause<K> {
    /// Converts a style clause to a different [`PropertyKey`].
    pub fn map_key<L>(self) -> StyleClause<L>
    where
        L: PropertyKey,
        K: Into<L>,
    {
        StyleClause {
            key: self.key.map_key(),
            value: self.value,
        }
    }

    /// Attempts to converts a style rule to a different [`PropertyKey`].
    pub fn try_map_key<L>(self) -> Result<StyleClause<L>, <K as TryInto<L>>::Error>
    where
        L: PropertyKey,
        K: TryInto<L>,
    {
        Ok(StyleClause {
            key: self.key.try_map_key()?,
            value: self.value,
        })
    }
}

/// A key that values can be assigned to in a style rule.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum StyleKey<K: PropertyKey = RawPropertyKey> {
    /// Assigns value to a property of the selected entity.
    Property(K),

    /// Assigns values to a cascade variable.
    Variable(String),
}

impl<K: PropertyKey> StyleKey<K> {
    /// Converts a style key to a different [`PropertyKey`].
    pub fn map_key<L>(self) -> StyleKey<L>
    where
        L: PropertyKey,
        K: Into<L>,
    {
        match self {
            Self::Property(k) => StyleKey::Property(k.into()),
            Self::Variable(v) => StyleKey::Variable(v),
        }
    }

    /// Attempts to converts a style key to a different [`PropertyKey`].
    pub fn try_map_key<L>(self) -> Result<StyleKey<L>, <K as TryInto<L>>::Error>
    where
        L: PropertyKey,
        K: TryInto<L>,
    {
        match self {
            Self::Property(k) => k.try_into().map(StyleKey::Property),
            Self::Variable(v) => Ok(StyleKey::Variable(v)),
        }
    }
}

/// Full stylesheet, a sequence of style rules.
#[derive(PartialEq, Eq, Debug)]
pub struct Stylesheet<K: PropertyKey = RawPropertyKey>(pub Vec<StyleRule<K>>);

impl<K: PropertyKey> Stylesheet<K> {
    /// Converts a stylesheet to a different [`PropertyKey`].
    ///
    /// All properties whose keys cannot be converted are removed.
    pub fn map_key<L>(self) -> Stylesheet<L>
    where
        L: PropertyKey,
        K: TryInto<L>,
    {
        Stylesheet(self.0.into_iter().map(StyleRule::map_key).collect())
    }
}

impl<K: PropertyKey> Default for Stylesheet<K> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
