//! The Javascript-side description of [`state`](crate::state)
//! and means of its generation.

use crate::state::{StateGraph, StateNode};
use aili_model::state;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_INTERFACES: &str = r"
    /**
     * Description of a node of the program state graph.
     */
    interface StateNode {
        /**
         * Name of the node's type, if any.
         */
        typeName?: string;
        /**
         * General category of the node's type.
         */
        typeKind: NodeTypeClass;
        /**
         * Numeric value of the node, if any.
         */
        value?: BigInt;
        /**
         * Edges that lead from the node.
         */
        outEdges?: StateEdge[];
    }
    /**
     * Description of an outgoing edge of a {@link StateNode}.
     */
    interface StateEdge {
        /**
         * Target node of the edge.
         */
        node: StateNode;
        /**
         * Label that describes the edge's semantics.
         */
        edgeLabel: EdgeLabel;
    }
";

/// General type category of a [`StateNodeDescription`].
///
/// Maps to [`aili_model::state::NodeTypeClass`].
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeTypeClass {
    /// Root node.
    ///
    /// See [`aili_model::state::NodeTypeClass::Root`].
    #[default]
    Root,
    /// Stack frame node.
    ///
    /// See [`aili_model::state::NodeTypeClass::Frame`].
    Frame,
    /// Elementary value node.
    ///
    /// See [`aili_model::state::NodeTypeClass::Atom`].
    Atom,
    /// Structured data node.
    ///
    /// See [`aili_model::state::NodeTypeClass::Struct`].
    Struct,
    /// Array node.
    ///
    /// See [`aili_model::state::NodeTypeClass::Array`].
    Array,
    /// Reference node.
    ///
    /// See [`aili_model::state::NodeTypeClass::Ref`].
    Ref,
}

impl From<NodeTypeClass> for state::NodeTypeClass {
    fn from(value: NodeTypeClass) -> Self {
        use NodeTypeClass::*;
        match value {
            Root => Self::Root,
            Frame => Self::Frame,
            Atom => Self::Atom,
            Struct => Self::Struct,
            Array => Self::Array,
            Ref => Self::Ref,
        }
    }
}

/// Label that determines the semantics of a program state edge.
///
/// Maps to [`aili_model::state::EdgeLabel`].
#[wasm_bindgen(getter_with_clone)]
pub struct EdgeLabel(state::EdgeLabel);

#[wasm_bindgen]
impl EdgeLabel {
    /// Edge that identifies the entry point stack frame.
    ///
    /// See [`aili_model::state::EdgeLabel::Main`].
    #[wasm_bindgen(getter, js_name = "MAIN")]
    pub fn main() -> Self {
        Self(state::EdgeLabel::Main)
    }

    /// Edge that indicates the succession of stack frames.
    ///
    /// See [`aili_model::state::EdgeLabel::Next`].
    #[wasm_bindgen(getter, js_name = "NEXT")]
    pub fn next() -> Self {
        Self(state::EdgeLabel::Next)
    }

    /// Edge that identifies the return value of a function.
    ///
    /// See [`aili_model::state::EdgeLabel::Result`].
    #[wasm_bindgen(getter, js_name = "RESULT")]
    pub fn result() -> Self {
        Self(state::EdgeLabel::Result)
    }

    /// Edge that identifies the pointee of a reference node.
    ///
    /// See [`aili_model::state::EdgeLabel::Deref`].
    #[wasm_bindgen(getter, js_name = "DEREF")]
    pub fn deref() -> Self {
        Self(state::EdgeLabel::Deref)
    }

    /// Edge that identifies the length of an array node.
    ///
    /// See [`aili_model::state::EdgeLabel::Length`].
    #[wasm_bindgen(getter, js_name = "LENGTH")]
    pub fn length() -> Self {
        Self(state::EdgeLabel::Length)
    }

    /// Edge that indicates a named variable.
    ///
    /// See [`aili_model::state::EdgeLabel::Named`].
    pub fn named(name: &str, discriminator: Option<usize>) -> Self {
        Self(state::EdgeLabel::Named(
            name.to_owned(),
            discriminator.unwrap_or_default(),
        ))
    }

    /// Edge that indicates an array item.
    ///
    /// See [`aili_model::state::EdgeLabel::Index`].
    pub fn index(index: usize) -> Self {
        Self(state::EdgeLabel::Index(index))
    }
}

#[wasm_bindgen]
extern "C" {
    /// Description of a [`StateNode`].
    #[wasm_bindgen(typescript_type = "StateNode")]
    pub type StateNodeDescription;

    /// Information about an outgoing edge of a [`StateNodeDescription`].
    #[wasm_bindgen(typescript_type = "StateEdge")]
    pub type StateEdgeDescription;

    /// Gets the type name of the node.
    ///
    /// Maps to [`aili_model::state::ProgramStateNode::node_type_id`].
    #[wasm_bindgen(method, getter, js_name = "typeName")]
    pub fn type_name(this: &StateNodeDescription) -> Option<String>;

    /// Gets the type class of the node.
    ///
    /// Maps to [`aili_model::state::ProgramStateNode::node_type_class`].
    #[wasm_bindgen(method, getter, js_name = "typeKind")]
    pub fn type_kind(this: &StateNodeDescription) -> NodeTypeClass;

    /// Gets the value of a node.
    ///
    /// Maps to [`aili_model::state::ProgramStateNode::value`].
    #[wasm_bindgen(method, getter)]
    pub fn value(this: &StateNodeDescription) -> Option<i64>;

    /// Outgoing edges of the node.
    ///
    /// Maps to [`aili_model::state::successors`].
    #[wasm_bindgen(method, getter, js_name = "outEdges")]
    pub fn out_edges(this: &StateNodeDescription) -> Option<Box<[StateEdgeDescription]>>;

    /// The target node of the edge.
    #[wasm_bindgen(method, getter)]
    pub fn node(this: &StateEdgeDescription) -> StateNodeDescription;

    /// Label that describes the semantics of the edge.
    #[wasm_bindgen(method, getter, js_name = "edgeLabel")]
    pub fn edge_label(this: &StateEdgeDescription) -> EdgeLabel;
}

#[wasm_bindgen]
impl StateGraph {
    /// Constructs a state graph based on a description.
    #[wasm_bindgen(constructor)]
    pub fn new(root: StateNodeDescription) -> Self {
        let mut nodes = Vec::new();
        Self::add_node(root, &mut nodes);
        Self(nodes.into_iter().map(|(_, node)| node).collect())
    }

    fn add_node(description: StateNodeDescription, nodes: &mut Vec<(JsValue, StateNode)>) -> usize {
        let existing_node_index = nodes
            .iter()
            .enumerate()
            .find(|(_, (js_node, _))| *js_node == description.obj)
            .map(|(i, _)| i);
        // If we have already seen the node, just return the reference to it
        if let Some(index) = existing_node_index {
            return index;
        }
        // Othwrwise we create a new node with a new index
        let this_node_index = nodes.len();
        let node = StateNode::from_description_without_successors(&description);
        // The node must be pushed before any successors are resolved,
        // otherwise the indices would get messed up
        nodes.push((description.obj.clone(), node));
        // Now resolve successors
        for edge in description.out_edges().iter().flatten() {
            let target_index = Self::add_node(edge.node(), nodes);
            nodes[this_node_index]
                .1
                .successors
                .insert(edge.edge_label().0, target_index);
        }
        this_node_index
    }
}

impl StateNode {
    fn from_description_without_successors(description: &StateNodeDescription) -> Self {
        StateNode {
            type_class: description.type_kind().into(),
            type_name: description.type_name(),
            value: description.value().map(state::NodeValue::Int),
            successors: HashMap::new(),
        }
    }
}
