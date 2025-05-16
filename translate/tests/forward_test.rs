//! Tests for [`VisTreeWriter`].

mod test_vis;

use aili_style::selectable::Selectable;
use aili_translate::{
    forward::{VisTreeWriter, VisTreeWriterWarning},
    property::{DisplayMode, FragmentKey, PropertyMap},
};
use std::collections::HashMap;
use test_vis::*;

macro_rules! construct {
    ( $ty:ident { $( $prop:ident $( : $val:expr )? ),* $(,)? } ) => {
        $ty {
            $( $prop $( : $val )? ,)*
            ..$ty::default()
        }
    };
}

/// Shorthand for cunstructing the argument of [`VisTreeWriter::update`].
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
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
    renderer.update(mapping![
        0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
    ]);
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.elements,
        expect_elements![{ tag_name: "cell".to_owned() }]
    );
}

#[test]
fn create_connector() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
    renderer.update(mapping![
        0 => { display: Some(DisplayMode::Connector) },
    ]);
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(vis_tree.connectors, expect_connectors![{}]);
}

#[test]
fn create_nothing() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
    renderer.update(mapping![0 => { display: None }]);
    let vis_tree = renderer.reclaim_vis_tree();
    assert!(vis_tree.connectors.is_empty());
    assert!(vis_tree.elements.is_empty());
}

#[test]
fn create_element_with_attributes() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
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
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.elements,
        expect_elements![{ tag_name: "cell".to_owned(), attributes }]
    );
}

#[test]
fn update_element_attributes() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
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
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.elements,
        expect_elements![{ tag_name: "cell".to_owned(), attributes: updated_attributes }]
    );
}

#[test]
fn create_element_with_parent() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
    renderer.update(mapping![
        0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
        1 => {
            display: Some(DisplayMode::ElementTag("cell".to_owned())),
            parent: Some(Selectable::node(0))
        },
    ]);
    let vis_tree = renderer.reclaim_vis_tree();
    expect_one_parent_and_child(&vis_tree);
}

#[test]
fn create_element_without_parent() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
    renderer.update(mapping![
        0 => {
            display: Some(DisplayMode::ElementTag("cell".to_owned())),
            parent: Some(Selectable::node(1)), // But there is no such node
        },
    ]);
    // The parent should simply not be set
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.elements,
        expect_elements![{ tag_name: "cell".to_owned() }]
    );
}

#[test]
fn create_element_with_connector_for_parent() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
    renderer.update(mapping![
        0 => { display: Some(DisplayMode::Connector) },
        1 => {
            display: Some(DisplayMode::ElementTag("cell".to_owned())),
            parent: Some(Selectable::node(0)) // Which is a connector
        },
    ]);
    // The parent should not be set, it is not an element
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.elements,
        expect_elements![{ tag_name: "cell".to_owned() }]
    );
}

#[test]
fn update_parent_of_element() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
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
    let vis_tree = renderer.reclaim_vis_tree();
    expect_one_parent_and_child(&vis_tree);
}

#[test]
fn unset_parent_of_element() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
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
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.elements,
        expect_elements![
            { tag_name: "cell".to_owned() },
            { tag_name: "cell".to_owned(), parent_index: None },
        ]
    );
}

#[test]
fn remove_parent_of_element() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
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
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.elements,
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
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
    renderer.update(mapping![
        0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
        1 => { display: Some(DisplayMode::ElementTag("kvt".to_owned())) },
        2 => {
            display: Some(DisplayMode::Connector),
            parent: Some(Selectable::node(0)),
            target: Some(Selectable::node(1)),
        },
    ]);
    let vis_tree = renderer.reclaim_vis_tree();
    let index_of_first = vis_tree
        .elements
        .iter()
        .enumerate()
        .find(|(_, e)| e.tag_name == "cell")
        .expect("One <cell> element should be present")
        .0;
    let index_of_second = vis_tree
        .elements
        .iter()
        .enumerate()
        .find(|(_, e)| e.tag_name == "kvt")
        .expect("One <kvt> element should be present")
        .0;
    assert_eq!(
        vis_tree.connectors,
        expect_connectors![{
            start: TestVisPin { target_index: Some(index_of_first), attributes: [].into() },
            end: TestVisPin { target_index: Some(index_of_second), attributes: [].into() },
        }]
    );
}

#[test]
fn change_element_into_connector() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
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
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.elements,
        expect_elements![
            // This is the old representation of element 0,
            // we do not have a garbage collector, so it stays
            { tag_name: "cell".to_owned() },
            // Element 1 should no longer be linked to its parent
            { tag_name: "cell".to_owned() },
        ]
    );
    assert_eq!(vis_tree.connectors, expect_connectors![{}]);
}

#[test]
fn change_connector_into_element() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
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
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.elements,
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
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
    renderer.update(mapping![
        0 => { display: Some(DisplayMode::ElementTag("cell".to_owned())) },
    ]);
    renderer.update(mapping![
        0 => { display: Some(DisplayMode::ElementTag("kvt".to_owned())) },
    ]);
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.elements,
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
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
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
        .reclaim_vis_tree()
        .elements
        .iter()
        .enumerate()
        .find(|(_, e)| e.parent_index == Some(2))
        .expect("Child element should have moved under new representation of its parent");
}

#[test]
fn swap_parent_and_child() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
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
    let vis_tree = renderer.reclaim_vis_tree();
    let element_2 = vis_tree.expect_find_element(|e| e.tag_name == "label");
    let element_1 = vis_tree.expect_find_element(|e| e.tag_name == "kvt");
    let element_0 = vis_tree.expect_find_element(|e| e.tag_name == "cell");
    assert_eq!(vis_tree.elements[element_2].parent_index, None);
    assert_eq!(vis_tree.elements[element_1].parent_index, Some(element_2));
    assert_eq!(vis_tree.elements[element_0].parent_index, Some(element_1));
}

#[test]
fn set_connector_fragment_attributes() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
    renderer.update(mapping![
        0 => {
            display: Some(DisplayMode::Connector),
            fragment_attributes: [
                (FragmentKey::Start, [("key".to_owned(), "a".to_owned())].into()),
                (FragmentKey::End, [("value".to_owned(), "b".to_owned())].into()),
            ]
            .into(),
        },
    ]);
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.connectors,
        expect_connectors![{
            start: TestVisPin {
                target_index: None,
                attributes: [("key".to_owned(), "a".to_owned())].into(),
            },
            end: TestVisPin {
                target_index: None,
                attributes: [("value".to_owned(), "b".to_owned())].into(),
            },
        }],
    );
}

#[test]
fn update_connector_fragment_attributes() {
    let mut renderer = VisTreeWriter::new(TestVisTree::default());
    renderer.update(mapping![
        0 => {
            display: Some(DisplayMode::Connector),
            fragment_attributes: [
                (
                    FragmentKey::Start,
                    [
                        ("a".to_owned(), "a".to_owned()),
                        ("b".to_owned(), "b".to_owned()),
                    ]
                    .into()),
            ]
            .into(),
        },
    ]);
    renderer.update(mapping![
        0 => {
            display: Some(DisplayMode::Connector),
            fragment_attributes: [
                (
                    FragmentKey::Start,
                    [
                        ("b".to_owned(), "d".to_owned()),
                        ("c".to_owned(), "c".to_owned()),
                    ]
                    .into()),
            ]
            .into(),
        },
    ]);
    let vis_tree = renderer.reclaim_vis_tree();
    assert_eq!(
        vis_tree.connectors,
        expect_connectors![{
            start: TestVisPin {
                target_index: None,
                attributes: [
                    ("b".to_owned(), "d".to_owned()),
                    ("c".to_owned(), "c".to_owned()),
                ]
                .into(),
            },
            end: TestVisPin { target_index: None, attributes: [].into() },
        }],
    );
}

#[test]
fn create_loop_in_vis_tree() {
    let mut warning_was_emited = false;
    let mut renderer =
        VisTreeWriter::new(TestVisTree::default()).with_warning_handler(Box::new(|warning| {
            match warning {
                VisTreeWriterWarning::VisStructureViolation(_) => warning_was_emited = true,
            }
        }));
    renderer.update(mapping![
        0 => {
            display: Some(DisplayMode::ElementTag("cell".to_owned())),
            parent: Some(Selectable::node(1)),
        },
        1 => {
            display: Some(DisplayMode::ElementTag("cell".to_owned())),
            parent: Some(Selectable::node(0)),
        },
    ]);
    drop(renderer);
    assert!(warning_was_emited);
}
