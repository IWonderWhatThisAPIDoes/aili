//! Implementation of [`ProgramStateGraph`] backed by a GDB session.

use crate::gdbmi::types::VariableObject;
use aili_model::state::*;
use aili_style::values::PropertyValue;
use derive_more::{Debug, Deref, DerefMut};
use std::collections::{BTreeMap, HashMap};

/// Identifiers of state nodes used by [`GdbStateGraph`].
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum GdbStateNodeId {
    /// Identifier of the root node.
    #[debug("::")]
    Root,

    /// Identifier of a stack frame node.
    #[debug("frame({_0})")]
    Frame(usize),

    /// Identifier of a node backed by a
    /// [GDB/MI variable object](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Variable-Objects.html).
    #[debug("var({:?})", _0.0)]
    VarObject(VariableObject),

    /// Identifier of the [`EdgeLabel::Length`] pseudo-node
    /// associated with a [`GdbStateNodeId::VarObject`] node.
    #[debug("var({:?}) len", _0.0)]
    Length(VariableObject),
}

/// Implementation of a [`ProgramStateGraph`] backed by a GDB session.
#[derive(Debug)]
pub struct GdbStateGraph {
    pub(crate) root_node: GdbStateNode,
    pub(crate) stack_trace: Vec<GdbStateNode>,
    pub(crate) variables: HashMap<VariableObject, GdbStateNodeForVariable>,
    pub(crate) length_nodes: HashMap<VariableObject, GdbStateNode>,
    pub(crate) address_mapping: BTreeMap<u64, VariableObject>,
    pub(crate) resolved_length_hints: HashMap<VariableObject, PropertyValue<GdbStateNodeId>>,
}

impl ProgramStateGraph for GdbStateGraph {
    type NodeId = GdbStateNodeId;
    type NodeRef<'a>
        = &'a GdbStateNode
    where
        Self: 'a;
    fn get(&self, id: &Self::NodeId) -> Option<Self::NodeRef<'_>> {
        match id {
            GdbStateNodeId::Root => Some(&self.root_node),
            GdbStateNodeId::Frame(i) => self.stack_trace.get(*i),
            GdbStateNodeId::VarObject(v) => self.variables.get(v).map(|v| &v.node),
            GdbStateNodeId::Length(v) => self.length_nodes.get(v),
        }
    }
}

impl RootedProgramStateGraph for GdbStateGraph {
    fn root(&self) -> Self::NodeId {
        GdbStateNodeId::Root
    }
}

/// Node of a [`GdbStateGraph`].
#[derive(Debug)]
pub struct GdbStateNode {
    pub(crate) type_class: NodeTypeClass,
    pub(crate) type_name: Option<String>,
    pub(crate) successors: Vec<(EdgeLabel, GdbStateNodeId)>,
    pub(crate) value: Option<NodeValue>,
}

impl ProgramStateNode for &GdbStateNode {
    type NodeId = GdbStateNodeId;
    type NodeTypeId<'a>
        = &'a str
    where
        Self: 'a;
    fn get_successor(&self, edge: &EdgeLabel) -> Option<Self::NodeId> {
        self.successors
            .iter()
            .find(|(e, _)| *e == *edge)
            .map(|(_, n)| n)
            .cloned()
    }
    fn node_type_class(&self) -> NodeTypeClass {
        self.type_class
    }
    fn node_type_id(&self) -> Option<Self::NodeTypeId<'_>> {
        match self.type_class {
            NodeTypeClass::Atom | NodeTypeClass::Struct | NodeTypeClass::Frame => {
                self.type_name.as_deref()
            }
            NodeTypeClass::Ref | NodeTypeClass::Root | NodeTypeClass::Array => None,
        }
    }
    fn successors(&self) -> impl Iterator<Item = (&EdgeLabel, Self::NodeId)> {
        self.successors.iter().map(|(e, n)| (e, n.clone()))
    }
    fn value(&self) -> Option<NodeValue> {
        self.value
    }
}

/// [`GdbStateNode`] with additional information related to variable objects.
#[derive(Debug, Deref, DerefMut)]
pub(crate) struct GdbStateNodeForVariable {
    /// The state node.
    #[deref]
    #[deref_mut]
    pub node: GdbStateNode,

    /// Address of the variable, if available
    pub address: Option<u64>,

    /// Reference to the main parent node, if any.
    ///
    /// If this is empty, the node is a heap-allocated object.
    pub parent: Option<GdbStateNodeId>,

    /// References to [`NodeTypeClass::Ref`] nodes whose
    /// [`EdgeLabel::Deref`] points to this node.
    pub referers: Vec<VariableObject>,
}

/// [`GdbStateNode`] with additional data for a node that
/// represents a [`VariableObject`].
impl GdbStateNodeForVariable {
    pub fn new(node: GdbStateNode, parent: Option<GdbStateNodeId>) -> Self {
        Self {
            node,
            parent,
            address: None,
            referers: Vec::new(),
        }
    }

    /// True if the node is associated with a top-level GDB variable object.
    pub fn is_top_level(&self) -> bool {
        // Node is not top-level if its parent is also a variable node
        !matches!(self.parent, Some(GdbStateNodeId::VarObject(_)))
    }
}
