//! `garnet-parser` — Rung 2 of the Garnet engineering ladder.
//!
//! Tokenizes and parses Mini-Spec v0.2 §2.1 (memory unit declarations) and
//! §4.1 (actor declarations with protocols and handlers). Per Mini-Spec
//! v0.2 §7, these are the two grammars a v0.2-compliant parser MUST handle.
//! §5.1–§5.3 (recursion guardrails) define normative MUST rules but do not
//! define a concrete grammar in v0.2; the parser is a structural no-op for
//! §5 and the README documents the gap for v0.3.
//!
//! ```
//! use garnet_parser::parse_source;
//!
//! let src = r#"
//! memory episodic conversations : Vector<Turn>
//!
//! actor Greeter {
//!   protocol hello(name: String) -> String
//!   on hello(name) {
//!     let g = "hi"
//!     g
//!   }
//! }
//! "#;
//!
//! let module = parse_source(src).expect("parses cleanly");
//! assert_eq!(module.items.len(), 2);
//! ```
//!
//! Anchor: *"Where there is no vision, the people perish." — Proverbs 29:18*

pub mod ast;
pub mod error;
pub mod grammar;
pub mod lexer;
pub mod parser;
pub mod token;

pub use ast::*;
pub use error::ParseError;
pub use lexer::Lexer;
pub use parser::Parser;
pub use token::{Span, Token, TokenKind};

/// Convenience entry point: lex + parse a complete source string into a
/// [`Module`]. Returns a span-attached [`ParseError`] on failure.
pub fn parse_source(src: &str) -> Result<Module, ParseError> {
    Parser::parse_source(src)
}

/// Convenience entry point: lex a complete source string into a token
/// stream (terminated by [`TokenKind::Eof`]).
pub fn lex_source(src: &str) -> Result<Vec<Token>, ParseError> {
    Lexer::new(src).lex()
}
