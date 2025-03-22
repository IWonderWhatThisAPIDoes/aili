//! Selectors identify selectable entities by the paths
//! that lead to them.

use crate::expression::Expression;
use aili_model::state::EdgeLabel;
use derive_more::{Debug, From};

/// Pattern against which an [`EdgeLabel`] can be matched.
#[derive(PartialEq, Eq, From, Debug)]
pub enum EdgeMatcher {
    /// Matches all edges.
    #[debug("*")]
    Any,

    /// Matches a particular edge label.
    #[from]
    #[debug("{_0:?}")]
    Exact(EdgeLabel),

    /// Matches all [`EdgeLabel::Index`] edges.
    #[debug("[_]")]
    AnyIndex,

    /// Matches all [`EdgeLabel::Named`] edges.
    #[debug("<\"_\">")]
    AnyNamed,

    /// Matches all [`EdgeLabel::Named`] edges with a particular name,
    /// but with any secondary index.
    #[debug("\"{_0}\"")]
    Named(String),
}

impl EdgeMatcher {
    /// Tests an [`EdgeLabel`] against an [`EdgeMatcher`].
    pub fn matches(&self, label: &EdgeLabel) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(pattern) => label == pattern,
            Self::AnyIndex => matches!(label, EdgeLabel::Index(_)),
            Self::AnyNamed => matches!(label, EdgeLabel::Named(_, _)),
            Self::Named(name) => {
                matches!(label, EdgeLabel::Named(edge_name, _) if edge_name == name)
            }
        }
    }
}

/// Unrestricted segment of a selector path.
/// Can be an edge matcher or a control flow construct.
pub enum SelectorSegment {
    /// Matches an edge.
    Match(EdgeMatcher),

    /// Matches a full selector path zero or more times.
    AnyNumberOfTimes(SelectorPath),

    /// Matches at least one of a set of selector paths.
    Branch(Vec<SelectorPath>),
}

impl SelectorSegment {
    /// Shorthand for a completely unrestricted selector segment
    /// that matches all edges to any depth.
    pub fn anything_any_number_of_times() -> Self {
        Self::AnyNumberOfTimes([SelectorSegment::Match(EdgeMatcher::Any).into()].into())
    }
}

/// A series of selector segments that must all match in sequence
/// in order to pass.
#[derive(From)]
#[from(forward)]
pub struct SelectorPath(pub Vec<RestrictedSelectorSegment>);

/// [`SelectorSegment`] that is optionally restricted by a condition.
/// If the condition does not evaluate to a [truthy](crate::values::PropertyValue::is_truthy)
/// value, the selector segment does not match anything.
pub struct RestrictedSelectorSegment {
    /// The selector segment that performs the initial match.
    pub segment: SelectorSegment,

    /// The condition that optionally further restricts the selector.
    /// Must be [truthy](crate::values::PropertyValue::is_truthy) to pass.
    pub condition: Option<Expression>,
}

impl From<SelectorSegment> for RestrictedSelectorSegment {
    fn from(segment: SelectorSegment) -> Self {
        Self {
            segment,
            condition: None,
        }
    }
}

/// Full selector, defined by a selector path that must match,
/// and tail decorators that specify which selectable element
/// was exactly selected.
pub struct Selector {
    /// Path that must match in order to select something.
    pub path: SelectorPath,

    /// Specifies whether the selector selects the last
    /// edge it matched instead of the node at the end of that edge.
    pub selects_edge: bool,

    /// Specifies whether the selector selects an extra element
    /// attached to the matched node or edge, instead of the node
    /// or edge directly.
    pub extra: Option<String>,
}

impl Selector {
    /// Shorthand for constructing a selector that matches a node.
    pub fn from_path(path: SelectorPath) -> Self {
        Self {
            path,
            selects_edge: false,
            extra: None,
        }
    }

    /// Shorthand for setting the [`Selector::selects_edge`] flag.
    pub fn selecting_edge(self) -> Self {
        Self {
            path: self.path,
            selects_edge: true,
            extra: self.extra,
        }
    }

    /// Shorthand for adding an [`Selector::extra`] tag.
    pub fn with_extra(self, extra: String) -> Self {
        Self {
            path: self.path,
            selects_edge: self.selects_edge,
            extra: Some(extra),
        }
    }
}

/// Selector that is limited to a single path
/// and exact matches for edges (edges other than [`EdgeMatcher::Exact`])
/// are not allowed.
///
/// These selectors can always unambiguously select at most one entity.
#[derive(Clone, PartialEq, Eq)]
pub struct LimitedSelector {
    /// Path that must be matched in order to select something.
    pub path: Vec<LimitedSelectorSegment>,

    /// Specifies whether the selector selects an extra element
    /// attached to the matched node or edge, instead of the node
    /// or edge directly.
    pub extra_label: Option<String>,
}

impl LimitedSelector {
    /// Shorthand for constructing a limited selector that matches a node.
    pub fn from_path(path: impl IntoIterator<Item = LimitedSelectorSegment>) -> Self {
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

/// Unambiguous [`EdgeLabel`] matcher that is optionally further restricted
/// by a condition. If the condition does not evaluate to a
/// [truthy](crate::values::PropertyValue::is_truthy)
/// value, the selector segment does not match anything.
#[derive(Clone, PartialEq, Eq)]
pub struct LimitedSelectorSegment {
    /// The edge label for the initial match.
    pub edge_label: EdgeLabel,

    /// The condition that optionally further restricts the selector.
    /// Must be [truthy](crate::values::PropertyValue::is_truthy) to pass.
    pub condition: Option<Expression>,
}

impl std::fmt::Debug for LimitedSelectorSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.edge_label)?;
        if let Some(condition) = &self.condition {
            write!(f, ".if({condition:?})")?;
        }
        Ok(())
    }
}

impl From<EdgeLabel> for LimitedSelectorSegment {
    fn from(edge_label: EdgeLabel) -> Self {
        Self {
            edge_label,
            condition: None,
        }
    }
}
