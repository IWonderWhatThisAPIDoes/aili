mod utils;

use aili_gdbstate::state::GdbStateGraph;
use aili_model::state::*;
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
fn global_variable() {
    let mut gdb = gdb_from_source("int global = 42; int main(void) {}");
    let state_graph = GdbStateGraph::new(&mut gdb).expect_ready().unwrap();
    let global = state_graph
        .get_at_root(&[EdgeLabel::Named("global".to_owned(), 0)])
        .unwrap();
    assert_eq!(global.node_type_class(), NodeTypeClass::Atom);
    assert_eq!(global.node_type_id(), Some("int"));
    assert_eq!(global.value(), Some(NodeValue::Int(42)));
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
