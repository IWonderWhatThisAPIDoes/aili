//! Contexts for expression evaluation.

use super::variable_pool::VariablePool;
use aili_model::state::{NodeTypeId, ProgramStateGraph, ProgramStateNode};

/// Provides stateful context for expression evaluation.
pub struct EvaluationContext<'a, T>
where
    T: ProgramStateGraph,
{
    /// Graph within which [`Select`](crate::stylesheet::expression::Expression::Select)
    /// expressions should be evaluated.
    pub graph: Option<&'a T>,

    /// Node that should be the origin for
    /// [`Select`](crate::stylesheet::expression::Expression::Select) expressions.
    pub select_origin: Option<T::NodeId>,

    /// Variable pool in which [`Variable`](crate::stylesheet::expression::Expression::Variable)
    /// expressions should be evaluated.
    pub variable_pool: Option<&'a VariablePool<&'a str, T::NodeId>>,

    /// Value that
    /// [`MagicVariableKey::EdgeIndex`](crate::stylesheet::expression::MagicVariableKey::EdgeIndex)
    /// should resolve to.
    pub edge_index: Option<usize>,

    /// Value that
    /// [`MagicVariableKey::EdgeName`](crate::stylesheet::expression::MagicVariableKey::EdgeName)
    /// should resolve to.
    pub edge_name: Option<&'a str>,

    /// Value that
    /// [`MagicVariableKey::EdgeDiscriminator`](crate::stylesheet::expression::MagicVariableKey::EdgeDiscriminator)
    /// should resolve to.
    pub edge_discriminator: Option<usize>,
}

impl<'a, T> EvaluationContext<'a, T>
where
    T: ProgramStateGraph,
{
    /// Constructs a context that provides no state.
    pub fn stateless() -> Self {
        Self::default()
    }

    /// Constructs a context that allows evaluation of
    /// [`Select`](crate::stylesheet::expression::Expression::Select)
    /// expressions over a graph.
    pub fn from_graph(graph: &'a T, select_origin: T::NodeId) -> Self {
        Self {
            graph: Some(graph),
            select_origin: Some(select_origin),
            variable_pool: None,
            edge_index: None,
            edge_discriminator: None,
            edge_name: None,
        }
    }

    /// Adds a variable pool for evaluating
    /// [`Variable`](crate::stylesheet::expression::Expression::Variable)
    /// expressions.
    pub fn with_variables(mut self, variable_pool: &'a VariablePool<&'a str, T::NodeId>) -> Self {
        self.variable_pool = Some(variable_pool);
        self
    }

    /// Adds an edge index for evaluating the
    /// [`MagicVariableKey::EdgeIndex`](crate::stylesheet::expression::MagicVariableKey::EdgeIndex)
    /// magic variable.
    pub fn with_edge_index(mut self, index: usize) -> Self {
        self.edge_index = Some(index);
        self
    }

    /// Adds an edge name for evaluating the
    /// [`MagicVariableKey::EdgeName`](crate::stylesheet::expression::MagicVariableKey::EdgeName)
    /// magic variable.
    pub fn with_edge_name(mut self, name: &'a str) -> Self {
        self.edge_name = Some(name);
        self
    }

    /// Adds an edge index for evaluating the
    /// [`MagicVariableKey::EdgeDiscriminator`](crate::stylesheet::expression::MagicVariableKey::EdgeDiscriminator)
    /// magic variable.
    pub fn with_edge_discriminator(mut self, discriminator: usize) -> Self {
        self.edge_discriminator = Some(discriminator);
        self
    }
}

impl<T> Default for EvaluationContext<'_, T>
where
    T: ProgramStateGraph,
{
    fn default() -> Self {
        Self {
            graph: None,
            select_origin: None,
            variable_pool: None,
            edge_index: None,
            edge_discriminator: None,
            edge_name: None,
        }
    }
}

/// Convenience alias for [`EvaluationContext`] that is inherently stateless.
pub type StatelessEvaluation = EvaluationContext<'static, Never>;

impl StatelessEvaluation {
    pub fn new() -> Self {
        Self::stateless()
    }
}

/// A value of this type can never be constructed.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Never {}

impl ProgramStateGraph for Never {
    type NodeId = Never;
    type NodeRef<'a> = Never;
    fn get(&self, _: &Self::NodeId) -> Option<Self::NodeRef<'_>> {
        None
    }
}

impl ProgramStateNode for Never {
    type NodeId = Never;
    type NodeTypeId<'a> = &'a str;
    fn get_successor(&self, _: &aili_model::state::EdgeLabel) -> Option<Self::NodeId> {
        unreachable!()
    }
    fn node_type_class(&self) -> aili_model::state::NodeTypeClass {
        unreachable!()
    }
    fn node_type_id(&self) -> Option<Self::NodeTypeId<'_>> {
        unreachable!()
    }
    fn successors(&self) -> impl Iterator<Item = (&aili_model::state::EdgeLabel, Self::NodeId)> {
        std::iter::empty()
    }
    fn value(&self) -> Option<aili_model::state::NodeValue> {
        unreachable!()
    }
}

impl NodeTypeId for Never {
    fn type_name(&self) -> &str {
        unreachable!()
    }
}
