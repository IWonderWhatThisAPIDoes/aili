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
