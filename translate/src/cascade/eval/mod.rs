//! Expression evaluation.

pub mod context;
mod evaluator;
#[cfg(test)]
mod test;

use crate::{property::PropertyValue, stylesheet::expression::Expression};
use context::EvaluationContext;
use evaluator::Evaluator;

/// Evaluates an expression in a provided context.
pub fn evaluate<T: EvaluationContext>(
    expression: &Expression,
    context: &T,
) -> PropertyValue<T::NodeId> {
    Evaluator(context).evaluate(expression)
}
