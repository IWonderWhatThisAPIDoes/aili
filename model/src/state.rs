//! Program state graph model.

use derive_more::{Debug, From};

/// Unique identifier of a program state node.
pub trait NodeId: Clone + std::fmt::Debug {}

impl<T: Clone + std::fmt::Debug> NodeId for T {}

/// Uniqie identifier of a program state node's type.
///
/// There are three kinds of nodes that are further split
/// into types:
///
/// - Stack frames
/// - Elementary values
/// - Object values
///
/// All of these are constrained by this trait.
/// If two or all of them are identified by the same type
/// (i. e. by the same implementation of this trait),
/// they do not need to be globally unique.
/// They must only be unique within each of the three
/// categories.
pub trait NodeTypeId: Clone + std::fmt::Debug + std::cmp::Eq {}

impl<T: Clone + std::fmt::Debug + std::cmp::Eq> NodeTypeId for T {}

/// Enumerates elementary arithmetic values for nodes.
#[derive(Clone, Copy, Eq, Debug, From)]
pub enum NodeValue {
    /// Boolean value.
    #[debug("{}", if *_0 { "true" } else  { "false" })]
    Bool(bool),

    // Signed integer value.
    #[debug("{_0}")]
    Int(i64),

    // Unsigned integer value.
    #[debug("{_0}")]
    Uint(u64),
}

impl PartialEq for NodeValue {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl PartialOrd for NodeValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (*self, *other) {
            (Self::Bool(left), Self::Bool(right)) => left.cmp(&right),
            (Self::Bool(left), Self::Int(right)) => i64::from(left).cmp(&right),
            (Self::Bool(left), Self::Uint(right)) => u64::from(left).cmp(&right),
            (Self::Int(left), Self::Bool(right)) => left.cmp(&right.into()),
            (Self::Int(left), Self::Int(right)) => left.cmp(&right),
            (Self::Int(left), Self::Uint(right)) => u64::try_from(left)
                .map(|left| left.cmp(&right))
                .unwrap_or(std::cmp::Ordering::Less),
            (Self::Uint(left), Self::Bool(right)) => left.cmp(&right.into()),
            (Self::Uint(left), Self::Int(right)) => u64::try_from(right)
                .map(|right| left.cmp(&right))
                .unwrap_or(std::cmp::Ordering::Greater),
            (Self::Uint(left), Self::Uint(right)) => left.cmp(&right),
        }
    }
}

/// Types of program state edges.
///
/// Each type has specific semantics which determine what kinds
/// of [`NodeType`] they can connect, but these are not enforced.
/// Nontheless, implementations should adhere to them.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EdgeLabel {
    /// Identifies the entry point (the bottom-most stack frame).
    ///
    /// ## Permitted Sources
    /// [`NodeType::Root`]
    ///
    /// ## Permitted Targets
    /// [`NodeType::Frame`]
    #[debug("main")]
    Main,

    /// Indicates an ordering relation between stack frames.
    ///
    /// ## Permitted Sources
    /// [`NodeType::Frame`]
    ///
    /// ## Permitted Targets
    /// [`NodeType::Frame`]
    #[debug("next")]
    Next,

    /// Identifies the return value of a function call.
    ///
    /// ## Permitted Sources
    /// [`NodeType::Frame`]
    ///
    /// ## Permitted Targets
    /// [`NodeType::Atom`], [`NodeType::Struct`], [`NodeType::Array`], [`NodeType::Ref`]
    #[debug("ret")]
    Result,

    /// Identifies the target of a reference node.
    ///
    /// ## Permitted Sources
    /// [`NodeType::Ref`]
    ///
    /// ## Permitted Targets
    /// [`NodeType::Atom`], [`NodeType::Struct`], [`NodeType::Array`], [`NodeType::Ref`]
    #[debug("ref")]
    Deref,

    /// Identifies an item of a sequence at a specific index.
    ///
    /// ## Permitted Sources
    /// [`NodeType::Array`]
    ///
    /// ## Permitted Targets
    /// [`NodeType::Atom`], [`NodeType::Struct`], [`NodeType::Array`], [`NodeType::Ref`]
    #[debug("[{_0}]")]
    Index(usize),

    /// Identifies a named variable - global, local, or member.
    ///
    /// The edges are secondarily distinguished by indices,
    /// since some languages allow multiple variables
    /// of the same name to exist in the same scope
    /// (edge labels must be unique within their starting node).
    /// Indices should always be sequential.
    ///
    /// ## Permitted Sources
    /// [`NodeType::Root`], [`NodeType::Frame`], [`NodeType::Struct`]
    ///
    /// ## Permitted Targets
    /// [`NodeType::Atom`], [`NodeType::Struct`], [`NodeType::Array`], [`NodeType::Ref`]
    #[debug("\"{_0}\"#{_1}")]
    Named(String, usize),

    /// Indicates a variable that stores the length of a sequence.
    ///
    /// ## Permitted Sources
    /// [`NodeType::Array`]
    ///
    /// ## Permitted Targets
    /// [`NodeType::Atom`]
    #[debug("len")]
    Length,
}

/// Types of program state nodes.
///
/// Each type has specific semantics which determine what types
/// of incoming and outgoing [`EdgeLabel`]s are allowed, but these
/// are not enforced. Nontheless, implementations should adhere to them.
#[derive(Clone, Debug)]
pub enum NodeType<FunId: NodeTypeId, AtomId: NodeTypeId, ObjId: NodeTypeId> {
    /// Type of the node that represents the program's global scope.
    ///
    /// ## Required Value
    /// No.
    ///
    /// ## Permitted Incoming Edges
    /// None.
    ///
    /// ## Permitted Outgoing Edges
    /// | Edge label           | Multiplicity | Semantics                             |
    /// |----------------------|--------------|---------------------------------------|
    /// | [`EdgeLabel::Main`]  | 1            | Entry point (bottom-most stack frame) |
    /// | [`EdgeLabel::Named`] | *            | Global variables                      |
    #[debug("root")]
    Root,

    /// Type of nodes that represent stack frames.
    ///
    /// Parametrized by an identifier of the function image.
    ///
    /// ## Required Value
    /// No.
    ///
    /// ## Permitted Incoming Edges
    /// | Edge label                                 | Multiplicity |
    /// |--------------------------------------------|--------------|
    /// | [`EdgeLabel::Main`] or [`EdgeLabel::Next`] | 1            |
    ///
    /// ## Permitted Outgoing Edges
    /// | Edge label            | Multiplicity | Semantics                   |
    /// |-----------------------|--------------|-----------------------------|
    /// | [`EdgeLabel::Next`]   | 0..1         | Stack frame above this one  |
    /// | [`EdgeLabel::Named`]  | *            | Local variables             |
    /// | [`EdgeLabel::Result`] | 0..1         | The function's return value |
    ///
    #[debug("fun:{_0}")]
    Frame(FunId),

    /// Type of nodes that represent elementary values.
    ///
    /// Parametrized by the type of the value.
    ///
    /// ## Required Value
    /// Yes.
    ///
    /// ## Permitted Incoming Edges
    /// | Edge label                                                                                     | Multiplicity |
    /// |------------------------------------------------------------------------------------------------|--------------|
    /// | [`EdgeLabel::Named`] or [`EdgeLabel::Index`] or [`EdgeLabel::Length`] or [`EdgeLabel::Result`] | 0..1         |
    /// | [`EdgeLabel::Deref`]                                                                           | *            |
    ///
    /// ## Permitted Outgoing Edges
    /// None.
    #[debug("val:{_0}")]
    Atom(AtomId),

    /// Type of nodes that represent structured values.
    ///
    /// Parametrized by the type of the value.
    ///
    /// ## Required Value
    /// No.
    ///
    /// ## Permitted Incoming Edges
    /// | Edge label                                                            | Multiplicity |
    /// |-----------------------------------------------------------------------|--------------|
    /// | [`EdgeLabel::Named`] or [`EdgeLabel::Index`] or [`EdgeLabel::Result`] | 0..1         |
    /// | [`EdgeLabel::Deref`]                                                  | *            |
    ///
    /// ## Permitted Outgoing Edges
    /// | Edge label           | Multiplicity | Semantics             |
    /// |----------------------|--------------|-----------------------|
    /// | [`EdgeLabel::Named`] | *            | Member variables      |
    #[debug("obj:{_0}")]
    Struct(ObjId),

    /// Type of nodes that represent sequence (array) values.
    ///
    /// ## Required Value
    /// No.
    ///
    /// ## Permitted Incoming Edges
    /// | Edge label                                                            | Multiplicity |
    /// |-----------------------------------------------------------------------|--------------|
    /// | [`EdgeLabel::Named`] or [`EdgeLabel::Index`] or [`EdgeLabel::Result`] | 0..1         |
    /// | [`EdgeLabel::Deref`]                                                  | *            |
    ///
    /// ## Permitted Outgoing Edges
    /// | Edge label            | Multiplicity | Semantics                                        |
    /// |-----------------------|--------------|--------------------------------------------------|
    /// | [`EdgeLabel::Index`]  | *            | Array entries                                    |
    /// | [`EdgeLabel::Length`] | 1            | Numeric value indicating the length of the array |
    #[debug("arr")]
    Array,

    /// Type of nodes that represent references.
    ///
    /// ## Required Value
    /// No.
    ///
    /// ## Permitted Incoming Edges
    /// | Edge label                                                            | Multiplicity |
    /// |-----------------------------------------------------------------------|--------------|
    /// | [`EdgeLabel::Named`] or [`EdgeLabel::Index`] or [`EdgeLabel::Result`] | 0..1         |
    /// | [`EdgeLabel::Deref`]                                                  | *            |
    ///
    /// ## Permitted Outgoing Edges
    /// | Edge label           | Multiplicity | Semantics                  |
    /// |----------------------|--------------|----------------------------|
    /// | [`EdgeLabel::Deref`] | 1            | The value being referenced |
    #[debug("ref")]
    Ref,
}

/// Reference to a program state node.
pub trait ProgramStateNodeRef {
    /// Type of unique identifiers for nodes.
    type NodeId: NodeId;

    /// Type of unique identifiers for [`NodeType::Frame`] node types.
    type FunId: NodeTypeId;

    /// Type of unique identifiers for [`NodeType::Atom`] node types.
    type AtomId: NodeTypeId;

    /// Type of unique identifiers for [`NodeType::Struct`] node types.
    type ObjId: NodeTypeId;

    /// Finds a successor node by navigating along a specified edge.
    fn get_successor(self, edge: &EdgeLabel) -> Option<Self::NodeId>;

    /// Iterates through the list of successors and the edges
    /// that lead to them. Edge labels are unique.
    fn successors<'a>(self) -> impl Iterator<Item = (&'a EdgeLabel, Self::NodeId)>
    where
        Self: 'a;

    /// Gets the type of the node.
    fn node_type<'a>(self) -> &'a NodeType<Self::FunId, Self::AtomId, Self::ObjId>
    where
        Self: 'a;

    /// Gets the value of the node, if any.
    fn value<'a>(self) -> Option<&'a NodeValue>
    where
        Self: 'a;
}

/// Container for a program state graph.
pub trait ProgramStateGraphRef {
    /// Type of unique identifiers for nodes.
    type NodeId: NodeId;

    /// Type of references to nodes.
    type NodeRef: ProgramStateNodeRef<NodeId = Self::NodeId>;

    /// Get the ID of the root node.
    fn root(self) -> Self::NodeId;

    /// Get a reference to a state node by its ID.
    fn get(self, id: Self::NodeId) -> Option<Self::NodeRef>;
}
