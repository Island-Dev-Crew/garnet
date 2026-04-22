//! # Garnet Parser v0.3
//!
//! Lexer and recursive-descent parser for the Garnet programming language,
//! covering all 90 EBNF productions from Mini-Spec v0.3.
//!
//! Rung 2.1 of the Garnet engineering ladder.
//!
//! ## Usage
//!
//! ```
//! let src = r#"def greet(name) { "Hello, #{name}!" }"#;
//! let module = garnet_parser::parse_source(src).expect("should parse");
//! assert_eq!(module.items.len(), 1);
//! ```

pub mod ast;
pub mod budget;
pub mod error;
pub mod grammar;
pub mod lexer;
pub mod parser;
pub mod token;

use ast::Module;
pub use budget::ParseBudget;
use error::ParseError;
use lexer::Lexer;
use parser::Parser;
use token::{Span, Token};

/// Lex and parse a Garnet source string into a Module AST using the
/// default `ParseBudget`. For bespoke budgets (e.g., in fuzz harnesses),
/// use `parse_source_with_budget()`.
pub fn parse_source(src: &str) -> Result<Module, ParseError> {
    parse_source_with_budget(src, ParseBudget::default())
}

/// Lex and parse a Garnet source string with a caller-supplied budget.
pub fn parse_source_with_budget(src: &str, budget: ParseBudget) -> Result<Module, ParseError> {
    budget.check_source_bytes(src.len())?;
    let tokens = lex_source_with_budget(src, budget)?;
    let mut p = Parser::with_budget(tokens, budget);
    let (safe, items) = grammar::parse_items(&mut p)?;
    let span = Span::new(0, src.len());
    Ok(Module { safe, items, span })
}

/// Lex a Garnet source string into a token stream using the default budget.
pub fn lex_source(src: &str) -> Result<Vec<Token>, ParseError> {
    lex_source_with_budget(src, ParseBudget::default())
}

/// Lex with a caller-supplied budget.
pub fn lex_source_with_budget(src: &str, budget: ParseBudget) -> Result<Vec<Token>, ParseError> {
    budget.check_source_bytes(src.len())?;
    let mut lexer = Lexer::with_budget(src, budget);
    lexer.lex()
}
