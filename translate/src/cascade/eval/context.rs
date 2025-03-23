//! Contexts for expression evaluation.

use crate::{
    property::{PropertyValue, Selectable},
    stylesheet::selector::LimitedSelector,
};
use aili_model::state::{NodeTypeId, ProgramStateGraph, ProgramStateNode, RootedProgramStateGraph};

/// Provides stateful context for expression evaluation.
pub trait EvaluationContext: ProgramStateGraph {
    /// Implementation of the [`Select`](Expression::Select) expression.
    fn select_entity(&self, _selector: &LimitedSelector) -> Option<Selectable<Self::NodeId>> {
        None
    }

    /// Implementation of the [`Variable`](Expression::Variable) expression.
    fn get_variable_value(&self, _name: &str) -> PropertyValue<Self::NodeId> {
        PropertyValue::Unset
    }
}

/// [`EvaluationContext`] that does not provide any state.
/// It has no variables and is not backed by a graph.
#[derive(Clone, Copy, Default)]
pub struct StatelessEvaluation;

impl ProgramStateGraph for StatelessEvaluation {
    type NodeId = Never;
    type Node = Never;
    fn get(&self, _: Self::NodeId) -> Option<&Self::Node> {
        None
    }
}

impl EvaluationContext for StatelessEvaluation {}

/// A value of this type can never be constructed.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Never {}

impl ProgramStateNode for Never {
    type NodeId = Never;
    type AtomId = Never;
    type FunId = Never;
    type ObjId = Never;
    fn get_successor(&self, _: &aili_model::state::EdgeLabel) -> Option<Self::NodeId> {
        unreachable!()
    }
    fn node_type(&self) -> &aili_model::state::NodeType<Self::FunId, Self::AtomId, Self::ObjId> {
        unreachable!()
    }
    fn successors(&self) -> impl Iterator<Item = (&aili_model::state::EdgeLabel, Self::NodeId)> {
        std::iter::empty()
    }
    fn value(&self) -> Option<&aili_model::state::NodeValue> {
        unreachable!()
    }
}

impl NodeTypeId for Never {
    fn type_name(&self) -> &str {
        unreachable!()
    }
}

/// [`EvaluationContext`] that provides a graph with an origin node to run selectors on.
pub struct EvaluationOnGraph<'a, T: ProgramStateGraph> {
    /// Graph within which [`Select`](crate::stylesheet::expression::Expression::Select)
    /// expressions should be evaluated.
    graph: &'a T,

    /// Node that should be the origin for
    /// [`Select`](crate::stylesheet::expression::Expression::Select) expressions.
    origin_node: T::NodeId,
}

impl<'a, T: ProgramStateGraph> EvaluationOnGraph<'a, T> {
    /// Constructs an evaluation context for a graph.
    pub fn new(graph: &'a T, origin_node: T::NodeId) -> Self {
        Self { graph, origin_node }
    }
}

impl<T: ProgramStateGraph> ProgramStateGraph for EvaluationOnGraph<'_, T> {
    type NodeId = T::NodeId;
    type Node = T::Node;
    fn get(&self, id: Self::NodeId) -> Option<&Self::Node> {
        self.graph.get(id)
    }
}

impl<T: ProgramStateGraph> EvaluationContext for EvaluationOnGraph<'_, T> {
    fn select_entity(&self, selector: &LimitedSelector) -> Option<Selectable<Self::NodeId>> {
        let mut current_node = self.origin_node.clone();
        for segment in &selector.path {
            // Find the edge specified (unambiguously) by the segmens
            // and move to the node at its end
            current_node = self
                .graph
                .get(current_node.clone())
                .and_then(|node| node.get_successor(segment))?;
        }
        Some(Selectable::node(current_node).with_extra(selector.extra_label.clone()))
    }
}

impl<T: RootedProgramStateGraph> EvaluationContext for T {
    /// Evaluates a [`LimitedSelector`] in the context of the root node
    /// of a rooted graph. This way, one may simply use a graph
    /// as an evaluation context instead of
    /// `EvaluationOnGraph::new(graph, graph.root())`, which has the same effect.
    fn select_entity(&self, selector: &LimitedSelector) -> Option<Selectable<Self::NodeId>> {
        EvaluationOnGraph::new(self, self.root()).select_entity(selector)
    }
}
