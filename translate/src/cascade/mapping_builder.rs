//! Helper for construction of [`EntityPropertyMapping`]s.

use crate::property::{DisplayMode, EntityPropertyMapping, PropertyKey};
use aili_model::state::{NodeId, ProgramStateGraph, ProgramStateNode};
use aili_style::{selectable::Selectable, values::PropertyValue};
use std::collections::{HashMap, hash_map::Entry};

/// Identifier of a property variable on an entity.
#[derive(PartialEq, Eq, Debug, Hash)]
struct EntityPropertyKey<T: NodeId>(Selectable<T>, PropertyKey);

/// Value assigned to a property variable based on a rule
#[derive(Debug)]
struct RulePropertyValue<T: NodeId> {
    /// Value assigned to the property.
    value: PropertyValue<T>,
    /// Index of the rule that assigned the value.
    /// Relevant for calculating precedence.
    static_precedence: usize,
    /// Whether the value was assigned explicitly
    /// or as the side effect of another assignment.
    passive: bool,
}

impl<T: NodeId> RulePropertyValue<T> {
    /// Overwrites the existing value with a new one, but only
    /// if the new value has greater or equal precedence.
    ///
    /// ## Return Value
    /// True if the new value was written, false otherwise.
    fn assign_new_value(&mut self, candidate_value: Self) -> bool {
        // Passive assignments take lower priority always,
        // otherwise the precedence is decided based on evaluation order
        let precedence = |value: &Self| (!value.passive, value.static_precedence);
        if precedence(&candidate_value) >= precedence(self) {
            *self = candidate_value;
            true
        } else {
            false
        }
    }
}

/// Helper object for constructing an [`EntityPropertyMapping`].
pub struct PropertyMappingBuilder<T: NodeId> {
    /// Values assigned to each property on each node.
    properties: HashMap<EntityPropertyKey<T>, RulePropertyValue<T>>,

    /// Stack that tracks the information necessary to assign auto-defaults.
    auto_stack: Vec<AutoAssignmentContext<T>>,
}

impl<T: NodeId> PropertyMappingBuilder<T> {
    /// Constructs an empty mapping builder.
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            auto_stack: vec![AutoAssignmentContext::default()],
        }
    }

    /// Pushes a context frame onto the builder.
    pub fn push(&mut self) {
        self.auto_stack
            .push(self.auto_stack.last().unwrap().clone());
    }

    /// Pops a context frame off the builder.
    pub fn pop(&mut self) {
        if self.auto_stack.len() > 1 {
            self.auto_stack.pop();
        }
    }

    /// Finalizes the property mapping.
    pub fn build(mut self, graph: &impl ProgramStateGraph<NodeId = T>) -> EntityPropertyMapping<T> {
        let mut mapping = EntityPropertyMapping::new();
        for (EntityPropertyKey(entity, property), RulePropertyValue { value, .. }) in
            std::mem::take(&mut self.properties)
        {
            // Insert the property map lazily
            let entity_properties = || mapping.0.entry(entity).or_default();
            match property {
                PropertyKey::Attribute(name) => {
                    let value = Self::to_true_value(value, graph);
                    // If value if Unset, the attribute should not be saved at all
                    if value != PropertyValue::Unset {
                        entity_properties()
                            .attributes
                            .insert(name, value.to_string());
                    }
                }
                PropertyKey::FragmentAttribute(fragment, name) => {
                    let value = Self::to_true_value(value, graph);
                    // If value is Unset, the attribute should not be saved at all
                    if value != PropertyValue::Unset {
                        entity_properties()
                            .fragment_attributes
                            .entry(fragment)
                            .or_default()
                            .insert(name, value.to_string());
                    }
                }
                PropertyKey::Display => {
                    let display_mode = match &value {
                        PropertyValue::Unset => None,
                        PropertyValue::Selection(sel) => {
                            if sel.is_node() {
                                graph
                                    .get(&sel.node_id)
                                    .and_then(|node| node.value())
                                    .map(PropertyValue::<T>::from)
                                    .as_ref()
                                    .map(PropertyValue::to_string)
                                    .map(DisplayMode::from_name)
                            } else {
                                None
                            }
                        }
                        _ => Some(DisplayMode::from_name(value.to_string())),
                    };
                    if display_mode.is_some() {
                        entity_properties().display = display_mode;
                    }
                }
                PropertyKey::Parent => {
                    if let PropertyValue::Selection(sel) = value {
                        entity_properties().parent = Some(*sel);
                    }
                }
                PropertyKey::Target => {
                    if let PropertyValue::Selection(sel) = value {
                        entity_properties().target = Some(*sel);
                    }
                }
                PropertyKey::Detach => {}
            }
        }
        mapping
    }

    /// Notifies the builder that an entity has been encountered.
    /// The builder may apply default appearences to it.
    pub fn selected_entity(
        &mut self,
        target: &Selectable<T>,
        select_origin: &T,
        static_precedence: usize,
    ) {
        // Edges that are selected are automatically displayed as conenctors
        if target.is_edge() {
            // Display as connector
            let display_key = EntityPropertyKey(target.clone(), PropertyKey::Display);
            let display_value = RulePropertyValue {
                value: PropertyValue::String(DisplayMode::CONNECTOR_NAME.to_owned()),
                static_precedence,
                passive: true,
            };
            self.write_property(display_key, display_value);
            // Parent is source
            let parent_key = EntityPropertyKey(target.clone(), PropertyKey::Parent);
            let parent_value = RulePropertyValue {
                value: PropertyValue::Selection(Selectable::node(target.node_id.clone()).into()),
                static_precedence,
                passive: true,
            };
            self.write_property(parent_key, parent_value);
            // Target is target
            let target_key = EntityPropertyKey(target.clone(), PropertyKey::Target);
            let target_value = RulePropertyValue {
                value: PropertyValue::Selection(Selectable::node(select_origin.clone()).into()),
                static_precedence,
                passive: true,
            };
            self.write_property(target_key, target_value);
        }
    }

    /// Assigns a value to a property key of a given entity.
    pub fn assign(
        &mut self,
        target: &Selectable<T>,
        key: &PropertyKey,
        value: PropertyValue<T>,
        static_precedence: usize,
    ) {
        let full_key = EntityPropertyKey(target.clone(), key.clone());
        let full_value = RulePropertyValue {
            value,
            static_precedence,
            passive: false,
        };
        let updated_property = self.write_property(full_key, full_value);
        // If we just chaned the display mode of an entity,
        // we should auto-assign common values to other properties
        if updated_property && *key == PropertyKey::Display {
            if target.is_node() {
                // If the display property of a node is explicitly
                // assigned, that node becomes the parent of its successors
                // by default
                self.auto_stack.last_mut().unwrap().parent = Some(target.clone());
                // Likewise, it is adopted by its predecessor, if any
                if let Some(parent) = self.prev_auto_frame().and_then(|f| f.parent.as_ref()) {
                    let parent_key = EntityPropertyKey(target.clone(), PropertyKey::Parent);
                    let parent_value = RulePropertyValue {
                        value: PropertyValue::Selection(parent.clone().into()),
                        static_precedence,
                        passive: true,
                    };
                    self.write_property(parent_key, parent_value);
                }
            }
            if target.is_extra() {
                // Extra will be adopted by its owner
                let parent_key = EntityPropertyKey(target.clone(), PropertyKey::Parent);
                let parent_value = RulePropertyValue {
                    value: PropertyValue::Selection(target.clone().without_extra().into()),
                    static_precedence,
                    passive: true,
                };
                self.write_property(parent_key, parent_value);
            }
        }
    }

    /// Retrieves the second-to-last [`AutoAssignmentContext`], if present.
    /// This should be used to acquire context from previous entities.
    fn prev_auto_frame(&self) -> Option<&AutoAssignmentContext<T>> {
        if self.auto_stack.len() > 1 {
            Some(&self.auto_stack[self.auto_stack.len() - 2])
        } else {
            None
        }
    }

    /// Shorthand for assigning a [`RulePropertyValue`] to an [`EntityPropertyKey`].
    ///
    /// ## Return value
    /// True if the property has been written, false if there was already
    /// a value with greater precedence present.
    fn write_property(&mut self, key: EntityPropertyKey<T>, value: RulePropertyValue<T>) -> bool {
        match self.properties.entry(key) {
            Entry::Occupied(mut existing) => existing.get_mut().assign_new_value(value),
            Entry::Vacant(entry) => {
                entry.insert(value);
                true
            }
        }
    }

    /// Converts a [`PropertyValue::Selection`] to an explicit value.
    fn to_true_value(
        value: PropertyValue<T>,
        graph: &impl ProgramStateGraph<NodeId = T>,
    ) -> PropertyValue<T> {
        if let PropertyValue::Selection(sel) = &value {
            if sel.is_node() {
                graph
                    .get(&sel.node_id)
                    .and_then(|node| node.value())
                    .map(Into::into)
                    .unwrap_or_default()
            } else {
                PropertyValue::Unset
            }
        } else {
            value
        }
    }
}

/// Information that must be carried around
/// in order to auto-assign [`PropertyKey::Parent`]
/// and [`PropertyKey::Target`] properties.
#[derive(Clone)]
struct AutoAssignmentContext<T: NodeId> {
    parent: Option<Selectable<T>>,
}

impl<T: NodeId> Default for AutoAssignmentContext<T> {
    fn default() -> Self {
        Self { parent: None }
    }
}

impl DisplayMode {
    const CONNECTOR_NAME: &'static str = "connector";

    fn from_name(name: String) -> Self {
        match name.as_str() {
            Self::CONNECTOR_NAME => Self::Connector,
            _ => Self::ElementTag(name),
        }
    }
}
