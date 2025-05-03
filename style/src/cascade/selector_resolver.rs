//! Helper for [`CascadeSelector`] resolution.

use super::style::{CascadeSelector, FlatSelectorSegment};
use crate::eval::{context::EvaluationContext, evaluate};
use aili_model::state::{EdgeLabel, NodeId, ProgramStateGraph};
use std::collections::{BTreeSet, HashSet};

/// Helper object for the resolution of stylesheets.
pub struct SelectorResolver<'a, T: NodeId> {
    /// The compiled selectors that are being resolved.
    selectors: &'a CascadeSelector,

    /// Pairs of nodes and selector sequence points
    /// that have already been matched.
    ///
    /// Each node can only be matched by each sequence point
    /// once. If it is matched again, the match fails.
    ///
    /// A sequence point is a [`FlatSelectorSegment::MatchNode`]
    /// transition in the state machine.
    matched_sequence_points: HashSet<(T, SelectorState)>,

    /// The resolution stack that tracks the current path to root.
    stack: Vec<ResolveFrame>,
}

impl<'a, T: NodeId> SelectorResolver<'a, T> {
    /// Constructs a new resolver that resolves a particular stylesheet.
    pub fn new(selectors: &'a CascadeSelector) -> Self {
        Self {
            selectors,
            matched_sequence_points: HashSet::new(),
            stack: vec![ResolveFrame {
                active_states: selectors.all_starting_states(),
            }],
        }
    }

    /// Notifies the resolver that an edge has been traversed.
    ///
    /// Advances all selectors that are awaiting an edge.
    pub fn push_edge(&mut self, edge_label: &EdgeLabel) {
        let active_states = self
            .stack
            .last()
            .expect("The bottommost stack frame should never be popped")
            .active_states
            .iter()
            .filter(|state| {
                self.selectors.0[state.rule_index]
                    .path
                    .get(state.instruction_index)
                    .is_some_and(|instruction| match instruction {
                        FlatSelectorSegment::MatchEdge(matcher) => matcher.matches(edge_label),
                        _ => false,
                    })
            })
            .copied()
            .map(SelectorState::advance)
            .collect();
        self.stack.push(ResolveFrame { active_states });
    }

    /// Pops the context of edge traversal.
    ///
    /// Reverts the resolver to the state before the last edge was traversed.
    pub fn pop_edge(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Resolves all selectors over a node.
    pub fn resolve_node(
        &mut self,
        node: T,
        eval_context: &EvaluationContext<impl ProgramStateGraph>,
    ) -> Vec<(usize, SelectionCaret)> {
        // States of the selector state machine that have been visited
        // while evaluating this node
        let mut visited_states = BTreeSet::new();
        // States that are yet to be visited and whether the node has already
        // been committed when we reach them
        let mut open_states = Vec::from_iter(
            self.stack
                .pop()
                .unwrap()
                .active_states
                .into_iter()
                .map(|s| (s, SelectionCaret::PrecedingEdge)),
        );
        // States that are blocked by an edge matcher
        // and must be resolved by traversing further down the graph
        let mut output_states = Vec::new();
        // Rules whose selector selected this element or a related entity
        let mut matched_rules = Vec::new();

        // Make a transitive closure of selector states reachable at this node
        while let Some((state, target)) = open_states.pop() {
            let selector = &self.selectors.0[state.rule_index].path;
            if state.instruction_index >= selector.len() {
                // We made it to the end of the selector
                // That means it has matched the node
                matched_rules.push((state.rule_index, target));
                continue;
            }
            // Proceed, unless we have been here already
            // This prevents infinite loops caused by poorly written selectors
            if !visited_states.insert(state) {
                continue;
            }
            match &selector[state.instruction_index] {
                FlatSelectorSegment::MatchEdge(_) => {
                    // Traversing an edge is only permitted if the node has already been committed
                    // This ensures the resolver halts by only allowing each edge to be traversed once
                    if target == SelectionCaret::Node {
                        // This is where we must halt and send the selector
                        // along the edge later on, after we are done with
                        // all partial matches on this node
                        output_states.push(state);
                    }
                    // TODO: Emit a warning if we fail this check?
                    // This can never happen when using flattened regular selectors
                    // but it is possible to manually construct a flat selector
                    // that does not uphold this invariant
                }
                FlatSelectorSegment::MatchNode => {
                    // Proceed only if the selector has never partially matched
                    // this node in this way
                    if self.matched_sequence_points.insert((node.clone(), state)) {
                        // Continue traversing the state machine linearly
                        // and commit to the node
                        open_states.push((state.advance(), SelectionCaret::Node));
                    }
                }
                FlatSelectorSegment::Restrict(condition) => {
                    // Proceed only if the condition holds
                    if evaluate(condition, eval_context).is_truthy() {
                        // continue traversing the state machine linearly
                        open_states.push((state.advance(), target));
                    }
                }
                FlatSelectorSegment::Branch(next_state) => {
                    // Continue both linearly and from the indicated state
                    open_states.push((state.jump(*next_state), target));
                    open_states.push((state.advance(), target));
                }
                FlatSelectorSegment::Jump(next_state) => {
                    // Continue only from the indicated state
                    open_states.push((state.jump(*next_state), target));
                }
            }
        }

        // Push back the frame that we popped earlier, with updates states
        self.stack.push(ResolveFrame {
            active_states: output_states,
        });
        matched_rules
    }

    /// Checks whether there are any seoectors that are awaiting an outgoing edge.
    ///
    /// If this returns false, calling [`SelectorResolver::push_edge`]
    /// or [`SelectorResolver::resolve_node`] will yield no new results.
    pub fn has_edges_to_resolve(&self) -> bool {
        !self.stack.last().unwrap().active_states.is_empty()
    }
}

impl CascadeSelector {
    /// Retrieves the list of all starting states of all selectors
    /// in a stalesheet.
    fn all_starting_states(&self) -> Vec<SelectorState> {
        (0..self.0.len())
            .map(|i| SelectorState {
                rule_index: i,
                instruction_index: 0,
            })
            .collect()
    }
}

/// Indicates what kind of entity a selector has last passed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SelectionCaret {
    /// The selector has passed a node, and has either selected it,
    /// or is awaiting an edge.
    Node,
    /// The selector has passed an edge, and is awaiting a node.
    PrecedingEdge,
}

/// Unique identifier of an instruction in a selector.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct SelectorState {
    /// Index of the selector.
    rule_index: usize,
    /// Index of the instruction within the selector.
    instruction_index: usize,
}

impl SelectorState {
    /// Constructs a new state id that targets the following instruction.
    fn advance(self) -> Self {
        self.jump(self.instruction_index + 1)
    }

    /// Constructs a new state id that targets the same rule,
    /// with a different instruction index.
    fn jump(self, next_instruction: usize) -> Self {
        Self {
            rule_index: self.rule_index,
            instruction_index: next_instruction,
        }
    }
}

/// Context frame of [`SelectorResolver`].
struct ResolveFrame {
    /// All states in all selectors where their state machines
    /// currently are.
    active_states: Vec<SelectorState>,
}
