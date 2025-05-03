//! Preprocessing of [`Stylesheet`]s to simplify matching.

use crate::stylesheet::{expression::Expression, selector::*, *};
use derive_more::Debug;

/// Compiled stylesheet that can be used to evaluate the cascade.
#[derive(Debug)]
pub struct CascadeStyle<K: PropertyKey = RawPropertyKey> {
    selectors: CascadeSelector,
    rules: Vec<CascadeStyleRule<K>>,
}

impl<K: PropertyKey> CascadeStyle<K> {
    /// Gets the compiled selectors of the stalesheet.
    pub fn selector_machine(&self) -> &CascadeSelector {
        &self.selectors
    }

    /// Gets a rule at a specified index.
    ///
    /// All indices exposed by [`CascadeStyle::selector_machine`]
    /// are valid.
    pub fn rule_at(&self, index: usize) -> &CascadeStyleRule<K> {
        &self.rules[index]
    }
}

impl<K: PropertyKey> From<Stylesheet<K>> for CascadeStyle<K> {
    fn from(value: Stylesheet<K>) -> Self {
        let (selectors, rules) = value
            .0
            .into_iter()
            .map(|mut rule| {
                let extra_label = rule.selector.extra.take();
                let selector = rule.selector.into();
                let body = CascadeStyleRule {
                    extra_label,
                    properties: rule.properties,
                };
                (selector, body)
            })
            .unzip();
        Self {
            selectors: CascadeSelector(selectors),
            rules,
        }
    }
}

/// Compiled bundle of selectors.
#[derive(Debug)]
pub struct CascadeSelector(pub(super) Vec<FlatSelector>);

/// Body of a single rule in a compiled [`CascadeStyle`].
///
/// Contains the body of the rule and an optional extra label.
#[derive(Debug)]
pub struct CascadeStyleRule<K: PropertyKey = RawPropertyKey> {
    /// Specifies whether the selector selects an extra element
    /// attached to the matched node or edge, instead of the node
    /// or edge directly.
    pub extra_label: Option<String>,

    /// Properties in the body of the original rule.
    pub properties: Vec<StyleClause<K>>,
}

/// [`Selector`] flattened to simplify matching against it.
///
/// The selector is represented as essentially a state machine
/// whose edges are the items of [`FlatSelector::path`]
/// and its nodes are indices of those items.
/// The input of the machine is program state nodes and edges
/// in the order they appear in the state graph.
#[derive(PartialEq, Eq)]
pub struct FlatSelector {
    /// State machine of the selector.
    ///
    /// It is mostly [`Selector::path`] flattened into
    /// the transition map of a state machine.
    ///
    /// [`Selector::selects_edge`] is also reflected here,
    /// near the end of the machine, where it is indicated
    /// by whether or not the last transition matches
    /// the target node.
    pub path: Vec<FlatSelectorSegment>,
}

impl std::fmt::Debug for FlatSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.path.iter().enumerate() {
            write!(f, "[{i}] {segment:?}; ")?;
        }
        Ok(())
    }
}

impl From<Selector> for FlatSelector {
    fn from(value: Selector) -> Self {
        let mut path = Vec::new();
        flatten_selector_path(value.path, &mut path);

        // Unless selector is meant to match an edge,
        // match the current node at the end
        if !value.selects_edge {
            path.push(FlatSelectorSegment::MatchNode);
        }

        Self { path }
    }
}

/// Transitions of the state machine.
#[derive(PartialEq, Eq, Debug)]
pub enum FlatSelectorSegment {
    /// Transition that matches a program state edge.
    ///
    /// ### Techical Note
    /// This must alternate with [`MatchNode`](FlatSelectorSegment::MatchNode)
    /// transitions. While it is not required because of a technical limitation,
    /// it is necessary to ensure that the selector always halts.
    /// A [`MatchNode`](FlatSelectorSegment::MatchNode) transition can only trigger
    /// once in a given run of the selector. The requirement of taking it before
    /// each [`MatchEdge`](FlatSelectorSegment::MatchEdge) transition places an upper
    /// bound on the number of times each edge can be traversed.
    ///
    /// If multiple [`MatchEdge`](FlatSelectorSegment::MatchEdge) transitions
    /// are taken in succession, the state machine rejects the path.
    #[debug("-> {_0:?}")]
    MatchEdge(EdgeMatcher),

    /// Transition that matches the current program state node.
    ///
    /// The node is always unambiguous, sice the state machine
    /// starts in the root node and only moves by traversing edges,
    /// always arriving at a specific node.
    #[debug("node")]
    MatchNode,

    /// Transition that does not take any input,
    /// but verifies a condition. The condition must evaluate
    /// to a [truthy](crate::values::PropertyValue::is_truthy)
    /// value in order to take the transition.
    #[debug("if ({_0:?})")]
    Restrict(Expression),

    /// Epsilon transition to a state specified by its index.
    ///
    /// Normally, the next state is implicitly calculated
    /// by incrementing the current state index by one.
    #[debug("j {_0}")]
    Jump(usize),

    /// Double epsilon transition to both the implicit next state
    /// and a state specified by its index.
    ///
    /// Both routes are taken simultaneously. The control flow branches out.
    #[debug("br {_0}")]
    Branch(usize),
}

/// Flattens a selector path to a part of a state machine.
fn flatten_selector_path(path: SelectorPath, output: &mut Vec<FlatSelectorSegment>) {
    for segment in path.0 {
        flatten_selector_segment(segment, output);
    }
}

/// Flattens a selector segment to a part of a state machine.
fn flatten_selector_segment(segment: SelectorSegment, output: &mut Vec<FlatSelectorSegment>) {
    match segment {
        SelectorSegment::Match(edge_matcher) => {
            // Before an edge is matched, we must commit to moving
            // to the current node
            output.push(FlatSelectorSegment::MatchNode);
            output.push(FlatSelectorSegment::MatchEdge(edge_matcher));
        }
        SelectorSegment::AnyNumberOfTimes(path) => {
            /*        +--------------+
             *       v                \
             * --> ( ) --> (path) --> ( )   ( ) -->
             *       \                      ^
             *     ^  +--------------------+
             *     |
             *     +--starting_index
             */

            // Save the index of the starting state
            let starting_index = output.len();
            // Exit the loop by putting a branch transition at its start
            // It is just a placeholder for now, we do not yet know
            // the branch destination state index
            output.push(FlatSelectorSegment::Branch(0));
            // Include the inner path inside the loop
            flatten_selector_path(path, output);
            // Jump back to the start of the loop (before the branch transition)
            output.push(FlatSelectorSegment::Jump(starting_index));
            // Now set the branch transition to go past the loop
            output[starting_index] = FlatSelectorSegment::Branch(output.len());
        }
        SelectorSegment::Branch(branches) => {
            /*                 +-----------------------------------------+
             *        +-------/-----------------------+                   \
             *       /       /                         v                   v
             * --> ( ) --> ( ) --> (branch1) --> ( )   (branch2) --> ( )   (branch3) --> ( ) -->
             *                                     \                    \                ^
             *                                      \                    +--------------+
             *                                       +---------------------------------+
             */

            // Save the index of the starting state
            // so we can correctly set up branch transitions later
            let starting_index = output.len();
            // This will contain the indices of states where each branch,
            // except the last one, ends
            let mut ending_indices = Vec::new();
            let branch_count = branches.len();
            // Jump to the start of each branch (except the first, we are already there)
            // Just placeholders for now, we do not yet know the indices of the target states
            for _ in 0..(branch_count - 1) {
                output.push(FlatSelectorSegment::Branch(0));
            }
            // Include the branches, one by one
            for (i, branch) in branches.into_iter().enumerate() {
                // Except first branch, we now know the target state of the branch
                // transitions at the start
                if i > 0 {
                    output[starting_index + i - 1] = FlatSelectorSegment::Branch(output.len());
                }
                // Include the body of the branch
                flatten_selector_path(branch, output);
                // Except last branch, jump past all the remaining branches
                // Just a placeholder for now, we do not yet know the target state index
                if i < branch_count - 1 {
                    ending_indices.push(output.len());
                    output.push(FlatSelectorSegment::Jump(0));
                }
            }
            // Now we know the index of the target state for the exit jump transitions
            for i in ending_indices {
                output[i] = FlatSelectorSegment::Jump(output.len());
            }
        }
        SelectorSegment::Condition(condition) => {
            // Match if the condition passes
            output.push(FlatSelectorSegment::Restrict(condition));
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use FlatSelectorSegment::*;
    use aili_model::state::EdgeLabel;

    #[test]
    fn flatten_empty_selector() {
        let original_selector = Selector {
            path: SelectorPath(vec![]),
            selects_edge: false,
            extra: None,
        };
        let expected_flat_selector = FlatSelector {
            path: vec![MatchNode],
        };
        assert_eq!(
            FlatSelector::from(original_selector),
            expected_flat_selector
        );
    }

    #[test]
    fn flatten_linear_selector() {
        let original_selector = Selector {
            path: SelectorPath(vec![
                SelectorSegment::Match(EdgeMatcher::AnyNamed),
                SelectorSegment::Match(EdgeLabel::Result.into()),
            ]),
            selects_edge: false,
            extra: None,
        };
        let expected_flat_selector = FlatSelector {
            path: vec![
                MatchNode,
                MatchEdge(EdgeMatcher::AnyNamed),
                MatchNode,
                MatchEdge(EdgeLabel::Result.into()),
                MatchNode,
            ],
        };
        assert_eq!(
            FlatSelector::from(original_selector),
            expected_flat_selector
        );
    }

    #[test]
    fn flatten_repeated_selector() {
        let original_selector = Selector {
            path: SelectorPath(vec![
                SelectorSegment::Match(EdgeLabel::Result.into()),
                SelectorSegment::AnyNumberOfTimes(SelectorPath(vec![
                    SelectorSegment::Match(EdgeMatcher::Any),
                    SelectorSegment::Match(EdgeMatcher::AnyIndex),
                ])),
                SelectorSegment::Match(EdgeLabel::Deref.into()),
            ]),
            selects_edge: false,
            extra: None,
        };
        let expected_flat_selector = FlatSelector {
            path: vec![
                MatchNode,
                MatchEdge(EdgeLabel::Result.into()),
                /* 2 */ Branch(8),
                MatchNode,
                MatchEdge(EdgeMatcher::Any),
                MatchNode,
                MatchEdge(EdgeMatcher::AnyIndex),
                Jump(2),
                /* 8 */ MatchNode,
                MatchEdge(EdgeLabel::Deref.into()),
                MatchNode,
            ],
        };
        assert_eq!(
            FlatSelector::from(original_selector),
            expected_flat_selector
        );
    }

    #[test]
    fn flatten_branched_selector() {
        let original_selector = Selector {
            path: SelectorPath(vec![
                SelectorSegment::Match(EdgeLabel::Result.into()),
                SelectorSegment::Branch(vec![
                    SelectorPath(vec![SelectorSegment::Match(EdgeMatcher::Any)]),
                    SelectorPath(vec![
                        SelectorSegment::Match(EdgeMatcher::AnyNamed),
                        SelectorSegment::Match(EdgeMatcher::Named("hello".to_owned())),
                    ]),
                    SelectorPath(vec![SelectorSegment::Match(EdgeMatcher::AnyIndex)]),
                ]),
                SelectorSegment::Match(EdgeLabel::Deref.into()),
            ]),
            selects_edge: false,
            extra: None,
        };
        let expected_flat_selector = FlatSelector {
            path: vec![
                MatchNode,
                MatchEdge(EdgeLabel::Result.into()),
                Branch(7),
                Branch(12),
                MatchNode,
                MatchEdge(EdgeMatcher::Any),
                Jump(14),
                /* 7 */ MatchNode,
                MatchEdge(EdgeMatcher::AnyNamed),
                MatchNode,
                MatchEdge(EdgeMatcher::Named("hello".to_owned())),
                Jump(14),
                /* 12 */ MatchNode,
                MatchEdge(EdgeMatcher::AnyIndex),
                /* 14 */ MatchNode,
                MatchEdge(EdgeLabel::Deref.into()),
                MatchNode,
            ],
        };
        assert_eq!(
            FlatSelector::from(original_selector),
            expected_flat_selector
        );
    }

    #[test]
    fn flatten_branched_and_repeated_selector() {
        let original_selector = Selector {
            path: SelectorPath(vec![
                SelectorSegment::Match(EdgeLabel::Main.into()),
                SelectorSegment::AnyNumberOfTimes(SelectorPath(vec![
                    SelectorSegment::Match(EdgeLabel::Next.into()),
                    SelectorSegment::Branch(vec![
                        SelectorPath(vec![SelectorSegment::AnyNumberOfTimes(SelectorPath(vec![
                            SelectorSegment::Match(EdgeLabel::Deref.into()),
                        ]))]),
                        SelectorPath(vec![
                            SelectorSegment::Match(EdgeMatcher::AnyIndex),
                            SelectorSegment::Match(EdgeMatcher::Any),
                        ]),
                    ]),
                ])),
            ]),
            selects_edge: false,
            extra: None,
        };
        let expected_flat_selector = FlatSelector {
            path: vec![
                MatchNode,
                MatchEdge(EdgeLabel::Main.into()),
                /* 2 */ Branch(16),
                MatchNode,
                MatchEdge(EdgeLabel::Next.into()),
                Branch(11),
                /* 6 */ Branch(10),
                MatchNode,
                MatchEdge(EdgeLabel::Deref.into()),
                Jump(6),
                /* 10 */ Jump(15),
                /* 11 */ MatchNode,
                MatchEdge(EdgeMatcher::AnyIndex),
                MatchNode,
                MatchEdge(EdgeMatcher::Any),
                /* 15 */ Jump(2),
                /* 16 */ MatchNode,
            ],
        };
        assert_eq!(
            FlatSelector::from(original_selector),
            expected_flat_selector
        );
    }
}
