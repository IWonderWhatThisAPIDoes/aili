//! Container for storing interpreter variables.

use crate::values::PropertyValue;
use aili_model::state::NodeId;
use std::collections::HashMap;

/// Container that stores variables for the interpreter in a layered stack structure.
pub struct VariablePool<K, T>(Vec<HashMap<K, PropertyValue<T>>>)
where
    K: std::hash::Hash + Eq,
    T: NodeId;

impl<K, T> VariablePool<K, T>
where
    K: std::hash::Hash + Eq,
    T: NodeId,
{
    /// Construct a new variable pool with one (permanent) frame.
    pub fn new() -> Self {
        Self(vec![HashMap::new()])
    }

    /// Pushes a variable pool frame.
    ///
    /// All variables assigned with [`VariablePool::insert`] belong
    /// to the new frame and will be discarded by a matching call
    /// to [`VariablePool::pop`].
    pub fn push(&mut self) {
        self.0.push(HashMap::new());
    }

    /// Pops a variable pool frame.
    ///
    /// All variables that have been assigned after
    /// the matching call to [`VariablePool::push`] are discarded.
    /// If they had values before then, their old values are reinstated.
    ///
    /// If there are no frames except the bottom, this operation does nothing.
    pub fn pop(&mut self) {
        if self.0.len() > 1 {
            self.0.pop();
        }
    }

    /// Accesses a variable value by its key.
    ///
    /// The most recent value assigned to the variable is returned,
    /// sans values that have been discarded by a call to [`VariablePool::pop`].
    pub fn get<Q>(&self, key: &Q) -> Option<&PropertyValue<T>>
    where
        Q: std::hash::Hash + Eq + ?Sized,
        K: std::borrow::Borrow<Q>,
    {
        self.0
            .iter()
            .rev()
            .filter_map(|frame| frame.get(key))
            .next()
    }

    /// Assigns a value to a variable by its key.
    ///
    /// The value will be discarded on the next call to [`VariablePool::pop`].
    /// If the variable already had a value, the old value will be reinstated.
    pub fn insert(&mut self, variable_name: K, value: PropertyValue<T>) {
        self.0
            .last_mut()
            .expect("The bottom frame of variable pool should never be popped")
            .insert(variable_name, value);
    }
}

impl<K, T> Default for VariablePool<K, T>
where
    K: std::hash::Hash + Eq,
    T: NodeId,
{
    fn default() -> Self {
        Self::new()
    }
}
