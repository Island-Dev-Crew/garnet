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
pub mod cst;
pub mod edition;
pub mod error;
pub mod grammar;
pub mod lexer;
pub mod parser;
pub mod token;

use ast::Module;
pub use budget::ParseBudget;
pub use edition::{Edition, EditionError};
use error::ParseError;
use lexer::Lexer;
use parser::Parser;
use token::{Span, Token, TokenKind};

/// Lex and parse a Garnet source string into both a Module AST and a CST root.
pub fn parse_source_cst(src: &str) -> Result<(Module, cst::CstNode), ParseError> {
    parse_source_cst_with_budget(src, ParseBudget::default())
}

/// Lex and parse a Garnet source string into both AST and CST with a caller-supplied budget.
pub fn parse_source_cst_with_budget(
    src: &str,
    budget: ParseBudget,
) -> Result<(Module, cst::CstNode), ParseError> {
    parse_source_cst_with_budget_and_edition(src, budget, Edition::default())
}

/// Lex and parse into AST + CST under an explicit [`Edition`] (default budget).
pub fn parse_source_cst_with_edition(
    src: &str,
    edition: Edition,
) -> Result<(Module, cst::CstNode), ParseError> {
    parse_source_cst_with_budget_and_edition(src, ParseBudget::default(), edition)
}

/// Lex and parse into AST + CST with a caller-supplied budget and [`Edition`].
pub fn parse_source_cst_with_budget_and_edition(
    src: &str,
    budget: ParseBudget,
    edition: Edition,
) -> Result<(Module, cst::CstNode), ParseError> {
    budget.check_source_bytes(src.len())?;
    let tokens = lex_source_with_budget_and_edition(src, budget, edition)?;
    check_token_nesting(&tokens, budget)?;
    let filtered_tokens: Vec<Token> = tokens
        .iter()
        .filter(|t| !matches!(t.kind, TokenKind::Whitespace(_) | TokenKind::Comment(_)))
        .cloned()
        .collect();
    let mut p = Parser::with_budget(filtered_tokens, budget);
    let (safe, items) = grammar::parse_items(&mut p)?;
    let span = Span::new(0, src.len());
    let module = Module { safe, items, span };
    let cst_root = cst::CstNode::from_ast_and_tokens(&module, tokens);
    Ok((module, cst_root))
}

/// Lex and parse a Garnet source string into a Module AST using the
/// default `ParseBudget`. For bespoke budgets (e.g., in fuzz harnesses),
/// use `parse_source_with_budget()`.
pub fn parse_source(src: &str) -> Result<Module, ParseError> {
    parse_source_with_budget(src, ParseBudget::default())
}

/// Lex and parse a Garnet source string with a caller-supplied budget.
pub fn parse_source_with_budget(src: &str, budget: ParseBudget) -> Result<Module, ParseError> {
    parse_source_with_budget_and_edition(src, budget, Edition::default())
}

/// Lex and parse a Garnet source string under an explicit [`Edition`] (default
/// budget). Source valid in two editions yields a byte-identical `Module` (the
/// one-canonical-IR invariant); editions gate only the lexical surface.
pub fn parse_source_with_edition(src: &str, edition: Edition) -> Result<Module, ParseError> {
    parse_source_with_budget_and_edition(src, ParseBudget::default(), edition)
}

/// Lex and parse a Garnet source string with a caller-supplied budget and
/// [`Edition`].
pub fn parse_source_with_budget_and_edition(
    src: &str,
    budget: ParseBudget,
    edition: Edition,
) -> Result<Module, ParseError> {
    budget.check_source_bytes(src.len())?;
    let tokens = lex_source_with_budget_and_edition(src, budget, edition)?;
    check_token_nesting(&tokens, budget)?;
    let filtered_tokens: Vec<Token> = tokens
        .iter()
        .filter(|t| !matches!(t.kind, TokenKind::Whitespace(_) | TokenKind::Comment(_)))
        .cloned()
        .collect();
    let mut p = Parser::with_budget(filtered_tokens, budget);
    let (safe, items) = grammar::parse_items(&mut p)?;
    let span = Span::new(0, src.len());
    Ok(Module { safe, items, span })
}

fn check_token_nesting(tokens: &[Token], budget: ParseBudget) -> Result<(), ParseError> {
    let mut depth = 0usize;
    for tok in tokens {
        match tok.kind {
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                depth = depth.saturating_add(1);
                budget.check_depth(depth, tok.span)?;
            }
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Lex a Garnet source string into a token stream using the default budget.
pub fn lex_source(src: &str) -> Result<Vec<Token>, ParseError> {
    lex_source_with_budget(src, ParseBudget::default())
}

/// Lex with a caller-supplied budget.
pub fn lex_source_with_budget(src: &str, budget: ParseBudget) -> Result<Vec<Token>, ParseError> {
    lex_source_with_budget_and_edition(src, budget, Edition::default())
}

/// Lex with a caller-supplied budget under an explicit [`Edition`]. The edition
/// governs only edition-gated reserved words (see [`Edition::is_reserved_ident`]).
pub fn lex_source_with_budget_and_edition(
    src: &str,
    budget: ParseBudget,
    edition: Edition,
) -> Result<Vec<Token>, ParseError> {
    budget.check_source_bytes(src.len())?;
    let mut lexer = Lexer::with_budget_and_edition(src, budget, edition);
    lexer.lex()
}
