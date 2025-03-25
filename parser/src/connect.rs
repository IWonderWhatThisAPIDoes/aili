//! Facilitates the connection between [`lexer`](super::lexer)
//! and [`grammar`](super::grammar).

use crate::{grammar::Token as GrammarInputToken, lexer::Token as LexOutputToken};

impl<'a> From<LexOutputToken<'a>> for GrammarInputToken<'a> {
    /// Converts lexer [`Token`](LexOutputToken)s
    /// to parser [`Token`](GrammarInputToken)s.
    ///
    /// This is necessary because each of the libraries we use
    /// creates its own token enumeration and they must be unified.
    fn from(value: LexOutputToken<'a>) -> Self {
        use LexOutputToken::*;
        match value {
            Unquoted(s) => Self::Unquoted(s),
            Quoted(s) => Self::Quoted(s),
            Int(i) => Self::Int(i),
            RestrictMatcher => Self::If,
            ManyMatcher => Self::Many,
            AltMatcher => Self::Alt,
            RootMatcher => Self::RootMatcher,
            EdgeMatcher => Self::EdgeMatcher,
            ExtraMatcher => Self::Extra,
            Plus => Self::Plus,
            Minus => Self::Minus,
            Not => Self::Not,
            Asterisk => Self::Asterisk,
            Slash => Self::Slash,
            Percent => Self::Percent,
            Equals => Self::Eq,
            NotEquals => Self::Ne,
            Less => Self::Lt,
            Greater => Self::Gt,
            LessEquals => Self::Le,
            GreaterEquals => Self::Ge,
            DoubleAnd => Self::And,
            DoubleOr => Self::Or,
            At => Self::At,
            Semicolon => Self::Semicolon,
            Comma => Self::Comma,
            Colon => Self::Colon,
            OpenBrace => Self::OpenBrace,
            CloseBrace => Self::CloseBrace,
            OpenParen => Self::OpenParen,
            CloseParen => Self::CloseParen,
            OpenBracket => Self::OpenBracket,
            CloseBracket => Self::CloseBracket,
            Hash => Self::Hash,
            Question => Self::Question,
        }
    }
}
