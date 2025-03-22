//! Expression evaluation.

#[cfg(test)]
mod test;

use crate::{
    property::{PropertyValue, Selectable},
    stylesheet::{expression::*, selector::LimitedSelector},
};
use aili_model::state::*;
use std::collections::HashMap;

/// Context in which stylesheet expressions are evaluated.
///
/// Some types of expressions require additional context data
/// in order to properly evaluate.
pub struct EvaluationContext<'a, T: ProgramStateGraphRef> {
    /// Graph within which [`Select`](Expression::Select) expressions
    /// should be evaluated.
    graph: T,

    /// Node that should be the origin for [`Select`](Expression::Select) expressions.
    current_node: T::NodeId,

    /// Variable pool from which variable values should be invoked.
    variable_pool: Option<&'a HashMap<&'a str, PropertyValue<T::NodeId>>>,
}

impl<'a, T: ProgramStateGraphRef> EvaluationContext<'a, T> {
    /// Constructs an evaluation context for a graph,
    /// starting at the root node, with no variables.
    pub fn of_graph(graph: T) -> Self {
        Self {
            current_node: graph.clone().root(),
            graph,
            variable_pool: None,
        }
    }

    /// Sets the origin node of the evaluation context.
    pub fn at_node(self, current_node: T::NodeId) -> Self {
        Self {
            current_node,
            graph: self.graph,
            variable_pool: self.variable_pool,
        }
    }

    /// Sets the variable pool of the evaluation context.
    pub fn with_variables(
        self,
        variable_pool: Option<&'a HashMap<&'a str, PropertyValue<T::NodeId>>>,
    ) -> Self {
        Self {
            current_node: self.current_node,
            graph: self.graph,
            variable_pool,
        }
    }

    /// Evaluates an expression in the context.
    pub fn evaluate(&self, expression: &Expression) -> PropertyValue<T::NodeId> {
        use Expression::*;
        match expression {
            Unset => PropertyValue::Unset,
            Bool(b) => (*b).into(),
            Int(i) => (*i).into(),
            String(s) => PropertyValue::String(s.clone()),
            UnaryOperator(operator, operand) => {
                self.unary_operator(*operator, self.evaluate(operand))
            }
            BinaryOperator(left, operator, right) => {
                self.binary_operator(*operator, self.evaluate(left), self.evaluate(right))
            }
            Conditional(condition, if_true, if_false) => {
                if self.evaluate(condition).is_truthy() {
                    self.evaluate(if_true)
                } else {
                    self.evaluate(if_false)
                }
            }
            Variable(name) => self
                .variable_pool
                .and_then(|pool| pool.get(name.as_str()))
                .cloned()
                .unwrap_or_default(),
            Select(selector) => self
                .match_limited_selector(selector)
                .map(PropertyValue::Selection)
                .unwrap_or_default(),
        }
    }

    /// Evaluates a unary operator expression in the context.
    fn unary_operator(
        &self,
        operator: UnaryOperator,
        operand: PropertyValue<T::NodeId>,
    ) -> PropertyValue<T::NodeId> {
        use self::NodeValue::*;
        use PropertyValue::*;
        use UnaryOperator::*;
        match operator {
            Plus => match self.coerce_to_value(operand) {
                Unset => Unset,
                Value(Int(i)) => i.into(),
                Value(Uint(u)) => u.into(),
                Value(Bool(b)) => u64::from(b).into(),
                String(s) => String(s),
                Selection(_) => unreachable!(),
            },
            Minus => match self.coerce_to_value(operand) {
                Unset => Unset,
                Value(Int(i)) => i.checked_neg().map(Into::into).unwrap_or_default(),
                Value(Uint(u)) => i64::try_from(u)
                    .map(std::ops::Neg::neg)
                    .map(Into::into)
                    .unwrap_or_default(),
                Value(Bool(b)) => (-i64::from(b)).into(),
                String(_) => Unset,
                Selection(_) => unreachable!(),
            },
            Not => (!operand.is_truthy()).into(),
            NodeValue => self
                .coerce_to_node(operand)
                .and_then(ProgramStateNodeRef::value)
                .cloned()
                .map(Into::into)
                .unwrap_or_default(),
            NodeIsA(type_class) => self
                .coerce_to_node(operand)
                .map(ProgramStateNodeRef::node_type)
                .map(NodeTypeClass::from)
                .is_some_and(|cls| cls == type_class)
                .into(),
            NodeTypeName => self
                .coerce_to_node(operand)
                .map(ProgramStateNodeRef::node_type)
                .and_then(NodeType::type_name)
                .map(str::to_owned)
                .map(Into::into)
                .unwrap_or_default(),
            IsSet => (!matches!(operand, PropertyValue::Unset)).into(),
        }
    }

    /// Evaluates a binary operator expression in the context.
    fn binary_operator(
        &self,
        operator: BinaryOperator,
        left: PropertyValue<T::NodeId>,
        right: PropertyValue<T::NodeId>,
    ) -> PropertyValue<T::NodeId> {
        use BinaryOperator::*;
        // Resolve logical operators first,
        // they are the only one that do not require extracting values from selections
        match operator {
            And => return (left.is_truthy() && right.is_truthy()).into(),
            Or => return (left.is_truthy() || right.is_truthy()).into(),
            _ => {}
        }
        // For all other operators, extract values from selections
        let left = self.coerce_to_value(left);
        let right = self.coerce_to_value(right);
        match operator {
            Plus => {
                // If either argument is a string, this is string concatenation.
                if matches!(left, PropertyValue::String(_))
                    || matches!(right, PropertyValue::String(_))
                {
                    return format!("{left}{right}").into();
                }
                // Try to coerce to numeric values
                match (left, right).try_into() {
                    Ok(NumericPair::Int(left, right)) => {
                        left.checked_add(right).map(Into::into).unwrap_or_default()
                    }
                    Ok(NumericPair::Uint(left, right)) => {
                        left.checked_add(right).map(Into::into).unwrap_or_default()
                    }
                    Err(_) => PropertyValue::Unset,
                }
            }
            Minus => match (left, right).try_into() {
                Ok(NumericPair::Int(left, right)) => {
                    left.checked_sub(right).map(Into::into).unwrap_or_default()
                }
                Ok(NumericPair::Uint(left, right)) => {
                    if left < right {
                        right
                            .checked_sub(left)
                            .and_then(|x| i64::try_from(x).ok())
                            .map(std::ops::Neg::neg)
                            .map(Into::into)
                            .unwrap_or_default()
                    } else {
                        left.checked_sub(right).map(Into::into).unwrap_or_default()
                    }
                }
                Err(_) => PropertyValue::Unset,
            },
            Mul => match (left, right).try_into() {
                Ok(NumericPair::Int(left, right)) => {
                    left.checked_mul(right).map(Into::into).unwrap_or_default()
                }
                Ok(NumericPair::Uint(left, right)) => {
                    left.checked_mul(right).map(Into::into).unwrap_or_default()
                }
                Err(_) => PropertyValue::Unset,
            },
            Div => match (left, right).try_into() {
                Ok(NumericPair::Int(left, right)) => left
                    .checked_div_euclid(right)
                    .map(Into::into)
                    .unwrap_or_default(),
                Ok(NumericPair::Uint(left, right)) => left
                    .checked_div_euclid(right)
                    .map(Into::into)
                    .unwrap_or_default(),
                Err(_) => PropertyValue::Unset,
            },
            Mod => match (left, right).try_into() {
                Ok(NumericPair::Int(left, right)) => left
                    .checked_rem_euclid(right)
                    .map(Into::into)
                    .unwrap_or_default(),
                Ok(NumericPair::Uint(left, right)) => left
                    .checked_rem_euclid(right)
                    .map(Into::into)
                    .unwrap_or_default(),
                Err(_) => PropertyValue::Unset,
            },
            Eq => (left == right).into(),
            Ne => (left != right).into(),
            Lt => (left < right).into(),
            Le => (left <= right).into(),
            Gt => (left > right).into(),
            Ge => (left >= right).into(),
            And | Or => unreachable!("This operator should have been resolved early"),
        }
    }

    /// Evaluates a limited selector against the context.
    pub fn match_limited_selector(
        &self,
        selector: &LimitedSelector,
    ) -> Option<Selectable<T::NodeId>> {
        let mut current_node = self.current_node.clone();
        for segment in &selector.path {
            // Find the edge specified (unambiguously) by the segmens
            // and move to the node at its end
            current_node = self
                .graph
                .clone()
                .get(current_node.clone())
                .and_then(|node| node.get_successor(&segment.edge_label))?;
            // Fail if a condition is set and fails
            if !segment
                .condition
                .as_ref()
                .map(|c| self.evaluate(c).is_truthy())
                .unwrap_or(true)
            {
                return None;
            }
        }
        Some(Selectable::node(current_node).with_extra(selector.extra_label.clone()))
    }

    /// Shorthand for retrieving the node that a property value is referencing, if any
    fn coerce_to_node(&self, value: PropertyValue<T::NodeId>) -> Option<T::NodeRef> {
        match value {
            PropertyValue::Selection(target) => {
                if target.edge_label.is_some() || target.extra_label.is_some() {
                    None
                } else {
                    self.graph.clone().get(target.node_id)
                }
            }
            _ => None,
        }
    }

    /// If a property value is a selector, this converts it to value by accessing
    /// the node's value.
    fn coerce_to_value(&self, value: PropertyValue<T::NodeId>) -> PropertyValue<T::NodeId> {
        match value {
            PropertyValue::Selection(target) => {
                if target.edge_label.is_some() || target.extra_label.is_some() {
                    PropertyValue::Unset
                } else {
                    self.graph
                        .clone()
                        .get(target.node_id)
                        .and_then(ProgramStateNodeRef::value)
                        .cloned()
                        .map(Into::into)
                        .unwrap_or_default()
                }
            }
            _ => value,
        }
    }
}

/// Helper for binary arithmetic operators.
enum NumericPair {
    /// Two values coercible to signed integer.
    Int(i64, i64),
    /// Two values coercible to unsigned integer.
    Uint(u64, u64),
}

/// Helper for binary arithmetic operators.
enum NumericValue {
    /// Value coercible to signed integer.
    Int(i64),
    /// Value coercible to unsigned integer.
    Uint(u64),
}

impl<T: NodeId> TryFrom<PropertyValue<T>> for NumericValue {
    type Error = ();
    fn try_from(value: PropertyValue<T>) -> Result<Self, Self::Error> {
        match value {
            PropertyValue::Value(NodeValue::Int(i)) => Ok(Self::Int(i)),
            PropertyValue::Value(NodeValue::Uint(u)) => Ok(Self::Uint(u)),
            PropertyValue::Value(NodeValue::Bool(b)) => Ok(Self::Uint(b.into())),
            _ => Err(()),
        }
    }
}

impl<T: NodeId> TryFrom<(PropertyValue<T>, PropertyValue<T>)> for NumericPair {
    type Error = ();
    fn try_from(value: (PropertyValue<T>, PropertyValue<T>)) -> Result<Self, Self::Error> {
        use NumericValue::*;
        match (value.0.try_into()?, value.1.try_into()?) {
            (Int(a), Int(b)) => Ok(Self::Int(a, b)),
            (Uint(a), Int(b)) => Ok(Self::Int(a.try_into().map_err(|_| ())?, b)),
            (Int(a), Uint(b)) => Ok(Self::Int(a, b.try_into().map_err(|_| ())?)),
            (Uint(a), Uint(b)) => Ok(Self::Uint(a, b)),
        }
    }
}
