//! Tokens of lexical analysis.

use derive_more::{Debug, Display, Error, From};
use logos::Logos;

/// Error type emited by a lexer if a token fails to parse.
#[derive(Clone, PartialEq, Eq, Debug, Display, From, Error, Default)]
pub enum LexerError {
    /// Lexer failed without providing any additional information.
    #[default]
    #[display("unspecified lexer error")]
    Generic,

    /// Integer literal is too big to be stored in an integer variable.
    #[display("invalid integer literal: {_0}")]
    ParseIntError(std::num::ParseIntError),

    /// Start of an unquoted token, but missing required alphabetic character.
    #[display("invalid unquoted token")]
    InvalidUnquoted,

    /// Start of a numeric literal followed immediately by alphabetic characters.
    #[display("alphabetic character in integer literal")]
    AlphaCharacterInNumber,

    /// Quoted string literal did not end before the end of a line or the end of input.
    #[display("unterminated string literal")]
    UnterminatedQuoted,
}

/// Additional data used by the lexer to track position in source.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SourceLocationInformation {
    /// Zero-based index of the line being read.
    pub line_index: usize,
    /// One-based offset of the last linefeed
    /// from the start of the input, in bytes.
    ///
    /// Value of zero means no linefeed has been encountered.
    pub line_offset: usize,
}

/// Tokens emited by the lexer.
///
/// Because it implements the [`Logos`] trait,
/// it provides a [`Token::lexer`] function for constructing
/// a lexer.
#[derive(Logos, Clone, Copy, PartialEq, Eq, Debug)]
#[logos(error = LexerError)]
#[logos(extras = SourceLocationInformation)]
#[logos(skip r"[ \r\t]|//[^\n]*")]
pub enum Token<'s> {
    // =========================================
    //             LOCATION TRACKING
    // =========================================
    #[regex(r"\n|/\*[^*]*\*+([^/][^*]*\*+)*/", |lex| {
        // Logos does not count lines on its own, so we have to do it manually
        for (offset, _) in lex.slice().bytes().enumerate().filter(|(_, c)| *c == b'\n') {
            // Find all newlines in the matched slice, if any, and increment the line counter
            lex.extras.line_index += 1;
            // Move the last linefeed offset
            lex.extras.line_offset = offset + lex.span().start + 1;
        }
        // Tell Logos not to emit a token
        logos::Filter::Skip
    })]
    // =========================================
    //                 LITERALS
    // =========================================
    /// Unquoted token that can be used as an identifier
    /// or coerced to a string literal.
    ///
    /// Tokens consists of alphanumeric characters and dashes.
    /// Leading dashes are allowed, but at least one non-dash character must be present,
    /// and the first one must be alphabetic.
    ///
    /// ## Examples
    /// Valid tokens:
    /// ```text
    /// hello
    /// hello-world
    /// --variable
    /// --variable-with-multiple-words
    /// --a123
    /// --a-123
    /// ```
    /// Invalid tokens:
    /// ```text
    /// 123
    /// 123abc
    /// --123
    /// --123abc
    /// --
    /// ```
    #[regex(r"-*[a-zA-Z][a-zA-Z\d\-]*")]
    #[regex(r"-{2,}", |_| Err(LexerError::InvalidUnquoted))]
    #[debug("{_0}")]
    Unquoted(&'s str),

    /// Double-quoted string literal.
    #[regex(r#""[^"\n]*""#, |lex| &lex.slice()[1..(lex.slice().len() - 1)])]
    #[regex(r#""[^"\n]*"#, |_| Err(LexerError::UnterminatedQuoted))]
    #[debug("{_0:?}")]
    Quoted(&'s str),

    /// Decimal integer literal.
    #[regex(r"\d+", |lex| lex.slice().parse())]
    #[regex(r"\d+[a-zA-Z][a-zA-Z\d]*", |_| Err(LexerError::AlphaCharacterInNumber))]
    #[debug("{_0}")]
    Int(u64),

    // =========================================
    //                 MATCHERS
    // =========================================
    /// Selector matcher that specifies
    /// a restriction condition. It is a separate token from the
    /// other matchers as it has a unique syntax.
    ///
    /// ## Examples
    /// ```text
    /// ::.if(@ == 0) {
    ///   property: value;
    /// }
    /// ```
    #[token(".if")]
    #[debug(".if")]
    RestrictMatcher,

    /// Selector matcher that specifies an iteration.
    #[token(".many")]
    #[debug(".many")]
    ManyMatcher,

    /// Selector matcher that specifies a branching.
    #[token(".alt")]
    #[debug(".alt")]
    AltMatcher,

    /// Selector matcher that matches the root element.
    /// Must be used at the start of a selector.
    /// Not allowed in limited selectors.
    #[token("::")]
    #[debug("::")]
    RootMatcher,

    /// Pseudo-element that selects an edge leading to the matched
    /// node instead of the node itself.
    #[token("::edge")]
    #[debug("::edge")]
    EdgeMatcher,

    /// Pseudo-element that selects a virtual element attached
    /// to the matched node or edge instead of the node or edge itself.
    #[token("::extra")]
    #[debug("::extra")]
    ExtraMatcher,

    // =========================================
    //                OPERATORS
    // =========================================
    #[token("+")]
    #[debug("[+]")]
    Plus,

    #[token("-")]
    #[debug("[-]")]
    Minus,

    #[token("!")]
    #[debug("!")]
    Not,

    #[token("*")]
    #[debug("*")]
    Asterisk,

    #[token("/")]
    #[debug("/")]
    Slash,

    #[token("%")]
    #[debug("%")]
    Percent,

    #[token("==")]
    #[debug("==")]
    Equals,

    #[token("!=")]
    #[debug("!=")]
    NotEquals,

    #[token("<")]
    #[debug("[<]")]
    Less,

    #[token(">")]
    #[debug("[>]")]
    Greater,

    #[token("<=")]
    #[debug("[<=]")]
    LessEquals,

    #[token(">=")]
    #[debug("[>=]")]
    GreaterEquals,

    #[token("&&")]
    #[debug("&&")]
    DoubleAnd,

    #[token("||")]
    #[debug("||")]
    DoubleOr,

    #[token("?")]
    #[debug("?")]
    Question,

    /// Operator that introduces a
    /// [`Select`](aili_style::stylesheet::expression::Expression::Select)
    /// expression.
    ///
    /// ## Syntax
    /// ```text
    /// @
    ///
    /// @( <limited-selector> )
    /// ```
    ///
    /// ## Examples
    /// ```text
    /// // Use the value of an element in a condition
    /// ::.if(@ == 0) {
    ///   display: unset;
    /// }
    ///
    /// // Check whether an element exists in a condition
    /// ::.if(@("a")) {
    ///   display: unset;
    /// }
    ///
    /// // Use the value of an element for a display property
    /// :: "a" {
    ///   value: @;
    /// }
    /// ```
    #[token("@")]
    #[debug("@")]
    At,

    // =========================================
    //                DELIMITERS
    // =========================================
    #[token(";")]
    #[debug("[;]")]
    Semicolon,

    #[token(",")]
    #[debug("[,]")]
    Comma,

    #[token(":")]
    #[debug("[:]")]
    Colon,

    #[token("{")]
    #[debug("[{{]")]
    OpenBrace,

    #[token("}")]
    #[debug("[}}]")]
    CloseBrace,

    #[token("(")]
    #[debug("[(]")]
    OpenParen,

    #[token(")")]
    #[debug("[)]")]
    CloseParen,

    #[token("[")]
    #[debug("<[>")]
    OpenBracket,

    #[token("]")]
    #[debug("<]>")]
    CloseBracket,

    #[token("#")]
    #[debug("#")]
    Hash,
}

#[cfg(test)]
mod test {
    use super::{
        LexerError::*,
        SourceLocationInformation,
        Token::{self, *},
    };
    use logos::Logos;

    #[test]
    fn valid_unquoted_tokens() {
        let tokens = Token::lexer("a abc  -b -a- -abc-def --xyz  abc0 -a0b")
            .collect::<Result<Vec<_>, _>>()
            .expect("Tokens should have parsed successfully");
        assert_eq!(
            tokens,
            vec![
                Unquoted("a"),
                Unquoted("abc"),
                Unquoted("-b"),
                Unquoted("-a-"),
                Unquoted("-abc-def"),
                Unquoted("--xyz"),
                Unquoted("abc0"),
                Unquoted("-a0b")
            ]
        );
    }

    #[test]
    fn invalid_unquoted_tokens() {
        let tokens = Token::lexer("-01 -- 123abc ---").collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Ok(Minus),
                Ok(Int(1)),
                Err(InvalidUnquoted),
                Err(AlphaCharacterInNumber),
                Err(InvalidUnquoted),
            ]
        );
    }

    #[test]
    fn valid_quoted_strings() {
        let tokens = Token::lexer("\"\"  \" \" \"a\" \"abc\"")
            .collect::<Result<Vec<_>, _>>()
            .expect("Tokens should have parsed");
        assert_eq!(
            tokens,
            vec![Quoted(""), Quoted(" "), Quoted("a"), Quoted("abc")]
        );
    }

    #[test]
    fn invalid_unterminated_quoted_string() {
        let tokens = Token::lexer("\"abc\n \"def\" \"xyz").collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                Err(UnterminatedQuoted),
                Ok(Quoted("def")),
                Err(UnterminatedQuoted)
            ]
        );
    }

    #[test]
    fn whitespace() {
        let tokens = Token::lexer("1  2 \n\t 3 \r\n4  \n")
            .collect::<Result<Vec<_>, _>>()
            .expect("Tokens should have parsed");
        assert_eq!(tokens, vec![Int(1), Int(2), Int(3), Int(4)]);
    }

    #[test]
    fn comments() {
        let tokens =
            Token::lexer("1// abc \n //* */ x\n def /* xyz \n \nw*/ 4 // 42").collect::<Vec<_>>();
        assert_eq!(tokens, vec![Ok(Int(1)), Ok(Unquoted("def")), Ok(Int(4))]);
    }

    #[test]
    fn tricky_block_comments() {
        let tokens = Token::lexer(
            "1 /**/ 2 /** */ 3 /* **/ 4 /***/ 5 /*/*/ 6 /*//*/ 7 /*/**/ 8 /** ***/ 9 /** /***/ 10",
        )
        .collect::<Vec<_>>();
        assert_eq!(tokens, (1..=10).map(Int).map(Ok).collect::<Vec<_>>());
    }

    #[test]
    fn unterminated_block_comments() {
        let test_cases = [
            "/*  *", "/**", "/*/*", "/*", "/* ", "/** ", "/*/* ", "/*/", "/* * ",
        ];
        for case in test_cases {
            Token::lexer(case)
                .collect::<Result<Vec<_>, _>>()
                .expect_err("Parsing should have failed");
        }
    }

    #[test]
    fn invalid_numeric_literals() {
        let tokens = Token::lexer("99999999999999999999 123abc123").collect::<Vec<_>>();
        assert!(matches!(
            tokens[..],
            [Err(ParseIntError(_)), Err(AlphaCharacterInNumber)]
        ));
    }

    #[test]
    fn source_locations() {
        let mut lexer = Token::lexer("a \nb // x\nc /* \n y \n */ d \n e");
        let expected_source_locations = [
            SourceLocationInformation {
                line_index: 0,
                line_offset: 0,
            },
            SourceLocationInformation {
                line_index: 1,
                line_offset: 3,
            },
            SourceLocationInformation {
                line_index: 2,
                line_offset: 10,
            },
            SourceLocationInformation {
                line_index: 4,
                line_offset: 20,
            },
            SourceLocationInformation {
                line_index: 5,
                line_offset: 27,
            },
        ];
        for expected in expected_source_locations {
            lexer
                .next()
                .expect("Lexer should not yet be at the end of input")
                .expect("Token should have parsed successfully");
            assert_eq!(lexer.extras, expected);
        }
        assert!(
            lexer.next().is_none(),
            "Lexer should have readhed the end of input"
        );
    }
}
