mod utils;

use aili_gdbstate::hints::PointerLengthHintKey;
use aili_gdbstate::state::GdbStateGraph;
use aili_model::state::*;
use aili_style::cascade::CascadeStyle;
use aili_style::stylesheet::expression::{Expression, LimitedSelector};
use aili_style::stylesheet::selector::{EdgeMatcher, Selector, SelectorSegment};
use aili_style::stylesheet::{StyleClause, StyleKey, StyleRule, Stylesheet};
use utils::future::ExpectReady as _;
use utils::gdb_from_source;

#[test]
fn minimal_sample_program() {
    let mut gdb = gdb_from_source("int main(void) {}");
    let state_graph = GdbStateGraph::new(&mut gdb)
        .expect_ready()
        .expect("Could not construct state graph");
    let main = state_graph
        .get_at_root(&[EdgeLabel::Main])
        .expect("Entry point node should be present");
    assert_eq!(main.node_type_class(), NodeTypeClass::Frame);
    assert_eq!(main.node_type_id(), Some("main"));
    assert_eq!(main.value(), None);
}

#[test]
fn basic_function_argument() {
    let mut gdb = gdb_from_source("int main(int argc) {}");
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let argc = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("argc".to_owned(), 0)])
        .unwrap();
    assert_eq!(argc.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(argc.node_type_id(), Some("int"));
    assert_eq!(argc.value(), Some(NodeValue::Int(1)));
}

#[test]
fn basic_local_variable() {
    let mut gdb = gdb_from_source(
        r"
        int main(void) {
            int local = 42;
            /* breakpoint */;
        }",
    );
    gdb.run_to_line(4).unwrap();
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let local = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("local".to_owned(), 0)])
        .unwrap();
    assert_eq!(local.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(local.node_type_id(), Some("int"));
    assert_eq!(local.value(), Some(NodeValue::Int(42)));
}

#[test]
fn no_op_update() {
    let mut gdb = gdb_from_source(
        r"
        int main(void) {
            int local = 42;
        }",
    );
    let mut state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    state_graph.update(&mut gdb).expect_ready().unwrap();
    let local =
        state_graph.get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("local".to_owned(), 0)]);
    let another_local_id =
        state_graph.get_id_at_root(&[EdgeLabel::Main, EdgeLabel::Named("local".to_owned(), 1)]);
    assert!(local.is_some());
    assert!(another_local_id.is_none());
}

#[test]
fn deeper_stack_trace() {
    let mut gdb = gdb_from_source(
        r"
        int f() { /* breakpoint */ }
        int g() { f(); }
        int main(void) { g(); }",
    );
    gdb.run_to_line(2).unwrap();
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let top_frame = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Next, EdgeLabel::Next])
        .unwrap();
    assert_eq!(top_frame.node_type_class(), NodeTypeClass::Frame);
    assert_eq!(top_frame.node_type_id(), Some("f"));
    assert_eq!(top_frame.value(), None);
}

#[test]
fn variable_shadowing() {
    let mut gdb = gdb_from_source(
        r"
        int main(void) {
            int a = -42; // a#0 (this variable should not be loaded)
            if (1) {
                unsigned a = 42; // a#1
                /* breakpoint */ a++;
            }
        }",
    );
    gdb.run_to_line(6).unwrap();
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let a0_id = state_graph.get_id_at_root(&[EdgeLabel::Main, EdgeLabel::Named("a".to_owned(), 0)]);
    let a1 = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("a".to_owned(), 1)])
        .unwrap();
    assert_eq!(a1.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(a1.node_type_id(), Some("unsigned int"));
    assert_eq!(a1.value(), Some(NodeValue::Int(42)));
    assert!(a0_id.is_none());
}

#[test]
fn multiple_variables_with_same_name() {
    let mut gdb = gdb_from_source(
        r"
        int main(void) {
            int a = -42; // a#0
            if (1) {
                unsigned a = 42; // a#1
                /* breakpoint */ a++;
            }
        }",
    );
    let mut state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    gdb.run_to_line(6).unwrap();
    state_graph.update(&mut gdb).expect_ready().unwrap();
    let a0 = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("a".to_owned(), 0)])
        .unwrap();
    let a1 = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("a".to_owned(), 1)])
        .unwrap();
    assert_eq!(a0.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(a0.node_type_id(), Some("int"));
    assert_eq!(a0.value(), Some(NodeValue::Int(-42)));
    assert_eq!(a1.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(a1.node_type_id(), Some("unsigned int"));
    assert_eq!(a1.value(), Some(NodeValue::Int(42)));
}

#[test]
fn variable_out_of_scope() {
    let mut gdb = gdb_from_source(
        r"
        int main(void) {
            int a = -42; // a#0
            if (1) {
                unsigned a = 42; // a#1
                /* breakpoint 1 */ a++;
            }
            /* breakpoint 2 */;
        }",
    );
    gdb.run_to_line(6).unwrap();
    // Construct at the first breakpoint
    // Variable a#0 is not going to be created immediately because of visibility
    let mut state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    gdb.run_to_line(8).unwrap();
    // Update at the second breakpoint
    // Variable a#0 should be loaded now, and a#1 should go out of scope
    state_graph.update(&mut gdb).expect_ready().unwrap();
    let a0 = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("a".to_owned(), 0)])
        .unwrap();
    let a1_id = state_graph.get_id_at_root(&[EdgeLabel::Main, EdgeLabel::Named("a".to_owned(), 1)]);
    assert_eq!(a0.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(a0.node_type_id(), Some("int"));
    assert_eq!(a0.value(), Some(NodeValue::Int(-42)));
    assert!(a1_id.is_none());
}

#[test]
fn structure_variables() {
    let mut gdb = gdb_from_source(
        r"
        struct pair {
            int first;
            char second;
        };
        int main(void) {
            struct pair p = {42};
            /* breakpoint */;
        }",
    );
    gdb.run_to_line(8).unwrap();
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let pair_id = state_graph
        .get_id_at_root(&[EdgeLabel::Main, EdgeLabel::Named("p".to_owned(), 0)])
        .unwrap();
    let pair = state_graph.get(&pair_id).unwrap();
    let first = state_graph
        .get_at(&pair_id, &[EdgeLabel::Named("first".to_owned(), 0)])
        .unwrap();
    let second = state_graph
        .get_at(&pair_id, &[EdgeLabel::Named("second".to_owned(), 0)])
        .unwrap();
    assert_eq!(pair.node_type_class(), NodeTypeClass::Struct);
    assert_eq!(pair.node_type_id(), Some("pair"));
    assert_eq!(pair.value(), None);
    assert_eq!(first.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(first.node_type_id(), Some("int"));
    assert_eq!(first.value(), Some(NodeValue::Int(42)));
    assert_eq!(second.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(second.node_type_id(), Some("char"));
    assert_eq!(second.value(), Some(NodeValue::Int(0)));
}

#[test]
fn static_array_variables() {
    let mut gdb = gdb_from_source(
        r"
        int main(void) {
            int array[] = {1, 2};
            /* breakpoint */;
        }",
    );
    gdb.run_to_line(4).unwrap();
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let array_id = state_graph
        .get_id_at_root(&[EdgeLabel::Main, EdgeLabel::Named("array".to_owned(), 0)])
        .unwrap();
    let array = state_graph.get(&array_id).unwrap();
    let first = state_graph
        .get_at(&array_id, &[EdgeLabel::Index(0)])
        .unwrap();
    let second = state_graph
        .get_at(&array_id, &[EdgeLabel::Index(1)])
        .unwrap();
    let length = state_graph.get_at(&array_id, &[EdgeLabel::Length]).unwrap();
    assert_eq!(array.node_type_class(), NodeTypeClass::Array);
    assert_eq!(array.node_type_id(), None);
    assert_eq!(array.value(), None);
    assert_eq!(first.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(first.node_type_id(), Some("int"));
    assert_eq!(first.value(), Some(NodeValue::Int(1)));
    assert_eq!(second.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(second.node_type_id(), Some("int"));
    assert_eq!(second.value(), Some(NodeValue::Int(2)));
    assert_eq!(length.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(length.node_type_id(), None);
    assert_eq!(length.value(), Some(NodeValue::Uint(2)));
}

#[test]
fn update_after_pushing_stack() {
    let mut gdb = gdb_from_source(
        r"
        int f() { /* breakpoint */ }
        int main(void) { f(); }",
    );
    // Construct the graph in main
    let mut state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    // Call a function and update the graph
    gdb.run_to_line(2).unwrap();
    state_graph.update(&mut gdb).expect_ready().unwrap();
    let top_frame = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Next])
        .unwrap();
    assert_eq!(top_frame.node_type_class(), NodeTypeClass::Frame);
    assert_eq!(top_frame.node_type_id(), Some("f"));
    assert_eq!(top_frame.value(), None);
}

#[test]
fn update_after_popping_stack() {
    let mut gdb = gdb_from_source(
        r"
        int f() { /* breakpoint 1 */ }
        int main(void) {
            f();
            /* breakpoint 2 */
        }",
    );
    // Construct the graph in function
    gdb.run_to_line(2).unwrap();
    let mut state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    // Return from the function and update the graph
    gdb.run_to_line(5).unwrap();
    state_graph.update(&mut gdb).expect_ready().unwrap();
    let main = state_graph.get_at_root(&[EdgeLabel::Main]).unwrap();
    // The upper frame should be gone
    assert!(main.get_successor(&EdgeLabel::Next).is_none());
}

#[test]
fn update_in_function_call() {
    let mut gdb = gdb_from_source(
        r"
        int f() { /* breakpoint */ }
        int main(void) {
            int a = 0;
            f();
        }",
    );
    // Construct the graph in function call
    gdb.run_to_line(2).unwrap();
    let mut state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    // Update it right away, we want to see what the update does
    state_graph.update(&mut gdb).expect_ready().unwrap();
    let function_frame = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Next])
        .unwrap();
    // No variables should have leaked from other frames
    assert!(function_frame.successors().next().is_none());
}

#[test]
fn pointer_argument() {
    let mut gdb = gdb_from_source("int main (int argc, const char* const * argv) {}");
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let argv = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("argv".to_owned(), 0)])
        .unwrap();
    let argv0 = state_graph
        .get_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("argv".to_owned(), 0),
            EdgeLabel::Deref,
        ])
        .unwrap();
    let argv00 = state_graph
        .get_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("argv".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Deref,
        ])
        .unwrap();
    assert_eq!(argv.node_type_class(), NodeTypeClass::Ref);
    assert_eq!(argv.node_type_id(), None);
    assert!(argv.value().is_some_and(|v| v != NodeValue::Uint(0)));
    assert_eq!(argv0.node_type_class(), NodeTypeClass::Ref);
    assert_eq!(argv0.node_type_id(), None);
    assert!(argv0.value().is_some_and(|v| v != NodeValue::Uint(0)));
    assert_eq!(argv00.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(argv00.node_type_id(), Some("char"));
}

#[test]
fn pointer_copying() {
    let mut gdb = gdb_from_source(
        r"
        int main (int argc, const char* const * argv) {
            const char* p = argv[0];
            /* breakpoint */;
        }",
    );
    gdb.run_to_line(4).unwrap();
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let argv00_id = state_graph
        .get_id_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("argv".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Deref,
        ])
        .unwrap();
    let deref_p_id = state_graph
        .get_id_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("p".to_owned(), 0),
            EdgeLabel::Deref,
        ])
        .unwrap();
    // Both dereferences should point to the same node
    assert_eq!(argv00_id, deref_p_id);
}

#[test]
fn pointer_update() {
    let mut gdb = gdb_from_source(
        r"
        int main (int argc, const char* const * argv) {
            const char* p = argv[0], * q = p;
            /* breakpoint 1 */;
            ++q;
            /* breakpoint 2 */;
        }",
    );
    gdb.run_to_line(4).unwrap();
    let mut state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    gdb.run_to_line(6).unwrap();
    state_graph.update(&mut gdb).expect_ready().unwrap();
    let p = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("p".to_owned(), 0)])
        .unwrap();
    let q = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("q".to_owned(), 0)])
        .unwrap();
    // The pointers should be offset by one byte
    match (p.value(), q.value()) {
        (Some(NodeValue::Uint(p)), Some(NodeValue::Uint(q))) => assert_eq!(q, p + 1),
        _ => panic!("Pointers have unexpected values"),
    }
    // The targeted nodes should be different
    let deref_p = p.get_successor(&EdgeLabel::Deref);
    let deref_q = q.get_successor(&EdgeLabel::Deref);
    assert!(deref_p.is_some());
    assert!(deref_q.is_some());
    assert_ne!(deref_p, deref_q);
}

#[test]
fn pointer_to_local() {
    let mut gdb = gdb_from_source(
        r"
        int main(void) {
            int a = 0;
            int* p = &a;
            int* q;
            int z = 0;
            // This pointer points to a variable that comes, both alphabetically
            // and in declaration order, after the pointer,
            // so the backend might encounter it first as well
            q = &z;
            /* breakpoint */;
        }",
    );
    gdb.run_to_line(11).unwrap();
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let a_id = state_graph
        .get_id_at_root(&[EdgeLabel::Main, EdgeLabel::Named("a".to_owned(), 0)])
        .unwrap();
    let z_id = state_graph
        .get_id_at_root(&[EdgeLabel::Main, EdgeLabel::Named("z".to_owned(), 0)])
        .unwrap();
    let p_deref_id = state_graph
        .get_id_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("p".to_owned(), 0),
            EdgeLabel::Deref,
        ])
        .unwrap();
    let q_deref_id = state_graph
        .get_id_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("q".to_owned(), 0),
            EdgeLabel::Deref,
        ])
        .unwrap();
    assert_eq!(a_id, p_deref_id);
    assert_eq!(z_id, q_deref_id);
}

#[test]
fn dangling_reference_detachment() {
    let mut gdb = gdb_from_source(
        r"
        int main(void) {
            int* p;
            if (1) {
                int a = 42;
                p = &a;
                /* breakpoint 1 */ a = 0;
            }
            /* breakpoint 2 */;
        }",
    );
    gdb.run_to_line(7).unwrap();
    let mut state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    gdb.run_to_line(9).unwrap();
    state_graph.update(&mut gdb).expect_ready().unwrap();
    let pointer = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("p".to_owned(), 0)])
        .unwrap();
    assert!(pointer.get_successor(&EdgeLabel::Deref).is_none());
}

#[test]
fn linked_list() {
    let mut gdb = gdb_from_source(
        r"
        #include<stdlib.h>

        struct node {
            struct node* next;
            int value;
        };

        int main(void) {
            struct node* head = (struct node*)malloc(sizeof(*head));
            head->value = 41;
            head->next = (struct node*)malloc(sizeof(*head));
            head->next->value = 42;
            head->next->next = (struct node*)malloc(sizeof(*head));
            head->next->next->value = 43;
            head->next->next->next = NULL;
            /* breakpoint */;
        }",
    );
    gdb.run_to_line(17).unwrap();
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let first = state_graph
        .get_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("head".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Named("value".to_owned(), 0),
        ])
        .unwrap();
    let second = state_graph
        .get_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("head".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Named("next".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Named("value".to_owned(), 0),
        ])
        .unwrap();
    let third = state_graph
        .get_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("head".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Named("next".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Named("next".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Named("value".to_owned(), 0),
        ])
        .unwrap();
    assert_eq!(first.value(), Some(NodeValue::Int(41)));
    assert_eq!(second.value(), Some(NodeValue::Int(42)));
    assert_eq!(third.value(), Some(NodeValue::Int(43)));
}

#[test]
fn reusing_deallocated_memory() {
    let mut gdb = gdb_from_source(
        r"
        #include <stdlib.h>

        int main(void) {
            int* p = (int*)malloc(sizeof(int));
            /* breakpoint 1 */;
            free(p);
            p = NULL;
            /* breakpoint 2 */;
            // This malloc might reuse the same address as the previous
            // The update at the last breakpoint should not fail because of this
            p = (int*)malloc(sizeof(int));
            /* breakpoint 3 */;
        }",
    );
    gdb.run_to_line(6).unwrap();
    let mut state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    gdb.run_to_line(9).unwrap();
    state_graph.update(&mut gdb).expect_ready().unwrap();
    gdb.run_to_line(13).unwrap();
    state_graph.update(&mut gdb).expect_ready().unwrap();
}

#[test]
fn constant_length_hint() {
    let hints = CascadeStyle::from(Stylesheet(vec![StyleRule {
        selector: Selector::from_path(
            [
                SelectorSegment::Match(EdgeLabel::Main.into()),
                SelectorSegment::Match(EdgeMatcher::Named("argv".to_owned())),
            ]
            .into(),
        ),
        properties: vec![StyleClause {
            key: StyleKey::Property(PointerLengthHintKey::Length),
            value: Expression::Int(3),
        }],
    }]));
    let mut gdb = gdb_from_source("int main(int argc, char** argv) {}");
    let state_graph = GdbStateGraph::new_with_hints(&mut gdb, &hints)
        .expect_ready()
        .unwrap();
    let argv_length = state_graph
        .get_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("argv".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Length,
        ])
        .unwrap();
    let argv_0 = state_graph.get_at_root(&[
        EdgeLabel::Main,
        EdgeLabel::Named("argv".to_owned(), 0),
        EdgeLabel::Deref,
        EdgeLabel::Index(0),
    ]);
    let argv_1 = state_graph.get_at_root(&[
        EdgeLabel::Main,
        EdgeLabel::Named("argv".to_owned(), 0),
        EdgeLabel::Deref,
        EdgeLabel::Index(1),
    ]);
    let argv_2 = state_graph.get_at_root(&[
        EdgeLabel::Main,
        EdgeLabel::Named("argv".to_owned(), 0),
        EdgeLabel::Deref,
        EdgeLabel::Index(2),
    ]);
    let argv_3 = state_graph.get_at_root(&[
        EdgeLabel::Main,
        EdgeLabel::Named("argv".to_owned(), 0),
        EdgeLabel::Deref,
        EdgeLabel::Index(3),
    ]);
    assert_eq!(argv_length.value(), Some(NodeValue::Uint(3)));
    assert!(argv_0.is_some());
    assert!(argv_1.is_some());
    assert!(argv_2.is_some());
    assert!(argv_3.is_none());
}

#[test]
fn length_hint_from_variable() {
    let hints = CascadeStyle::from(Stylesheet(vec![StyleRule {
        selector: Selector::from_path(
            [
                SelectorSegment::Match(EdgeLabel::Main.into()),
                SelectorSegment::Match(EdgeMatcher::Named("argv".to_owned())),
            ]
            .into(),
        ),
        properties: vec![
            StyleClause {
                key: StyleKey::Variable("--len".to_owned()),
                value: Expression::Int(3),
            },
            StyleClause {
                key: StyleKey::Property(PointerLengthHintKey::Length),
                value: Expression::Variable("--len".to_owned()),
            },
        ],
    }]));
    let mut gdb = gdb_from_source("int main(int argc, char** argv) {}");
    let state_graph = GdbStateGraph::new_with_hints(&mut gdb, &hints)
        .expect_ready()
        .unwrap();
    let argv_length = state_graph
        .get_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("argv".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Length,
        ])
        .unwrap();
    let argv_0 = state_graph.get_at_root(&[
        EdgeLabel::Main,
        EdgeLabel::Named("argv".to_owned(), 0),
        EdgeLabel::Deref,
        EdgeLabel::Index(0),
    ]);
    let argv_1 = state_graph.get_at_root(&[
        EdgeLabel::Main,
        EdgeLabel::Named("argv".to_owned(), 0),
        EdgeLabel::Deref,
        EdgeLabel::Index(1),
    ]);
    let argv_2 = state_graph.get_at_root(&[
        EdgeLabel::Main,
        EdgeLabel::Named("argv".to_owned(), 0),
        EdgeLabel::Deref,
        EdgeLabel::Index(2),
    ]);
    let argv_3 = state_graph.get_at_root(&[
        EdgeLabel::Main,
        EdgeLabel::Named("argv".to_owned(), 0),
        EdgeLabel::Deref,
        EdgeLabel::Index(3),
    ]);
    assert_eq!(argv_length.value(), Some(NodeValue::Uint(3)));
    assert!(argv_0.is_some());
    assert!(argv_1.is_some());
    assert!(argv_2.is_some());
    assert!(argv_3.is_none());
}

#[test]
fn variable_length_hint() {
    let hints = CascadeStyle::from(Stylesheet(vec![
        StyleRule {
            selector: Selector::from_path([SelectorSegment::Match(EdgeLabel::Main.into())].into()),
            properties: vec![StyleClause {
                key: StyleKey::Variable("--argc".to_owned()),
                value: Expression::Select(
                    LimitedSelector::from_path([EdgeLabel::Named("argc".to_owned(), 0).into()])
                        .into(),
                ),
            }],
        },
        StyleRule {
            selector: Selector::from_path(
                [
                    SelectorSegment::Match(EdgeLabel::Main.into()),
                    SelectorSegment::Match(EdgeMatcher::Named("argv".to_owned())),
                ]
                .into(),
            ),
            properties: vec![StyleClause {
                key: StyleKey::Property(PointerLengthHintKey::Length),
                value: Expression::Variable("--argc".to_owned()),
            }],
        },
    ]));
    let mut gdb = gdb_from_source("int main(int argc, char** argv) {}");
    let state_graph = GdbStateGraph::new_with_hints(&mut gdb, &hints)
        .expect_ready()
        .unwrap();
    let argc = state_graph
        .get_at_root(&[EdgeLabel::Main, EdgeLabel::Named("argc".to_owned(), 0)])
        .unwrap();
    let argv_length = state_graph
        .get_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("argv".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Length,
        ])
        .unwrap();
    assert_eq!(argc.value(), argv_length.value());
}

#[test]
fn resize_array_with_length_hint() {
    let hints = CascadeStyle::from(Stylesheet(vec![
        StyleRule {
            selector: Selector::from_path([SelectorSegment::Match(EdgeLabel::Main.into())].into()),
            properties: vec![StyleClause {
                key: StyleKey::Variable("--len".to_owned()),
                value: Expression::Select(
                    LimitedSelector::from_path([EdgeLabel::Named("len".to_owned(), 0).into()])
                        .into(),
                ),
            }],
        },
        StyleRule {
            selector: Selector::from_path(
                [
                    SelectorSegment::Match(EdgeLabel::Main.into()),
                    SelectorSegment::Match(EdgeMatcher::Named("p".to_owned())),
                ]
                .into(),
            ),
            properties: vec![StyleClause {
                key: StyleKey::Property(PointerLengthHintKey::Length),
                value: Expression::Variable("--len".to_owned()),
            }],
        },
    ]));
    let mut gdb = gdb_from_source(
        r"
        #include <stdlib.h>

        int main() {
            int len = 2;
            int* p = malloc(len * sizeof(*p));
            /* breakpoint 1 */;
            len = 4;
            p = malloc(len * sizeof(*p));
            /* breakpoint 2 */;
        }",
    );
    gdb.run_to_line(7).unwrap();
    let mut state_graph = GdbStateGraph::new_with_hints(&mut gdb, &hints)
        .expect_ready()
        .unwrap();
    gdb.run_to_line(10).unwrap();
    state_graph
        .update_with_hints(&mut gdb, &hints)
        .expect_ready()
        .unwrap();
    let p_length = state_graph
        .get_at_root(&[
            EdgeLabel::Main,
            EdgeLabel::Named("p".to_owned(), 0),
            EdgeLabel::Deref,
            EdgeLabel::Length,
        ])
        .unwrap();
    let p_3 = state_graph.get_at_root(&[
        EdgeLabel::Main,
        EdgeLabel::Named("p".to_owned(), 0),
        EdgeLabel::Deref,
        EdgeLabel::Index(3),
    ]);
    assert_eq!(p_length.value(), Some(NodeValue::Uint(4)));
    assert!(p_3.is_some());
}
