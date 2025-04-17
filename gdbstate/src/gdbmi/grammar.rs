//! Lexical and syntactic analysis of outputs from GDB.

use super::raw_output::*;
use derive_more::{Debug, Display, Error};
use logos::Logos;
use pomelo::pomelo;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display, Error, Default)]
#[display("input was not recognized as a GDB/MI record")]
pub struct ParseError;

/// Parses a single line of
/// [GDB/MI](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI.html)
/// output.
///
/// Result records (`^`) and asynchronous execution records (`*`) are supported.
pub fn parse_gdbmi_record(input: &str) -> Result<Record, ParseError> {
    let lexer = parser::Token::lexer(input);
    let mut parser = parser::Parser::new();
    for token in lexer {
        parser.parse(token?)?;
    }
    parser.end_of_input()
}

pomelo! {
    %include {
        use super::{*, Debug};
    }

    %error ParseError;

    %token
    /// Type of tokens accepted by the syntactic parser.
    #[derive(Clone, PartialEq, Eq, Debug, Logos)]
    #[logos(error = ParseError)]
    pub enum Token<'s> {};

    // Underlying types of nonterminal symbols
    %type output         Record;
    %type record         Record;
    %type exec_record    AsyncExecRecord;
    %type exec_record1   AsyncExecRecord;
    %type exec_class     AsyncExecClass;
    %type result_record  ResultRecord;
    %type result_record1 ResultRecord;
    %type result_class   ResultClass;
    %type results        ResultTuple;
    %type result         ResultEntry;
    %type value          Value;
    %type values         Vec<Value>;

    // ========================================
    //            TERMINAL SYMBOLS
    // ========================================

    %type
    #[regex(r"[a-zA-Z][a-zA-Z0-9\-_]*")]
    #[debug("{_0}")]
    Unquoted &'s str;

    %type
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let string_contents = &lex.slice()[1..(lex.slice().len() - 1)];
        double_quoted_escapes(string_contents).map_err(|_| ParseError)
    })]
    #[debug("{_0:?}")]
    Quoted String;

    %type
    #[regex(r"\d+")]
    #[debug("{_0}")]
    Numeric &'s str;

    %type
    #[token("^")]
    #[debug("^")]
    Caret;

    %type
    #[token("*")]
    #[debug("*")]
    Asterisk;

    %type
    #[token(",")]
    #[debug("<,>")]
    Comma;

    %type
    #[token("=")]
    #[debug("=")]
    Equals;

    %type
    #[token("[")]
    #[debug("<[>")]
    OpenBracket;

    %type
    #[token("]")]
    #[debug("<]>")]
    CloseBracket;

    %type
    #[token("{")]
    #[debug("<{{>")]
    OpenBrace;

    %type
    #[token("}")]
    #[debug("<}}>")]
    CloseBrace;

    %type
    #[regex(r"\n|\r|\r\n")]
    #[debug("<nl>")]
    Eol;

    // ========================================
    //                 GRAMMAR
    // ========================================

    // Starting nonterminal symbol
    output ::= record(r) Eol?                                { r }

    // Output records
    // Only result records are parsed, other kinds are not needed for testing
    record ::= result_record(r)                              { Record::Result(r) }
    record ::= exec_record(r)                                { Record::AsyncExec(r) }

    // Result record
    result_record ::= result_record1;
    result_record ::= result_record1(mut r) Comma results(e) { r.results = e; r }
    result_record1 ::= Numeric?(n) Caret result_class(c)     { ResultRecord { token: n.map(str::to_owned), result_class: c, results: ResultTuple::default() } }
    result_class ::= Unquoted(s)                             { s.parse().map_err(|_| ParseError)? }

    // Async-exec record
    exec_record ::= exec_record1;
    exec_record ::= exec_record1(mut r) Comma results(e)     { r.results = e; r }
    exec_record1 ::= Asterisk exec_class(c)                  { AsyncExecRecord { async_exec_class: c, results: ResultTuple::default() } }
    exec_class ::= Unquoted(s)                               { s.parse().map_err(|_| ParseError)? }

    // Parameters returned together with a successful result record
    results ::= result(e)                                    { ResultTuple(vec![e]) }
    results ::= results(mut r) Comma result(e)               { r.0.push(e); r }
    result ::= Unquoted(k) Equals value(v)                   { ResultEntry { key: k.to_owned(), value: v } }
    values ::= value(e)                                      { vec![e] }
    values ::= values(mut v) Comma value(e)                  { v.push(e); v }
    value ::= Quoted(s)                                      { Value::Const(s.to_owned()) }
    value ::= OpenBrace results?(r) CloseBrace               { Value::Tuple(r.unwrap_or_default()) }
    value ::= OpenBracket results(r) CloseBracket            { Value::TupleList(r) }
    value ::= OpenBracket values?(r) CloseBracket            { Value::List(r.unwrap_or_default()) }
}

/// Resolves escape sequences in a string literal.
fn double_quoted_escapes(literal: &str) -> Result<String, ParseError> {
    let mut output = String::with_capacity(literal.len());
    let mut input = literal.chars().peekable();
    while let Some(c) = input.next() {
        if c != '\\' {
            output.push(c);
            continue;
        }
        match input.next() {
            Some('a') => output.push('\x07'),
            Some('b') => output.push('\x08'),
            Some('t') => output.push('\t'),
            Some('n') => output.push('\n'),
            Some('v') => output.push('\x0b'),
            Some('f') => output.push('\x0c'),
            Some('r') => output.push('\r'),
            Some('"') => output.push('"'),
            Some('\'') => output.push('\''),
            Some('\\') => output.push('\\'),
            Some('x') => {
                let mut code_point = input
                    .next()
                    .and_then(|c| c.to_digit(16))
                    .ok_or(ParseError)?;
                if let Some(c) = input.peek().and_then(|c| c.to_digit(16)) {
                    // Eat the character if it is good
                    input.next();
                    // Add the extra character to the code point
                    code_point = code_point * 16 + c;
                }
                if code_point > 0x7f {
                    return Err(ParseError);
                }
                output.push(code_point as u8 as char);
            }
            Some(c) if c.is_digit(8) => {
                let mut code_point = c.to_digit(8).unwrap();
                for _ in 0..=1 {
                    if let Some(c) = input.peek().and_then(|c| c.to_digit(8)) {
                        // Eat the character if it is good
                        input.next();
                        // Add the extra character to the code point
                        code_point = code_point * 8 + c;
                    }
                }
                if code_point > 0x7f {
                    return Err(ParseError);
                }
                output.push(code_point as u8 as char);
            }
            _ => return Err(ParseError),
        }
    }
    Ok(output)
}

#[derive(Clone, Debug, Display, Error)]
#[display("{_0:?} is not a valid result class")]
#[error(ignore)]
pub struct InvalidResultClass(String);

impl std::str::FromStr for ResultClass {
    type Err = InvalidResultClass;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "done" => Ok(Self::Done),
            "running" => Ok(Self::Running),
            "connected" => Ok(Self::Connected),
            "error" => Ok(Self::Error),
            "exit" => Ok(Self::Exit),
            _ => Err(InvalidResultClass(s.into())),
        }
    }
}

impl std::str::FromStr for AsyncExecClass {
    type Err = InvalidResultClass;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "running" => Ok(Self::Running),
            "stopped" => Ok(Self::Stopped),
            _ => Err(InvalidResultClass(s.into())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn result_record_with_no_data() {
        let result =
            parse_gdbmi_record("^done\r\n").expect("Input should have parsed successfully");
        let expected = ResultRecord {
            token: None,
            result_class: ResultClass::Done,
            results: ResultTuple::default(),
        }
        .into();
        assert_eq!(result, expected);
    }

    #[test]
    fn result_record_different_class() {
        let result =
            parse_gdbmi_record("^exit\r\n").expect("Input should have parsed successfully");
        let expected = ResultRecord {
            token: None,
            result_class: ResultClass::Exit,
            results: ResultTuple::default(),
        }
        .into();
        assert_eq!(result, expected);
    }

    #[test]
    fn result_record_invalid_class() {
        parse_gdbmi_record("^none\r\n").expect_err("Input should not have parsed");
    }

    #[test]
    fn result_record_with_token() {
        let result =
            parse_gdbmi_record("123^done\r\n").expect("Input should have parsed successfully");
        let expected = ResultRecord {
            token: Some("123".to_owned()),
            result_class: ResultClass::Done,
            results: ResultTuple::default(),
        }
        .into();
        assert_eq!(result, expected);
    }

    #[test]
    fn result_record_with_value() {
        let result = parse_gdbmi_record("^done,value=\"1\"\r\n")
            .expect("Input should have parsed successfully");
        let expected = ResultRecord {
            token: None,
            result_class: ResultClass::Done,
            results: ResultTuple(vec![ResultEntry {
                key: "value".to_owned(),
                value: Value::Const("1".to_owned()),
            }]),
        }
        .into();
        assert_eq!(result, expected);
    }

    #[test]
    fn result_record_with_tuples() {
        let result = parse_gdbmi_record("^done,a={},b={a=\"1\",b=\"2\"}")
            .expect("Input should have parsed successfully");
        let expected = ResultRecord {
            token: None,
            result_class: ResultClass::Done,
            results: ResultTuple(vec![
                ResultEntry {
                    key: "a".to_owned(),
                    value: Value::Tuple(ResultTuple::default()),
                },
                ResultEntry {
                    key: "b".to_owned(),
                    value: Value::Tuple(ResultTuple(vec![
                        ResultEntry {
                            key: "a".to_owned(),
                            value: Value::Const("1".to_owned()),
                        },
                        ResultEntry {
                            key: "b".to_owned(),
                            value: Value::Const("2".to_owned()),
                        },
                    ])),
                },
            ]),
        }
        .into();
        assert_eq!(result, expected);
    }

    #[test]
    fn result_record_with_lists() {
        let result = parse_gdbmi_record("^done,a=[],b=[\"1\",\"2\"],c=[a=\"1\",b=\"2\"]")
            .expect("Input should have parsed successfully");
        let expected = ResultRecord {
            token: None,
            result_class: ResultClass::Done,
            results: ResultTuple(vec![
                ResultEntry {
                    key: "a".to_owned(),
                    value: Value::List(Vec::new()),
                },
                ResultEntry {
                    key: "b".to_owned(),
                    value: Value::List(vec![
                        Value::Const("1".to_owned()),
                        Value::Const("2".to_owned()),
                    ]),
                },
                ResultEntry {
                    key: "c".to_owned(),
                    value: Value::TupleList(ResultTuple(vec![
                        ResultEntry {
                            key: "a".to_owned(),
                            value: Value::Const("1".to_owned()),
                        },
                        ResultEntry {
                            key: "b".to_owned(),
                            value: Value::Const("2".to_owned()),
                        },
                    ])),
                },
            ]),
        }
        .into();
        assert_eq!(result, expected);
    }

    #[test]
    fn escape_sequences() {
        let result = parse_gdbmi_record(r#"^done,value="\\\"\n\12\123\x0\x42\0\\""#)
            .expect("Input should have parsed successfully");
        let expected = ResultRecord {
            token: None,
            result_class: ResultClass::Done,
            results: ResultTuple(vec![ResultEntry {
                key: "value".to_owned(),
                value: Value::Const("\\\"\n\x0a\x53\0\x42\0\\".to_owned()),
            }]),
        }
        .into();
        assert_eq!(result, expected);
    }

    #[test]
    fn async_exec_record() {
        let result = parse_gdbmi_record(r#"*stopped,reason="breakpoint""#)
            .expect("Input should have parsed successfully");
        let expected = AsyncExecRecord {
            async_exec_class: AsyncExecClass::Stopped,
            results: ResultTuple(vec![ResultEntry {
                key: "reason".to_owned(),
                value: Value::Const("breakpoint".to_owned()),
            }]),
        }
        .into();
        assert_eq!(result, expected);
    }
}
