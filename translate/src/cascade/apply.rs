//! Evaluation of an entire stylesheet.

use super::{
    eval::{
        context::{EvaluationContext, EvaluationOnGraph},
        evaluate,
        variable_pool::VariablePool,
    },
    flat_selector::FlatSelectorSegment,
    flat_stylesheet::FlatStylesheet,
};
use crate::{
    property::*,
    stylesheet::{StyleKey, expression::LimitedSelector, selector::EdgeMatcher},
};
use aili_model::state::{
    EdgeLabel, NodeId, NodeValue, ProgramStateGraph, ProgramStateNode, RootedProgramStateGraph,
};
use std::collections::{HashMap, HashSet};

/// Applies a stylesheet to a graph.
pub fn apply_stylesheet<T: RootedProgramStateGraph>(
    stylesheet: &FlatStylesheet,
    graph: &T,
) -> EntityPropertyMapping<T::NodeId> {
    let mut helper = ApplyStylesheet::new(stylesheet, graph);
    helper.run();
    helper.result()
}

/// Name of the magic variable that stores the index
/// of the [`EdgeLabel::Index`] edge that leads to
/// the current node, if any.
pub const VARIABLE_INDEX: &str = "--INDEX";

/// Name of the magic variable that stores the name
/// of the [`EdgeLabel::Named`] edge that leads to
/// the current node, if any
pub const VARIABLE_NAME: &str = "--NAME";

/// Name of the magic variable that stores the discriminator
/// of the [`EdgeLabel::Named`] edge that leads to
/// the current node, if any
pub const VARIABLE_DISCRIMINATOR: &str = "--DISCRIMINATOR";

/// Identifier of a property variable on an entity.
#[derive(PartialEq, Eq, Debug, Hash)]
struct EntityPropertyKey<T: NodeId>(Selectable<T>, PropertyKey);

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
    properties: HashMap<EntityPropertyKey<T::NodeId>, PropertyValue<T::NodeId>>,

    /// Variables that are active at the moment
    variable_pool: VariablePool<&'a str, T::NodeId>,
}

struct GraphPoolEvaluationContext<'a, T: ProgramStateGraph> {
    graph: &'a T,
    origin: T::NodeId,
    variable_pool: &'a VariablePool<&'a str, T::NodeId>,
}

impl<T: ProgramStateGraph> ProgramStateGraph for GraphPoolEvaluationContext<'_, T> {
    type NodeRef<'a>
        = T::NodeRef<'a>
    where
        Self: 'a;
    type NodeId = T::NodeId;
    fn get(&self, id: &Self::NodeId) -> Option<Self::NodeRef<'_>> {
        self.graph.get(id)
    }
}

impl<T: ProgramStateGraph> EvaluationContext for GraphPoolEvaluationContext<'_, T> {
    fn select_entity(&self, selector: &LimitedSelector) -> Option<Selectable<Self::NodeId>> {
        EvaluationOnGraph::new(self.graph, self.origin.clone()).select_entity(selector)
    }
    fn get_variable_value(&self, name: &str) -> PropertyValue<Self::NodeId> {
        self.variable_pool.get(name).cloned().unwrap_or_default()
    }
}

impl<'a, 'g, T: RootedProgramStateGraph> ApplyStylesheet<'a, 'g, T> {
    fn new(stylesheet: &'a FlatStylesheet, graph: &'g T) -> Self {
        Self {
            graph,
            stylesheet,
            matched_sequence_points: HashSet::new(),
            properties: HashMap::new(),
            variable_pool: VariablePool::new(),
        }
    }

    fn result(self) -> EntityPropertyMapping<T::NodeId> {
        let mut mapping = EntityPropertyMapping::new();
        for (EntityPropertyKey(entity, property), value) in self.properties {
            let entity_properties = mapping.0.entry(entity).or_insert_with(PropertyMap::default);
            match property {
                PropertyKey::Attribute(name) => {
                    let value = if let PropertyValue::Selection(sel) = &value {
                        if sel.extra_label.is_none() && sel.edge_label.is_none() {
                            self.graph
                                .get(&sel.node_id)
                                .and_then(|node| node.value())
                                .map(Into::into)
                                .unwrap_or_default()
                        } else {
                            PropertyValue::Unset
                        }
                    } else {
                        value
                    };
                    entity_properties.attributes.insert(name, value.to_string());
                }
                PropertyKey::Display => {
                    entity_properties.display = match &value {
                        PropertyValue::Unset => None,
                        PropertyValue::Selection(sel) => {
                            if sel.extra_label.is_none() && sel.edge_label.is_none() {
                                self.graph
                                    .get(&sel.node_id)
                                    .and_then(|node| node.value())
                                    .map(PropertyValue::<T::NodeId>::from)
                                    .as_ref()
                                    .map(PropertyValue::to_string)
                                    .map(DisplayMode::from_name)
                            } else {
                                None
                            }
                        }
                        _ => Some(DisplayMode::from_name(value.to_string())),
                    }
                }
                PropertyKey::Parent => {
                    if let PropertyValue::Selection(sel) = value {
                        entity_properties.parent = Some(*sel);
                    }
                }
                PropertyKey::Target => {
                    if let PropertyValue::Selection(sel) = value {
                        entity_properties.target = Some(*sel);
                    }
                }
                PropertyKey::Detach => {}
            }
        }
        mapping
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
                    if evaluate(condition, &self.evaluation_context(node.clone())).is_truthy() {
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
        let Some(node) = self.graph.get(&starting_node) else {
            return;
        };
        for (edge_label, successor_node) in node.successors() {
            // Push a state so we can pop it later
            self.variable_pool.push();
            self.create_edge_identifier_variables(edge_label);
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
            // Discard all variables that were created here
            self.variable_pool.pop();
        }
    }

    fn selected_entity(&mut self, target: Selectable<T::NodeId>, rule_index: usize) {
        let properties = &self.stylesheet.0[rule_index].properties;
        for property in properties {
            let value = evaluate(
                &property.value,
                &self.evaluation_context(target.node_id.clone()),
            );
            match &property.key {
                StyleKey::Property(key) => {
                    let property_key = EntityPropertyKey(target.clone(), key.clone());
                    self.properties.insert(property_key, value);
                }
                StyleKey::Variable(name) => {
                    self.variable_pool.insert(name, value);
                }
            }
        }
    }

    fn evaluation_context(&self, origin: T::NodeId) -> impl EvaluationContext<NodeId = T::NodeId> {
        GraphPoolEvaluationContext {
            graph: self.graph,
            origin,
            variable_pool: &self.variable_pool,
        }
    }

    fn create_edge_identifier_variables(&mut self, edge_label: &EdgeLabel) {
        match edge_label {
            EdgeLabel::Index(i) => {
                self.variable_pool.insert(
                    VARIABLE_INDEX,
                    PropertyValue::Value(NodeValue::Uint(*i as u64)),
                );
                self.variable_pool
                    .insert(VARIABLE_NAME, PropertyValue::Unset);
                self.variable_pool
                    .insert(VARIABLE_DISCRIMINATOR, PropertyValue::Unset);
            }
            EdgeLabel::Named(name, i) => {
                self.variable_pool
                    .insert(VARIABLE_INDEX, PropertyValue::Unset);
                self.variable_pool
                    .insert(VARIABLE_NAME, PropertyValue::String(name.clone()));
                self.variable_pool.insert(
                    VARIABLE_DISCRIMINATOR,
                    PropertyValue::Value(NodeValue::Uint(*i as u64)),
                );
            }
            _ => {
                // Clear all the variables that may have been set in previous steps
                self.variable_pool
                    .insert(VARIABLE_INDEX, PropertyValue::Unset);
                self.variable_pool
                    .insert(VARIABLE_NAME, PropertyValue::Unset);
                self.variable_pool
                    .insert(VARIABLE_DISCRIMINATOR, PropertyValue::Unset);
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

impl DisplayMode {
    const CONNECTOR_NAME: &'static str = "connector";

    fn from_name(name: String) -> Self {
        match name.as_str() {
            Self::CONNECTOR_NAME => Self::Connector,
            _ => Self::ElementTag(name),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{PropertyKey::*, *};
    use crate::{
        cascade::test_graph::TestGraph,
        stylesheet::{StyleKey::*, expression::*, selector::*, *},
    };

    #[test]
    fn apply_stylesheet_with_one_rule() {
        // .many(*) "a" {
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
            properties: vec![StyleClause {
                key: Property(Display),
                value: Expression::String("cell".to_owned()),
            }],
        }]));
        let expected_properties = PropertyMap {
            display: Some(DisplayMode::ElementTag("cell".to_owned())),
            ..PropertyMap::default()
        };
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (Selectable::node(5), expected_properties.clone()),
                (Selectable::node(6), expected_properties.clone()),
                (Selectable::node(7), expected_properties.clone()),
                (Selectable::node(10), expected_properties.clone()),
                (Selectable::node(11), expected_properties.clone()),
                (Selectable::node(12), expected_properties.clone()),
            ]
            .into()
        );
    }

    #[test]
    fn apply_stylesheet_with_multiple_rules() {
        // .many(*) [] {
        //   display: "cell";
        // }
        // :: main .many(next) {
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
                properties: vec![StyleClause {
                    key: Property(Display),
                    value: Expression::String("cell".to_owned()),
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
                    StyleClause {
                        key: Property(Display),
                        value: Expression::String("kvt".to_owned()),
                    },
                    StyleClause {
                        key: Property(Attribute("title".to_owned())),
                        value: Expression::Int(42),
                    },
                ],
            },
        ]));
        let expected_properties_1 = PropertyMap {
            display: Some(DisplayMode::ElementTag("cell".to_owned())),
            ..PropertyMap::default()
        };
        let expected_properties_2 = PropertyMap {
            display: Some(DisplayMode::ElementTag("kvt".to_owned())),
            attributes: [("title".to_owned(), "42".to_owned())].into(),
            ..PropertyMap::default()
        };
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (Selectable::node(1), expected_properties_2.clone()),
                (Selectable::node(2), expected_properties_2.clone()),
                (Selectable::node(3), expected_properties_2.clone()),
                (Selectable::node(4), expected_properties_2.clone()),
                (Selectable::node(8), expected_properties_1.clone()),
                (Selectable::node(12), expected_properties_1.clone()),
                (Selectable::node(13), expected_properties_1.clone()),
            ]
            .into()
        );
    }

    #[test]
    fn select_extra_entity() {
        // :: main::extra {
        //   display: "cell";
        // }
        //
        // :: main next::extra(abc) {
        //   display: "kvt";
        // }
        let stylesheet = FlatStylesheet::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into()).into()].into(),
                )
                .with_extra("".to_owned()),
                properties: vec![StyleClause {
                    key: Property(Display),
                    value: Expression::String("cell".to_owned()),
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
                properties: vec![StyleClause {
                    key: Property(Display),
                    value: Expression::String("kvt".to_owned()),
                }],
            },
        ]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (
                    Selectable::node(1).with_extra(Some("".to_owned())),
                    PropertyMap {
                        display: Some(DisplayMode::ElementTag("cell".to_owned())),
                        ..PropertyMap::default()
                    },
                ),
                (
                    Selectable::node(2).with_extra(Some("abc".to_owned())),
                    PropertyMap {
                        display: Some(DisplayMode::ElementTag("kvt".to_owned())),
                        ..PropertyMap::default()
                    },
                ),
            ]
            .into()
        );
    }

    #[test]
    fn select_edge() {
        // .many(*).if(@("a"#0))::edge {
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
            properties: vec![StyleClause {
                key: Property(Display),
                value: Expression::String("cell".to_owned()),
            }],
        }]));
        let expected_properties = PropertyMap {
            display: Some(DisplayMode::ElementTag("cell".to_owned())),
            ..PropertyMap::default()
        };
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (
                    Selectable::edge(0, EdgeLabel::Main),
                    expected_properties.clone(),
                ),
                (
                    Selectable::edge(0, EdgeLabel::Named("a".to_owned(), 0)),
                    expected_properties.clone(),
                ),
                (
                    Selectable::edge(1, EdgeLabel::Named("a".to_owned(), 0)),
                    expected_properties.clone(),
                ),
                (
                    Selectable::edge(2, EdgeLabel::Next),
                    expected_properties.clone(),
                ),
                (
                    Selectable::edge(5, EdgeLabel::Named("a".to_owned(), 0)),
                    expected_properties.clone(),
                ),
                (
                    Selectable::edge(5, EdgeLabel::Deref),
                    expected_properties.clone(),
                ),
                (
                    Selectable::edge(7, EdgeLabel::Deref),
                    expected_properties.clone(),
                ),
                (
                    Selectable::edge(12, EdgeLabel::Deref),
                    expected_properties.clone(),
                ),
            ]
            .into()
        );
    }

    #[test]
    fn coerce_values() {
        // :: {
        //   display: true;
        //   target: @(main);
        // }
        //
        // :: "a" {
        //   value: @;
        //   display: @([0]);
        // }
        let stylesheet = FlatStylesheet::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![
                    StyleClause {
                        key: Property(Display),
                        value: Expression::Bool(true),
                    },
                    StyleClause {
                        key: Property(Target),
                        value: Expression::Select(
                            LimitedSelector::from_path([EdgeLabel::Main]).into(),
                        ),
                    },
                ],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into()].into(),
                ),
                properties: vec![
                    StyleClause {
                        key: Property(Attribute("value".to_owned())),
                        value: Expression::Select(LimitedSelector::default().into()),
                    },
                    StyleClause {
                        key: Property(Display),
                        value: Expression::Select(
                            LimitedSelector::from_path([EdgeLabel::Index(0)]).into(),
                        ),
                    },
                ],
            },
        ]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (
                    Selectable::node(0),
                    PropertyMap {
                        display: Some(DisplayMode::ElementTag("true".to_owned())),
                        target: Some(Selectable::node(1)),
                        ..PropertyMap::default()
                    },
                ),
                (
                    Selectable::node(5),
                    PropertyMap {
                        display: None,
                        attributes: HashMap::from_iter([(
                            "value".to_owned(),
                            TestGraph::NUMERIC_NODE_VALUE.to_string()
                        )]),
                        ..PropertyMap::default()
                    },
                ),
            ]
            .into()
        );
    }

    /// This test verifies simple saving and restoring of variables.
    ///
    /// Root node saves a reference to itself in a variable,
    /// which is then recalled by a successor node.
    #[test]
    fn save_variable_at_root() {
        // :: {
        //   --root: @;
        // }
        //
        // :: main {
        //   parent: --root;
        // }
        let stylesheet = FlatStylesheet::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--root".to_owned()),
                    value: Expression::Select(LimitedSelector::default().into()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into()).into()].into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Parent),
                    value: Expression::Variable("--root".to_owned()),
                }],
            },
        ]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [(
                Selectable::node(1),
                PropertyMap {
                    // Reference to the root node should have been loaded from the variable
                    parent: Some(Selectable::node(0)),
                    ..PropertyMap::default()
                },
            )]
            .into()
        );
    }

    /// This test ensures that evaluation of individual clauses in a rule
    /// is sequentially consistent.
    ///
    /// When clauses depend on one another, they must be evaluated
    /// in the order they are written.
    #[test]
    fn variable_assignment_sequential_consistency() {
        // :: {
        //   --i: 0;
        //   a: --i;
        //   --i: --i + 1;
        //   b: --i;
        //   --i: --i + 2;
        //   c: --i;
        // }
        let stylesheet = FlatStylesheet::from(Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: Variable("--i".to_owned()),
                    value: Expression::Int(0),
                },
                StyleClause {
                    key: Property(Attribute("a".to_owned())),
                    value: Expression::Variable("--i".to_owned()),
                },
                StyleClause {
                    key: Variable("--i".to_owned()),
                    value: Expression::BinaryOperator(
                        Expression::Variable("--i".to_owned()).into(),
                        BinaryOperator::Plus,
                        Expression::Int(1).into(),
                    ),
                },
                StyleClause {
                    key: Property(Attribute("b".to_owned())),
                    value: Expression::Variable("--i".to_owned()),
                },
                StyleClause {
                    key: Variable("--i".to_owned()),
                    value: Expression::BinaryOperator(
                        Expression::Variable("--i".to_owned()).into(),
                        BinaryOperator::Plus,
                        Expression::Int(2).into(),
                    ),
                },
                StyleClause {
                    key: Property(Attribute("c".to_owned())),
                    value: Expression::Variable("--i".to_owned()),
                },
            ],
        }]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [(
                Selectable::node(0),
                PropertyMap {
                    // Reference to the root node should have been loaded from the variable
                    attributes: HashMap::from_iter([
                        ("a".to_owned(), "0".to_owned()),
                        ("b".to_owned(), "1".to_owned()),
                        ("c".to_owned(), "3".to_owned()),
                    ]),
                    ..PropertyMap::default()
                },
            )]
            .into()
        );
    }

    /// This test servesas a proof of concept of depth limitation
    /// and verifies that it works asexpected.
    ///
    /// A depth-tracking variable is initialized in the root node
    /// and then incremented on each match. Nodes only match until
    /// the variable reaches the depth limit.
    ///
    /// Note that the continuation condition is inside of the `.many`
    /// matcher instead of after it. This is more efficient as the
    /// condition is verified on every iteration, not just at the end,
    /// and the selector stops traversing as soon as depth limit is reached.
    /// If the condition were outside the `.many` matcher,
    /// the resolver would traverse the graph to arbitrary depth and then
    /// filter out the nodes that exceed the depth limit.
    #[test]
    fn max_depth_using_variables() {
        // :: {
        //   --depth: 0;
        // }
        //
        // :: main .many(next.if(--depth < 3)) {
        //   value: --depth;
        //   --depth: --depth + 1;
        // }
        let stylesheet = FlatStylesheet::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--depth".to_owned()),
                    value: Expression::Int(0),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::Match(EdgeLabel::Main.into()).into(),
                        SelectorSegment::AnyNumberOfTimes(
                            [RestrictedSelectorSegment {
                                segment: SelectorSegment::Match(EdgeLabel::Next.into()),
                                condition: Some(Expression::BinaryOperator(
                                    Expression::Variable("--depth".to_owned()).into(),
                                    BinaryOperator::Lt,
                                    Expression::Int(3).into(),
                                )),
                            }]
                            .into(),
                        )
                        .into(),
                    ]
                    .into(),
                ),
                properties: vec![
                    StyleClause {
                        key: Property(Attribute("value".to_owned())),
                        value: Expression::Variable("--depth".to_owned()),
                    },
                    StyleClause {
                        key: Variable("--depth".to_owned()),
                        value: Expression::BinaryOperator(
                            Expression::Variable("--depth".to_owned()).into(),
                            BinaryOperator::Plus,
                            Expression::Int(1).into(),
                        ),
                    },
                ],
            },
        ]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (
                    Selectable::node(1),
                    PropertyMap {
                        attributes: HashMap::from_iter([("value".to_owned(), "0".to_owned())]),
                        ..PropertyMap::default()
                    },
                ),
                (
                    Selectable::node(2),
                    PropertyMap {
                        attributes: HashMap::from_iter([("value".to_owned(), "1".to_owned())]),
                        ..PropertyMap::default()
                    },
                ),
                (
                    Selectable::node(3),
                    PropertyMap {
                        attributes: HashMap::from_iter([("value".to_owned(), "2".to_owned())]),
                        ..PropertyMap::default()
                    },
                ),
            ]
            .into()
        );
    }

    #[test]
    fn magic_edge_label_variables() {
        // .many(*).if(isset(--INDEX)) {
        //   value: --INDEX;
        // }
        let stylesheet = FlatStylesheet::from(Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [RestrictedSelectorSegment {
                    segment: SelectorSegment::anything_any_number_of_times(),
                    condition: Some(Expression::UnaryOperator(
                        UnaryOperator::IsSet,
                        Expression::Variable(VARIABLE_INDEX.to_owned()).into(),
                    )),
                }]
                .into(),
            ),
            properties: vec![StyleClause {
                key: Property(Attribute("value".to_owned())),
                value: Expression::Variable(VARIABLE_INDEX.to_owned()),
            }],
        }]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(
            resolved,
            [
                (
                    Selectable::node(8),
                    PropertyMap {
                        attributes: HashMap::from_iter([("value".to_owned(), "0".to_owned())]),
                        ..PropertyMap::default()
                    },
                ),
                (
                    Selectable::node(12),
                    PropertyMap {
                        attributes: HashMap::from_iter([("value".to_owned(), "1".to_owned())]),
                        ..PropertyMap::default()
                    },
                ),
                (
                    Selectable::node(13),
                    PropertyMap {
                        attributes: HashMap::from_iter([("value".to_owned(), "0".to_owned())]),
                        ..PropertyMap::default()
                    },
                ),
            ]
            .into()
        );
    }
}
