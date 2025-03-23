//! Evaluation of an entire stylesheet.

use super::{
    eval::{context::EvaluationOnGraph, evaluate},
    flat_selector::FlatSelectorSegment,
    flat_stylesheet::FlatStylesheet,
};
use crate::{
    property::{PropertyValue, Selectable},
    stylesheet::{StylePropertyKey, selector::EdgeMatcher},
};
use aili_model::state::{EdgeLabel, NodeId, ProgramStateNode, RootedProgramStateGraph};
use std::collections::{HashMap, HashSet};

/// Applies a stylesheet to a graph.
pub fn apply_stylesheet<'a, T: RootedProgramStateGraph>(
    stylesheet: &'a FlatStylesheet,
    graph: &T,
) -> HashMap<PropertyKey<'a, T::NodeId>, PropertyValue<T::NodeId>> {
    let mut helper = ApplyStylesheet::new(stylesheet, graph);
    helper.run();
    helper.result()
}

/// Identifier of a property variable on an entity.
#[derive(PartialEq, Eq, Debug, Hash)]
pub struct PropertyKey<'a, T: NodeId>(pub Selectable<T>, pub &'a str);

/// Helper for stylesheet applications.
struct ApplyStylesheet<'a, 'g, T: RootedProgramStateGraph> {
    /// The graph being traversed.
    graph: &'g T,

    /// The stylesheet being evaluated.
    stylesheet: &'a FlatStylesheet,

    /// Pairs of nodes and selector sequence points
    /// that have already been matched.
    ///
    /// Each node can only be matched by each sequence point
    /// once. If it is matched again, the match fails.
    ///
    /// A sequence point is a [`MatchNode`](super::flat_selector::FlatSelectorSegment::MatchNode)
    /// transition in the state machine.
    matched_sequence_points: HashSet<(T::NodeId, SequencePointRef)>,

    /// Values assigned to each property on each node.
    properties: HashMap<PropertyKey<'a, T::NodeId>, PropertyValue<T::NodeId>>,
}

impl<'a, 'g, T: RootedProgramStateGraph> ApplyStylesheet<'a, 'g, T> {
    fn new(stylesheet: &'a FlatStylesheet, graph: &'g T) -> Self {
        Self {
            graph,
            stylesheet,
            matched_sequence_points: HashSet::new(),
            properties: HashMap::new(),
        }
    }

    fn result(self) -> HashMap<PropertyKey<'a, T::NodeId>, PropertyValue<T::NodeId>> {
        self.properties
    }

    fn run(&mut self) {
        let starting_states = (0..self.stylesheet.0.len()).map(|rule_index| SequencePointRef {
            rule_index,
            state_index: 0,
        });
        self.run_from(self.graph.root(), starting_states, None, None);
    }

    /// Traverses depth-first from a specified node and evaluates the selector.
    fn run_from(
        &mut self,
        node: T::NodeId,
        starting_states: impl IntoIterator<Item = SequencePointRef>,
        previous_node: Option<T::NodeId>,
        previous_edge: Option<&EdgeLabel>,
    ) {
        let output_states =
            self.resolve_node(node.clone(), starting_states, previous_node, previous_edge);

        // This is our termination condition:
        // We stop once there is nothing else to explore
        if output_states.is_empty() {
            return;
        }

        // Traverse down the tree through all edges
        self.traverse_outgoing_edges(node, &output_states);
    }

    /// Runs segments of the state machine at a given node.
    fn resolve_node(
        &mut self,
        node: T::NodeId,
        starting_states: impl IntoIterator<Item = SequencePointRef>,
        previous_node: Option<T::NodeId>,
        previous_edge: Option<&EdgeLabel>,
    ) -> Vec<(&'a EdgeMatcher, SequencePointRef)> {
        // States of the selector state machine that have been visited
        // while evaluating this node
        let mut visited_states = HashSet::new();
        // States that are yet to be visited and whether the node has already
        // been committed when we reach them
        let mut open_states = Vec::from_iter(starting_states.into_iter().map(|s| (s, false)));
        // States that are blocked by an edge matcher
        // and must be resolved by traversing further down the graph
        let mut output_states = Vec::new();

        // Make a transitive closure of selector states reachable at this node
        while let Some((state, committed)) = open_states.pop() {
            let selector = &self.stylesheet.0[state.rule_index].machine;
            if state.state_index >= selector.path.len() {
                // We made it to the end of the selector
                // That means it has matched the node
                let selected = if committed {
                    Selectable::node(node.clone())
                } else if let Some(selected) = previous_node.clone().and_then(|node| {
                    previous_edge
                        .cloned()
                        .map(|edge| Selectable::edge(node, edge))
                }) {
                    selected
                } else {
                    continue;
                }
                .with_extra(selector.extra.clone());
                self.selected_entity(selected, state.rule_index);
                continue;
            }
            // Proceed, unless we have been here already
            // This prevents infinite loops caused by poorly written selectors
            if !visited_states.insert(state) {
                continue;
            }
            match &selector.path[state.state_index] {
                FlatSelectorSegment::MatchEdge(matcher) => {
                    // Traversing an edge is only permitted if the node has already been committed
                    // This ensures the resolver halts by only allowing each edge to be traversed once
                    if committed {
                        // This is where we must halt and send the selector
                        // along the edge later on, after we are done with
                        // all partial matches on this node
                        output_states.push((matcher, state));
                    }
                    // TODO: Emit a warning if we fail this check?
                    // This can never happen when using flattened regular selectors
                    // but it is possible to manually construct a flat selector
                    // that does not uphold this invariant
                }
                FlatSelectorSegment::MatchNode => {
                    // Proceed only if the selector has never partially matched
                    // this node in this way
                    if self.matched_sequence_points.insert((node.clone(), state)) {
                        // Continue traversing the state machine linearly
                        // and commit to the node
                        open_states.push((state.advance(), true));
                    }
                }
                FlatSelectorSegment::Restrict(condition) => {
                    // Proceed only if the condition holds
                    if evaluate(condition, &EvaluationOnGraph::new(self.graph, node.clone()))
                        .is_truthy()
                    {
                        // continue traversing the state machine linearly
                        open_states.push((state.advance(), committed));
                    }
                }
                FlatSelectorSegment::Branch(next_state) => {
                    // Continue both linearly and from the indicated state
                    open_states.push((state.jump(*next_state), committed));
                    open_states.push((state.advance(), committed));
                }
                FlatSelectorSegment::Jump(next_state) => {
                    // Continue only from the indicated state
                    open_states.push((state.jump(*next_state), committed));
                }
            }
        }

        output_states
    }

    /// Traverses depth-first through all outgoing edges of a node.
    fn traverse_outgoing_edges(
        &mut self,
        starting_node: T::NodeId,
        output_states: &Vec<(&EdgeMatcher, SequencePointRef)>,
    ) {
        let successors = self
            .graph
            .get(starting_node.clone())
            .into_iter()
            .flat_map(|node| node.successors());
        for (edge_label, successor_node) in successors {
            // Start traversing from the next node, from all the states where this node ended
            // and the edge matches the blocking matcher
            let starting_states = output_states
                .iter()
                .copied()
                .filter(|(matcher, _)| matcher.matches(edge_label))
                .map(|(_, state)| state.advance());
            self.run_from(
                successor_node,
                starting_states,
                Some(starting_node.clone()),
                Some(edge_label),
            );
        }
    }

    fn selected_entity(&mut self, target: Selectable<T::NodeId>, rule_index: usize) {
        let properties = &self.stylesheet.0[rule_index].properties;
        for property in properties {
            match &property.key {
                StylePropertyKey::Property(name) => {
                    self.properties.insert(
                        PropertyKey(target.clone(), name),
                        evaluate(
                            &property.value,
                            &EvaluationOnGraph::new(self.graph, target.node_id.clone()),
                        ),
                    );
                }
                StylePropertyKey::Variable(_) => {
                    todo!()
                }
            }
        }
    }
}

/// Reference to a selector sequence point.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SequencePointRef {
    /// Index of the rule whose selector is being referenced
    /// within the stylesheet.
    rule_index: usize,
    /// Index of the state within the selector.
    state_index: usize,
}

impl SequencePointRef {
    fn advance(self) -> Self {
        self.jump(self.state_index + 1)
    }

    fn jump(self, next_state: usize) -> Self {
        Self {
            rule_index: self.rule_index,
            state_index: next_state,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        cascade::test_graph::TestGraph,
        stylesheet::{StylePropertyKey::*, expression::*, selector::*, *},
    };

    #[test]
    fn apply_stylesheet_with_one_rule() {
        // iter(*) "a" {
        //   display: "cell";
        // }
        let stylesheet = FlatStylesheet::from(Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [
                    SelectorSegment::anything_any_number_of_times().into(),
                    SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
                ]
                .into(),
            ),
            properties: vec![StyleRuleProperty {
                key: Property("display".to_owned()),
                value: Expression::String("cell".to_owned()).into(),
            }],
        }]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (
                    PropertyKey(Selectable::node(5), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(6), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(7), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(10), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(11), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(12), "display"),
                    "cell".to_owned().into()
                ),
            ]
            .into()
        );
    }

    #[test]
    fn apply_stylesheet_with_multiple_rules() {
        // iter(*) index {
        //   display: "cell";
        // }
        // main iter(next) {
        //   display: "kvt";
        //   title: 42;
        // }
        let stylesheet = FlatStylesheet::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::anything_any_number_of_times().into(),
                        SelectorSegment::Match(EdgeMatcher::AnyIndex).into(),
                    ]
                    .into(),
                ),
                properties: vec![StyleRuleProperty {
                    key: Property("display".to_owned()),
                    value: Expression::String("cell".to_owned()).into(),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::Match(EdgeLabel::Main.into()).into(),
                        SelectorSegment::AnyNumberOfTimes(
                            [SelectorSegment::Match(EdgeLabel::Next.into()).into()].into(),
                        )
                        .into(),
                    ]
                    .into(),
                ),
                properties: vec![
                    StyleRuleProperty {
                        key: Property("display".to_owned()),
                        value: Expression::String("kvt".to_owned()).into(),
                    },
                    StyleRuleProperty {
                        key: Property("title".to_owned()),
                        value: Expression::Int(42).into(),
                    },
                ],
            },
        ]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (PropertyKey(Selectable::node(1), "title"), 42u64.into()),
                (PropertyKey(Selectable::node(2), "title"), 42u64.into()),
                (PropertyKey(Selectable::node(3), "title"), 42u64.into()),
                (PropertyKey(Selectable::node(4), "title"), 42u64.into()),
                (
                    PropertyKey(Selectable::node(1), "display"),
                    "kvt".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(2), "display"),
                    "kvt".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(3), "display"),
                    "kvt".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(4), "display"),
                    "kvt".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(8), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(12), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::node(13), "display"),
                    "cell".to_owned().into()
                ),
            ]
            .into()
        );
    }

    #[test]
    fn select_extra_entity() {
        // main::extra {
        //   display: "cell";
        // }
        //
        // main next::extra(abc) {
        //   display: "kvt";
        // }
        let stylesheet = FlatStylesheet::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into()).into()].into(),
                )
                .with_extra("".to_owned()),
                properties: vec![StyleRuleProperty {
                    key: Property("display".to_owned()),
                    value: Expression::String("cell".to_owned()).into(),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::Match(EdgeLabel::Main.into()).into(),
                        SelectorSegment::Match(EdgeLabel::Next.into()).into(),
                    ]
                    .into(),
                )
                .with_extra("abc".to_owned()),
                properties: vec![StyleRuleProperty {
                    key: Property("display".to_owned()),
                    value: Expression::String("kvt".to_owned()).into(),
                }],
            },
        ]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (
                    PropertyKey(
                        Selectable::node(1).with_extra(Some("".to_owned())),
                        "display"
                    ),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(
                        Selectable::node(2).with_extra(Some("abc".to_owned())),
                        "display"
                    ),
                    "kvt".to_owned().into()
                ),
            ]
            .into()
        );
    }

    #[test]
    fn select_edge() {
        // iter(*).if(@("a"#0))::edge {
        //   display: "cell";
        // }
        let stylesheet = FlatStylesheet::from(Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [RestrictedSelectorSegment {
                    segment: SelectorSegment::anything_any_number_of_times(),
                    condition: Expression::Select(
                        LimitedSelector::from_path([EdgeLabel::Named("a".to_owned(), 0)]).into(),
                    )
                    .into(),
                }]
                .into(),
            )
            .selecting_edge(),
            properties: vec![StyleRuleProperty {
                key: Property("display".to_owned()),
                value: Expression::String("cell".to_owned()).into(),
            }],
        }]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (
                    PropertyKey(Selectable::edge(0, EdgeLabel::Main), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(
                        Selectable::edge(0, EdgeLabel::Named("a".to_owned(), 0)),
                        "display"
                    ),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(
                        Selectable::edge(1, EdgeLabel::Named("a".to_owned(), 0)),
                        "display"
                    ),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::edge(2, EdgeLabel::Next), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(
                        Selectable::edge(5, EdgeLabel::Named("a".to_owned(), 0)),
                        "display"
                    ),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::edge(5, EdgeLabel::Deref), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::edge(7, EdgeLabel::Deref), "display"),
                    "cell".to_owned().into()
                ),
                (
                    PropertyKey(Selectable::edge(12, EdgeLabel::Deref), "display"),
                    "cell".to_owned().into()
                ),
            ]
            .into()
        );
    }
}
