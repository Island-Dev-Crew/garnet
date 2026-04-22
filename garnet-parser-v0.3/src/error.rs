//! Garnet v0.3 parse errors with miette diagnostic rendering.

use crate::token::Span;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ParseError {
    #[error("unexpected character '{ch}'")]
    #[diagnostic(help("this character is not part of Garnet's lexical grammar"))]
    UnexpectedChar {
        ch: char,
        #[label("here")]
        span: Span,
    },

    #[error("unterminated string literal")]
    #[diagnostic(help("add a closing '\"' to complete the string"))]
    UnterminatedString {
        #[label("string starts here")]
        span: Span,
    },

    #[error("invalid integer literal")]
    #[diagnostic(help("integer value is out of range for i64"))]
    InvalidInt {
        #[label("here")]
        span: Span,
    },

    #[error("invalid float literal")]
    #[diagnostic(help("float value is malformed or out of range"))]
    InvalidFloat {
        #[label("here")]
        span: Span,
    },

    #[error("expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
        #[label("found {found} here")]
        span: Span,
    },

    #[error("unexpected end of file, expected {expected}")]
    UnexpectedEof {
        expected: String,
        #[label("file ends here")]
        span: Span,
    },

    #[error("parse budget exceeded ({axis}: {actual} > limit {limit})")]
    #[diagnostic(help(
        "the parser refuses to process adversarially large or deeply-nested input. \
         if this is legitimate, construct a ParseBudget with a higher limit and use \
         parse_source_with_budget()."
    ))]
    BudgetExceeded {
        axis: &'static str,
        limit: usize,
        actual: usize,
        #[label("budget exceeded here")]
        span: Span,
    },
}

impl ParseError {
    pub fn unexpected_token(expected: &str, found: &str, span: Span) -> Self {
        ParseError::UnexpectedToken {
            expected: expected.to_string(),
            found: found.to_string(),
            span,
        }
    }

    pub fn unexpected_eof(expected: &str, span: Span) -> Self {
        ParseError::UnexpectedEof {
            expected: expected.to_string(),
            span,
        }
    }

    pub fn budget_exceeded(axis: &'static str, limit: usize, actual: usize, span: Span) -> Self {
        ParseError::BudgetExceeded {
            axis,
            limit,
            actual,
            span,
        }
    }
}
