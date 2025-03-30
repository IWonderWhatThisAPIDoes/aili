//! Identifiers for selectable entities.

use aili_model::state::{EdgeLabel, NodeId};

/// Unique identifier of any entity that can be selected
/// by the translator and converted into a visual entity.
///
/// Any state node and edge can be selected.
/// Additionally, "extra" entities can be attached to them,
/// which allows each state entity to produce multiple visual elements.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Selectable<T: NodeId> {
    /// Identifier of the state node that has either been selected
    /// or is the starting point of the selected edge.
    pub node_id: T,
    /// If present, is the label of the selected outgoing edge
    /// of the node specified by [`node_id`](Self::node_id).
    /// Otherwise, the node itself has been selected.
    pub edge_label: Option<EdgeLabel>,
    /// If the selected entity is an extra,
    /// this is its string identifier.
    pub extra_label: Option<String>,
}

impl<T: NodeId> Selectable<T> {
    /// Identifies a node as a selectable entity.
    pub fn node(node_id: T) -> Self {
        Self {
            node_id,
            edge_label: None,
            extra_label: None,
        }
    }

    /// Identifies an edge as a selectable entity.
    /// An edge is identified by its source node and label.
    pub fn edge(node_id: T, edge_label: EdgeLabel) -> Self {
        Self {
            node_id,
            edge_label: Some(edge_label),
            extra_label: None,
        }
    }

    /// Adds an extra label to the identifier. The identifier no longer
    /// refers to a program state entity, but to a virtual entity
    /// that can be mapped to a visual.
    pub fn with_extra(mut self, extra_label: Option<String>) -> Self {
        self.extra_label = extra_label;
        self
    }

    /// Checks whether the selection is a main node (i. e. not an extra).
    pub fn is_node(&self) -> bool {
        self.edge_label.is_none() && self.extra_label.is_none()
    }

    /// Checks whether the selection is a main edge (i. e. not an extra).
    pub fn is_edge(&self) -> bool {
        self.edge_label.is_some() && self.extra_label.is_none()
    }

    /// Checks whether the selection is an extra entity.
    pub fn is_extra(&self) -> bool {
        self.extra_label.is_some()
    }
}

impl<T: NodeId> std::fmt::Debug for Selectable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.node_id)?;
        if let Some(edge) = &self.edge_label {
            write!(f, " {:?}::edge", edge)?;
        }
        if let Some(extra) = &self.extra_label {
            if extra.is_empty() {
                write!(f, "::extra")?;
            } else {
                write!(f, "::extra({extra:?})")?;
            }
        }
        Ok(())
    }
}
