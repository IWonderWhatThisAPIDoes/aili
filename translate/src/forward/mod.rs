//! Forwarding results of the [cascade](crate::cascade)
//! to the [visualization tree](aili_model::vis).

mod debug;

use crate::property::{DisplayMode, EntityPropertyMapping, FragmentKey, PropertyMap};
use aili_model::{state::NodeId, vis::*};
use aili_style::selectable::Selectable;
use derive_more::Display;
use std::collections::HashMap;

/// Describes an occurrence in the renderer
/// that should not arise when using it as intended
/// and is likely indicative of an error in the input.
#[derive(Debug, Display)]
pub enum RendererWarning<T: NodeId> {
    /// The resolved stylesheet has caused a cycle in the visualization tree.
    #[display("detected loop in vis tree near {_0:?}")]
    VisStructureViolation(Selectable<T>),
}

/// Updates the structure of a [`VisTree`] to reflect
/// changes in stylesheet resolution.
pub struct Renderer<'w, T: NodeId, V: VisTree> {
    /// The target visualization tree.
    vis_tree: V,

    /// Selectable entity whose associated visual element
    /// is currently at the root of the visualization tree.
    current_root: Option<Selectable<T>>,

    /// Associated visual elements and current properties
    /// of all visualized entities.
    current_mappping: HashMap<Selectable<T>, EntityRendering<T, V>>,

    /// Handler that processes warnings emited by the renderer.
    warning_handler: Option<Box<dyn FnMut(RendererWarning<T>) + 'w>>,
}

impl<'w, T: NodeId, V: VisTree> Renderer<'w, T, V> {
    /// Constructs a new renderer that renders into a tree.
    pub fn new(vis_tree: V) -> Self {
        Self {
            vis_tree,
            current_root: None,
            current_mappping: HashMap::new(),
            warning_handler: None,
        }
    }

    /// Adds a warning handler to the renderer.
    pub fn set_warning_handler(
        &mut self,
        warning_handler: Option<Box<dyn FnMut(RendererWarning<T>) + 'w>>,
    ) {
        self.warning_handler = warning_handler;
    }

    /// Adds a warning handler to the renderer.
    pub fn with_warning_handler(
        mut self,
        warning_handler: Box<dyn FnMut(RendererWarning<T>) + 'w>,
    ) -> Self {
        self.set_warning_handler(Some(warning_handler));
        self
    }

    /// Consumes self and returns the [`VisTree`] that was passed
    /// to the constructor.
    pub fn reclaim_vis_tree(self) -> V {
        self.vis_tree
    }

    /// Changes the root element.
    pub fn update_root(&mut self, new_root: Option<Selectable<T>>) {
        // Do nothing if the root is up-to-date
        if new_root == self.current_root {
            return;
        }
        self.current_root = new_root;
        // Propagate the update to the vis tree
        self.forward_update_root();
    }

    /// Updates the properties of all visual elements.
    pub fn update(&mut self, mut new_mapping: EntityPropertyMapping<T>) {
        let mut updated_mapping = HashMap::new();
        // Create renderings for entities that are not yet rendered and update those that are
        for (key, new_properties) in new_mapping.0.drain() {
            if let Some(new_entity_mapping) = self.update_or_create_rendering(&key, new_properties)
            {
                updated_mapping.insert(key, new_entity_mapping);
            }
        }
        // current_mapping now only contains entities that were rendered, but are no longer
        // supposed to be, so we destroy their renderings
        for (_, mapping) in std::mem::take(&mut self.current_mappping).drain() {
            self.remove_rendering(mapping);
        }
        // Put the new mapping in its place
        self.current_mappping = updated_mapping;
        // Inter-entity relationships should only be updated now,
        // after all entity recreating is completed
        self.update_inter_entity_relations();
        // Root element may have changed or its rendering may have been recreated
        self.forward_update_root();
    }

    /// Updates the parent-child and pin-target relationships of all active visual entities.
    fn update_inter_entity_relations(&mut self) {
        let mut retry_element_insertions = Vec::new();
        for (selectable, mapping) in &self.current_mappping {
            match &mapping.vis_handle {
                EitherVisHandle::Element(handle) => {
                    let mut element = self
                        .vis_tree
                        .get_element(handle)
                        .expect("The handle should remain valid");
                    let parent_handle = mapping
                        .properties
                        .parent
                        .as_ref()
                        .and_then(|key| self.current_mappping.get(key))
                        .and_then(|mapping| mapping.vis_handle.element());
                    match element.insert_into(parent_handle) {
                        Ok(()) => {}
                        Err(ParentAssignmentError::InvalidHandle(_)) => {
                            panic!("The handle should remain valid")
                        }
                        Err(ParentAssignmentError::StructureViolation) => {
                            // This does not always mean that the user has supplied
                            // an invalid stylesheet. It may happen when we intend
                            // to swap a parent and its child.
                            //
                            // In such cases, that cannot happen
                            // unless we disconnect one first and reconnect it later.
                            element
                                .insert_into(None)
                                .expect("Detachment should never fail");
                            retry_element_insertions.push((handle, parent_handle, selectable));
                        }
                    }
                }
                EitherVisHandle::Connector(handle) => {
                    let mut connector = self
                        .vis_tree
                        .get_connector(handle)
                        .expect("The handle should remain valid");
                    let start_handle = mapping
                        .properties
                        .parent
                        .as_ref()
                        .and_then(|key| self.current_mappping.get(key))
                        .and_then(|mapping| mapping.vis_handle.element());
                    let end_handle = mapping
                        .properties
                        .target
                        .as_ref()
                        .and_then(|key| self.current_mappping.get(key))
                        .and_then(|mapping| mapping.vis_handle.element());
                    connector
                        .start_mut()
                        .attach_to(start_handle)
                        .expect("The handle should remain valid");
                    connector
                        .end_mut()
                        .attach_to(end_handle)
                        .expect("The handle should remain valid");
                }
            }
        }
        // We have inserted everything except a few elements that we have detached
        // from their parents. This is where we retry failed assignments
        for (child_handle, parent_handle, selectable) in retry_element_insertions {
            let result = self
                .vis_tree
                .get_element(child_handle)
                .expect("The handle should remain valid")
                .insert_into(parent_handle);
            // If the insertion fails again, we know for sure it is because
            // the user forced a loop with their stylesheet
            match result {
                Ok(_) => {}
                Err(ParentAssignmentError::InvalidHandle(_)) => {
                    panic!("The handle should remain valid")
                }
                Err(ParentAssignmentError::StructureViolation) => {
                    if let Some(warning_handler) = &mut self.warning_handler {
                        warning_handler(RendererWarning::VisStructureViolation(selectable.clone()));
                    }
                }
            }
        }
    }

    /// Updates the existing rendering for an entity if possible,
    /// or creates a new one
    fn update_or_create_rendering(
        &mut self,
        key: &Selectable<T>,
        new_properties: PropertyMap<T>,
    ) -> Option<EntityRendering<T, V>> {
        // Get the existing mapping for the entity and remove it from the container
        if let Some(mut old_mapping) = self.current_mappping.remove(key) {
            if old_mapping.properties.display == new_properties.display {
                // The entity is already displayed and its display mode has not changed,
                // so we update the existing rendering instead of creating a new one
                self.update_attributes(&mut old_mapping, new_properties);
                Some(old_mapping)
            } else {
                // The entity's display mode has changed, so we destroy its existing
                // rendering and create a new one
                self.remove_rendering(old_mapping);
                self.try_create_rendering(new_properties)
            }
        } else {
            // The entity is not displayed yet, so we create a new rendering for it
            self.try_create_rendering(new_properties)
        }
    }

    /// Detaches and drops an existing entity rendering.
    fn remove_rendering(&mut self, mapping: EntityRendering<T, V>) {
        match mapping.vis_handle {
            EitherVisHandle::Element(handle) => {
                // Remove the element from its parent
                if let Ok(mut element) = self.vis_tree.get_element(&handle) {
                    element
                        .insert_into(None)
                        .expect("Detachment should never fail");
                }
            }
            EitherVisHandle::Connector(handle) => {
                // Remove the connector from both its endpoints
                if let Ok(mut connector) = self.vis_tree.get_connector(&handle) {
                    connector
                        .start_mut()
                        .attach_to(None)
                        .expect("Detachment should never fail");
                    connector
                        .end_mut()
                        .attach_to(None)
                        .expect("Detachment should never fail");
                }
            }
        }
    }

    /// Creates a new rendering for an entity bassed on its properties.
    /// If [`PropertyMap::display`] is [`None`], no rendering is created.
    fn try_create_rendering(
        &mut self,
        properties: PropertyMap<T>,
    ) -> Option<EntityRendering<T, V>> {
        let vis_handle = match &properties.display {
            Some(DisplayMode::ElementTag(tag_name)) => {
                let handle = self.vis_tree.add_element(tag_name);
                let mut element = self
                    .vis_tree
                    .get_element(&handle)
                    .expect("The element was just created");
                Self::set_attributes(
                    &mut element,
                    properties
                        .attributes
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str())),
                );
                EitherVisHandle::Element(handle)
            }
            Some(DisplayMode::Connector) => {
                let handle = self.vis_tree.add_connector();
                let mut connector = self
                    .vis_tree
                    .get_connector(&handle)
                    .expect("The connector was just created");
                Self::set_attributes(
                    &mut connector,
                    properties
                        .attributes
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str())),
                );
                if let Some(start_attrs) = properties.fragment_attributes.get(&FragmentKey::Start) {
                    Self::set_attributes(
                        &mut connector.start_mut(),
                        start_attrs.iter().map(|(k, v)| (k.as_str(), v.as_str())),
                    );
                }
                if let Some(end_attrs) = properties.fragment_attributes.get(&FragmentKey::End) {
                    Self::set_attributes(
                        &mut connector.end_mut(),
                        end_attrs.iter().map(|(k, v)| (k.as_str(), v.as_str())),
                    );
                }
                EitherVisHandle::Connector(handle)
            }
            // If display is not set, do not render the entity at all
            None => return None,
        };

        Some(EntityRendering {
            vis_handle,
            properties,
        })
    }

    /// Updates the attributes of a visual entity to reflect a stylesheet update.
    fn update_attributes(
        &mut self,
        mapping: &mut EntityRendering<T, V>,
        properties: PropertyMap<T>,
    ) {
        match &mapping.vis_handle {
            EitherVisHandle::Element(handle) => {
                let mut element = self
                    .vis_tree
                    .get_element(handle)
                    .expect("The handle should remain valid");
                Self::update_attribute_map(
                    &mut element,
                    std::mem::take(&mut mapping.properties.attributes),
                    properties
                        .attributes
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str())),
                );
                mapping.properties = properties;
            }
            EitherVisHandle::Connector(handle) => {
                let mut connector = self
                    .vis_tree
                    .get_connector(handle)
                    .expect("The handle should remain valid");
                Self::update_attribute_map(
                    &mut connector,
                    std::mem::take(&mut mapping.properties.attributes),
                    properties
                        .attributes
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str())),
                );
                Self::update_attribute_map(
                    &mut connector.start_mut(),
                    mapping
                        .properties
                        .fragment_attributes
                        .remove(&FragmentKey::Start)
                        .unwrap_or_default(),
                    properties
                        .fragment_attributes
                        .get(&FragmentKey::Start)
                        .into_iter()
                        .flatten()
                        .map(|(k, v)| (k.as_str(), v.as_str())),
                );
                Self::update_attribute_map(
                    &mut connector.end_mut(),
                    mapping
                        .properties
                        .fragment_attributes
                        .remove(&FragmentKey::End)
                        .unwrap_or_default(),
                    properties
                        .fragment_attributes
                        .get(&FragmentKey::End)
                        .into_iter()
                        .flatten()
                        .map(|(k, v)| (k.as_str(), v.as_str())),
                );
                mapping.properties = properties;
            }
        }
    }

    /// Initializes attributes of a visual entity.
    fn set_attributes<'a>(
        target: &mut impl AttributeMap,
        values: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) {
        for (key, value) in values {
            target.set_attribute(key, Some(value));
        }
    }

    /// Updates attributes of a visual entity.
    fn update_attribute_map<'a>(
        target: &mut impl AttributeMap,
        mut old_values: HashMap<String, String>,
        values: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) {
        for (key, value) in values {
            target.set_attribute(key, Some(value));
            old_values.remove(key);
        }
        for key in old_values.keys() {
            target.set_attribute(key, None);
        }
    }

    /// Updates the root element in the visualization tree.
    ///
    /// This can be called in reaction to an external change of the root entity,
    /// or because the root entity's visualization has been recreated.
    fn forward_update_root(&mut self) {
        let root_handle = self
            .current_root
            .as_ref()
            .and_then(|key| self.current_mappping.get(key))
            .and_then(|mapping| mapping.vis_handle.element());
        self.vis_tree
            .set_root(root_handle)
            .expect("The handle should remain valid");
    }
}

/// Represents a selectable entity that has a visual representation.
struct EntityRendering<T: NodeId, V: VisTree> {
    /// Handle to the visual associated with the entity.
    vis_handle: EitherVisHandle<V::ElementHandle, V::ConnectorHandle>,

    /// Current properties of the visual.
    properties: PropertyMap<T>,
}

/// Handle to a visual entity.
enum EitherVisHandle<E: VisHandle, C: VisHandle> {
    /// The visual entity is an element.
    Element(E),

    /// The visual entity is a connector.
    Connector(C),
}

impl<E: VisHandle, C: VisHandle> EitherVisHandle<E, C> {
    fn element(&self) -> Option<&E> {
        match self {
            Self::Element(h) => Some(h),
            Self::Connector(_) => None,
        }
    }
}
