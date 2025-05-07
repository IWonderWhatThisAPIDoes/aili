//! Expression evaluation.

pub mod context;
mod evaluator;
pub mod variable_pool;

use crate::{stylesheet::expression::Expression, values::PropertyValue};
use aili_model::state::ProgramStateGraph;
use context::EvaluationContext;
use evaluator::Evaluator;

/// Evaluates an expression in a provided context.
pub fn evaluate<T: ProgramStateGraph>(
    expression: &Expression,
    context: &EvaluationContext<T>,
) -> PropertyValue<T::NodeId> {
    Evaluator(context).evaluate(expression)
}

/// If a [`PropertyValue`] is a [`PropertyValue::Selection`],
/// evaluates the node and returns its value.
///
/// Otherwise, the value is returned as is.
pub fn unwrap_node_value<T: ProgramStateGraph>(
    value: PropertyValue<T::NodeId>,
    context: &EvaluationContext<T>,
) -> PropertyValue<T::NodeId> {
    Evaluator(context).coerce_to_value(value)
}
