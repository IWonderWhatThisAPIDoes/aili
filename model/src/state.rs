//! Program state graph model.

use derive_more::{Debug, From};

/// Unique identifier of a program state node.
pub trait NodeId: Clone + std::fmt::Debug + Eq + std::hash::Hash {}

impl<T: Clone + std::fmt::Debug + Eq + std::hash::Hash> NodeId for T {}

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
pub trait NodeTypeId: Clone + std::cmp::Eq {
    /// Get the specific name of the type.
    fn type_name(&self) -> &str;
}

// Blanket implementation for convenience,
// allow the use of anything that can be aliased as
// a str for a type identifier
impl<T> NodeTypeId for T
where
    T: Clone + std::cmp::Eq + std::borrow::Borrow<str>,
{
    fn type_name(&self) -> &str {
        self.borrow()
    }
}

/// Enumerates elementary arithmetic values for nodes.
#[derive(Clone, Copy, Eq, Debug, From)]
pub enum NodeValue {
    /// Boolean value.
    #[debug("{}", if *_0 { "true" } else  { "false" })]
    Bool(bool),

    /// Signed integer value.
    #[debug("{_0}")]
    Int(i64),

    /// Unsigned integer value.
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
/// of [`NodeTypeClass`] they can connect, but these are not enforced.
/// Nontheless, implementations should adhere to them.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EdgeLabel {
    /// Identifies the entry point (the bottom-most stack frame).
    ///
    /// ## Permitted Sources
    /// [`NodeTypeClass::Root`]
    ///
    /// ## Permitted Targets
    /// [`NodeTypeClass::Frame`]
    #[debug("main")]
    Main,

    /// Indicates an ordering relation between stack frames.
    ///
    /// ## Permitted Sources
    /// [`NodeTypeClass::Frame`]
    ///
    /// ## Permitted Targets
    /// [`NodeTypeClass::Frame`]
    #[debug("next")]
    Next,

    /// Identifies the return value of a function call.
    ///
    /// ## Permitted Sources
    /// [`NodeTypeClass::Frame`]
    ///
    /// ## Permitted Targets
    /// [`NodeTypeClass::Atom`], [`NodeTypeClass::Struct`], [`NodeTypeClass::Array`], [`NodeTypeClass::Ref`]
    #[debug("ret")]
    Result,

    /// Identifies the target of a reference node.
    ///
    /// ## Permitted Sources
    /// [`NodeTypeClass::Ref`]
    ///
    /// ## Permitted Targets
    /// [`NodeTypeClass::Atom`], [`NodeTypeClass::Struct`], [`NodeTypeClass::Array`], [`NodeTypeClass::Ref`]
    #[debug("ref")]
    Deref,

    /// Identifies an item of a sequence at a specific index.
    ///
    /// ## Permitted Sources
    /// [`NodeTypeClass::Array`]
    ///
    /// ## Permitted Targets
    /// [`NodeTypeClass::Atom`], [`NodeTypeClass::Struct`], [`NodeTypeClass::Array`], [`NodeTypeClass::Ref`]
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
    /// [`NodeTypeClass::Root`], [`NodeTypeClass::Frame`], [`NodeTypeClass::Struct`]
    ///
    /// ## Permitted Targets
    /// [`NodeTypeClass::Atom`], [`NodeTypeClass::Struct`], [`NodeTypeClass::Array`], [`NodeTypeClass::Ref`]
    #[debug("{_0:?}#{_1}")]
    Named(String, usize),

    /// Indicates a variable that stores the length of a sequence.
    ///
    /// ## Permitted Sources
    /// [`NodeTypeClass::Array`]
    ///
    /// ## Permitted Targets
    /// [`NodeTypeClass::Atom`]
    #[debug("len")]
    Length,
}

/// Categories of types of program state nodes.
///
/// Each type has specific semantics which determine what types
/// of incoming and outgoing [`EdgeLabel`]s are allowed, but these
/// are not enforced. Nontheless, implementations should adhere to them.
///
/// Some types can be further classified using [`NodeTypeId`]s.
/// It is not enforced either.
///
/// Nodes of some types may be characterized with a [`NodeValue`],
/// usualy a numeric one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeTypeClass {
    /// Type of the node that represents the program's global scope.
    ///
    /// ## Properties
    /// | Property | Usage |
    /// |----------|-------|
    /// | Value    | No    |
    /// | Type ID  | No    |
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
    /// ## Properties
    /// | Property | Usage    | Notes                             |
    /// |----------|----------|-----------------------------------|
    /// | Value    | Optional | Address of the stack frame        |
    /// | Type ID  | Yes      | Distinguishes different functions |
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
    #[debug("fun")]
    Frame,

    /// Type of nodes that represent elementary values.
    ///
    /// Parametrized by the type of the value.
    ///
    /// ## Properties
    /// | Property | Usage | Notes                                    |
    /// |----------|-------|------------------------------------------|
    /// | Value    | Yes   | The elementary value                     |
    /// | Type ID  | Yes   | Distinguishes different elementary types |
    ///
    /// ## Permitted Incoming Edges
    /// | Edge label                                                                                     | Multiplicity |
    /// |------------------------------------------------------------------------------------------------|--------------|
    /// | [`EdgeLabel::Named`] or [`EdgeLabel::Index`] or [`EdgeLabel::Length`] or [`EdgeLabel::Result`] | 0..1         |
    /// | [`EdgeLabel::Deref`]                                                                           | *            |
    ///
    /// ## Permitted Outgoing Edges
    /// None.
    #[debug("val")]
    Atom,

    /// Type of nodes that represent structured values.
    ///
    /// Parametrized by the type of the value.
    ///
    /// ## Properties
    /// | Property | Usage | Notes                           |
    /// |----------|-------|---------------------------------|
    /// | Value    | No    |                                 |
    /// | Type ID  | Yes   | Distinguishes different classes |
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
    #[debug("struct")]
    Struct,

    /// Type of nodes that represent sequence (array) values.
    ///
    /// ## Properties
    /// | Property | Usage |
    /// |----------|-------|
    /// | Value    | No    |
    /// | Type ID  | No    |
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
    /// ## Properties
    /// | Property | Usage    | Notes                                   |
    /// |----------|----------|-----------------------------------------|
    /// | Value    | Optional | Address of the reference                |
    /// | Type ID  | Optional | Distinguishes different reference types |
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

/// Node in the program state graph.
pub trait ProgramStateNode {
    /// Type of unique identifiers for nodes.
    type NodeId: NodeId;

    /// Type of identifiers that firther identify node types
    /// of the same [`NodeTypeClass`] category.
    type NodeTypeId<'a>: NodeTypeId + 'a
    where
        Self: 'a;

    /// Finds a successor node by navigating along a specified edge.
    fn get_successor(&self, edge: &EdgeLabel) -> Option<Self::NodeId>;

    /// Iterates through the list of successors and the edges
    /// that lead to them. Edge labels are unique.
    fn successors(&self) -> impl Iterator<Item = (&EdgeLabel, Self::NodeId)>;

    /// Gets the categorical type of the node.
    fn node_type_class(&self) -> NodeTypeClass;

    /// Gets the specific type ID of the node.
    fn node_type_id(&self) -> Option<Self::NodeTypeId<'_>>;

    /// Gets the value of the node, if any.
    fn value(&self) -> Option<NodeValue>;
}

/// Container for a program state graph.
pub trait ProgramStateGraph {
    /// Type of unique identifiers for nodes.
    type NodeId: NodeId;

    /// Type of references to nodes.
    type NodeRef<'a>: ProgramStateNode<NodeId = Self::NodeId> + 'a
    where
        Self: 'a;

    /// Get a reference to a state node by its ID.
    fn get(&self, id: &Self::NodeId) -> Option<Self::NodeRef<'_>>;

    /// Get the ID of a state node by its path from a reference node.
    fn get_id_at<'b>(
        &self,
        origin_id: &Self::NodeId,
        path: impl IntoIterator<Item = &'b EdgeLabel>,
    ) -> Option<Self::NodeId> {
        let mut id = origin_id.clone();
        for edge_label in path {
            id = self.get(&id)?.get_successor(edge_label)?;
        }
        Some(id)
    }

    /// Get a reference to a state node by its path from a reference node.
    fn get_at<'b>(
        &self,
        origin_id: &Self::NodeId,
        path: impl IntoIterator<Item = &'b EdgeLabel>,
    ) -> Option<Self::NodeRef<'_>> {
        self.get(&self.get_id_at(origin_id, path)?)
    }
}

/// [`ProgramStateGraph`] that additionally allows accessing the root node.
pub trait RootedProgramStateGraph: ProgramStateGraph {
    /// Get the ID of the root node.
    fn root(&self) -> Self::NodeId;

    /// Get the ID of a state node by its full path from the root.
    fn get_id_at_root<'b>(
        &self,
        path: impl IntoIterator<Item = &'b EdgeLabel>,
    ) -> Option<Self::NodeId> {
        self.get_id_at(&self.root(), path)
    }

    /// Get a reference to a state node by its full path from the root.
    fn get_at_root<'b>(
        &self,
        path: impl IntoIterator<Item = &'b EdgeLabel>,
    ) -> Option<Self::NodeRef<'_>> {
        self.get_at(&self.root(), path)
    }
}
