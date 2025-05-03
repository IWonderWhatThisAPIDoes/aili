//! Evaluation of an entire stylesheet.

use super::mapping_builder::PropertyMappingBuilder;
use crate::property::{EntityPropertyMapping, PropertyKey};
use aili_model::state::{EdgeLabel, ProgramStateNode, RootedProgramStateGraph};
use aili_style::{
    cascade::{
        selector_resolver::{SelectionCaret, SelectorResolver},
        style::CascadeStyle,
    },
    eval::{context::EvaluationContext, evaluate, variable_pool::VariablePool},
    selectable::Selectable,
    stylesheet::StyleKey,
};

/// Applies a stylesheet to a graph.
pub fn apply_stylesheet<T: RootedProgramStateGraph>(
    stylesheet: &CascadeStyle<PropertyKey>,
    graph: &T,
) -> EntityPropertyMapping<T::NodeId> {
    let mut helper = ApplyStylesheet::new(stylesheet, graph);
    helper.run();
    helper.result()
}

/// Helper for stylesheet applications.
struct ApplyStylesheet<'a, 'g, T: RootedProgramStateGraph> {
    /// The graph being traversed.
    graph: &'g T,

    /// The stylesheet being evaluated.
    stylesheet: &'a CascadeStyle<PropertyKey>,

    /// Resolver that tracks the stylesheet's selectors.
    resolver: SelectorResolver<'a, PropertyKey, T::NodeId>,

    /// Builder that constructs the resulting mapping.
    mapping: PropertyMappingBuilder<T::NodeId>,

    /// Variables that are active at the moment
    variable_pool: VariablePool<&'a str, T::NodeId>,
}

impl<'a, 'g, T: RootedProgramStateGraph> ApplyStylesheet<'a, 'g, T> {
    fn new(stylesheet: &'a CascadeStyle<PropertyKey>, graph: &'g T) -> Self {
        Self {
            graph,
            stylesheet,
            resolver: SelectorResolver::new(stylesheet),
            mapping: PropertyMappingBuilder::new(),
            variable_pool: VariablePool::new(),
        }
    }

    fn result(self) -> EntityPropertyMapping<T::NodeId> {
        self.mapping.build(self.graph)
    }

    fn run(&mut self) {
        self.run_from(self.graph.root(), None, None);
    }

    /// Traverses depth-first from a specified node and evaluates the selector.
    fn run_from(
        &mut self,
        node: T::NodeId,
        previous_node: Option<T::NodeId>,
        previous_edge: Option<&EdgeLabel>,
    ) {
        let matched_rules = self.resolve_node(node.clone(), previous_edge);

        self.mapping.push();

        self.resolve_matched_rules(&node, previous_node, previous_edge, matched_rules);

        // This is our termination condition:
        // We stop once there is nothing else to explore
        if self.resolver.has_edges_to_resolve() {
            // Traverse down the tree through all edges
            self.traverse_outgoing_edges(node);
        }

        self.mapping.pop();
    }

    fn resolve_matched_rules(
        &mut self,
        node: &T::NodeId,
        previous_node: Option<T::NodeId>,
        previous_edge: Option<&EdgeLabel>,
        mut matched_rules: Vec<(usize, SelectionCaret)>,
    ) {
        // Resolve rules in correct order
        matched_rules.sort_by_cached_key(|&(rule_index, caret)| {
            let has_extra = self.stylesheet.0[rule_index].machine.extra.is_some();
            // Primary ordering: incoming edge before node
            // Secondary ordering: nodes and edges before extras
            // Tertiary ordering: declaration order in the stylesheet
            (caret == SelectionCaret::Node, has_extra, rule_index)
        });

        // Resolve all entities that matched
        for (rule_index, caret) in matched_rules {
            let mut selected = if caret == SelectionCaret::Node {
                Selectable::node(node.clone())
            } else if let Some(selected) = previous_node.clone().and_then(|node| {
                previous_edge
                    .cloned()
                    .map(|edge| Selectable::edge(node, edge))
            }) {
                selected
            } else {
                continue;
            };
            selected.extra_label = self.stylesheet.0[rule_index].machine.extra.clone();
            self.selected_entity(&selected, node, rule_index, previous_edge);
        }
    }

    /// Runs segments of the state machine at a given node.
    fn resolve_node(
        &mut self,
        node: T::NodeId,
        previous_edge: Option<&EdgeLabel>,
    ) -> Vec<(usize, SelectionCaret)> {
        let context =
            Self::evaluation_context(self.graph, &self.variable_pool, node.clone(), previous_edge);
        self.resolver.resolve_node(node, &context)
    }

    /// Traverses depth-first through all outgoing edges of a node.
    fn traverse_outgoing_edges(&mut self, starting_node: T::NodeId) {
        let Some(node) = self.graph.get(&starting_node) else {
            return;
        };
        for (edge_label, successor_node) in node.successors() {
            // Push a state so we can pop it later
            self.variable_pool.push();
            self.resolver.push_edge(edge_label);
            // Resolve the following edge and node
            self.run_from(
                successor_node,
                Some(starting_node.clone()),
                Some(edge_label),
            );
            // Discard all variables that were created here
            self.resolver.pop_edge();
            self.variable_pool.pop();
        }
    }

    fn selected_entity(
        &mut self,
        target: &Selectable<T::NodeId>,
        select_origin: &T::NodeId,
        rule_index: usize,
        previous_edge: Option<&EdgeLabel>,
    ) {
        // Adjust the mapping to the new entity
        self.mapping
            .selected_entity(target, select_origin, rule_index);
        // Extra entities get their own variable scope
        // so they cannot affect anything outside
        if target.is_extra() {
            self.variable_pool.push();
        }
        let properties = &self.stylesheet.0[rule_index].properties;
        for property in properties {
            let context = Self::evaluation_context(
                self.graph,
                &self.variable_pool,
                select_origin.clone(),
                previous_edge,
            );
            let value = evaluate(&property.value, &context);
            match &property.key {
                StyleKey::Property(key) => {
                    self.mapping.assign(target, key, value, rule_index);
                }
                StyleKey::Variable(name) => {
                    self.variable_pool.insert(name, value);
                }
            }
        }
        if target.is_extra() {
            self.variable_pool.pop();
        }
    }

    fn evaluation_context<'b>(
        graph: &'b T,
        variable_pool: &'b VariablePool<&'b str, T::NodeId>,
        origin: T::NodeId,
        previous_edge: Option<&'b EdgeLabel>,
    ) -> EvaluationContext<'b, T> {
        let mut context =
            EvaluationContext::from_graph(graph, origin).with_variables(variable_pool);
        match previous_edge {
            Some(EdgeLabel::Index(index)) => context = context.with_edge_index(*index),
            Some(EdgeLabel::Named(name, discriminator)) => {
                context = context
                    .with_edge_name(name.as_str())
                    .with_edge_discriminator(*discriminator)
            }
            _ => {}
        }
        context
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        cascade::test_graph::TestGraph,
        property::{PropertyKey::*, *},
    };
    use aili_style::{
        selectable::Selectable,
        stylesheet::{StyleKey::*, expression::*, selector::*, *},
    };

    #[test]
    fn apply_stylesheet_with_one_rule() {
        // .many(*) "a" {
        //   value: 42;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [
                    SelectorSegment::anything_any_number_of_times(),
                    SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
                ]
                .into(),
            ),
            properties: vec![StyleClause {
                key: Property(Attribute("value".to_owned())),
                value: Expression::Int(42),
            }],
        }]));
        let expected_properties =
            PropertyMap::new().with_attribute("value".to_owned(), "42".to_owned());
        let expected_mapping = [
            (Selectable::node(5), expected_properties.clone()),
            (Selectable::node(6), expected_properties.clone()),
            (Selectable::node(7), expected_properties.clone()),
            (Selectable::node(10), expected_properties.clone()),
            (Selectable::node(11), expected_properties.clone()),
            (Selectable::node(12), expected_properties.clone()),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn apply_stylesheet_with_multiple_rules() {
        // .many(*) [] {
        //   value: "cell";
        // }
        // :: main .many(next) {
        //   value: "kvt";
        //   title: 42;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::anything_any_number_of_times(),
                        SelectorSegment::Match(EdgeMatcher::AnyIndex),
                    ]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".to_owned())),
                    value: Expression::String("cell".to_owned()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::Match(EdgeLabel::Main.into()),
                        SelectorSegment::AnyNumberOfTimes(
                            [SelectorSegment::Match(EdgeLabel::Next.into())].into(),
                        ),
                    ]
                    .into(),
                ),
                properties: vec![
                    StyleClause {
                        key: Property(Attribute("value".to_owned())),
                        value: Expression::String("kvt".to_owned()),
                    },
                    StyleClause {
                        key: Property(Attribute("title".to_owned())),
                        value: Expression::Int(42),
                    },
                ],
            },
        ]));
        let expected_properties_1 =
            PropertyMap::new().with_attribute("value".to_owned(), "cell".to_owned());
        let expected_properties_2 = PropertyMap::new()
            .with_attribute("value".to_owned(), "kvt".to_owned())
            .with_attribute("title".to_owned(), "42".to_owned());
        let expected_mapping = [
            (Selectable::node(1), expected_properties_2.clone()),
            (Selectable::node(2), expected_properties_2.clone()),
            (Selectable::node(3), expected_properties_2.clone()),
            (Selectable::node(4), expected_properties_2.clone()),
            (Selectable::node(8), expected_properties_1.clone()),
            (Selectable::node(12), expected_properties_1.clone()),
            (Selectable::node(13), expected_properties_1.clone()),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn select_extra_entity() {
        // :: main::extra {
        //   display: "cell";
        // }
        //
        // :: main next::extra(abc) {
        //   display: "kvt";
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into())].into(),
                )
                .with_extra("".to_owned()),
                properties: vec![StyleClause {
                    key: Property(Display),
                    value: Expression::String("cell".to_owned()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::Match(EdgeLabel::Main.into()),
                        SelectorSegment::Match(EdgeLabel::Next.into()),
                    ]
                    .into(),
                )
                .with_extra("abc".to_owned()),
                properties: vec![StyleClause {
                    key: Property(Display),
                    value: Expression::String("kvt".to_owned()),
                }],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(1).with_extra("".to_owned()),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    // Parent is assigned automatically
                    .with_parent(Selectable::node(1)),
            ),
            (
                Selectable::node(2).with_extra("abc".to_owned()),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("kvt".to_owned()))
                    // Parent is assigned automatically
                    .with_parent(Selectable::node(2)),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn select_edge() {
        // .many(*).if(@("a"#0))::edge { }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [
                    SelectorSegment::anything_any_number_of_times(),
                    SelectorSegment::Condition(Expression::Select(
                        LimitedSelector::from_path([EdgeLabel::Named("a".to_owned(), 0).into()])
                            .into(),
                    )),
                ]
                .into(),
            )
            .selecting_edge(),
            // These are edges, so all we need to do is select them,
            // properties do not need to be assigned
            properties: Vec::new(),
        }]));
        // Display, parent, and target are assigned automatically
        let expected_mapping = [
            (
                Selectable::edge(0, EdgeLabel::Main),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(0))
                    .with_target(Selectable::node(1)),
            ),
            (
                Selectable::edge(0, EdgeLabel::Named("a".to_owned(), 0)),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(0))
                    .with_target(Selectable::node(5)),
            ),
            (
                Selectable::edge(1, EdgeLabel::Named("a".to_owned(), 0)),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(1))
                    .with_target(Selectable::node(10)),
            ),
            (
                Selectable::edge(2, EdgeLabel::Next),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(2))
                    .with_target(Selectable::node(3)),
            ),
            (
                Selectable::edge(5, EdgeLabel::Named("a".to_owned(), 0)),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(5))
                    .with_target(Selectable::node(6)),
            ),
            (
                Selectable::edge(5, EdgeLabel::Deref),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(5))
                    .with_target(Selectable::node(10)),
            ),
            (
                Selectable::edge(7, EdgeLabel::Deref),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(7))
                    .with_target(Selectable::node(5)),
            ),
            (
                Selectable::edge(12, EdgeLabel::Deref),
                PropertyMap::new()
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(12))
                    .with_target(Selectable::node(10)),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn coerce_values() {
        // :: {
        //   display: true;
        //   target: @(main);
        // }
        //
        // :: "a" {
        //   value: @;
        //   display: @([0]);
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![
                    StyleClause {
                        key: Property(Display),
                        value: Expression::Bool(true),
                    },
                    StyleClause {
                        key: Property(Target),
                        value: Expression::Select(
                            LimitedSelector::from_path([EdgeLabel::Main.into()]).into(),
                        ),
                    },
                ],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeMatcher::Named("a".to_owned()))].into(),
                ),
                properties: vec![
                    StyleClause {
                        key: Property(Attribute("value".to_owned())),
                        value: Expression::Select(LimitedSelector::default().into()),
                    },
                    StyleClause {
                        key: Property(Display),
                        value: Expression::Select(
                            LimitedSelector::from_path([EdgeLabel::Index(0).into()]).into(),
                        ),
                    },
                ],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(0),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("true".to_owned()))
                    .with_target(Selectable::node(1)),
            ),
            (
                Selectable::node(5),
                PropertyMap::new()
                    // Parent is assigned automatically
                    .with_parent(Selectable::node(0))
                    .with_attribute(
                        "value".to_owned(),
                        TestGraph::NUMERIC_NODE_VALUE.to_string(),
                    ),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test verifies simple saving and restoring of variables.
    ///
    /// Root node saves a reference to itself in a variable,
    /// which is then recalled by a successor node.
    #[test]
    fn save_variable_at_root() {
        // :: {
        //   --root: @;
        // }
        //
        // :: main {
        //   parent: --root;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--root".to_owned()),
                    value: Expression::Select(LimitedSelector::default().into()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into())].into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Parent),
                    value: Expression::Variable("--root".to_owned()),
                }],
            },
        ]));
        let expected_mapping = [(
            Selectable::node(1),
            // Reference to the root node should have been loaded from the variable
            PropertyMap::new().with_parent(Selectable::node(0)),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test ensures that evaluation of individual clauses in a rule
    /// is sequentially consistent.
    ///
    /// When clauses depend on one another, they must be evaluated
    /// in the order they are written.
    #[test]
    fn variable_assignment_sequential_consistency() {
        // :: {
        //   --i: 0;
        //   a: --i;
        //   --i: --i + 1;
        //   b: --i;
        //   --i: --i + 2;
        //   c: --i;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: Variable("--i".to_owned()),
                    value: Expression::Int(0),
                },
                StyleClause {
                    key: Property(Attribute("a".to_owned())),
                    value: Expression::Variable("--i".to_owned()),
                },
                StyleClause {
                    key: Variable("--i".to_owned()),
                    value: Expression::BinaryOperator(
                        Expression::Variable("--i".to_owned()).into(),
                        BinaryOperator::Plus,
                        Expression::Int(1).into(),
                    ),
                },
                StyleClause {
                    key: Property(Attribute("b".to_owned())),
                    value: Expression::Variable("--i".to_owned()),
                },
                StyleClause {
                    key: Variable("--i".to_owned()),
                    value: Expression::BinaryOperator(
                        Expression::Variable("--i".to_owned()).into(),
                        BinaryOperator::Plus,
                        Expression::Int(2).into(),
                    ),
                },
                StyleClause {
                    key: Property(Attribute("c".to_owned())),
                    value: Expression::Variable("--i".to_owned()),
                },
            ],
        }]));
        let expected_mapping = [(
            Selectable::node(0),
            PropertyMap::new()
                .with_attribute("a".to_owned(), "0".to_owned())
                .with_attribute("b".to_owned(), "1".to_owned())
                .with_attribute("c".to_owned(), "3".to_owned()),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test serves as a proof of concept of depth limitation
    /// and verifies that it works as expected.
    ///
    /// A depth-tracking variable is initialized in the root node
    /// and then incremented on each match. Nodes only match until
    /// the variable reaches the depth limit.
    ///
    /// Note that the continuation condition is inside of the `.many`
    /// matcher instead of after it. This is more efficient as the
    /// condition is verified on every iteration, not just at the end,
    /// and the selector stops traversing as soon as depth limit is reached.
    /// If the condition were outside the `.many` matcher,
    /// the resolver would traverse the graph to arbitrary depth and then
    /// filter out the nodes that exceed the depth limit.
    #[test]
    fn max_depth_using_variables() {
        // :: {
        //   --depth: 0;
        // }
        //
        // :: main .many(next.if(--depth < 3)) {
        //   value: --depth;
        //   --depth: --depth + 1;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--depth".to_owned()),
                    value: Expression::Int(0),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::Match(EdgeLabel::Main.into()),
                        SelectorSegment::AnyNumberOfTimes(
                            [
                                SelectorSegment::Match(EdgeLabel::Next.into()),
                                SelectorSegment::Condition(Expression::BinaryOperator(
                                    Expression::Variable("--depth".to_owned()).into(),
                                    BinaryOperator::Lt,
                                    Expression::Int(3).into(),
                                )),
                            ]
                            .into(),
                        ),
                    ]
                    .into(),
                ),
                properties: vec![
                    StyleClause {
                        key: Property(Attribute("value".to_owned())),
                        value: Expression::Variable("--depth".to_owned()),
                    },
                    StyleClause {
                        key: Variable("--depth".to_owned()),
                        value: Expression::BinaryOperator(
                            Expression::Variable("--depth".to_owned()).into(),
                            BinaryOperator::Plus,
                            Expression::Int(1).into(),
                        ),
                    },
                ],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(1),
                PropertyMap::new().with_attribute("value".to_owned(), "0".to_owned()),
            ),
            (
                Selectable::node(2),
                PropertyMap::new().with_attribute("value".to_owned(), "1".to_owned()),
            ),
            (
                Selectable::node(3),
                PropertyMap::new().with_attribute("value".to_owned(), "2".to_owned()),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn index_edge_magic_variables() {
        // .many(*).if(isset(--INDEX)) {
        //   value: --INDEX;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [
                    SelectorSegment::anything_any_number_of_times(),
                    SelectorSegment::Condition(Expression::UnaryOperator(
                        UnaryOperator::IsSet,
                        Expression::MagicVariable(MagicVariableKey::EdgeIndex).into(),
                    )),
                ]
                .into(),
            ),
            properties: vec![StyleClause {
                key: Property(Attribute("value".to_owned())),
                value: Expression::MagicVariable(MagicVariableKey::EdgeIndex),
            }],
        }]));
        let expected_mapping = [
            (
                Selectable::node(8),
                PropertyMap::new().with_attribute("value".to_owned(), "0".to_owned()),
            ),
            (
                Selectable::node(12),
                PropertyMap::new().with_attribute("value".to_owned(), "1".to_owned()),
            ),
            (
                Selectable::node(13),
                PropertyMap::new().with_attribute("value".to_owned(), "0".to_owned()),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn named_edge_magic_variables() {
        // .many(*).if((isset(--NAME) && --NAME) != "a" || --DISCRIMINATOR) {
        //   value: --NAME + --DISCRIMINATOR;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [
                    SelectorSegment::anything_any_number_of_times(),
                    SelectorSegment::Condition(Expression::BinaryOperator(
                        Expression::BinaryOperator(
                            Expression::UnaryOperator(
                                UnaryOperator::IsSet,
                                Expression::MagicVariable(MagicVariableKey::EdgeName).into(),
                            )
                            .into(),
                            BinaryOperator::And,
                            Expression::BinaryOperator(
                                Expression::MagicVariable(MagicVariableKey::EdgeName).into(),
                                BinaryOperator::Ne,
                                Expression::String("a".to_owned()).into(),
                            )
                            .into(),
                        )
                        .into(),
                        BinaryOperator::Or,
                        Expression::MagicVariable(MagicVariableKey::EdgeDiscriminator).into(),
                    )),
                ]
                .into(),
            ),
            properties: vec![StyleClause {
                key: Property(Attribute("value".to_owned())),
                value: Expression::BinaryOperator(
                    Expression::MagicVariable(MagicVariableKey::EdgeName).into(),
                    BinaryOperator::Plus,
                    Expression::MagicVariable(MagicVariableKey::EdgeDiscriminator).into(),
                ),
            }],
        }]));
        let expected_mapping = [
            (
                Selectable::node(7),
                PropertyMap::new().with_attribute("value".to_owned(), "b0".to_owned()),
            ),
            (
                Selectable::node(12),
                PropertyMap::new().with_attribute("value".to_owned(), "a1".to_owned()),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test case reproduces a discovered bug where
    /// select expressions run from the body of a rule
    /// that selects an edge are not evaluated correctly.
    ///
    /// Select expressions should be evaluated
    /// relative to the target node.
    #[test]
    fn select_expressions_in_edge_rule() {
        // :: {
        //   --root: @(main);
        // }
        //
        // :: main::edge {
        //   parent: --root;
        //   target: @(next);
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--root".to_owned()),
                    value: Expression::Select(
                        LimitedSelector::from_path([EdgeLabel::Main.into()]).into(),
                    ),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into())].into(),
                )
                .selecting_edge(),
                properties: vec![
                    StyleClause {
                        key: Property(Parent),
                        value: Expression::Variable("--root".to_owned()),
                    },
                    StyleClause {
                        key: Property(Target),
                        value: Expression::Select(
                            LimitedSelector::from_path([EdgeLabel::Next.into()]).into(),
                        ),
                    },
                ],
            },
        ]));
        let expected_mapping = [(
            Selectable::edge(0, EdgeLabel::Main),
            PropertyMap::new()
                .with_display(DisplayMode::Connector) // Assigned automatically
                .with_parent(Selectable::node(1))
                .with_target(Selectable::node(2)),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test case verifies that select expressions
    /// in the bodies of rules that select extra entities
    /// are relative to the owning element.
    #[test]
    fn select_expressions_in_extra_rule() {
        // :: ::extra {
        //   parent: @;
        // }
        //
        // :: main::edge::extra {
        //   parent: @;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default().with_extra("".to_owned()),
                properties: vec![StyleClause {
                    key: Property(Parent),
                    value: Expression::Select(LimitedSelector::default().into()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into())].into(),
                )
                .selecting_edge()
                .with_extra("".to_owned()),
                properties: vec![StyleClause {
                    key: Property(Parent),
                    value: Expression::Select(LimitedSelector::default().into()),
                }],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(0).with_extra("".to_owned()),
                PropertyMap::new().with_parent(Selectable::node(0)),
            ),
            (
                Selectable::edge(0, EdgeLabel::Main).with_extra("".to_owned()),
                PropertyMap::new().with_parent(Selectable::node(1)),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test verifies that rules are applied in order of declaration.
    ///
    /// The last rule should override properties set by earlier rules,
    /// even if they are resolved through different paths.
    #[test]
    fn rule_precedence_in_declaration_order() {
        // :: "a" .many(*) ref {
        //   value: cell;
        // }
        //
        // :: main .many(next) "a" {
        //   value: kvt;
        // }
        //
        // .many(*) "b" {
        //   value: graph;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
                        SelectorSegment::anything_any_number_of_times(),
                        SelectorSegment::Match(EdgeLabel::Deref.into()),
                    ]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".to_owned())),
                    value: Expression::String("cell".to_owned()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::Match(EdgeLabel::Main.into()),
                        SelectorSegment::AnyNumberOfTimes(
                            [SelectorSegment::Match(EdgeLabel::Next.into())].into(),
                        ),
                        SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
                    ]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".to_owned())),
                    value: Expression::String("kvt".to_owned()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::anything_any_number_of_times(),
                        SelectorSegment::Match(EdgeMatcher::Named("b".to_owned())),
                    ]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".to_owned())),
                    value: Expression::String("graph".to_owned()),
                }],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(5),
                PropertyMap::new().with_attribute("value".to_owned(), "cell".to_owned()),
            ),
            (
                Selectable::node(7),
                PropertyMap::new().with_attribute("value".to_owned(), "graph".to_owned()),
            ),
            (
                Selectable::node(9),
                PropertyMap::new().with_attribute("value".to_owned(), "cell".to_owned()),
            ),
            (
                Selectable::node(10),
                PropertyMap::new().with_attribute("value".to_owned(), "kvt".to_owned()),
            ),
            (
                Selectable::node(12),
                PropertyMap::new().with_attribute("value".to_owned(), "cell".to_owned()),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test case reproduces a discovered bug where
    /// variables assigned by earlier rules are not accessible
    /// in later rules, even in the same run.
    #[test]
    fn variable_sequential_consistency_across_rules() {
        // :: {
        //   --a: a;
        // }
        //
        // :: {
        //   value: --a + --b;
        // }
        //
        // :: {
        //   --b: b;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--a".into()),
                    value: Expression::String("a".to_owned()),
                }],
            },
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".into())),
                    value: Expression::BinaryOperator(
                        Expression::Variable("--a".to_owned()).into(),
                        BinaryOperator::Plus,
                        Expression::Variable("--b".to_owned()).into(),
                    ),
                }],
            },
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--b".into()),
                    value: Expression::String("b".to_owned()),
                }],
            },
        ]));
        let expected_mapping = [(
            Selectable::node(0),
            PropertyMap::new().with_attribute("value".to_owned(), "a".to_owned()),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test verifies that variables are inherited correctly
    /// when `::edge` and `::extra` matchers are involved.
    ///
    /// - `::edge` selector should have access to variables
    ///   assigned by its source node
    /// - `::edge` selector should provide variables for its
    ///   target node
    /// - `::extra` matchers should have access to variables
    ///   assigned by their owner entities
    /// - Variables assigned by `::extra` matchers should not
    ///   be visible from anywhere else
    ///
    /// In essence, the variable scope propagation graph
    /// should look as follows.
    /// ```text
    /// [node] --> [edge] --> [node]
    ///     \          \
    ///      v          v
    ///     [extra]    [extra]
    /// ```
    #[test]
    fn variable_scopes_with_edge_and_extra_matchers() {
        // :: main {
        //   value: --a + --b + --c + --d + --e;
        // }
        //
        // :: main::edge::extra {
        //   value: --a + --b + --c + --d + --e;
        //   --e: e;
        // }
        //
        // :: main::edge {
        //   value: --a + --b + --c + --d + --e;
        //   --d: d;
        // }
        //
        // :: ::extra {
        //   value: --a + --b + --c + --d + --e;
        //   --b: b;
        // }
        //
        // :: ::extra(other) {
        //   value: --a + --b + --c + --d + --e;
        //   --c: c;
        // }
        //
        // :: {
        //   value: --a + --b + --c + --d + --e;
        //   --a: a;
        // }
        let value_assignment = StyleClause {
            key: Property(Attribute("value".to_owned())),
            value: Expression::BinaryOperator(
                Expression::BinaryOperator(
                    Expression::BinaryOperator(
                        Expression::BinaryOperator(
                            Expression::Variable("--a".to_owned()).into(),
                            BinaryOperator::Plus,
                            Expression::Variable("--b".to_owned()).into(),
                        )
                        .into(),
                        BinaryOperator::Plus,
                        Expression::Variable("--c".to_owned()).into(),
                    )
                    .into(),
                    BinaryOperator::Plus,
                    Expression::Variable("--d".to_owned()).into(),
                )
                .into(),
                BinaryOperator::Plus,
                Expression::Variable("--e".to_owned()).into(),
            ),
        };
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into())].into(),
                ),
                properties: vec![value_assignment.clone()],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into())].into(),
                )
                .selecting_edge()
                .with_extra("".to_owned()),
                properties: vec![
                    value_assignment.clone(),
                    StyleClause {
                        key: Variable("--e".to_owned()),
                        value: Expression::String("e".to_owned()),
                    },
                ],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into())].into(),
                )
                .selecting_edge(),
                properties: vec![
                    value_assignment.clone(),
                    StyleClause {
                        key: Variable("--d".to_owned()),
                        value: Expression::String("d".to_owned()),
                    },
                ],
            },
            StyleRule {
                selector: Selector::default().with_extra("".to_owned()),
                properties: vec![
                    value_assignment.clone(),
                    StyleClause {
                        key: Variable("--b".to_owned()),
                        value: Expression::String("b".to_owned()),
                    },
                ],
            },
            StyleRule {
                selector: Selector::default().with_extra("other".to_owned()),
                properties: vec![
                    value_assignment.clone(),
                    StyleClause {
                        key: Variable("--c".to_owned()),
                        value: Expression::String("c".to_owned()),
                    },
                ],
            },
            StyleRule {
                selector: Selector::default(),
                properties: vec![
                    value_assignment.clone(),
                    StyleClause {
                        key: Variable("--a".to_owned()),
                        value: Expression::String("a".to_owned()),
                    },
                ],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(0).with_extra("".to_owned()),
                PropertyMap::new().with_attribute("value".to_owned(), "a".to_owned()),
            ),
            (
                Selectable::node(0).with_extra("other".to_owned()),
                PropertyMap::new().with_attribute("value".to_owned(), "a".to_owned()),
            ),
            (
                Selectable::edge(0, EdgeLabel::Main),
                PropertyMap::new()
                    .with_attribute("value".to_owned(), "a".to_owned())
                    // Display, parent, and target assigned automatically
                    .with_display(DisplayMode::Connector)
                    .with_parent(Selectable::node(0))
                    .with_target(Selectable::node(1)),
            ),
            (
                Selectable::edge(0, EdgeLabel::Main).with_extra("".to_owned()),
                PropertyMap::new().with_attribute("value".to_owned(), "ad".to_owned()),
            ),
            (
                Selectable::node(1),
                PropertyMap::new().with_attribute("value".to_owned(), "ad".to_owned()),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    /// This test verifies that if [`PropertyValue::Unset`]
    /// is assigned to a property, the attribute will not
    /// exist in the result.
    #[test]
    fn assigning_unset_erases_property() {
        // :: {
        //   value: none;
        //   display: none;
        //   parent: none;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: Property(Attribute("value".to_owned())),
                    value: Expression::Unset.to_owned(),
                },
                StyleClause {
                    key: Property(Display),
                    value: Expression::Unset.to_owned(),
                },
                StyleClause {
                    key: Property(Parent),
                    value: Expression::Unset.to_owned(),
                },
            ],
        }]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        // The element should not have an entry at all
        assert_eq!(resolved, EntityPropertyMapping::new());
    }

    /// This test verifies that if the same rule
    /// assigns the same property more than once,
    /// the last assignment counts.
    ///
    /// The same rule for variables is already verified by
    /// [`variable_assignment_sequential_consistency`].
    #[test]
    fn latter_property_assignments_take_priority() {
        // :: {
        //   display: connector;
        //   display: none;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: Property(Display),
                    value: Expression::String("connector".to_owned()),
                },
                StyleClause {
                    key: Property(Display),
                    value: Expression::Unset.to_owned(),
                },
            ],
        }]));
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        // Display property was removed by last assignment,
        // so the mapping should be empty
        assert_eq!(resolved, EntityPropertyMapping::new());
    }

    #[test]
    fn automatic_node_parent_assignment() {
        // :: {
        //   display: graph;
        // }
        //
        // :: .alt(main, main "a", "a", "a" ref "a") {
        //   display: cell;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Property(Display),
                    value: Expression::String("graph".to_owned()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Branch(vec![
                        [SelectorSegment::Match(EdgeLabel::Main.into())].into(),
                        [
                            SelectorSegment::Match(EdgeLabel::Main.into()),
                            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
                        ]
                        .into(),
                        [SelectorSegment::Match(EdgeMatcher::Named("a".to_owned()))].into(),
                        [
                            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
                            SelectorSegment::Match(EdgeLabel::Deref.into()),
                            SelectorSegment::Match(EdgeMatcher::Named("a".to_owned())),
                        ]
                        .into(),
                    ])]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Display),
                    value: Expression::String("cell".to_owned()),
                }],
            },
        ]));
        let expected_mapping = [
            (
                Selectable::node(0),
                PropertyMap::new().with_display(DisplayMode::ElementTag("graph".to_owned())),
            ),
            (
                Selectable::node(1),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    .with_parent(Selectable::node(0)),
            ),
            (
                Selectable::node(5),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    .with_parent(Selectable::node(0)),
            ),
            (
                Selectable::node(10),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    // This node was reached by the (:: main next "a") selector,
                    // so its default parent is resolved along that path
                    .with_parent(Selectable::node(1)),
            ),
            (
                Selectable::node(11),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    // These two nodes were reached by the (:: "a" ref "a") selector,
                    // so although node 10 is along the way, it does not participate
                    // in parent assignment
                    .with_parent(Selectable::node(5)),
            ),
            (
                Selectable::node(12),
                PropertyMap::new()
                    .with_display(DisplayMode::ElementTag("cell".to_owned()))
                    .with_parent(Selectable::node(5)),
            ),
        ]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn fragment_attributes() {
        // :: {
        //   start/value: 42;
        //   end/key: abc;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: Property(FragmentAttribute(FragmentKey::Start, "value".to_owned())),
                    value: Expression::Int(42),
                },
                StyleClause {
                    key: Property(FragmentAttribute(FragmentKey::End, "key".to_owned())),
                    value: Expression::String("abc".to_owned()),
                },
            ],
        }]));
        let expected_mapping = [(
            Selectable::node(0),
            PropertyMap::new()
                .with_fragment_attribute(FragmentKey::Start, "value".to_owned(), "42".to_owned())
                .with_fragment_attribute(FragmentKey::End, "key".to_owned(), "abc".to_owned()),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn dynamic_index_matcher() {
        // :: {
        //   --i: 1;
        // }
        //
        // .if(@([--i])) {
        //   value: abc;
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--i".to_owned()),
                    value: Expression::Int(1),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [
                        SelectorSegment::anything_any_number_of_times(),
                        SelectorSegment::Condition(Expression::Select(
                            LimitedSelector::from_path([LimitedEdgeMatcher::DynIndex(
                                Expression::Variable("--i".to_owned()),
                            )])
                            .into(),
                        )),
                    ]
                    .into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".to_owned())),
                    value: Expression::String("abc".to_owned()),
                }],
            },
        ]));
        let expected_mapping = [(
            Selectable::node(11),
            PropertyMap::new().with_attribute("value".to_owned(), "abc".to_owned()),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }

    #[test]
    fn select_origin_override() {
        // :: {
        //   --root: @;
        // }
        //
        // :: main {
        //   value: @((--root) "a");
        // }
        let stylesheet = CascadeStyle::from(Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: Variable("--root".to_owned()),
                    value: Expression::Select(LimitedSelector::default().into()),
                }],
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into())].into(),
                ),
                properties: vec![StyleClause {
                    key: Property(Attribute("value".to_owned())),
                    value: Expression::Select(
                        LimitedSelector::from_path([EdgeLabel::Named("a".to_owned(), 0).into()])
                            .with_origin(Expression::Variable("--root".to_owned()))
                            .into(),
                    ),
                }],
            },
        ]));
        let expected_mapping = [(
            Selectable::node(1),
            PropertyMap::new().with_attribute(
                "value".to_owned(),
                TestGraph::NUMERIC_NODE_VALUE.to_string(),
            ),
        )]
        .into();
        let resolved = apply_stylesheet(&stylesheet, &TestGraph::default_graph());
        assert_eq!(resolved, expected_mapping);
    }
}
