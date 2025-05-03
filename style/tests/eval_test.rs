//! Unit tests for expression evaluation.

mod test_graph;

use aili_model::state::RootedProgramStateGraph as _;
use aili_style::{
    eval::{context::EvaluationContext, evaluate},
    stylesheet::expression::{
        BinaryOperator as BinaryOp,
        Expression::{self, *},
        UnaryOperator as UnaryOp,
    },
    values::PropertyValue,
};
use test_graph::TestGraph;

/// Evaluate an expression at the root node of the [`TestGraph::default_graph`].
fn eval_on_default_graph(expression: &Expression) -> PropertyValue<usize> {
    let graph = TestGraph::default_graph();
    let context = EvaluationContext::from_graph(&graph, graph.root());
    evaluate(expression, &context)
}

#[test]
fn unary_plus_returns_string_unchanged() {
    let expr = UnaryOperator(UnaryOp::Plus, String("hello".to_owned()).into());
    assert_eq!(eval_on_default_graph(&expr), "hello".to_owned().into());
}

#[test]
fn unary_plus_returns_int_unchanged() {
    let expr = UnaryOperator(UnaryOp::Plus, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), 42u64.into());
}

#[test]
fn unary_plus_converts_true_to_one() {
    let expr = UnaryOperator(UnaryOp::Plus, Bool(true).into());
    assert_eq!(eval_on_default_graph(&expr), 1u64.into());
}

#[test]
fn unary_plus_converts_false_to_zero() {
    let expr = UnaryOperator(UnaryOp::Plus, Bool(false).into());
    assert_eq!(eval_on_default_graph(&expr), 0u64.into());
}

#[test]
fn unary_plus_returns_unset_unchanged() {
    let expr = UnaryOperator(UnaryOp::Plus, Unset.into());
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn unary_plus_extracts_value_from_node() {
    let expr = UnaryOperator(
        UnaryOp::Plus,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(
        eval_on_default_graph(&expr),
        TestGraph::NUMERIC_NODE_VALUE.into()
    );
}

#[test]
fn unary_plus_unwraps_valueless_node() {
    let expr = UnaryOperator(
        UnaryOp::Plus,
        Select(TestGraph::valueless_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn unary_plus_on_missing_node_returns_unset() {
    let expr = UnaryOperator(
        UnaryOp::Plus,
        Select(TestGraph::missing_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn unary_minus_on_string_returns_unset() {
    let expr = UnaryOperator(UnaryOp::Minus, String("hello".to_owned()).into());
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn unary_minus_negates_int() {
    let expr = UnaryOperator(UnaryOp::Minus, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), (-42i64).into());
}

#[test]
fn unary_minus_returns_unset_on_overflow() {
    let expr = UnaryOperator(UnaryOp::Minus, Int(u64::MAX).into());
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn unary_minus_extracts_value_from_node() {
    let expr = UnaryOperator(
        UnaryOp::Minus,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(
        eval_on_default_graph(&expr),
        (-(TestGraph::NUMERIC_NODE_VALUE as i64)).into()
    );
}

#[test]
fn not_nonzero_is_false() {
    let expr = UnaryOperator(UnaryOp::Not, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn not_zero_is_true() {
    let expr = UnaryOperator(UnaryOp::Not, Int(0).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn not_true_is_false() {
    let expr = UnaryOperator(UnaryOp::Not, Bool(true).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn not_unset_is_true() {
    let expr = UnaryOperator(UnaryOp::Not, Unset.into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn not_nonempty_string_is_false() {
    let expr = UnaryOperator(UnaryOp::Not, String("hello".to_owned()).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn not_empty_string_is_true() {
    let expr = UnaryOperator(UnaryOp::Not, String("".to_owned()).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn not_numeric_node_is_false() {
    let expr = UnaryOperator(
        UnaryOp::Not,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn not_valueless_node_is_false() {
    let expr = UnaryOperator(
        UnaryOp::Not,
        Select(TestGraph::valueless_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn not_missing_node_is_true() {
    let expr = UnaryOperator(
        UnaryOp::Not,
        Select(TestGraph::missing_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn value_extracts_value_from_node() {
    let expr = UnaryOperator(
        UnaryOp::NodeValue,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(
        eval_on_default_graph(&expr),
        TestGraph::NUMERIC_NODE_VALUE.into()
    );
}

#[test]
fn value_of_valueless_node_is_unset() {
    let expr = UnaryOperator(
        UnaryOp::NodeValue,
        Select(TestGraph::valueless_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn value_of_missing_node_is_unset() {
    let expr = UnaryOperator(
        UnaryOp::NodeValue,
        Select(TestGraph::missing_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn value_of_int_is_unset() {
    let expr = UnaryOperator(UnaryOp::NodeValue, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn isset_unset_is_false() {
    let expr = UnaryOperator(UnaryOp::IsSet, Unset.into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn isset_false_is_true() {
    let expr = UnaryOperator(UnaryOp::IsSet, Bool(false).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn isset_numeric_node_is_true() {
    let expr = UnaryOperator(
        UnaryOp::IsSet,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn isset_valueless_node_is_true() {
    let expr = UnaryOperator(
        UnaryOp::IsSet,
        Select(TestGraph::valueless_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn isset_missing_node_is_false() {
    let expr = UnaryOperator(
        UnaryOp::IsSet,
        Select(TestGraph::missing_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn binary_plus_concatenates_strings() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Plus,
        String("world".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), "helloworld".to_owned().into());
}

#[test]
fn string_plus_unset_returns_string_unchanged() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Plus,
        Unset.into(),
    );
    assert_eq!(eval_on_default_graph(&expr), "hello".to_owned().into());
}

#[test]
fn string_plus_number_serializes_number() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Plus,
        Int(42).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), "hello42".to_owned().into());
}

#[test]
fn bool_plus_string_serializes_bool() {
    let expr = BinaryOperator(
        Bool(true).into(),
        BinaryOp::Plus,
        String("hello".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), "truehello".to_owned().into());
}

#[test]
fn bool_plus_int_coerces_bool_to_int() {
    let expr = BinaryOperator(Bool(true).into(), BinaryOp::Plus, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), 43u64.into());
}

#[test]
fn unset_plus_int_is_unset() {
    let expr = BinaryOperator(Unset.into(), BinaryOp::Plus, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn bool_minus_uint_coerces_bool_to_int() {
    let expr = BinaryOperator(Bool(true).into(), BinaryOp::Minus, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), (-41i64).into());
}

#[test]
fn string_minus_int_is_unset() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Minus,
        Int(42).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn int_minus_unset_is_unset() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Minus, Unset.into());
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn node_minus_int_extracts_value_from_node() {
    let expr = BinaryOperator(
        Select(TestGraph::numeric_node_selector().into()).into(),
        BinaryOp::Minus,
        Int(0).into(),
    );
    assert_eq!(
        eval_on_default_graph(&expr),
        TestGraph::NUMERIC_NODE_VALUE.into()
    );
}

#[test]
fn uint_times_uint_multiplies_values() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Mul, Int(17).into());
    assert_eq!(eval_on_default_graph(&expr), 714u64.into());
}

#[test]
fn int_times_int_multiplies_values() {
    let expr = BinaryOperator(
        UnaryOperator(UnaryOp::Minus, Int(42).into()).into(),
        BinaryOp::Mul,
        UnaryOperator(UnaryOp::Minus, Int(17).into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), 714i64.into());
}

#[test]
fn int_times_bool_coerces_bool_to_int() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Mul, Bool(true).into());
    assert_eq!(eval_on_default_graph(&expr), 42u64.into());
}

#[test]
fn uint_times_uint_returns_unset_on_overflow() {
    let expr = BinaryOperator(Int(u64::MAX).into(), BinaryOp::Mul, Int(2).into());
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn int_over_int_divides_values() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Div, Int(5).into());
    assert_eq!(eval_on_default_graph(&expr), 8u64.into());
}

#[test]
fn negative_dividend_is_divided_the_euclidean_way() {
    let expr = BinaryOperator(
        UnaryOperator(UnaryOp::Minus, Int(42).into()).into(),
        BinaryOp::Div,
        Int(5).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), (-9i64).into());
}

#[test]
fn zero_division_is_unset() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Div, Int(0).into());
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn modulo_calculates_remainder() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Mod, Int(5).into());
    assert_eq!(eval_on_default_graph(&expr), 2u64.into());
}

#[test]
fn modulo_with_negative_dividend_is_calculated_the_euclidean_way() {
    let expr = BinaryOperator(
        UnaryOperator(UnaryOp::Minus, Int(42).into()).into(),
        BinaryOp::Mod,
        Int(5).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), (3i64).into());
}

#[test]
fn zero_modulo_is_unset() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Mod, Int(0).into());
    assert_eq!(eval_on_default_graph(&expr), PropertyValue::Unset);
}

#[test]
fn unset_equals_unset() {
    let expr = BinaryOperator(Unset.into(), BinaryOp::Eq, Unset.into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn one_equals_true() {
    let expr = BinaryOperator(Int(1).into(), BinaryOp::Eq, Bool(true).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn string_equals_same_string() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Eq,
        String("hello".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn string_does_not_equal_different_string() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Eq,
        String("world".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn string_does_not_equal_int() {
    let expr = BinaryOperator(String("42".to_owned()).into(), BinaryOp::Eq, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn int_equals_node_with_same_value() {
    let expr = BinaryOperator(
        Int(TestGraph::NUMERIC_NODE_VALUE).into(),
        BinaryOp::Eq,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn empty_node_equals_unset() {
    let expr = BinaryOperator(
        Select(TestGraph::valueless_node_selector().into()).into(),
        BinaryOp::Eq,
        Unset.into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn empty_node_equals_missing_node() {
    let expr = BinaryOperator(
        Select(TestGraph::valueless_node_selector().into()).into(),
        BinaryOp::Eq,
        Select(TestGraph::missing_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn empty_node_does_not_equal_numeric_node() {
    let expr = BinaryOperator(
        Select(TestGraph::valueless_node_selector().into()).into(),
        BinaryOp::Eq,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn unset_is_not_different_from_unset() {
    let expr = BinaryOperator(Unset.into(), BinaryOp::Ne, Unset.into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn string_is_not_different_from_same_string() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Ne,
        String("hello".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn string_is_different_from_different_string() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Ne,
        String("world".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn int_is_different_from_different_int() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Ne, Int(12).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn empty_node_is_not_different_from_numeric_node_with_same_value() {
    let expr = BinaryOperator(
        Select(TestGraph::valueless_node_selector().into()).into(),
        BinaryOp::Ne,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn small_number_is_less_than_large_number() {
    let expr = BinaryOperator(Bool(true).into(), BinaryOp::Lt, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn number_is_not_less_than_itself() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Lt, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn string_is_not_less_than_itself() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Lt,
        String("hello".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn string_is_not_less_than_different_string() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Lt,
        String("world".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn unset_is_not_less_than_number() {
    let expr = BinaryOperator(Unset.into(), BinaryOp::Lt, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn boolean_is_less_than_numeric_node_with_larger_value() {
    let expr = BinaryOperator(
        Bool(true).into(),
        BinaryOp::Lt,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn small_number_is_less_or_equal_to_large_number() {
    let expr = BinaryOperator(Bool(true).into(), BinaryOp::Le, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn number_is_less_or_equal_to_itself() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Le, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn string_is_less_or_equal_to_itself() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Le,
        String("hello".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn string_is_not_less_or_equal_to_different_string() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Le,
        String("world".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn boolean_is_less_or_equal_to_numeric_node_with_larger_value() {
    let expr = BinaryOperator(
        Bool(true).into(),
        BinaryOp::Le,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn small_number_is_not_greater_than_large_number() {
    let expr = BinaryOperator(Bool(true).into(), BinaryOp::Gt, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn number_is_not_greater_than_itself() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Gt, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn string_is_not_greater_than_itself() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Gt,
        String("hello".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn string_is_not_greater_than_different_string() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Gt,
        String("world".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn boolean_is_not_greater_than_numeric_node_with_larger_value() {
    let expr = BinaryOperator(
        Bool(true).into(),
        BinaryOp::Gt,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn small_number_is_not_greater_or_equal_to_large_number() {
    let expr = BinaryOperator(Bool(true).into(), BinaryOp::Gt, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn number_is_greater_or_equal_to_itself() {
    let expr = BinaryOperator(Int(42).into(), BinaryOp::Ge, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn string_is_greater_or_equal_to_itself() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Ge,
        String("hello".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn string_is_not_greater_or_equal_to_different_string() {
    let expr = BinaryOperator(
        String("hello".to_owned()).into(),
        BinaryOp::Ge,
        String("world".to_owned()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn boolean_is_not_greater_or_equal_to_numeric_node_with_larger_value() {
    let expr = BinaryOperator(
        Bool(true).into(),
        BinaryOp::Ge,
        Select(TestGraph::numeric_node_selector().into()).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn true_and_nonzero_is_true() {
    let expr = BinaryOperator(Bool(true).into(), BinaryOp::And, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn node_and_unset_is_false() {
    let expr = BinaryOperator(
        Select(TestGraph::valueless_node_selector().into()).into(),
        BinaryOp::And,
        Unset.into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn missing_node_and_true_is_false() {
    let expr = BinaryOperator(
        Select(TestGraph::missing_node_selector().into()).into(),
        BinaryOp::And,
        Bool(true).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn empty_stirng_and_zero_is_false() {
    let expr = BinaryOperator(String("".to_owned()).into(), BinaryOp::And, Int(0).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn true_or_nonzero_is_true() {
    let expr = BinaryOperator(Bool(true).into(), BinaryOp::Or, Int(42).into());
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn node_or_unset_is_true() {
    let expr = BinaryOperator(
        Select(TestGraph::valueless_node_selector().into()).into(),
        BinaryOp::Or,
        Unset.into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn missing_node_or_true_is_true() {
    let expr = BinaryOperator(
        Select(TestGraph::missing_node_selector().into()).into(),
        BinaryOp::Or,
        Bool(true).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), true.into());
}

#[test]
fn empty_stirng_or_zero_is_false() {
    let expr = BinaryOperator(String("".to_owned()).into(), BinaryOp::Or, Int(0).into());
    assert_eq!(eval_on_default_graph(&expr), false.into());
}

#[test]
fn conditional_returns_second_argument_if_first_is_truthy() {
    let expr = Conditional(
        String("hello".to_owned()).into(),
        String("world".to_owned()).into(),
        Int(42).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), "world".to_owned().into());
}

#[test]
fn conditional_returns_third_argument_if_first_is_falsy() {
    let expr = Conditional(
        String("".to_owned()).into(),
        String("world".to_owned()).into(),
        Int(42).into(),
    );
    assert_eq!(eval_on_default_graph(&expr), 42u64.into());
}
