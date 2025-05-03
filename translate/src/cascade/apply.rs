//! Evaluation of an entire stylesheet.

use super::mapping_builder::PropertyMappingBuilder;
use crate::property::{EntityPropertyMapping, PropertyKey};
use aili_model::state::{EdgeLabel, ProgramStateNode, RootedProgramStateGraph};
use aili_style::{
    cascade::{CascadeStyle, SelectionCaret, SelectorResolver},
    eval::{context::EvaluationContext, evaluate, variable_pool::VariablePool},
    selectable::Selectable,
    stylesheet::StyleKey,
};

/// Applies a stylesheet to a graph.
pub fn apply_stylesheet<T: RootedProgramStateGraph>(
    stylesheet: &CascadeStyle<PropertyKey>,
    graph: &T,
) -> EntityPropertyMapping<T::NodeId> {
    let mut helper = ApplyStylesheet::new(stylesheet, graph);
    helper.run();
    helper.result()
}

/// Helper for stylesheet applications.
struct ApplyStylesheet<'a, 'g, T: RootedProgramStateGraph> {
    /// The graph being traversed.
    graph: &'g T,

    /// The stylesheet being evaluated.
    stylesheet: &'a CascadeStyle<PropertyKey>,

    /// Resolver that tracks the stylesheet's selectors.
    resolver: SelectorResolver<'a, T::NodeId>,

    /// Builder that constructs the resulting mapping.
    mapping: PropertyMappingBuilder<T::NodeId>,

    /// Variables that are active at the moment
    variable_pool: VariablePool<&'a str, T::NodeId>,
}

impl<'a, 'g, T: RootedProgramStateGraph> ApplyStylesheet<'a, 'g, T> {
    fn new(stylesheet: &'a CascadeStyle<PropertyKey>, graph: &'g T) -> Self {
        Self {
            graph,
            stylesheet,
            resolver: SelectorResolver::new(stylesheet.selector_machine()),
            mapping: PropertyMappingBuilder::new(),
            variable_pool: VariablePool::new(),
        }
    }

    fn result(self) -> EntityPropertyMapping<T::NodeId> {
        self.mapping.build(self.graph)
    }

    fn run(&mut self) {
        self.run_from(self.graph.root(), None, None);
    }

    /// Traverses depth-first from a specified node and evaluates the selector.
    fn run_from(
        &mut self,
        node: T::NodeId,
        previous_node: Option<T::NodeId>,
        previous_edge: Option<&EdgeLabel>,
    ) {
        let matched_rules = self.resolve_node(node.clone(), previous_edge);

        self.mapping.push();

        self.resolve_matched_rules(&node, previous_node, previous_edge, matched_rules);

        // This is our termination condition:
        // We stop once there is nothing else to explore
        if self.resolver.has_edges_to_resolve() {
            // Traverse down the tree through all edges
            self.traverse_outgoing_edges(node);
        }

        self.mapping.pop();
    }

    fn resolve_matched_rules(
        &mut self,
        node: &T::NodeId,
        previous_node: Option<T::NodeId>,
        previous_edge: Option<&EdgeLabel>,
        mut matched_rules: Vec<(usize, SelectionCaret)>,
    ) {
        // Resolve rules in correct order
        matched_rules.sort_by_cached_key(|&(rule_index, caret)| {
            let has_extra = self.stylesheet.rule_at(rule_index).extra_label.is_some();
            // Primary ordering: incoming edge before node
            // Secondary ordering: nodes and edges before extras
            // Tertiary ordering: declaration order in the stylesheet
            (caret == SelectionCaret::Node, has_extra, rule_index)
        });

        // Resolve all entities that matched
        for (rule_index, caret) in matched_rules {
            let mut selected = if caret == SelectionCaret::Node {
                Selectable::node(node.clone())
            } else if let Some(selected) = previous_node.clone().and_then(|node| {
                previous_edge
                    .cloned()
                    .map(|edge| Selectable::edge(node, edge))
            }) {
                selected
            } else {
                continue;
            };
            selected.extra_label = self.stylesheet.rule_at(rule_index).extra_label.clone();
            self.selected_entity(&selected, node, rule_index, previous_edge);
        }
    }

    /// Runs segments of the state machine at a given node.
    fn resolve_node(
        &mut self,
        node: T::NodeId,
        previous_edge: Option<&EdgeLabel>,
    ) -> Vec<(usize, SelectionCaret)> {
        let context =
            Self::evaluation_context(self.graph, &self.variable_pool, node.clone(), previous_edge);
        self.resolver.resolve_node(node, &context)
    }

    /// Traverses depth-first through all outgoing edges of a node.
    fn traverse_outgoing_edges(&mut self, starting_node: T::NodeId) {
        let Some(node) = self.graph.get(&starting_node) else {
            return;
        };
        for (edge_label, successor_node) in node.successors() {
            // Push a state so we can pop it later
            self.variable_pool.push();
            self.resolver.push_edge(edge_label);
            // Resolve the following edge and node
            self.run_from(
                successor_node,
                Some(starting_node.clone()),
                Some(edge_label),
            );
            // Discard all variables that were created here
            self.resolver.pop_edge();
            self.variable_pool.pop();
        }
    }

    fn selected_entity(
        &mut self,
        target: &Selectable<T::NodeId>,
        select_origin: &T::NodeId,
        rule_index: usize,
        previous_edge: Option<&EdgeLabel>,
    ) {
        // Adjust the mapping to the new entity
        self.mapping
            .selected_entity(target, select_origin, rule_index);
        // Extra entities get their own variable scope
        // so they cannot affect anything outside
        if target.is_extra() {
            self.variable_pool.push();
        }
        let properties = &self.stylesheet.rule_at(rule_index).properties;
        for property in properties {
            let context = Self::evaluation_context(
                self.graph,
                &self.variable_pool,
                select_origin.clone(),
                previous_edge,
            );
            let value = evaluate(&property.value, &context);
            match &property.key {
                StyleKey::Property(key) => {
                    self.mapping.assign(target, key, value, rule_index);
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

    fn evaluation_context<'b>(
        graph: &'b T,
        variable_pool: &'b VariablePool<&'b str, T::NodeId>,
        origin: T::NodeId,
        previous_edge: Option<&'b EdgeLabel>,
    ) -> EvaluationContext<'b, T> {
        let mut context =
            EvaluationContext::from_graph(graph, origin).with_variables(variable_pool);
        match previous_edge {
            Some(EdgeLabel::Index(index)) => context = context.with_edge_index(*index),
            Some(EdgeLabel::Named(name, discriminator)) => {
                context = context
                    .with_edge_name(name.as_str())
                    .with_edge_discriminator(*discriminator)
            }
            _ => {}
        }
        context
    }
}
