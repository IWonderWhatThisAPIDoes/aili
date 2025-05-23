//! Keys and values of properties of visualizable entities.

pub mod symbols;

use aili_model::state::NodeId;
use aili_style::selectable::Selectable;
use derive_more::{Debug, From};
use std::collections::HashMap;

/// A key that values can be assigned to on a selectable entity.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum PropertyKey {
    /// Assigns value to an attribute of the selected entity.
    Attribute(String),

    /// Assigns value to an attribute of a fragment of the selected entity.
    FragmentAttribute(FragmentKey, String),

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
#[derive(Clone, PartialEq, Eq)]
pub struct PropertyMap<T: NodeId> {
    /// Attributes with string values.
    pub attributes: HashMap<String, String>,

    /// Attributes with string values that belong to fragments
    /// of the visual element, rather than the element as a whole.
    pub fragment_attributes: HashMap<FragmentKey, HashMap<String, String>>,

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

impl<T: NodeId> PropertyMap<T> {
    /// Constructs an empty property map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a display mode to the property map.
    pub fn with_display(mut self, display_mode: DisplayMode) -> Self {
        self.display = Some(display_mode);
        self
    }

    /// Adds a parent reference to the property map.
    pub fn with_parent(mut self, parent: Selectable<T>) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Adds a target reference to the property map.
    pub fn with_target(mut self, target: Selectable<T>) -> Self {
        self.target = Some(target);
        self
    }

    /// Adds an attribute value to the property map.
    pub fn with_attribute(mut self, attribute_name: String, attribute_value: String) -> Self {
        self.attributes.insert(attribute_name, attribute_value);
        self
    }

    /// Adds an attribute value of a fragment to the property map
    pub fn with_fragment_attribute(
        mut self,
        fragment: FragmentKey,
        attribute_name: String,
        attribute_value: String,
    ) -> Self {
        self.fragment_attributes
            .entry(fragment)
            .or_default()
            .insert(attribute_name, attribute_value);
        self
    }
}

impl<T: NodeId> Default for PropertyMap<T> {
    fn default() -> Self {
        Self {
            attributes: HashMap::default(),
            fragment_attributes: HashMap::default(),
            display: None,
            parent: None,
            target: None,
        }
    }
}

impl<T: NodeId> std::fmt::Debug for PropertyMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;
        if let Some(display) = &self.display {
            write!(f, "display: {display:?}; ")?;
        }
        if let Some(parent) = &self.parent {
            write!(f, "parent: {parent:?}; ")?;
        }
        if let Some(target) = &self.target {
            write!(f, "target: {target:?}; ")?;
        }
        for (key, value) in &self.attributes {
            write!(f, "{key:?}: {value:?}; ")?;
        }
        for (fragment, attributes) in &self.fragment_attributes {
            for (key, value) in attributes {
                write!(f, "{fragment:?}/{key:?}: {value:?}; ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

/// Ways to visualize an entity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DisplayMode {
    /// Entity is displayed as a connector.
    #[debug("connector")]
    Connector,

    /// Entity is displayed as an element with the provided tag name.
    #[debug("<{_0}>")]
    ElementTag(String),
}

/// Identifies fragments of entities that are recognized by the renderer.
///
/// [`PropertyMap`]s can assign attributes not just to whole entities,
/// but to their fragments as well. These keys identify the known fragment types.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum FragmentKey {
    /// The start endpoint of an entity with [`DisplayMode::Connector`].
    ///
    /// If the display mode is not [`DisplayMode::Connector`],
    /// attributes assigned to this fragment are ignored.
    #[debug("start")]
    Start,

    /// The end endpoint of an entity with [`DisplayMode::Connector`].
    ///
    /// If the display mode is not [`DisplayMode::Connector`],
    /// attributes assigned to this fragment are ignored.
    #[debug("end")]
    End,
}

/// Represents the mapping between selectable entities and their display
/// properties, computed by evaluating the cascade.
#[derive(Clone, PartialEq, Eq, From, Debug)]
#[from(forward)]
pub struct EntityPropertyMapping<T: NodeId>(pub HashMap<Selectable<T>, PropertyMap<T>>);

impl<T: NodeId> EntityPropertyMapping<T> {
    /// Constructs an empty property mapping.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: NodeId> Default for EntityPropertyMapping<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
