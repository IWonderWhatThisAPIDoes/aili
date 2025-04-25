#![doc = include_str!("../README.md")]

mod connect;
mod grammar;
mod lexer;
mod mock_error_handler;
mod report;
pub mod symbols;

use aili_translate::stylesheet::Stylesheet;
use derive_more::{Display, Error, From};
use grammar::{ErrorManager, Parser};
use lexer::Token;
use logos::Logos;
use report::FilteredErrorHandler;

pub use grammar::{ParseFailure, SyntaxError};
pub use lexer::LexerError;

/// Error type that indicates recoverable lexer or parser input errors.
#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
#[display("line {line_number}: {error_data}")]
pub struct ParseError {
    /// Information about the error.
    #[error(source)]
    pub error_data: ParseErrorInfo,
    /// One-based number of the line where the error occurred.
    pub line_number: usize,
}

/// Internal data for recoverable lexer or parser errors.
#[derive(Clone, PartialEq, Eq, Debug, Display, Error, From)]
pub enum ParseErrorInfo {
    /// Error originating from the lexer.
    LexerError(LexerError),

    /// Error originating from the parser.
    SyntaxError(SyntaxError),
}

/// Parses a [`Stylesheet`].
///
/// The parse function attempts error recovery by discarding unparsable
/// tokens. The returned stylesheet is a parsable portion of the input.
/// An error is only returned if the parser irrecoverably fails.
pub fn parse_stylesheet(
    source: &str,
    error_handler: impl FnMut(ParseError),
) -> Result<Stylesheet, ParseFailure> {
    let lexer = Token::lexer(source);
    // Wrap error handler and lexer in a RefCell so we can access it
    // from both parser and the main loop
    let shared = std::cell::RefCell::new((lexer, FilteredErrorHandler::new(error_handler)));
    let report_error = |error_data| {
        let (lexer, error_handler) = &mut *shared.borrow_mut();
        error_handler.handle_error(ParseError {
            error_data,
            line_number: lexer.extras.line_index + 1,
        });
    };
    // Wrap this in a callback because otherwise the borrow
    // would not be dropped in time and error reporting would fail
    let next_token_from_lexer = || shared.borrow_mut().0.next();
    // Forward syntax errors to the handler
    let parser_extra = ErrorManager::new(|err| report_error(err.into()));
    let mut parser = Parser::new(parser_extra);
    while let Some(token) = next_token_from_lexer() {
        match token {
            Ok(token) => {
                parser.parse(token.into())?;
                shared.borrow_mut().1.token_parsed();
            }
            Err(err) => report_error(err.into()),
        }
    }
    // Push end token so we get relevant error descriptions
    parser.parse(grammar::Token::End)?;
    Ok(parser.end_of_input()?.0)
}

#[cfg(test)]
mod test {
    use super::{ParseError, parse_stylesheet};
    use crate::{
        grammar::{self, SyntaxError},
        lexer::LexerError,
        mock_error_handler::ExpectErrors,
        symbols::InvalidSymbol,
    };
    use aili_model::state::{EdgeLabel, NodeTypeClass};
    use aili_translate::{
        property::{FragmentKey, PropertyKey},
        stylesheet::{expression::*, selector::*, *},
    };

    #[test]
    fn minimal_empty_rule() {
        let source = ":: { }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: Vec::new(),
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn assign_unquoted_to_unquoted() {
        let source = ":: { abc:def }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Property(PropertyKey::Attribute("abc".to_owned())),
                value: Expression::String("def".to_owned()),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn assign_single_letter_to_single_letter() {
        let source = ":: { a:b }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Property(PropertyKey::Attribute("a".to_owned())),
                value: Expression::String("b".to_owned()),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn multiple_clauses_with_trailing_semicolon() {
        let source = ":: { a: 1; b: 2; }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Attribute("a".to_owned())),
                    value: Expression::Int(1),
                },
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Attribute("b".to_owned())),
                    value: Expression::Int(2),
                },
            ],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn variable_invocations() {
        let source = ":: { --i: --j }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Variable("--i".to_owned()),
                value: Expression::Variable("--j".to_owned()),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn arihhmetic_operators() {
        let source = ":: { a: -1 - 3 * 2 + 4 / 2 % +5 }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Property(PropertyKey::Attribute("a".to_owned())),
                value: Expression::BinaryOperator(
                    Expression::BinaryOperator(
                        Expression::UnaryOperator(
                            expression::UnaryOperator::Minus,
                            Expression::Int(1).into(),
                        )
                        .into(),
                        BinaryOperator::Minus,
                        Expression::BinaryOperator(
                            Expression::Int(3).into(),
                            BinaryOperator::Mul,
                            Expression::Int(2).into(),
                        )
                        .into(),
                    )
                    .into(),
                    BinaryOperator::Plus,
                    Expression::BinaryOperator(
                        Expression::BinaryOperator(
                            Expression::Int(4).into(),
                            BinaryOperator::Div,
                            Expression::Int(2).into(),
                        )
                        .into(),
                        BinaryOperator::Mod,
                        Expression::UnaryOperator(
                            expression::UnaryOperator::Plus,
                            Expression::Int(5).into(),
                        )
                        .into(),
                    )
                    .into(),
                ),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn empty_select_expression() {
        let source = ":: { value: @ }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Property(PropertyKey::Attribute("value".to_owned())),
                value: Expression::Select(LimitedSelector::default().into()),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn logical_operators() {
        let source = ":: { value: @ || --a && !--b || --i == 0 }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Property(PropertyKey::Attribute("value".to_owned())),
                value: Expression::BinaryOperator(
                    Expression::BinaryOperator(
                        Expression::Select(LimitedSelector::default().into()).into(),
                        BinaryOperator::Or,
                        Expression::BinaryOperator(
                            Expression::Variable("--a".to_owned()).into(),
                            BinaryOperator::And,
                            Expression::UnaryOperator(
                                expression::UnaryOperator::Not,
                                Expression::Variable("--b".to_owned()).into(),
                            )
                            .into(),
                        )
                        .into(),
                    )
                    .into(),
                    BinaryOperator::Or,
                    Expression::BinaryOperator(
                        Expression::Variable("--i".to_owned()).into(),
                        BinaryOperator::Eq,
                        Expression::Int(0).into(),
                    )
                    .into(),
                ),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn select_expression_with_path() {
        let source = ":: { value: @(\"a\" [42]) }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Property(PropertyKey::Attribute("value".to_owned())),
                value: Expression::Select(
                    LimitedSelector::from_path([
                        EdgeLabel::Named("a".to_owned(), 0).into(),
                        EdgeLabel::Index(42).into(),
                    ])
                    .into(),
                ),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn ternary_operator() {
        let source = ":: { value: --a && --b ? \"true\" : 1 + --a }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Property(PropertyKey::Attribute("value".to_owned())),
                value: Expression::Conditional(
                    Expression::BinaryOperator(
                        Expression::Variable("--a".to_owned()).into(),
                        BinaryOperator::And,
                        Expression::Variable("--b".to_owned()).into(),
                    )
                    .into(),
                    Expression::String("true".to_owned()).into(),
                    Expression::BinaryOperator(
                        Expression::Int(1).into(),
                        BinaryOperator::Plus,
                        Expression::Variable("--a".to_owned()).into(),
                    )
                    .into(),
                ),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn selector_edge_matchers() {
        let source = "main next ret ref len [] [42] \"a\" \"b\"#1 * % { }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::from_path(SelectorPath(
                std::iter::once(SelectorSegment::anything_any_number_of_times())
                    .chain(
                        [
                            EdgeLabel::Main.into(),
                            EdgeLabel::Next.into(),
                            EdgeLabel::Result.into(),
                            EdgeLabel::Deref.into(),
                            EdgeLabel::Length.into(),
                            EdgeMatcher::AnyIndex,
                            EdgeLabel::Index(42).into(),
                            EdgeMatcher::Named("a".to_owned()),
                            EdgeLabel::Named("b".to_owned(), 1).into(),
                            EdgeMatcher::Any,
                            EdgeMatcher::AnyNamed,
                        ]
                        .into_iter()
                        .map(SelectorSegment::Match),
                    )
                    .collect(),
            )),
            properties: Vec::new(),
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn selector_pseudo_elements() {
        let source =
            "::::edge { } ::::extra { } ::::extra(hello-world) { } :: main::edge::extra { }";
        let expected_stylesheet = Stylesheet(vec![
            StyleRule {
                selector: Selector::default().selecting_edge(),
                properties: Vec::new(),
            },
            StyleRule {
                selector: Selector::default().with_extra("".to_owned()),
                properties: Vec::new(),
            },
            StyleRule {
                selector: Selector::default().with_extra("hello-world".to_owned()),
                properties: Vec::new(),
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeLabel::Main.into())].into(),
                )
                .selecting_edge()
                .with_extra("".to_owned()),
                properties: Vec::new(),
            },
        ]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn branched_selectors() {
        let source = ":: .many(.alt(next ret, .many(%))) { }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [SelectorSegment::AnyNumberOfTimes(
                    [SelectorSegment::Branch(vec![
                        [
                            SelectorSegment::Match(EdgeLabel::Next.into()),
                            SelectorSegment::Match(EdgeLabel::Result.into()),
                        ]
                        .into(),
                        [SelectorSegment::AnyNumberOfTimes(
                            [SelectorSegment::Match(EdgeMatcher::AnyNamed)].into(),
                        )]
                        .into(),
                    ])]
                    .into(),
                )]
                .into(),
            ),
            properties: Vec::new(),
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn special_property_keys() {
        let source = ":: { display: unset; \"display\": \"unset\"; parent: true; target: false; \"--i\": 1 }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Display),
                    value: Expression::Unset,
                },
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Attribute("display".to_owned())),
                    value: Expression::String("unset".to_owned()),
                },
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Parent),
                    value: Expression::Bool(true),
                },
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Target),
                    value: Expression::Bool(false),
                },
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Attribute("--i".to_owned())),
                    value: Expression::Int(1),
                },
            ],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn restricted_selector() {
        let source = ":: .many(*.if(--c)).if(--i == 0) { }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [
                    SelectorSegment::AnyNumberOfTimes(
                        [
                            SelectorSegment::Match(EdgeMatcher::Any),
                            SelectorSegment::Condition(Expression::Variable("--c".to_owned())),
                        ]
                        .into(),
                    ),
                    SelectorSegment::Condition(Expression::BinaryOperator(
                        Expression::Variable("--i".to_owned()).into(),
                        BinaryOperator::Eq,
                        Expression::Int(0).into(),
                    )),
                ]
                .into(),
            ),
            properties: Vec::new(),
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn named_operators() {
        let source = ":: { a: isset(--i); b: is-root(@); c: typename(@); d: val(@); }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Attribute("a".to_owned())),
                    value: Expression::UnaryOperator(
                        expression::UnaryOperator::IsSet,
                        Expression::Variable("--i".to_owned()).into(),
                    ),
                },
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Attribute("b".to_owned())),
                    value: Expression::UnaryOperator(
                        expression::UnaryOperator::NodeIsA(NodeTypeClass::Root),
                        Expression::Select(LimitedSelector::default().into()).into(),
                    ),
                },
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Attribute("c".to_owned())),
                    value: Expression::UnaryOperator(
                        expression::UnaryOperator::NodeTypeName,
                        Expression::Select(LimitedSelector::default().into()).into(),
                    ),
                },
                StyleClause {
                    key: StyleKey::Property(PropertyKey::Attribute("d".to_owned())),
                    value: Expression::UnaryOperator(
                        expression::UnaryOperator::NodeValue,
                        Expression::Select(LimitedSelector::default().into()).into(),
                    ),
                },
            ],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn conditional_operator_precedence() {
        let source = ":: { a: 1 ? --a && 2 ? 3 : 4 : --a && 5 }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Property(PropertyKey::Attribute("a".to_owned())),
                value: Expression::Conditional(
                    Expression::Int(1).into(),
                    Expression::Conditional(
                        Expression::BinaryOperator(
                            Expression::Variable("--a".to_owned()).into(),
                            BinaryOperator::And,
                            Expression::Int(2).into(),
                        )
                        .into(),
                        Expression::Int(3).into(),
                        Expression::Int(4).into(),
                    )
                    .into(),
                    Expression::BinaryOperator(
                        Expression::Variable("--a".to_owned()).into(),
                        BinaryOperator::And,
                        Expression::Int(5).into(),
                    )
                    .into(),
                ),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn invalid_selector() {
        // The affected rules should be discarded, but all others should be retained
        let source = ":: { } # { }  main > } { } }";
        let expected_stylesheet = Stylesheet(vec![
            StyleRule {
                selector: Selector::default(),
                properties: Vec::new(),
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::anything_any_number_of_times()].into(),
                ),
                properties: Vec::new(),
            },
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::anything_any_number_of_times()].into(),
                ),
                properties: Vec::new(),
            },
        ]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::some().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn unclosed_rule_body() {
        let source = ":: % { } :: { --a: b ";
        let expected_stylesheet = Stylesheet(vec![
            StyleRule {
                selector: Selector::from_path(
                    [SelectorSegment::Match(EdgeMatcher::AnyNamed)].into(),
                ),
                properties: Vec::new(),
            },
            StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: StyleKey::Variable("--a".to_owned()),
                    value: Expression::String("b".to_owned()),
                }],
            },
        ]);
        let parsed_stylesheet = parse_stylesheet(
            source,
            ExpectErrors::exact([ParseError {
                error_data: SyntaxError::UnterminatedRule.into(),
                line_number: 1,
            }])
            .f(),
        )
        .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn missing_semicolon() {
        let source = ":: { a: a; b: b /* missing semicolon */ x: x; c: c }";
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::some().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(Stylesheet::default(), parsed_stylesheet);
    }

    #[test]
    fn empty_clause_right_hand_side() {
        let source = ":: { a: a; b: /* missing rhs */; c: c }";
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::some().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(Stylesheet::default(), parsed_stylesheet);
    }

    #[test]
    fn missing_clause_separator() {
        let source = ":: { a: a; b /* missing colon */ b; c: c }";
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::some().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(Stylesheet::default(), parsed_stylesheet);
    }

    #[test]
    fn missing_clause_separator_and_right_hand_side() {
        let source = ":: { a: a; b; c: c }";
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::some().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(Stylesheet::default(), parsed_stylesheet);
    }

    #[test]
    fn multiple_tokens_on_left_hand_side() {
        let source = ":: { a: a; b b: b; c: c }";
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::some().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(Stylesheet::default(), parsed_stylesheet);
    }

    #[test]
    fn invalid_token_on_left_hand_side() {
        let source = ":: { a: a; 42: b; c: c }";
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::some().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(Stylesheet::default(), parsed_stylesheet);
    }

    #[test]
    fn extra_semicolon() {
        let source = ":: { a: a; ; c: c }";
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::some().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(Stylesheet::default(), parsed_stylesheet);
    }

    /// This test verifies that a parser stack overflow is handled
    /// gracefully and the right error is returned.
    ///
    /// Parser stack overflow can be triggered by deeply nesting
    /// right-associative expressions (either parenthesized, or right-associative
    /// by design.
    #[test]
    fn stack_overflow() {
        let source = r"*.if(
            0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ?
                0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ?
                    0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ?
                        0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ?
                            0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ? 0 ?
                                0 ? 0 ? 0 ? 0 ? 0 ? 0 : 0 : 0 : 0 : 0 : 0 :
                            0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 :
                        0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 :
                    0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 :
                0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 :
            0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 : 0 
        ) { }";
        let result = parse_stylesheet(source, ExpectErrors::none().f());
        assert_eq!(result, Err(grammar::ParseFailure::StackOverflow));
    }

    #[test]
    fn fragment_attributes() {
        let source = ":: { start/a: a; end/b: b }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![
                StyleClause {
                    key: StyleKey::Property(PropertyKey::FragmentAttribute(
                        FragmentKey::Start,
                        "a".to_owned(),
                    )),
                    value: Expression::String("a".to_owned()),
                },
                StyleClause {
                    key: StyleKey::Property(PropertyKey::FragmentAttribute(
                        FragmentKey::End,
                        "b".to_owned(),
                    )),
                    value: Expression::String("b".to_owned()),
                },
            ],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn invalid_fragment_key() {
        let source = ":: { not-a-fragment/value: none }";
        let expected_error = ParseError {
            error_data: SyntaxError::InvalidFragment(InvalidSymbol("not-a-fragment".to_owned()))
                .into(),
            line_number: 1,
        };
        parse_stylesheet(source, ExpectErrors::exact([expected_error]).f())
            .expect("Stylesheet should have parsed");
    }

    #[test]
    fn type_assertions_in_selectors() {
        let source = ":: :struct :\"struct\" :vector :\"vector\" { }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [
                    SelectorSegment::Condition(Expression::UnaryOperator(
                        UnaryOperator::NodeIsA(NodeTypeClass::Struct),
                        Expression::Select(LimitedSelector::default().into()).into(),
                    )),
                    SelectorSegment::Condition(Expression::BinaryOperator(
                        Expression::UnaryOperator(
                            UnaryOperator::NodeTypeName,
                            Expression::Select(LimitedSelector::default().into()).into(),
                        )
                        .into(),
                        BinaryOperator::Eq,
                        Expression::String("struct".to_owned()).into(),
                    )),
                    SelectorSegment::Condition(Expression::BinaryOperator(
                        Expression::UnaryOperator(
                            UnaryOperator::NodeTypeName,
                            Expression::Select(LimitedSelector::default().into()).into(),
                        )
                        .into(),
                        BinaryOperator::Eq,
                        Expression::String("vector".to_owned()).into(),
                    )),
                    SelectorSegment::Condition(Expression::BinaryOperator(
                        Expression::UnaryOperator(
                            UnaryOperator::NodeTypeName,
                            Expression::Select(LimitedSelector::default().into()).into(),
                        )
                        .into(),
                        BinaryOperator::Eq,
                        Expression::String("vector".to_owned()).into(),
                    )),
                ]
                .into(),
            ),
            properties: Vec::new(),
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn dynamic_index_matcher_in_expression() {
        let source = ":: { parent: @([--len - 1][--i]) }";
        let index_expression = Expression::BinaryOperator(
            Expression::Variable("--len".to_owned()).into(),
            BinaryOperator::Minus,
            Expression::Int(1).into(),
        );
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Property(PropertyKey::Parent),
                value: Expression::Select(
                    LimitedSelector::from_path([
                        LimitedEdgeMatcher::DynIndex(index_expression),
                        LimitedEdgeMatcher::DynIndex(Expression::Variable("--i".to_owned())),
                    ])
                    .into(),
                ),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn dynamic_index_matcher_in_selector() {
        let source = ":: [--len - 1] [--i] { }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::from_path(
                [
                    // Dynamic index matcher unrolls
                    SelectorSegment::Match(EdgeMatcher::AnyIndex),
                    SelectorSegment::Condition(Expression::BinaryOperator(
                        Expression::MagicVariable(MagicVariableKey::EdgeIndex).into(),
                        BinaryOperator::Eq,
                        Expression::BinaryOperator(
                            Expression::Variable("--len".to_owned()).into(),
                            BinaryOperator::Minus,
                            Expression::Int(1).into(),
                        )
                        .into(),
                    )),
                    SelectorSegment::Match(EdgeMatcher::AnyIndex),
                    SelectorSegment::Condition(Expression::BinaryOperator(
                        Expression::MagicVariable(MagicVariableKey::EdgeIndex).into(),
                        BinaryOperator::Eq,
                        Expression::Variable("--i".to_owned()).into(),
                    )),
                ]
                .into(),
            ),
            properties: Vec::new(),
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn select_origin_override() {
        let source = ":: { parent: @((@) main) }";
        let expected_stylesheet = Stylesheet(vec![StyleRule {
            selector: Selector::default(),
            properties: vec![StyleClause {
                key: StyleKey::Property(PropertyKey::Parent),
                value: Expression::Select(
                    LimitedSelector::from_path([EdgeLabel::Main.into()])
                        .with_origin(Expression::Select(LimitedSelector::default().into()))
                        .into(),
                ),
            }],
        }]);
        let parsed_stylesheet = parse_stylesheet(source, ExpectErrors::none().f())
            .expect("Stylesheet should have parsed");
        assert_eq!(expected_stylesheet, parsed_stylesheet);
    }

    #[test]
    fn error_cooldown() {
        let source = r#" /* first line ends here */
        /      /* invalid */
        :: abc /* 2 valid tokens */
        /      /* invalid (not reported, still in cooldown) */
        :: { } /* 3 valid tokens (next error will be reported) */
        "         unterminated string (lexer error, will be reported)
        ::     /* 1 valid token */
        123abc /* invalid number (lexer error, not reported) */
        :: abc /* 2 valid tokens */
        $      /* weird character (lexer error, not reported, still in cooldown) */
        "#;
        let expected_errors = [
            ParseError {
                error_data: SyntaxError::UnexpectedToken.into(),
                line_number: 2,
            },
            ParseError {
                error_data: LexerError::UnterminatedQuoted.into(),
                line_number: 6,
            },
        ];
        parse_stylesheet(source, ExpectErrors::exact(expected_errors).f())
            .expect("Stylesheet should have parsed");
    }
}
