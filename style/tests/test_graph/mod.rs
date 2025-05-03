//! Stub program state graphs for testing.

use aili_model::state::*;
use aili_style::stylesheet::expression::LimitedSelector;
use std::collections::HashMap;

/// Stub graph for testing graph-based code.
pub struct TestGraph(Vec<TestNode>);

impl TestGraph {
    /// Shorthand for a pre-constructed graph for running tests.
    pub fn default_graph() -> Self {
        use EdgeLabel::*;
        Self(vec![
            // 0 - root and valueless node
            TestNode([(Named("a".to_owned(), 0), 1)].into(), None),
            // 1 - numeric node
            TestNode([].into(), Some(NodeValue::Uint(Self::NUMERIC_NODE_VALUE))),
        ])
    }

    /// Constructs a selector that selects a valueless node
    /// in the [`default_graph`](TestGraph::default_graph).
    pub fn valueless_node_selector() -> LimitedSelector {
        LimitedSelector::from_path([])
    }

    /// Constructs a selector that matches a numeric-valued node
    /// in the [`default_graph`](TestGraph::default_graph).
    pub fn numeric_node_selector() -> LimitedSelector {
        LimitedSelector::from_path([EdgeLabel::Named("a".into(), 0).into()])
    }

    /// Constructs a selector that does not match a node
    /// in the [`default_graph`](TestGraph::default_graph).
    pub fn missing_node_selector() -> LimitedSelector {
        LimitedSelector::from_path([EdgeLabel::Result.into()])
    }

    /// Value of the node selected by
    /// [`numeric_node_selector`](TestGraph::numeric_node_selector)
    /// in the [`default_graph`](TestGraph::default_graph)
    pub const NUMERIC_NODE_VALUE: u64 = 37;
}

impl ProgramStateGraph for TestGraph {
    type NodeId = usize;
    type NodeRef<'a> = &'a TestNode;
    fn get(&self, id: &Self::NodeId) -> Option<Self::NodeRef<'_>> {
        self.0.get(*id)
    }
}

impl RootedProgramStateGraph for TestGraph {
    fn root(&self) -> Self::NodeId {
        0
    }
}

/// Node of [`TestGraph`].
pub struct TestNode(HashMap<EdgeLabel, usize>, Option<NodeValue>);

impl ProgramStateNode for &TestNode {
    type NodeId = usize;
    type NodeTypeId<'a>
        = &'a str
    where
        Self: 'a;
    fn get_successor(&self, edge: &EdgeLabel) -> Option<Self::NodeId> {
        self.0.get(edge).copied()
    }
    fn successors(&self) -> impl Iterator<Item = (&EdgeLabel, Self::NodeId)> {
        self.0.iter().map(|(k, v)| (k, *v))
    }
    fn node_type_class(&self) -> NodeTypeClass {
        NodeTypeClass::Root
    }
    fn node_type_id(&self) -> Option<Self::NodeTypeId<'_>> {
        None
    }
    fn value(&self) -> Option<NodeValue> {
        self.1
    }
}
