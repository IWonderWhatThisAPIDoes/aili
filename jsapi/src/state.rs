//! Mockup implementation of [`aili_model::state::ProgramStateGraph`].

use aili_model::state;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Wasm-side implementation of [`aili_model::state::ProgramStateGraph`].
#[wasm_bindgen]
pub struct StateGraph(pub(crate) Vec<StateNode>);

impl state::ProgramStateGraph for StateGraph {
    type NodeId = usize;
    type NodeRef<'a> = &'a StateNode;
    fn get(&self, id: &Self::NodeId) -> Option<Self::NodeRef<'_>> {
        self.0.get(*id)
    }
}

impl state::RootedProgramStateGraph for StateGraph {
    fn root(&self) -> Self::NodeId {
        assert!(!self.0.is_empty(), "State graph cannot be empty");
        0
    }
}

/// A node of [`StateGraph`].
#[derive(Debug)]
pub struct StateNode {
    pub(crate) type_class: state::NodeTypeClass,
    pub(crate) type_name: Option<String>,
    pub(crate) value: Option<state::NodeValue>,
    pub(crate) successors: HashMap<state::EdgeLabel, usize>,
}

impl state::ProgramStateNode for &StateNode {
    type NodeId = usize;
    type NodeTypeId<'a>
        = &'a str
    where
        Self: 'a;
    fn value(&self) -> Option<state::NodeValue> {
        self.value
    }
    fn node_type_class(&self) -> state::NodeTypeClass {
        self.type_class
    }
    fn node_type_id(&self) -> Option<Self::NodeTypeId<'_>> {
        self.type_name.as_deref()
    }
    fn get_successor(&self, edge: &state::EdgeLabel) -> Option<Self::NodeId> {
        self.successors.get(edge).cloned()
    }
    fn successors(&self) -> impl Iterator<Item = (&state::EdgeLabel, Self::NodeId)> {
        self.successors.iter().map(|(e, i)| (e, *i))
    }
}
