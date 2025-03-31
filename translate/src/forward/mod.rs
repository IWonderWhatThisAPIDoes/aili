//! Forwarding results of the [cascade](crate::cascade)
//! to the [visualization tree](aili_model::vis).

mod debug;
mod test_vis;

use crate::property::{DisplayMode, EntityPropertyMapping, PropertyMap, Selectable};
use aili_model::{state::NodeId, vis::*};
use std::collections::HashMap;

/// Updates the structure of a [`VisTree`] to reflect
/// changes in stylesheet resolution.
pub struct Renderer<T: NodeId, V: VisTree> {
    /// The target visualization tree.
    vis_tree: V,

    /// Selectable entity whose associated visual element
    /// is currently at the root of the visualization tree.
    current_root: Option<Selectable<T>>,

    /// Associated visual elements and current properties
    /// of all visualized entities.
    current_mappping: HashMap<Selectable<T>, EntityRendering<T, V>>,
}

impl<T: NodeId, V: VisTree> Renderer<T, V> {
    /// Constructs a new renderer that renders into a tree.
    pub fn new(vis_tree: V) -> Self {
        Self {
            vis_tree,
            current_root: None,
            current_mappping: HashMap::new(),
        }
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
        for mapping in self.current_mappping.values() {
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
                            // to swap a parent and its child. That cannot happen
                            // unless we disconnect one first and reconnect it later.
                            element
                                .insert_into(None)
                                .expect("Detachment should never fail");
                            retry_element_insertions.push((handle, parent_handle));
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
        for (child_handle, parent_handle) in retry_element_insertions {
            // If the insertion fails again, we know for sure it is because
            // the user forced a loop with their stylesheet
            // todo: log the error
            let _ = self
                .vis_tree
                .get_element(child_handle)
                .expect("The handle should remain valid")
                .insert_into(parent_handle);
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

#[cfg(test)]
mod test {
    use super::{test_vis::*, *};
    use std::collections::HashMap;

    macro_rules! construct {
        ( $ty:ident { $( $prop:ident $( : $val:expr )? ),* $(,)? } ) => {
            $ty {
                $( $prop $( : $val )? ,)*
                ..$ty::default()
            }
        };
    }

    /// Shorthand for cunstructing the argument of [`Renderer::update`].
    macro_rules! mapping {
        ( $( $id:expr => { $($fill:tt)* } ),* $(,)? ) => {
            [
                $((
                    Selectable::node($id),
                    construct!(PropertyMap { $($fill)* })
                )),*
            ].into()
        };
    }

    /// Shorthand for constructing a set of expected [`TestVisElement`]s.
    macro_rules! expect_elements {
        ( $( { $($fill:tt)* } ),* $(,)? ) => {
            vec![
                $(
                    construct!(TestVisElement { $($fill)* })
                ),*
            ]
        };
    }

    /// Shorthand for constructing a set of expected [`TestVisConnector`]s.
    macro_rules! expect_connectors {
        ( $( { $($fill:tt)* } ),* $(,)? ) => {
            vec![
                $(
                    construct!(TestVisConnector { $($fill)* })
                ),*
            ]
        };
    }

    /// Shorthand for verifying that a collection contains
    /// two elements, with one being the parent of the other
    fn expect_one_parent_and_child(tree: &TestVisTree) {
        let parent_index = tree
            .elements
            .iter()
            .enumerate()
            .find(|(_, e)| e.parent_index.is_none())
            .expect("One element without a parent should be present")
            .0;
        tree.elements
            .iter()
            .enumerate()
            .find(|(_, e)| e.parent_index == Some(parent_index))
            .expect("One element that is a child of the other should be present");
    }

    #[test]
    fn create_element() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
        ]);
        assert_eq!(
            renderer.vis_tree.elements,
            expect_elements![{ tag_name: "cell".to_owned() }]
        );
    }

    #[test]
    fn create_connector() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::Connector) },
        ]);
        assert_eq!(renderer.vis_tree.connectors, expect_connectors![{}]);
    }

    #[test]
    fn create_nothing() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![0 => { display: None }]);
        assert!(renderer.vis_tree.connectors.is_empty());
        assert!(renderer.vis_tree.elements.is_empty());
    }

    #[test]
    fn create_element_with_attributes() {
        let mut renderer = Renderer::new(TestVisTree::default());
        let attributes = HashMap::from_iter([
            ("hello".to_owned(), "world".to_owned()),
            ("a".to_owned(), "b".to_owned()),
        ]);
        renderer.update(mapping![
            0 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                attributes: attributes.clone(),
            },
        ]);
        assert_eq!(
            renderer.vis_tree.elements,
            expect_elements![{ tag_name: "cell".to_owned(), attributes }]
        );
    }

    #[test]
    fn update_element_attributes() {
        let mut renderer = Renderer::new(TestVisTree::default());
        let attributes = HashMap::from_iter([
            ("hello".to_owned(), "world".to_owned()),
            ("a".to_owned(), "b".to_owned()),
        ]);
        let updated_attributes = HashMap::from_iter([
            ("a".to_owned(), "c".to_owned()),
            ("x".to_owned(), "y".to_owned()),
        ]);
        renderer.update(mapping![
            0 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                attributes,
            },
        ]);
        renderer.update(mapping![
            0 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                attributes: updated_attributes.clone(),
            },
        ]);
        // The element should not be recreated, only its attributes should be updated.
        // Attributes should be added, updated, and deleted
        assert_eq!(
            renderer.vis_tree.elements,
            expect_elements![{ tag_name: "cell".to_owned(), attributes: updated_attributes }]
        );
    }

    #[test]
    fn create_element_with_parent() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0))
            },
        ]);
        expect_one_parent_and_child(&renderer.vis_tree);
    }

    #[test]
    fn create_element_without_parent() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(1)), // But there is no such node
            },
        ]);
        // The parent should simply not be set
        assert_eq!(
            renderer.vis_tree.elements,
            expect_elements![{ tag_name: "cell".to_owned() }]
        );
    }

    #[test]
    fn create_element_with_connector_for_parent() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::Connector) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)) // Which is a connector
            },
        ]);
        // The parent should not be set, it is not an element
        assert_eq!(
            renderer.vis_tree.elements,
            expect_elements![{ tag_name: "cell".to_owned() }]
        );
    }

    #[test]
    fn update_parent_of_element() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
        ]);
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)),
            },
        ]);
        expect_one_parent_and_child(&renderer.vis_tree);
    }

    #[test]
    fn unset_parent_of_element() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)),
            },
        ]);
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
        ]);
        // The parent should have been removed
        assert_eq!(
            renderer.vis_tree.elements,
            expect_elements![
                { tag_name: "cell".to_owned() },
                { tag_name: "cell".to_owned(), parent_index: None },
            ]
        );
    }

    #[test]
    fn remove_parent_of_element() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)),
            },
        ]);
        renderer.update(mapping![
            0 => { display: None },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)),
            },
        ]);
        assert_eq!(
            renderer.vis_tree.elements,
            expect_elements![
                // This is the old representation of element 0,
                // we do not have a garbage collector, so it stays
                { tag_name: "cell".to_owned() },
                // This is element 1, it should not have its parent set anymore
                { tag_name: "cell".to_owned(), parent_index: None },
            ]
        );
    }

    #[test]
    fn create_connector_with_pins() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => { display: Some(DisplayMode::ElementTag("kvt".to_owned())) },
            2 => {
                display: Some(DisplayMode::Connector),
                parent: Some(Selectable::node(0)),
                target: Some(Selectable::node(1)),
            },
        ]);
        let index_of_first = renderer
            .vis_tree
            .elements
            .iter()
            .enumerate()
            .find(|(_, e)| e.tag_name == "cell")
            .expect("One <cell> element should be present")
            .0;
        let index_of_second = renderer
            .vis_tree
            .elements
            .iter()
            .enumerate()
            .find(|(_, e)| e.tag_name == "kvt")
            .expect("One <kvt> element should be present")
            .0;
        assert_eq!(
            renderer.vis_tree.connectors,
            expect_connectors![{
                start: TestVisPin { target_index: Some(index_of_first) },
                end: TestVisPin { target_index: Some(index_of_second) },
            }]
        );
    }

    #[test]
    fn change_element_into_connector() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)),
            },
        ]);
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::Connector) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)),
            },
        ]);
        assert_eq!(
            renderer.vis_tree.elements,
            expect_elements![
                // This is the old representation of element 0,
                // we do not have a garbage collector, so it stays
                { tag_name: "cell".to_owned() },
                // Element 1 should no longer be linked to its parent
                { tag_name: "cell".to_owned() },
            ]
        );
        assert_eq!(renderer.vis_tree.connectors, expect_connectors![{}]);
    }

    #[test]
    fn change_connector_into_element() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::Connector) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)),
            },
        ]);
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)),
            },
        ]);
        assert_eq!(
            renderer.vis_tree.elements,
            expect_elements![
                // Element 1 should have automatically attached
                // itself to its parent
                { tag_name: "cell".to_owned(), parent_index: Some(1) },
                // Element 0
                { tag_name: "cell".to_owned() },
            ]
        );
    }

    #[test]
    fn change_element_tag_name() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
        ]);
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("kvt".to_owned())) },
        ]);
        assert_eq!(
            renderer.vis_tree.elements,
            expect_elements![
                // This is the old representation
                // W do not have GC, so it stays
                { tag_name: "cell".to_owned() },
                // Element should have been recreated
                { tag_name: "kvt".to_owned() },
            ]
        );
    }

    #[test]
    fn change_parent_element_tag_name() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)),
            },
        ]);
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("kvt".to_owned())) },
            1 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(0)),
            },
        ]);
        // Child's parent should now be the last inserted node
        // since it has just been recreated
        renderer
            .vis_tree
            .elements
            .iter()
            .enumerate()
            .find(|(_, e)| e.parent_index == Some(2))
            .expect("Child element should have moved under new representation of its parent");
    }

    #[test]
    fn swap_parent_and_child() {
        let mut renderer = Renderer::new(TestVisTree::default());
        renderer.update(mapping![
            0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
            1 => {
                display: Some(DisplayMode::ElementTag("kvt".to_owned())),
                parent: Some(Selectable::node(0)),
            },
            2 => {
                display: Some(DisplayMode::ElementTag("label".to_owned())),
                parent: Some(Selectable::node(1)),
            },
        ]);
        renderer.update(mapping![
            0 => {
                display: Some(DisplayMode::ElementTag("cell".to_owned())),
                parent: Some(Selectable::node(1)),
            },
            1 => {
                display: Some(DisplayMode::ElementTag("kvt".to_owned())),
                parent: Some(Selectable::node(2)),
            },
            2 => { display: Some(DisplayMode::ElementTag("label".to_owned())) },
        ]);
        // Verify the structural order between nodes
        // (they may have been inserted in any order, so we must reconstruct the order)
        let element_2 = renderer
            .vis_tree
            .expect_find_element(|e| e.tag_name == "label");
        let element_1 = renderer
            .vis_tree
            .expect_find_element(|e| e.tag_name == "kvt");
        let element_0 = renderer
            .vis_tree
            .expect_find_element(|e| e.tag_name == "cell");
        assert_eq!(renderer.vis_tree.elements[element_2].parent_index, None);
        assert_eq!(
            renderer.vis_tree.elements[element_1].parent_index,
            Some(element_2)
        );
        assert_eq!(
            renderer.vis_tree.elements[element_0].parent_index,
            Some(element_1)
        );
    }
}
