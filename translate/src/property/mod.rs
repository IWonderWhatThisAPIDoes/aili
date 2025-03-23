//! Keys and values of properties of visualizable entities.

mod selectable;
mod values;

use aili_model::state::NodeId;
pub use selectable::Selectable;
use std::collections::HashMap;
pub use values::PropertyValue;

/// A key that values can be assigned to on a selectable entity.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum PropertyKey {
    /// Assigns value to an attribute of the selected entity.
    Attribute(String),

    /// Assigns value
    /// Modifies the display mode of the selected entity.
    Display,

    /// Modifies the parent reference of the selected entity.
    Parent,

    /// Modifies the connector target reference of the selected entity.
    Target,

    /// Modifies the detachment mode of the selected entity.
    Detach,
}

/// Properties of a visual element, pre-processed to the required form.
#[derive(Default)]
pub struct PropertyMap<T: NodeId> {
    /// Attributes with string values.
    pub attributes: HashMap<String, String>,

    /// Display mode of the entity.
    pub display: Option<DisplayMode>,

    /// Entity whose visualization should be the parent
    /// of this entity's visualization, or its starting
    /// point if [`display`](PropertyMap::display)
    /// is [`Connector`](DisplayMode::Connector).
    pub parent: Option<Selectable<T>>,

    /// Entity whose visualization should be the end point
    /// of this entity's visualization if [`display`](PropertyMap::display)
    /// is [`Connector`](DisplayMode::Connector).
    pub target: Option<Selectable<T>>,
}

/// Ways to visualize an entity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DisplayMode {
    /// Entity is displayed as a connector.
    Connector,

    /// Entity is displayed as an element with the provided tag name.
    ElementTag(String),
}
