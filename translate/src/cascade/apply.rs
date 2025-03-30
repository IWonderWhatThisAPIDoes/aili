//! Evaluation of an entire stylesheet.

use super::{
    eval::{
        context::{EvaluationContext, EvaluationOnGraph},
        evaluate,
        variable_pool::VariablePool,
    },
    style::{CascadeStyle, FlatSelectorSegment},
};
use crate::{
    property::*,
    stylesheet::{StyleKey, expression::LimitedSelector, selector::EdgeMatcher},
};
use aili_model::state::{
    EdgeLabel, NodeId, NodeValue, ProgramStateGraph, ProgramStateNode, RootedProgramStateGraph,
};
use std::collections::{HashMap, HashSet, hash_map::Entry};

/// Applies a stylesheet to a graph.
pub fn apply_stylesheet<T: RootedProgramStateGraph>(
    stylesheet: &CascadeStyle,
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

/// Value assigned to a property variable based on a rule
#[derive(Debug)]
struct RulePropertyValue<T: NodeId> {
    /// Value assigned to the property.
    value: PropertyValue<T>,
    /// Index of the rule that assigned the value.
    /// Relevant for calculating precedence.
    rule_index: usize,
    /// Whether the value was assigned explicitly
    /// or as the side effect of another assignment.
    passive: bool,
}

impl<T: NodeId> RulePropertyValue<T> {
    /// Overwrites the existing value with a new one, but only
    /// if the new value has greater or equal precedence.
    ///
    /// ## Return Value
    /// True if the new value was written, false otherwise.
    fn assign_new_value(&mut self, candidate_value: Self) -> bool {
        // Passive assignments take lower priority always,
        // otherwise the precedence is decided based on evaluation order
        let precedence = |value: &Self| (!value.passive, value.rule_index);
        if precedence(&candidate_value) >= precedence(self) {
            *self = candidate_value;
            true
        } else {
            false
        }
    }
}

/// Helper for stylesheet applications.
struct ApplyStylesheet<'a, 'g, T: RootedProgramStateGraph> {
    /// The graph being traversed.
    graph: &'g T,

    /// The stylesheet being evaluated.
    stylesheet: &'a CascadeStyle,

    /// Pairs of nodes and selector sequence points
    /// that have already been matched.
    ///
    /// Each node can only be matched by each sequence point
    /// once. If it is matched again, the match fails.
    ///
    /// A sequence point is a [`MatchNode`](super::style::FlatSelectorSegment::MatchNode)
    /// transition in the state machine.
    matched_sequence_points: HashSet<(T::NodeId, SequencePointRef)>,

    /// Values assigned to each property on each node.
    properties: HashMap<EntityPropertyKey<T::NodeId>, RulePropertyValue<T::NodeId>>,

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
    fn new(stylesheet: &'a CascadeStyle, graph: &'g T) -> Self {
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
        for (EntityPropertyKey(entity, property), RulePropertyValue { value, .. }) in
            self.properties
        {
            // Insert the property map lazily
            let entity_properties = || mapping.0.entry(entity).or_insert_with(PropertyMap::default);
            match property {
                PropertyKey::Attribute(name) => {
                    let value = if let PropertyValue::Selection(sel) = &value {
                        if sel.is_node() {
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
                    // If value if Unset, the attribute should not be saved at all
                    if value != PropertyValue::Unset {
                        entity_properties()
                            .attributes
                            .insert(name, value.to_string());
                    }
                }
                PropertyKey::Display => {
                    let display_mode = match &value {
                        PropertyValue::Unset => None,
                        PropertyValue::Selection(sel) => {
                            if sel.is_node() {
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
                    };
                    if display_mode.is_some() {
                        entity_properties().display = display_mode;
                    }
                }
                PropertyKey::Parent => {
                    if let PropertyValue::Selection(sel) = value {
                        entity_properties().parent = Some(*sel);
                    }
                }
                PropertyKey::Target => {
                    if let PropertyValue::Selection(sel) = value {
                        entity_properties().target = Some(*sel);
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
        self.run_from(
            self.graph.root(),
            starting_states,
            None,
            None,
            &AutoAssignmentContext::default(),
        );
    }

    /// Traverses depth-first from a specified node and evaluates the selector.
    fn run_from(
        &mut self,
        node: T::NodeId,
        starting_states: impl IntoIterator<Item = SequencePointRef>,
        previous_node: Option<T::NodeId>,
        previous_edge: Option<&EdgeLabel>,
        auto_context: &AutoAssignmentContext<T::NodeId>,
    ) {
        let ResolveNodeResult {
            output_states,
            mut matched_rules,
        } = self.resolve_node(node.clone(), starting_states);

        // Resolve rules in correct order
        matched_rules.sort_by_cached_key(|&(rule_index, matched_node)| {
            let has_extra = self.stylesheet.0[rule_index].machine.extra.is_some();
            // Primary ordering: incoming edge before node
            // Secondary ordering: nodes and edges before extras
            // Tertiary ordering: declaration order in the stylesheet
            (matched_node, has_extra, rule_index)
        });

        // The auto-assignment context we are going to pass along
        // to the next layer
        let mut new_auto_context = auto_context.clone();

        // Resolve all entities that matched
        for (rule_index, selected_node) in matched_rules {
            let selected = if selected_node {
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
            .with_extra(self.stylesheet.0[rule_index].machine.extra.clone());
            self.selected_entity(
                selected,
                &node,
                rule_index,
                auto_context,
                &mut new_auto_context,
            );
        }

        // This is our termination condition:
        // We stop once there is nothing else to explore
        if output_states.is_empty() {
            return;
        }

        // Traverse down the tree through all edges
        self.traverse_outgoing_edges(node, &output_states, &new_auto_context);
    }

    /// Runs segments of the state machine at a given node.
    fn resolve_node(
        &mut self,
        node: T::NodeId,
        starting_states: impl IntoIterator<Item = SequencePointRef>,
    ) -> ResolveNodeResult<'a> {
        // States of the selector state machine that have been visited
        // while evaluating this node
        let mut visited_states = HashSet::new();
        // States that are yet to be visited and whether the node has already
        // been committed when we reach them
        let mut open_states = Vec::from_iter(starting_states.into_iter().map(|s| (s, false)));
        // States that are blocked by an edge matcher
        // and must be resolved by traversing further down the graph
        let mut output_states = Vec::new();
        // Rules whose selector selected this element or a related entity
        let mut matched_rules = Vec::new();

        // Make a transitive closure of selector states reachable at this node
        while let Some((state, committed)) = open_states.pop() {
            let selector = &self.stylesheet.0[state.rule_index].machine;
            if state.state_index >= selector.path.len() {
                // We made it to the end of the selector
                // That means it has matched the node
                matched_rules.push((state.rule_index, committed));
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

        ResolveNodeResult {
            output_states,
            matched_rules,
        }
    }

    /// Traverses depth-first through all outgoing edges of a node.
    fn traverse_outgoing_edges(
        &mut self,
        starting_node: T::NodeId,
        output_states: &Vec<(&EdgeMatcher, SequencePointRef)>,
        auto_context: &AutoAssignmentContext<T::NodeId>,
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
                auto_context,
            );
            // Discard all variables that were created here
            self.variable_pool.pop();
        }
    }

    fn selected_entity(
        &mut self,
        target: Selectable<T::NodeId>,
        select_origin: &T::NodeId,
        rule_index: usize,
        input_auto_context: &AutoAssignmentContext<T::NodeId>,
        output_auto_context: &mut AutoAssignmentContext<T::NodeId>,
    ) {
        // Edges that are selected are automatically displayed as conenctors
        if target.is_edge() {
            // Display as connector
            let display_key = EntityPropertyKey(target.clone(), PropertyKey::Display);
            let display_value = RulePropertyValue {
                value: PropertyValue::String(DisplayMode::CONNECTOR_NAME.to_owned()),
                rule_index,
                passive: true,
            };
            self.write_property(display_key, display_value);
            // Parent is source
            let parent_key = EntityPropertyKey(target.clone(), PropertyKey::Parent);
            let parent_value = RulePropertyValue {
                value: PropertyValue::Selection(Selectable::node(target.node_id.clone()).into()),
                rule_index,
                passive: true,
            };
            self.write_property(parent_key, parent_value);
            // Target is target
            let target_key = EntityPropertyKey(target.clone(), PropertyKey::Target);
            let target_value = RulePropertyValue {
                value: PropertyValue::Selection(Selectable::node(select_origin.clone()).into()),
                rule_index,
                passive: true,
            };
            self.write_property(target_key, target_value);
        }
        // Extra entities get their own variable scope
        // so they cannot affect anything outside
        if target.is_extra() {
            self.variable_pool.push();
        }
        let properties = &self.stylesheet.0[rule_index].properties;
        for property in properties {
            let value = evaluate(
                &property.value,
                &self.evaluation_context(select_origin.clone()),
            );
            match &property.key {
                StyleKey::Property(key) => {
                    let full_key = EntityPropertyKey(target.clone(), key.clone());
                    let full_value = RulePropertyValue {
                        value,
                        rule_index,
                        passive: false,
                    };
                    let updated_property = self.write_property(full_key, full_value);
                    // If we just chaned the display mode of an entity,
                    // we should auto-assign common values to other properties
                    if updated_property && *key == PropertyKey::Display {
                        if target.is_node() {
                            // If the display property of a node is explicitly
                            // assigned, that node becomes the parent of its successors
                            // by default
                            output_auto_context.parent = Some(target.clone());
                            // Likewise, it is adopted by its predecessor, if any
                            if let Some(parent) = &input_auto_context.parent {
                                let parent_key =
                                    EntityPropertyKey(target.clone(), PropertyKey::Parent);
                                let parent_value = RulePropertyValue {
                                    value: PropertyValue::Selection(parent.clone().into()),
                                    rule_index,
                                    passive: true,
                                };
                                self.write_property(parent_key, parent_value);
                            }
                        }
                        if target.is_extra() {
                            // Extra will be adopted by its owner
                            let parent_key = EntityPropertyKey(target.clone(), PropertyKey::Parent);
                            let parent_value = RulePropertyValue {
                                value: PropertyValue::Selection(
                                    target.clone().with_extra(None).into(),
                                ),
                                rule_index,
                                passive: true,
                            };
                            self.write_property(parent_key, parent_value);
                        }
                    }
                }
                StyleKey::Variable(name) => {
                    self.variable_pool.insert(name, value);
                }
            }
        }
        if target.is_extra() {
            self.variable_pool.pop();
        }
    }

    fn evaluation_context(&self, origin: T::NodeId) -> impl EvaluationContext<NodeId = T::NodeId> {
        GraphPoolEvaluationContext {
            graph: self.graph,
            origin,
            variable_pool: &self.variable_pool,
        }
    }

    fn write_property(
        &mut self,
        key: EntityPropertyKey<T::NodeId>,
        value: RulePropertyValue<T::NodeId>,
    ) -> bool {
        match self.properties.entry(key) {
            Entry::Occupied(mut existing) => existing.get_mut().assign_new_value(value),
            Entry::Vacant(entry) => {
                entry.insert(value);
                true
            }
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

/// Result of [`ApplyStylesheet::resolve_node`].
struct ResolveNodeResult<'a> {
    /// States in which the selectors await
    output_states: Vec<(&'a EdgeMatcher, SequencePointRef)>,
    /// Rules that have selected the node,
    /// with a flag indicating whether it was the node that was selected
    /// (true), or the edge leading to it (false)
    matched_rules: Vec<(usize, bool)>,
}

/// Information that must be carried around
/// in order to auto-assign [`PropertyKey::Parent`]
/// and [`PropertyKey::Target`] properties.
#[derive(Clone)]
struct AutoAssignmentContext<T: NodeId> {
    parent: Option<Selectable<T>>,
}

impl<T: NodeId> Default for AutoAssignmentContext<T> {
    fn default() -> Self {
        Self { parent: None }
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
        //   value: 42;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [
                    SelectorSegment::anything_any_number_of_times().into(),
                    SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
                ]
                .into(),
            ),
            properties: vec![StyleClause {
                key: Property(Attribute("value".to_owned())),
                value: Expression::Int(42),
            }],
        }]));
        let expected_properties =
            PropertyMap::new().with_attribute("value".to_owned(), "42".to_owned());
        let expected_mapping = [
            (Selectable::node(5), expected_properties.clone()),
            (Selectable::node(6), expected_properties.clone()),
            (Selectable::node(7), expected_properties.clone()),
            (Selectable::node(10), expected_properties.clone()),
            (Selectable::node(11), expected_properties.clone()),
            (Selectable::node(12), expected_properties.clone()),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn apply_stylesheet_with_multiple_rules() {
        // .many(*) [] {
        //   value: "cell";
        // }
        // :: main .many(next) {
        //   value: "kvt";
        //   title: 42;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::anything_any_number_of_times().into(),
                        SelectorSegment::Match(EdgeMatcher::AnyIndex).into(),
                    ]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".to_owned())),
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
                        key: Property(Attribute("value".to_owned())),
                        value: Expression::String("kvt".to_owned()),
                    },
                    StyleClause {
                        key: Property(Attribute("title".to_owned())),
                        value: Expression::Int(42),
                    },
                ],
            },
        ]));
        let expected_properties_1 =
            PropertyMap::new().with_attribute("value".to_owned(), "cell".to_owned());
        let expected_properties_2 = PropertyMap::new()
            .with_attribute("value".to_owned(), "kvt".to_owned())
            .with_attribute("title".to_owned(), "42".to_owned());
        let expected_mapping = [
            (Selectable::node(1), expected_properties_2.clone()),
            (Selectable::node(2), expected_properties_2.clone()),
            (Selectable::node(3), expected_properties_2.clone()),
            (Selectable::node(4), expected_properties_2.clone()),
            (Selectable::node(8), expected_properties_1.clone()),
            (Selectable::node(12), expected_properties_1.clone()),
            (Selectable::node(13), expected_properties_1.clone()),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
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
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
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
        let expected_mapping = [
            (
                Selectable::node(1).with_extra(Some("".to_owned())),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    // Parent is assigned automatically
                    .with_parent(Selectable::node(1)),
            ),
            (
                Selectable::node(2).with_extra(Some("abc".to_owned())),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("kvt".to_owned()))
                    // Parent is assigned automatically
                    .with_parent(Selectable::node(2)),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn select_edge() {
        // .many(*).if(@("a"#0))::edge { }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
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
            // These are edges, so all we need to do is select them,
            // properties do not need to be assigned
            properties: Vec::new(),
        }]));
        // Display, parent, and target are assigned automatically
        let expected_mapping = [
            (
                Selectable::edge(0, EdgeLabel::Main),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(0))
                    .with_target(Selectable::node(1)),
            ),
            (
                Selectable::edge(0, EdgeLabel::Named("a".to_owned(), 0)),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(0))
                    .with_target(Selectable::node(5)),
            ),
            (
                Selectable::edge(1, EdgeLabel::Named("a".to_owned(), 0)),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(1))
                    .with_target(Selectable::node(10)),
            ),
            (
                Selectable::edge(2, EdgeLabel::Next),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(2))
                    .with_target(Selectable::node(3)),
            ),
            (
                Selectable::edge(5, EdgeLabel::Named("a".to_owned(), 0)),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(5))
                    .with_target(Selectable::node(6)),
            ),
            (
                Selectable::edge(5, EdgeLabel::Deref),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(5))
                    .with_target(Selectable::node(10)),
            ),
            (
                Selectable::edge(7, EdgeLabel::Deref),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(7))
                    .with_target(Selectable::node(5)),
            ),
            (
                Selectable::edge(12, EdgeLabel::Deref),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(12))
                    .with_target(Selectable::node(10)),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
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
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
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
        let expected_mapping = [
            (
                Selectable::node(0),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("true".to_owned()))
                    .with_target(Selectable::node(1)),
            ),
            (
                Selectable::node(5),
                PropertyMap::new()
                    // Parent is assigned automatically
                    .with_parent(Selectable::node(0))
                    .with_attribute(
                        "value".to_owned(),
                        TestGraph::NUMERIC_NODE_VALUE.to_string(),
                    ),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
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
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
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
        let expected_mapping = [(
            Selectable::node(1),
            // Reference to the root node should have been loaded from the variable
            PropertyMap::new().with_parent(Selectable::node(0)),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
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
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
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
        let expected_mapping = [(
            Selectable::node(0),
            PropertyMap::new()
                .with_attribute("a".to_owned(), "0".to_owned())
                .with_attribute("b".to_owned(), "1".to_owned())
                .with_attribute("c".to_owned(), "3".to_owned()),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test serves as a proof of concept of depth limitation
    /// and verifies that it works as expected.
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
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
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
        let expected_mapping = [
            (
                Selectable::node(1),
                PropertyMap::new().with_attribute("value".to_owned(), "0".to_owned()),
            ),
            (
                Selectable::node(2),
                PropertyMap::new().with_attribute("value".to_owned(), "1".to_owned()),
            ),
            (
                Selectable::node(3),
                PropertyMap::new().with_attribute("value".to_owned(), "2".to_owned()),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn magic_edge_label_variables() {
        // .many(*).if(isset(--INDEX)) {
        //   value: --INDEX;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
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
        let expected_mapping = [
            (
                Selectable::node(8),
                PropertyMap::new().with_attribute("value".to_owned(), "0".to_owned()),
            ),
            (
                Selectable::node(12),
                PropertyMap::new().with_attribute("value".to_owned(), "1".to_owned()),
            ),
            (
                Selectable::node(13),
                PropertyMap::new().with_attribute("value".to_owned(), "0".to_owned()),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test case reproduces a discovered bug where
    /// select expressions run from the body of a rule
    /// that selects an edge are not evaluated correctly.
    ///
    /// Select expressions should be evaluated
    /// relative to the target node.
    #[test]
    fn select_expressions_in_edge_rule() {
        // :: {
        //   --root: @(main);
        // }
        //
        // :: main::edge {
        //   parent: --root;
        //   target: @(next);
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--root".to_owned()),
                    value: Expression::Select(LimitedSelector::from_path([EdgeLabel::Main]).into()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into()).into()].into(),
                )
                .selecting_edge(),
                properties: vec![
                    StyleClause {
                        key: Property(Parent),
                        value: Expression::Variable("--root".to_owned()),
                    },
                    StyleClause {
                        key: Property(Target),
                        value: Expression::Select(
                            LimitedSelector::from_path([EdgeLabel::Next]).into(),
                        ),
                    },
                ],
            },
        ]));
        let expected_mapping = [(
            Selectable::edge(0, EdgeLabel::Main),
            PropertyMap::new()
                .with_display(DisplayMode::Connector) // Assigned automatically
                .with_parent(Selectable::node(1))
                .with_target(Selectable::node(2)),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test case verifies that select expressions
    /// in the bodies of rules that select extra entities
    /// are relative to the owning element.
    #[test]
    fn select_expressions_in_extra_rule() {
        // :: ::extra {
        //   parent: @;
        // }
        //
        // :: main::edge::extra {
        //   parent: @;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default().with_extra("".to_owned()),
                properties: vec![StyleClause {
                    key: Property(Parent),
                    value: Expression::Select(LimitedSelector::default().into()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into()).into()].into(),
                )
                .selecting_edge()
                .with_extra("".to_owned()),
                properties: vec![StyleClause {
                    key: Property(Parent),
                    value: Expression::Select(LimitedSelector::default().into()),
                }],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(0).with_extra(Some("".to_owned())),
                PropertyMap::new().with_parent(Selectable::node(0)),
            ),
            (
                Selectable::edge(0, EdgeLabel::Main).with_extra(Some("".to_owned())),
                PropertyMap::new().with_parent(Selectable::node(1)),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test verifies that rules are applied in order of declaration.
    ///
    /// The last rule should override properties set by earlier rules,
    /// even if they are resolved through different paths.
    #[test]
    fn rule_precedence_in_declaration_order() {
        // :: "a" .many(*) ref {
        //   value: cell;
        // }
        //
        // :: main .many(next) "a" {
        //   value: kvt;
        // }
        //
        // .many(*) "b" {
        //   value: graph;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
                        SelectorSegment::anything_any_number_of_times().into(),
                        SelectorSegment::Match(EdgeLabel::Deref.into()).into(),
                    ]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".to_owned())),
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
                        SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
                    ]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".to_owned())),
                    value: Expression::String("kvt".to_owned()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::anything_any_number_of_times().into(),
                        SelectorSegment::Match(EdgeMatcher::Named("b".to_owned())).into(),
                    ]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".to_owned())),
                    value: Expression::String("graph".to_owned()),
                }],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(5),
                PropertyMap::new().with_attribute("value".to_owned(), "cell".to_owned()),
            ),
            (
                Selectable::node(7),
                PropertyMap::new().with_attribute("value".to_owned(), "graph".to_owned()),
            ),
            (
                Selectable::node(9),
                PropertyMap::new().with_attribute("value".to_owned(), "cell".to_owned()),
            ),
            (
                Selectable::node(10),
                PropertyMap::new().with_attribute("value".to_owned(), "kvt".to_owned()),
            ),
            (
                Selectable::node(12),
                PropertyMap::new().with_attribute("value".to_owned(), "cell".to_owned()),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test case reproduces a discovered bug where
    /// variables assigned by earlier rules are not accessible
    /// in later rules, even in the same run.
    #[test]
    fn variable_sequential_consistency_across_rules() {
        // :: {
        //   --a: a;
        // }
        //
        // :: {
        //   value: --a + --b;
        // }
        //
        // :: {
        //   --b: b;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--a".into()),
                    value: Expression::String("a".to_owned()),
                }],
            },
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".into())),
                    value: Expression::BinaryOperator(
                        Expression::Variable("--a".to_owned()).into(),
                        BinaryOperator::Plus,
                        Expression::Variable("--b".to_owned()).into(),
                    ),
                }],
            },
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--b".into()),
                    value: Expression::String("b".to_owned()),
                }],
            },
        ]));
        let expected_mapping = [(
            Selectable::node(0),
            PropertyMap::new().with_attribute("value".to_owned(), "a".to_owned()),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test verifies that variables are inherited correctly
    /// when `::edge` and `::extra` matchers are involved.
    ///
    /// - `::edge` selector should have access to variables
    ///   assigned by its source node
    /// - `::edge` selector should provide variables for its
    ///   target node
    /// - `::extra` matchers should have access to variables
    ///   assigned by their owner entities
    /// - Variables assigned by `::extra` matchers should not
    ///   be visible from anywhere else
    ///
    /// In essence, the variable scope propagation graph
    /// should look as follows.
    /// ```text
    /// [node] --> [edge] --> [node]
    ///     \          \
    ///      v          v
    ///     [extra]    [extra]
    /// ```
    #[test]
    fn variable_scopes_with_edge_and_extra_matchers() {
        // :: main {
        //   value: --a + --b + --c + --d + --e;
        // }
        //
        // :: main::edge::extra {
        //   value: --a + --b + --c + --d + --e;
        //   --e: e;
        // }
        //
        // :: main::edge {
        //   value: --a + --b + --c + --d + --e;
        //   --d: d;
        // }
        //
        // :: ::extra {
        //   value: --a + --b + --c + --d + --e;
        //   --b: b;
        // }
        //
        // :: ::extra(other) {
        //   value: --a + --b + --c + --d + --e;
        //   --c: c;
        // }
        //
        // :: {
        //   value: --a + --b + --c + --d + --e;
        //   --a: a;
        // }
        let value_assignment = StyleClause {
            key: Property(Attribute("value".to_owned())),
            value: Expression::BinaryOperator(
                Expression::BinaryOperator(
                    Expression::BinaryOperator(
                        Expression::BinaryOperator(
                            Expression::Variable("--a".to_owned()).into(),
                            BinaryOperator::Plus,
                            Expression::Variable("--b".to_owned()).into(),
                        )
                        .into(),
                        BinaryOperator::Plus,
                        Expression::Variable("--c".to_owned()).into(),
                    )
                    .into(),
                    BinaryOperator::Plus,
                    Expression::Variable("--d".to_owned()).into(),
                )
                .into(),
                BinaryOperator::Plus,
                Expression::Variable("--e".to_owned()).into(),
            ),
        };
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into()).into()].into(),
                ),
                properties: vec![value_assignment.clone()],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into()).into()].into(),
                )
                .selecting_edge()
                .with_extra("".to_owned()),
                properties: vec![
                    value_assignment.clone(),
                    StyleClause {
                        key: Variable("--e".to_owned()),
                        value: Expression::String("e".to_owned()),
                    },
                ],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into()).into()].into(),
                )
                .selecting_edge(),
                properties: vec![
                    value_assignment.clone(),
                    StyleClause {
                        key: Variable("--d".to_owned()),
                        value: Expression::String("d".to_owned()),
                    },
                ],
            },
            StyleRule {
                selector: Selector::default().with_extra("".to_owned()),
                properties: vec![
                    value_assignment.clone(),
                    StyleClause {
                        key: Variable("--b".to_owned()),
                        value: Expression::String("b".to_owned()),
                    },
                ],
            },
            StyleRule {
                selector: Selector::default().with_extra("other".to_owned()),
                properties: vec![
                    value_assignment.clone(),
                    StyleClause {
                        key: Variable("--c".to_owned()),
                        value: Expression::String("c".to_owned()),
                    },
                ],
            },
            StyleRule {
                selector: Selector::default(),
                properties: vec![
                    value_assignment.clone(),
                    StyleClause {
                        key: Variable("--a".to_owned()),
                        value: Expression::String("a".to_owned()),
                    },
                ],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(0).with_extra(Some("".to_owned())),
                PropertyMap::new().with_attribute("value".to_owned(), "a".to_owned()),
            ),
            (
                Selectable::node(0).with_extra(Some("other".to_owned())),
                PropertyMap::new().with_attribute("value".to_owned(), "a".to_owned()),
            ),
            (
                Selectable::edge(0, EdgeLabel::Main),
                PropertyMap::new()
                    .with_attribute("value".to_owned(), "a".to_owned())
                    // Display, parent, and target assigned automatically
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(0))
                    .with_target(Selectable::node(1)),
            ),
            (
                Selectable::edge(0, EdgeLabel::Main).with_extra(Some("".to_owned())),
                PropertyMap::new().with_attribute("value".to_owned(), "ad".to_owned()),
            ),
            (
                Selectable::node(1),
                PropertyMap::new().with_attribute("value".to_owned(), "ad".to_owned()),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test verifies that if [`PropertyValue::Unset`]
    /// is assigned to a property, the attribute will not
    /// exist in the result.
    #[test]
    fn assigning_unset_erases_property() {
        // :: {
        //   value: none;
        //   display: none;
        //   parent: none;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: Property(Attribute("value".to_owned())),
                    value: Expression::Unset.to_owned(),
                },
                StyleClause {
                    key: Property(Display),
                    value: Expression::Unset.to_owned(),
                },
                StyleClause {
                    key: Property(Parent),
                    value: Expression::Unset.to_owned(),
                },
            ],
        }]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        // The element should not have an entry at all
        assert_eq!(resolved, EntityPropertyMapping::new());
    }

    /// This test verifies that if the same rule
    /// assigns the same property more than once,
    /// the last assignment counts.
    ///
    /// The same rule for variables is already verified by
    /// [`variable_assignment_sequential_consistency`].
    #[test]
    fn latter_property_assignments_take_priority() {
        // :: {
        //   display: connector;
        //   display: none;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: Property(Display),
                    value: Expression::String("connector".to_owned()),
                },
                StyleClause {
                    key: Property(Display),
                    value: Expression::Unset.to_owned(),
                },
            ],
        }]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        // Display property was removed by last assignment,
        // so the mapping should be empty
        assert_eq!(resolved, EntityPropertyMapping::new());
    }

    #[test]
    fn automatic_node_parent_assignment() {
        // :: {
        //   display: graph;
        // }
        //
        // :: .alt(main, main "a", "a", "a" ref "a") {
        //   display: cell;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Property(Display),
                    value: Expression::String("graph".to_owned()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Branch(vec![
                        [SelectorSegment::Match(EdgeLabel::Main.into()).into()].into(),
                        [
                            SelectorSegment::Match(EdgeLabel::Main.into()).into(),
                            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
                        ]
                        .into(),
                        [SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into()].into(),
                        [
                            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
                            SelectorSegment::Match(EdgeLabel::Deref.into()).into(),
                            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())).into(),
                        ]
                        .into(),
                    ])
                    .into()]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Display),
                    value: Expression::String("cell".to_owned()),
                }],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(0),
                PropertyMap::new().with_display(DisplayMode::ElementTag("graph".to_owned())),
            ),
            (
                Selectable::node(1),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    .with_parent(Selectable::node(0)),
            ),
            (
                Selectable::node(5),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    .with_parent(Selectable::node(0)),
            ),
            (
                Selectable::node(10),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    // This node was reached by the (:: main next "a") selector,
                    // so its default parent is resolved along that path
                    .with_parent(Selectable::node(1)),
            ),
            (
                Selectable::node(11),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    // These two nodes were reached by the (:: "a" ref "a") selector,
                    // so although node 10 is along the way, it does not participate
                    // in parent assignment
                    .with_parent(Selectable::node(5)),
            ),
            (
                Selectable::node(12),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    .with_parent(Selectable::node(5)),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }
}
