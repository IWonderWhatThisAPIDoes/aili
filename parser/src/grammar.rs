//! Grammar for syntactic analysis
//! and additional semantic analysis.

#![expect(clippy::let_unit_value, reason = "Clippy is triggered by macro output")]

use crate::symbols::*;
use aili_model::state::EdgeLabel;
use aili_style::stylesheet::{expression::*, selector::*, *};
use derive_more::{Display, Error, From};
use pomelo::pomelo;

/// Error type returned by a parser when it irrecoverably fails.
///
/// Recoverable errors are recovered and the parser returns
/// a [`Stylesheet`] that is the valid portion of the input.
#[derive(Clone, PartialEq, Eq, Debug, Display, Error, From, Default)]
pub enum ParseFailure {
    /// Parser failed without providing any additional information.
    #[default]
    #[display("irrecoverable parser failure")]
    Generic,

    /// Parser's stack has overflown.
    #[display("parser stack overflow")]
    StackOverflow,
}

/// Error type emited by a parser when incorrect syntax
/// is encountered.
///
/// Errors of this kind are generally recoverable.
#[derive(Clone, PartialEq, Eq, Debug, Display, Error)]
pub enum SyntaxError {
    /// Parser expected additional tokens at the end of input.
    #[display("unexpected end of input")]
    UnexpectedEnd,

    /// Parser encountered an unexpected token.
    #[display("unexpected token")]
    UnexpectedToken,

    /// An unrecognized edge name was used in a selector.
    #[display("unknown edge label name {:?}", _0.0)]
    InvalidEdgeLabel(InvalidSymbol),

    /// An unrecognized token was used as a function name.
    #[display("unknown function name {:?}", _0.0)]
    InvalidFunction(InvalidSymbol),

    /// An invalid literal was used in an expression.
    #[display("token {:?} which is not a literal cannot appear in an expression", _0.0)]
    InvalidUnquoted(InvalidSymbol),

    /// Missing closing brace at the end of input.
    #[display("last rule is missing a closing delimiter")]
    UnterminatedRule,
}

/// Additional state object for a parser.
///
/// This state object facilitates error reporting and recovery.
pub struct ErrorManager<'a> {
    /// Handler that will be invoked when a recoverable syntax error
    /// is encountered.
    error_handler: Box<dyn FnMut(SyntaxError) + 'a>,
    /// True when the parser is in error recovery state.
    is_recovering: bool,
}

impl Default for ErrorManager<'static> {
    fn default() -> Self {
        Self {
            error_handler: Box::new(|_| {}),
            is_recovering: false,
        }
    }
}

impl<'a> ErrorManager<'a> {
    /// Constructs an extra state object with the default initial state
    /// and a provided error handler.
    pub fn new<F: FnMut(SyntaxError) + 'a>(error_handler: F) -> Self {
        Self {
            error_handler: Box::new(error_handler),
            is_recovering: false,
        }
    }

    /// Wraps the result of an operation that can fail with a syntax error.
    ///
    /// If the operation fails (i. e. the provided result is [`Err`]),
    /// falls back to the default value and switches to recovery
    /// state which can later be resolved by [`ErrorManager::recover`].
    fn try_or<T>(&mut self, result: Result<T, SyntaxError>, default: T) -> T {
        match result {
            Ok(x) => x,
            Err(err) => {
                (self.error_handler)(err);
                self.is_recovering = true;
                default
            }
        }
    }

    /// Signals that Pomelo's `%syntax_error` hook has been triggered.
    ///
    /// Switches to recovery state,
    /// which can later be resolved by [`ErrorManager::recover`].
    fn syntax_error_trigger(&mut self, error: SyntaxError) {
        (self.error_handler)(error);
        self.is_recovering = true;
    }

    /// Signals that the parser has shifted the special `error` nonterminal
    /// in an effort to recover from a syntax error.
    ///
    /// This will be called immediately after [`ErrorManager::syntax_error_trigger`]
    /// (assuming the `error` nonterminal is successfully shifted),
    /// but if multiple syntax errors are present in a row, Pomelo silences
    /// the `%syntax_error` hook, so additional calls to
    /// [`ErrorManager::syntax_error_trigger`] in short succession may be skipped.
    fn shift_error(&mut self) {
        // We trust Pomelo on its judgement to let us know about an error
        // if it is necessary, so we do not emit anything here,
        // just switch back to recovery if we have already exited
        self.is_recovering = true;
    }

    /// Signals that a rule was missing a closing delimiter.
    ///
    /// As this is an easily recoverable error (parser just imagines the closing
    /// brace being there), and because we are at the end of input where
    /// further recovery does not make sense, the parser does not
    /// enter recovery state.
    fn unterminated_rule(&mut self) {
        (self.error_handler)(SyntaxError::UnterminatedRule);
    }

    /// Signals that the parser has reached a state where it can
    /// safely discard a part of input if it is errorneous.
    ///
    /// Exits recovery state.
    ///
    /// ## Return Value
    /// True if the parser was in recovery state (and input should
    /// thus be discarded), false otherwise.
    fn recover(&mut self) -> bool {
        std::mem::take(&mut self.is_recovering)
    }
}

pomelo! {
    %include {
        use super::*;
        use Expression::BinaryOperator as Bop;
        use Expression::UnaryOperator as Uop;
        use UnaryOperator::*;
        use BinaryOperator::*;
        use UnaryOperator::Plus as UnaryPlus;
        use UnaryOperator::Minus as UnaryMinus;
        use BinaryOperator::Plus as BinaryPlus;
        use BinaryOperator::Minus as BinaryMinus;
    }

    // Syntactic analysis errors
    %error ParseFailure;
    %stack_overflow { ParseFailure::StackOverflow }
    %syntax_error {
        let error = if token.is_some_and(|t| t != Token::End) {
            SyntaxError::UnexpectedToken
        } else {
            SyntaxError::UnexpectedEnd
        };
        extra.syntax_error_trigger(error);
        Ok(())
    }

    // Use extra state data to propagate errors
    %extra_argument ErrorManager<'a>;

    %token
    /// Type of tokens accepted by the syntactic parser.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Token<'a> {};

    // Underlying types of nonterminal symbols
    %type stylesheet Stylesheet;
    %type sheet_part Stylesheet;
    %type rule       StyleRule;
    %type body       Vec<StyleClause>;
    %type proplist   Vec<StyleClause>;
    %type proplist1  Vec<StyleClause>;
    %type clause     StyleClause;
    %type lvalue     StyleKey;
    %type rvalue     Expression;
    %type selector   Selector;
    %type selector1  Selector;
    %type selector2  Selector;
    %type condition  Expression;
    %type path       SelectorPath;
    %type segment    SelectorSegment;
    %type pathlist   Vec<SelectorPath>;
    %type limsel     LimitedSelector;
    %type limsel1    LimitedSelector;
    %type limpath    Vec<LimitedEdgeMatcher>;
    %type limseg     LimitedEdgeMatcher;
    %type matcher    EdgeMatcher;
    %type exact      EdgeLabel;
    %type extra      String;
    %type index      Expression;
    %type expr       Expression;
    %type rexpr      Expression;
    %type bop        BinaryOperator;
    %type uop        UnaryOperator;

    // Underlying types of terminal symbols
    %type Unquoted   &'a str;
    %type Quoted     &'a str;
    %type Int        u64;

    %type
    /// Special terminal symbol to signalize end of input.
    ///
    /// This symbol may be optionally pushed before ending the parser.
    /// This is necessary to get correct error reporting and partial results
    /// in case of failure at the end of input, which Pomelo does not do
    /// on its own.
    ///
    /// Calling [`Parser::end_of_input`] without pushing this token
    /// may lead to incorrect error reporting and loss of partial results
    /// in case of a failure at this point.
    End;

    // Operator precedence
    %right Question;
    %left Or;
    %left And;
    %left Eq Ne;
    %left Lt Le Gt Ge;
    %left Plus Minus;
    %left Asterisk Slash Percent;
    %nonassoc Not;

    // ======================================
    //                GRAMMAR
    // ======================================

    // The starting nonterminal symbol
    // Accept the End symbol optionally
    stylesheet ::= sheet_part(s) End?                  { s }

    // Rules in the stylesheet
    sheet_part ::=                                     { Stylesheet::default() }
    sheet_part ::= sheet_part(mut s) rule(r)           { if !extra.recover() { s.0.push(r) } s }
    rule ::= selector(s) body(b)                       { StyleRule { selector: s, properties: b } }
    rule ::= error                                     { extra.shift_error(); StyleRule::default() }

    // Rule body (the part that is not a selector)
    body ::= OpenBrace proplist CloseBrace;
    body ::= OpenBrace proplist(l) End                 { extra.unterminated_rule(); l }
    proplist ::= proplist1;
    proplist ::= proplist1(mut l) clause(c)            { l.push(c); l }
    proplist1 ::=                                      { Vec::new() }
    proplist1 ::= proplist1(mut l) clause(c) Semicolon { l.push(c); l }
    clause ::= lvalue(l) Colon rvalue(r)               { StyleClause { key: l, value: r } }
    lvalue ::= Quoted(s)                               { StyleKey::Property(RawPropertyKey::QuotedProperty(s.to_owned())) }
    lvalue ::= Unquoted(s)                             { if is_variable_name(s) {
                                                             StyleKey::Variable(s.to_owned())
                                                         } else {
                                                             StyleKey::Property(RawPropertyKey::Property(s.to_owned()))
                                                       } }
    lvalue ::= Unquoted(f) Slash Unquoted|Quoted(s)    { StyleKey::Property(RawPropertyKey::FragmentProperty(f.to_owned(), s.to_owned())) }
    rvalue ::= rexpr;
    rvalue ::= Unquoted(s)                             { resolve_unquoted_expression(s).unwrap_or_else(|InvalidSymbol(s)| Expression::String(s)) }

    // Selectors
    selector ::= selector1;
    selector ::= selector1(s) extra(e)                 { s.with_extra(e) }
    selector1 ::= selector2;
    selector1 ::= selector2(s) EdgeMatcher             { s.selecting_edge() }
    selector2 ::= RootMatcher path(p)                  { Selector::from_path(p) }
    selector2 ::= path(p)                              { selector_from_not_root(p.0) }
    condition ::= If OpenParen expr CloseParen;
    condition ::= Colon Unquoted(s)                    { type_match_condition(s, true) }
    condition ::= Colon Quoted(s)                      { type_match_condition(s, false) }
    path ::=                                           { [].into() }
    path ::= path(mut p) segment(s)                    { p.0.push(s); p }
    path ::= path(mut p) index(e)                      { if let Expression::Int(i) = e {
                                                             p.0.push(SelectorSegment::Match(EdgeLabel::Index(i as usize).into()))
                                                         } else {
                                                             p.0.push(SelectorSegment::Match(EdgeMatcher::AnyIndex));
                                                             p.0.push(SelectorSegment::Condition(index_match_condition(e)));
                                                         }
                                                         p }
    segment ::= matcher(m)                             { SelectorSegment::Match(m) }
    segment ::= Many OpenParen path(p) CloseParen      { SelectorSegment::AnyNumberOfTimes(p) }
    segment ::= Alt OpenParen pathlist(l) CloseParen   { SelectorSegment::Branch(l) }
    segment ::= condition(c)                           { SelectorSegment::Condition(c) }
    pathlist ::= path(p)                               { vec![p] }
    pathlist ::= pathlist(mut l) Comma path(p)         { l.push(p); l }

    // Limited selectors
    limsel ::= limsel1;
    limsel ::= limsel1(s) extra(e)                     { s.with_extra(e) }
    limsel1 ::= limpath(p)                             { LimitedSelector::from_path(p) }
    limsel1 ::= OpenParen expr(o) CloseParen limpath(p) { LimitedSelector::from_path(p).with_origin(o) }
    limpath ::=                                        { Vec::new() }
    limpath ::= limpath(mut p) limseg(s)               { p.push(s); p }
    limseg ::= exact(e)                                { e.into() }
    limseg ::= index(e)                                { if let Expression::Int(i) = e { EdgeLabel::Index(i as usize).into() } else { LimitedEdgeMatcher::DynIndex(e) } }
    limseg ::= Quoted(s)                               { EdgeLabel::Named(s.to_owned(), 0).into() }

    // Matchers in selectors (both full and limited)
    matcher ::= Asterisk                               { EdgeMatcher::Any }
    matcher ::= OpenBracket CloseBracket               { EdgeMatcher::AnyIndex }
    matcher ::= Quoted(s)                              { EdgeMatcher::Named(s.to_owned()) }
    matcher ::= Percent                                { EdgeMatcher::AnyNamed }
    matcher ::= exact(e)                               { EdgeMatcher::Exact(e) }
    exact ::= Quoted(s) Hash Int(i)                    { EdgeLabel::Named(s.to_owned(), i as usize) }
    exact ::= Unquoted(s)                              { extra.try_or(edge_label_from_name(s).map_err(SyntaxError::InvalidEdgeLabel), EdgeLabel::Main) }
    extra ::= Extra                                    { String::new() }
    extra ::= Extra OpenParen Unquoted(s) CloseParen   { s.to_owned() }
    index ::= OpenBracket expr CloseBracket;

    // Expressions
    expr ::= rexpr;
    expr ::= Unquoted(s)                               { extra.try_or(resolve_unquoted_expression(s).map_err(SyntaxError::InvalidUnquoted), Expression::Unset) }
    rexpr ::= OpenParen expr CloseParen;
    rexpr ::= Quoted(s)                                { Expression::String(s.to_owned()) }
    rexpr ::= Int(i)                                   { Expression::Int(i) }
    rexpr ::= Unquoted(s) OpenParen expr(e) CloseParen { Uop(extra.try_or(unary_function_by_name(s).map_err(SyntaxError::InvalidFunction), UnaryPlus), e.into()) }
    rexpr ::= Plus expr(e) [Not]                       { Uop(UnaryPlus, e.into()) }
    rexpr ::= Minus expr(e) [Not]                      { Uop(UnaryMinus, e.into()) }
    rexpr ::= Not expr(e)                              { Uop(Not, e.into()) }
    rexpr ::= expr(l) Plus expr(r)                     { Bop(l.into(), BinaryPlus, r.into()) }
    rexpr ::= expr(l) Minus expr(r)                    { Bop(l.into(), BinaryMinus, r.into()) }
    rexpr ::= expr(l) Asterisk expr(r)                 { Bop(l.into(), Mul, r.into()) }
    rexpr ::= expr(l) Slash expr(r)                    { Bop(l.into(), Div, r.into()) }
    rexpr ::= expr(l) Percent expr(r)                  { Bop(l.into(), Mod, r.into()) }
    rexpr ::= expr(l) Eq expr(r)                       { Bop(l.into(), Eq, r.into()) }
    rexpr ::= expr(l) Ne expr(r)                       { Bop(l.into(), Ne, r.into()) }
    rexpr ::= expr(l) Lt expr(r)                       { Bop(l.into(), Lt, r.into()) }
    rexpr ::= expr(l) Le expr(r)                       { Bop(l.into(), Le, r.into()) }
    rexpr ::= expr(l) Gt expr(r)                       { Bop(l.into(), Gt, r.into()) }
    rexpr ::= expr(l) Ge expr(r)                       { Bop(l.into(), Ge, r.into()) }
    rexpr ::= expr(l) And expr(r)                      { Bop(l.into(), And, r.into()) }
    rexpr ::= expr(l) Or expr(r)                       { Bop(l.into(), Or, r.into()) }
    rexpr ::= expr(c) Question expr(t) Colon expr(f)   { Expression::Conditional(c.into(), t.into(), f.into()) }
    rexpr ::= At                                       { Expression::Select(LimitedSelector::default().into()) }
    rexpr ::= At OpenParen limsel(s) CloseParen        { Expression::Select(s.into()) }
}

/// Shorthand for constructing a selector from a path that does not
/// start at the root node.
///
/// This is done by prepending a [`SelectorSegment::anything_any_number_of_times`]
/// to the path.
fn selector_from_not_root(segments: impl IntoIterator<Item = SelectorSegment>) -> Selector {
    let starting_segment = SelectorSegment::anything_any_number_of_times();
    let segments = std::iter::once(starting_segment).chain(segments).collect();
    Selector::from_path(SelectorPath(segments))
}

/// Shorthand for constructing an expression that verifies
/// the type name of a node.
fn type_match_condition(type_name: &str, allow_special_names: bool) -> Expression {
    let resolved_special_name = if allow_special_names {
        node_type_class_by_name(type_name).ok()
    } else {
        None
    };
    if let Some(type_class) = resolved_special_name {
        // is-xxx(@)
        Expression::UnaryOperator(
            UnaryOperator::NodeIsA(type_class),
            Expression::Select(LimitedSelector::default().into()).into(),
        )
    } else {
        // typename(@) == --type-name
        Expression::BinaryOperator(
            Expression::UnaryOperator(
                UnaryOperator::NodeTypeName,
                Expression::Select(LimitedSelector::default().into()).into(),
            )
            .into(),
            BinaryOperator::Eq,
            Expression::String(type_name.to_owned()).into(),
        )
    }
}

/// Shorthand for constructing an expression that verifies the index of an edge
fn index_match_condition(index: Expression) -> Expression {
    Expression::BinaryOperator(
        Box::new(Expression::MagicVariable(MagicVariableKey::EdgeIndex)),
        BinaryOperator::Eq,
        index.into(),
    )
}

// Re-export types generated by Pomelo
pub use parser::{Parser, Token};

#[cfg(test)]
mod test {
    use super::{Token::*, *};
    use crate::mock_error_handler::ExpectErrors;

    #[test]
    fn empty_stylesheet() {
        Parser::new(ErrorManager::new(ExpectErrors::none().f()))
            .end_of_input()
            .expect("Empty input should be valid");
    }

    #[test]
    fn smallest_empty_rule() {
        let mut parser = Parser::new(ErrorManager::new(ExpectErrors::none().f()));
        let tokens = [RootMatcher, OpenBrace, CloseBrace];
        for token in tokens {
            parser
                .parse(token)
                .expect("Token should have been accepted");
        }
        parser
            .end_of_input()
            .expect("Parser should have been in valid terminating state");
    }

    #[test]
    fn end_before_rule_body() {
        let mut parser = Parser::new(ErrorManager::new(
            ExpectErrors::exact([SyntaxError::UnexpectedEnd]).f(),
        ));
        parser
            .parse(RootMatcher)
            .expect("Root matcher should be valid here");
        parser
            .parse(End)
            .expect("End token shoud have been accepted");
        let stylesheet = parser
            .end_of_input()
            .expect("Parser should have been in valid terminating state")
            .0;
        assert_eq!(stylesheet, Stylesheet::default());
    }

    #[test]
    fn end_inside_rule_body() {
        let mut parser = Parser::new(ErrorManager::new(
            ExpectErrors::exact([SyntaxError::UnterminatedRule]).f(),
        ));
        let tokens = [
            RootMatcher,
            OpenBrace,
            Unquoted("display"),
            Colon,
            Unquoted("none"),
            End,
        ];
        for token in tokens {
            parser
                .parse(token)
                .expect("Token should have been accepted");
        }
        let stylesheet = parser
            .end_of_input()
            .expect("Parser should have been in valid terminating state")
            .0;
        assert_eq!(
            stylesheet,
            Stylesheet(vec![StyleRule {
                selector: Selector::default(),
                properties: vec![StyleClause {
                    key: StyleKey::Property(RawPropertyKey::Property("display".to_owned())),
                    value: Expression::Unset
                }]
            }])
        );
    }

    #[test]
    fn push_unexpected_token() {
        let mut parser = Parser::new(ErrorManager::new(ExpectErrors::some().f()));
        let tokens = [RootMatcher, At, Hash, Slash, End];
        for token in tokens {
            parser
                .parse(token)
                .expect("Token should have been accepted");
        }
        parser
            .end_of_input()
            .expect("Parser should have been in valid terminating state");
    }
}
