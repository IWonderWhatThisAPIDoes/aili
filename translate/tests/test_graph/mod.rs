//! Stub program state graphs for testing.

use aili_model::state::*;
use std::collections::HashMap;

/// Stub graph for testing graph-based code.
pub struct TestGraph(Vec<TestNode>);

impl TestGraph {
    /// Shorthand for a pre-constructed graph for running tests.
    pub fn default_graph() -> Self {
        /*          main          next           next         next
         *       +------->([1])--------->([2])--------->([3])------>([4])
         *      /            \                             \          |
         *     /              \               ref           \         |
         * ([0])          +----\-----------------------+     | "a"    |
         *     \         /      \                       \   /         |
         *      \       v        \                "b"    \ v      ret |
         *   "a" +-->([5] 37)-----\------>([6] 3)------>([7])         |
         *            /   \  "a"   \          \                      /
         *       [0] /     \        \ "a"      \ "a"                /
         *          /       \        v    "a"   v       [0]        v
         *         v         +---->([10])---->([11])---------->([13])
         *      ([8])       ref      ^ \          \             /
         *        |                  |  \          \ [1]       /
         *        | ref              |   \ "a"#1    v         / ref
         *        v              ref |    +------>([12])<----+
         *      ([9])                |             /
         *                           +------------+
         */
        use EdgeLabel::*;
        Self(vec![
            /* 0 */
            TestNode([(Main, 1), (Named("a".to_owned(), 0), 5)].into(), None),
            /* 1 */
            TestNode([(Next, 2), (Named("a".to_owned(), 0), 10)].into(), None),
            /* 2 */ TestNode([(Next, 3)].into(), None),
            /* 3 */
            TestNode([(Next, 4), (Named("a".to_owned(), 0), 7)].into(), None),
            /* 4 */ TestNode([(Result, 13)].into(), None),
            /* 5 */
            TestNode(
                [(Named("a".to_owned(), 0), 6), (Index(0), 8), (Deref, 10)].into(),
                Some(Self::NUMERIC_NODE_VALUE.into()),
            ),
            /* 6 */
            TestNode(
                [
                    (Named("a".to_owned(), 0), 11),
                    (Named("b".to_owned(), 0), 7),
                ]
                .into(),
                Some(3u64.into()),
            ),
            /* 7 */ TestNode([(Deref, 5)].into(), None),
            /* 8 */ TestNode([(Deref, 9)].into(), None),
            /* 9 */ TestNode([].into(), None),
            /* 10 */
            TestNode(
                [
                    (Named("a".to_owned(), 0), 11),
                    (Named("a".to_owned(), 1), 12),
                ]
                .into(),
                None,
            ),
            /* 11 */ TestNode([(Index(0), 13), (Index(1), 12)].into(), None),
            /* 12 */ TestNode([(Deref, 10)].into(), None),
            /* 13 */ TestNode([(Deref, 12)].into(), None),
        ])
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
