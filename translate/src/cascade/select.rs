//! Matching against selectors.

use super::{
    eval::{context::EvaluationOnGraph, evaluate},
    flat_selector::FlatSelectorSegment,
};
use crate::stylesheet::selector::EdgeMatcher;
use aili_model::state::{ProgramStateNode, RootedProgramStateGraph};
use std::collections::HashSet;

/// Tests a selector against all nodes in a graph.
pub fn get_selector_matches<T: RootedProgramStateGraph>(
    path: &Vec<FlatSelectorSegment>,
    graph: &T,
) -> HashSet<T::NodeId> {
    let mut helper = GetSelectorMatches::new(path, graph);
    helper.run();
    helper.results()
}

/// Helper for matching selectors against graphs.
struct GetSelectorMatches<'a, T: RootedProgramStateGraph> {
    /// The graph being traversed.
    graph: &'a T,

    /// The selector being matched.
    path: &'a Vec<FlatSelectorSegment>,

    /// Pairs of nodes and selector sequence points
    /// that have already been matched.
    ///
    /// Each node can only be matched by each sequence point
    /// once. If it is matched again, the match fails.
    ///
    /// A sequence point is a [`FlatSelectorSegment::MatchNode`]
    /// transition in the state machine.
    matched_sequence_points: HashSet<(T::NodeId, usize)>,

    /// Nodes that have been selected by the selector.
    selected_nodes: HashSet<T::NodeId>,
}

impl<'a, T: RootedProgramStateGraph> GetSelectorMatches<'a, T> {
    /// Construct a selector matching helper
    fn new(path: &'a Vec<FlatSelectorSegment>, graph: &'a T) -> Self {
        Self {
            path,
            graph,
            matched_sequence_points: HashSet::new(),
            selected_nodes: HashSet::new(),
        }
    }

    /// Retrieves the list of nodes that have been selected.
    fn results(self) -> HashSet<T::NodeId> {
        self.selected_nodes
    }

    /// Evaluates the selector.
    fn run(&mut self) {
        self.run_from(self.graph.root(), [0]);
    }

    /// Traverses depth-first from a specified node and evaluates the selector.
    fn run_from(&mut self, node: T::NodeId, starting_states: impl IntoIterator<Item = usize>) {
        let output_states = self.resolve_node(node.clone(), starting_states);

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
        starting_states: impl IntoIterator<Item = usize>,
    ) -> Vec<(&'a EdgeMatcher, usize)> {
        // States of the selector state machine that have been visited
        // while evaluating this node
        let mut visited_states = vec![false; self.path.len()];
        // States that are yet to be visited and whether the node has already
        // been committed when we reach them
        let mut open_states = Vec::from_iter(starting_states.into_iter().map(|s| (s, false)));
        // States that are blocked by an edge matcher
        // and must be resolved by traversing further down the graph
        let mut output_states = Vec::new();

        // Make a transitive closure of selector states reachable at this node
        while let Some((state, committed)) = open_states.pop() {
            if state >= self.path.len() {
                // We made it to the end of the selector
                // That means it has matched the node
                self.selected_nodes.insert(node.clone());
                continue;
            }
            // Proceed, unless we have been here already
            // This prevents infinite loops caused by poorly written selectors
            if visited_states[state] {
                continue;
            }
            visited_states[state] = true;
            match &self.path[state] {
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
                        open_states.push((state + 1, true));
                    }
                }
                FlatSelectorSegment::Restrict(condition) => {
                    // Proceed only if the condition holds
                    if evaluate(condition, &EvaluationOnGraph::new(self.graph, node.clone()))
                        .is_truthy()
                    {
                        // continue traversing the state machine linearly
                        open_states.push((state + 1, committed));
                    }
                }
                FlatSelectorSegment::Branch(next_state) => {
                    // Continue both linearly and from the indicated state
                    open_states.push((*next_state, committed));
                    open_states.push((state + 1, committed));
                }
                FlatSelectorSegment::Jump(next_state) => {
                    // Continue only from the indicated state
                    open_states.push((*next_state, committed));
                }
            }
        }

        output_states
    }

    /// Traverses depth-first through all outgoing edges of a node.
    fn traverse_outgoing_edges(
        &mut self,
        starting_node: T::NodeId,
        output_states: &Vec<(&EdgeMatcher, usize)>,
    ) {
        let Some(starting_node) = self.graph.get(&starting_node) else {
            return;
        };
        for (edge_label, successor_node) in starting_node.successors() {
            // Start traversing from the next node, from all the states where this node ended
            // and the edge matches the blocking matcher
            let starting_states = output_states
                .iter()
                .copied()
                .filter(|(matcher, _)| matcher.matches(edge_label))
                .map(|(_, state)| state + 1);
            self.run_from(successor_node, starting_states);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        cascade::{flat_selector::FlatSelector, test_graph::TestGraph},
        stylesheet::{expression::*, selector::*},
    };
    use aili_model::state::EdgeLabel;

    #[test]
    fn select_main_and_any_number_of_named() {
        // main iter(named)
        let selector = FlatSelector::from(Selector::from_path(
            [
                SelectorSegment::Match(EdgeLabel::Main.into()).into(),
                SelectorSegment::AnyNumberOfTimes(
                    [SelectorSegment::Match(EdgeMatcher::AnyNamed).into()].into(),
                )
                .into(),
            ]
            .into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::simple_graph());
        assert_eq!(matched, [1, 2, 3, 5].into());
    }

    #[test]
    fn select_all_stack_nodes() {
        // main iter(next)
        let selector = FlatSelector::from(Selector::from_path(
            [
                SelectorSegment::Match(EdgeLabel::Main.into()).into(),
                SelectorSegment::AnyNumberOfTimes(
                    [SelectorSegment::Match(EdgeLabel::Next.into()).into()].into(),
                )
                .into(),
            ]
            .into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::default_graph());
        assert_eq!(matched, [1, 2, 3, 4].into());
    }

    #[test]
    fn select_named_anywhere() {
        // iter(*) "a"
        let selector = FlatSelector::from(Selector::from_path(
            [
                SelectorSegment::anything_any_number_of_times().into(),
                SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
            ]
            .into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::default_graph());
        assert_eq!(matched, [5, 6, 7, 10, 11, 12].into());
    }

    #[test]
    fn select_named_successor_of_named_anywhere() {
        // iter(*) "a" "a"
        let selector = FlatSelector::from(Selector::from_path(
            [
                SelectorSegment::anything_any_number_of_times().into(),
                SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
                SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
            ]
            .into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::default_graph());
        assert_eq!(matched, [6, 11, 12].into());
    }

    #[test]
    fn select_dereference_anywhere_after_double_named() {
        // "a" "a" iter(*) deref
        let selector = FlatSelector::from(Selector::from_path(
            [
                SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
                SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
                SelectorSegment::anything_any_number_of_times().into(),
                SelectorSegment::Match(EdgeLabel::Deref.into()).into(),
            ]
            .into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::default_graph());
        assert_eq!(matched, [5, 9, 10, 12].into());
    }

    #[test]
    fn select_anything_after_result_anywhere() {
        // iter(*) result iter(*)
        let selector = FlatSelector::from(Selector::from_path(
            [
                SelectorSegment::anything_any_number_of_times().into(),
                SelectorSegment::Match(EdgeLabel::Result.into()).into(),
                SelectorSegment::anything_any_number_of_times().into(),
            ]
            .into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::default_graph());
        assert_eq!(matched, [10, 11, 12, 13].into());
    }

    #[test]
    fn select_next_frame_or_named() {
        // main iter(or(next, "a"))
        let selector = FlatSelector::from(Selector::from_path(
            [
                SelectorSegment::Match(EdgeLabel::Main.into()).into(),
                SelectorSegment::AnyNumberOfTimes(
                    [SelectorSegment::Branch(vec![
                        [SelectorSegment::Match(EdgeLabel::Next.into()).into()].into(),
                        [SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into()].into(),
                    ])
                    .into()]
                    .into(),
                )
                .into(),
            ]
            .into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::default_graph());
        assert_eq!(matched, [1, 2, 3, 4, 7, 10, 11, 12].into());
    }

    #[test]
    fn degenerate_repeated_empty_path() {
        // iter()
        let selector = FlatSelector::from(Selector::from_path(
            [SelectorSegment::AnyNumberOfTimes([].into()).into()].into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::default_graph());
        assert_eq!(matched, [0].into());
    }

    #[test]
    fn degenerate_repeated_empty_branch() {
        // iter(or(named, ))
        let selector = FlatSelector::from(Selector::from_path(
            [SelectorSegment::AnyNumberOfTimes(
                [SelectorSegment::Branch(vec![
                    [SelectorSegment::Match(EdgeMatcher::AnyNamed).into()].into(),
                    [].into(),
                ])
                .into()]
                .into(),
            )
            .into()]
            .into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::default_graph());
        assert_eq!(matched, [0, 5, 6, 7, 11].into());
    }

    #[test]
    fn match_with_lookahead() {
        // iter(*).if(@(deref))
        let selector = FlatSelector::from(Selector::from_path(
            [RestrictedSelectorSegment {
                segment: SelectorSegment::anything_any_number_of_times(),
                condition: Some(Expression::Select(
                    LimitedSelector::from_path([EdgeLabel::Deref]).into(),
                )),
            }]
            .into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::default_graph());
        assert_eq!(matched, [5, 7, 8, 12, 13].into());
    }

    #[test]
    fn select_stack_top_node() {
        // main iter(next).if(!@(next))
        let selector = FlatSelector::from(Selector::from_path(
            [
                SelectorSegment::Match(EdgeLabel::Main.into()).into(),
                RestrictedSelectorSegment {
                    segment: SelectorSegment::AnyNumberOfTimes(
                        [SelectorSegment::Match(EdgeLabel::Next.into()).into()].into(),
                    ),
                    condition: Some(Expression::UnaryOperator(
                        UnaryOperator::Not,
                        Expression::Select(LimitedSelector::from_path([EdgeLabel::Next]).into())
                            .into(),
                    )),
                },
            ]
            .into(),
        ));
        let matched = get_selector_matches(&selector.path, &TestGraph::default_graph());
        assert_eq!(matched, [4].into());
    }
}
