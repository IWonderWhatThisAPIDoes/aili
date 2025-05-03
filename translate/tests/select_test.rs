//! Tests for matching with [`SelectorResolver`].

mod test_graph;

use aili_model::state::{EdgeLabel, ProgramStateNode, RootedProgramStateGraph};
use aili_style::{
    cascade::{CascadeSelector, CascadeStyle, SelectorResolver},
    eval::context::EvaluationContext,
    stylesheet::{StyleRule, Stylesheet, expression::*, selector::*},
};
use std::collections::HashSet;
use test_graph::TestGraph;

/// Constructs a compiled stylesheet from one selector
fn construct_style(selector: Selector) -> CascadeStyle {
    Stylesheet(vec![StyleRule {
        selector,
        properties: Vec::new(),
    }])
    .into()
}

/// Tests a selector against all nodes in a graph.
fn get_selector_matches<T: RootedProgramStateGraph>(
    path: &CascadeSelector,
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

    /// Resolver that tracks the selector
    resolver: SelectorResolver<'a, T::NodeId>,

    /// Nodes that have been selected by the selector.
    selected_nodes: HashSet<T::NodeId>,
}

impl<'a, T: RootedProgramStateGraph> GetSelectorMatches<'a, T> {
    /// Construct a selector matching helper
    fn new(selector: &'a CascadeSelector, graph: &'a T) -> Self {
        Self {
            graph,
            resolver: SelectorResolver::new(selector),
            selected_nodes: HashSet::new(),
        }
    }

    /// Retrieves the list of nodes that have been selected.
    fn results(self) -> HashSet<T::NodeId> {
        self.selected_nodes
    }

    /// Evaluates the selector.
    fn run(&mut self) {
        self.run_from(self.graph.root());
    }

    /// Traverses depth-first from a specified node and evaluates the selector.
    fn run_from(&mut self, node: T::NodeId) {
        self.resolve_node(node.clone());

        // This is our termination condition:
        // We stop once there is nothing else to explore
        if !self.resolver.has_edges_to_resolve() {
            return;
        }

        // Traverse down the tree through all edges
        self.traverse_outgoing_edges(node);
    }

    /// Runs segments of the state machine at a given node.
    fn resolve_node(&mut self, node: T::NodeId) {
        let context = EvaluationContext::from_graph(self.graph, node.clone());
        let matched = self.resolver.resolve_node(node.clone(), &context);
        if !matched.is_empty() {
            self.selected_nodes.insert(node);
        }
    }

    /// Traverses depth-first through all outgoing edges of a node.
    fn traverse_outgoing_edges(&mut self, starting_node: T::NodeId) {
        let Some(starting_node) = self.graph.get(&starting_node) else {
            return;
        };
        for (edge_label, successor_node) in starting_node.successors() {
            self.resolver.push_edge(edge_label);
            self.run_from(successor_node);
            self.resolver.pop_edge();
        }
    }
}

#[test]
fn select_main_and_any_number_of_named() {
    // main iter(named)
    let style = construct_style(Selector::from_path(
        [
            SelectorSegment::Match(EdgeLabel::Main.into()),
            SelectorSegment::AnyNumberOfTimes(
                [SelectorSegment::Match(EdgeMatcher::AnyNamed)].into(),
            ),
        ]
        .into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [1, 10, 11, 12].into());
}

#[test]
fn select_all_stack_nodes() {
    // main iter(next)
    let style = construct_style(Selector::from_path(
        [
            SelectorSegment::Match(EdgeLabel::Main.into()),
            SelectorSegment::AnyNumberOfTimes(
                [SelectorSegment::Match(EdgeLabel::Next.into())].into(),
            ),
        ]
        .into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [1, 2, 3, 4].into());
}

#[test]
fn select_named_anywhere() {
    // iter(*) "a"
    let style = construct_style(Selector::from_path(
        [
            SelectorSegment::anything_any_number_of_times(),
            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
        ]
        .into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [5, 6, 7, 10, 11, 12].into());
}

#[test]
fn select_named_successor_of_named_anywhere() {
    // iter(*) "a" "a"
    let style = construct_style(Selector::from_path(
        [
            SelectorSegment::anything_any_number_of_times(),
            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
        ]
        .into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [6, 11, 12].into());
}

#[test]
fn select_dereference_anywhere_after_double_named() {
    // "a" "a" iter(*) deref
    let style = construct_style(Selector::from_path(
        [
            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
            SelectorSegment::anything_any_number_of_times(),
            SelectorSegment::Match(EdgeLabel::Deref.into()),
        ]
        .into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [5, 9, 10, 12].into());
}

#[test]
fn select_anything_after_result_anywhere() {
    // iter(*) result iter(*)
    let style = construct_style(Selector::from_path(
        [
            SelectorSegment::anything_any_number_of_times(),
            SelectorSegment::Match(EdgeLabel::Result.into()),
            SelectorSegment::anything_any_number_of_times(),
        ]
        .into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [10, 11, 12, 13].into());
}

#[test]
fn select_next_frame_or_named() {
    // main iter(or(next, "a"))
    let style = construct_style(Selector::from_path(
        [
            SelectorSegment::Match(EdgeLabel::Main.into()),
            SelectorSegment::AnyNumberOfTimes(
                [SelectorSegment::Branch(vec![
                    [SelectorSegment::Match(EdgeLabel::Next.into())].into(),
                    [SelectorSegment::Match(EdgeMatcher::Named("a".to_owned()))].into(),
                ])]
                .into(),
            ),
        ]
        .into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [1, 2, 3, 4, 7, 10, 11, 12].into());
}

#[test]
fn degenerate_repeated_empty_path() {
    // iter()
    let style = construct_style(Selector::from_path(
        [SelectorSegment::AnyNumberOfTimes([].into())].into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [0].into());
}

#[test]
fn degenerate_repeated_empty_branch() {
    // iter(or(named, ))
    let style = construct_style(Selector::from_path(
        [SelectorSegment::AnyNumberOfTimes(
            [SelectorSegment::Branch(vec![
                [SelectorSegment::Match(EdgeMatcher::AnyNamed)].into(),
                [].into(),
            ])]
            .into(),
        )]
        .into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [0, 5, 6, 7, 11].into());
}

#[test]
fn match_with_lookahead() {
    // iter(*).if(@(deref))
    let style = construct_style(Selector::from_path(
        [
            SelectorSegment::anything_any_number_of_times(),
            SelectorSegment::Condition(Expression::Select(
                LimitedSelector::from_path([EdgeLabel::Deref.into()]).into(),
            )),
        ]
        .into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [5, 7, 8, 12, 13].into());
}

#[test]
fn select_stack_top_node() {
    // main iter(next).if(!@(next))
    let style = construct_style(Selector::from_path(
        [
            SelectorSegment::Match(EdgeLabel::Main.into()),
            SelectorSegment::AnyNumberOfTimes(
                [SelectorSegment::Match(EdgeLabel::Next.into())].into(),
            ),
            SelectorSegment::Condition(Expression::UnaryOperator(
                UnaryOperator::Not,
                Expression::Select(LimitedSelector::from_path([EdgeLabel::Next.into()]).into())
                    .into(),
            )),
        ]
        .into(),
    ));
    let matched = get_selector_matches(style.selector_machine(), &TestGraph::default_graph());
    assert_eq!(matched, [4].into());
}
