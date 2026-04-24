//! Garnet v0.3 parser cursor — token stream navigation and expect/eat helpers.
//!
//! **v3.3 hardening:** `depth` field + `enter_depth()` helper implement
//! the depth axis of `ParseBudget`. Grammar code calls `enter_depth()` at
//! the top of each recursive production to prevent ParensBomb / brace-bomb
//! stack overflow DOS.

use crate::budget::ParseBudget;
use crate::error::ParseError;
use crate::token::{describe_kind, Span, Token, TokenKind};
use std::cell::Cell;
use std::rc::Rc;

/// Static fallback token returned when the parser is asked to peek past the
/// end of an empty token stream. Cannot be `const` because `Token` contains
/// a non-const-constructible `Span`. Callers that mutate would never reach
/// this — `peek` returns `&Token`, never `&mut Token`.
fn eof_token() -> &'static Token {
    use std::sync::OnceLock;
    static EOF: OnceLock<Token> = OnceLock::new();
    EOF.get_or_init(|| Token {
        kind: TokenKind::Eof,
        span: Span::new(0, 0),
    })
}

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub budget: ParseBudget,
    /// Current nesting depth of recursive productions. `Rc<Cell<usize>>`
    /// so `DepthGuard` can hold an independent clone of the Rc, which
    /// doesn't borrow `self` and thus lets grammar code call recursive
    /// `&mut Parser` productions while the guard is live.
    /// See `budget::ParseBudget::max_depth`.
    depth: Rc<Cell<usize>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self::with_budget(tokens, ParseBudget::default())
    }

    pub fn with_budget(tokens: Vec<Token>, budget: ParseBudget) -> Self {
        Self {
            tokens,
            pos: 0,
            budget,
            depth: Rc::new(Cell::new(0)),
        }
    }

    /// Increment the nesting depth counter and return a RAII guard that
    /// decrements it on drop. Fails with `BudgetExceeded` if the new
    /// depth exceeds `budget.max_depth`.
    ///
    /// Takes `&self` and returns a guard that holds an independent
    /// `Rc<Cell>` clone — so grammar code can call `parse_x(p: &mut Parser)`
    /// recursively while the guard is live without running into the
    /// borrow checker.
    ///
    /// A 100-deep `((((...` chain now fails with a clean error instead
    /// of blowing the stack.
    pub fn enter_depth(&self) -> Result<DepthGuard, ParseError> {
        let new_depth = self.depth.get() + 1;
        let span = self.peek().span;
        self.budget.check_depth(new_depth, span)?;
        self.depth.set(new_depth);
        Ok(DepthGuard {
            depth: Rc::clone(&self.depth),
        })
    }

    /// Current depth — for tests.
    pub fn depth(&self) -> usize {
        self.depth.get()
    }
}

/// RAII guard that decrements the parser's depth counter when dropped.
/// Returned by `Parser::enter_depth`. Holds an independent `Rc<Cell>`
/// clone so it does not borrow the `Parser` itself.
pub struct DepthGuard {
    depth: Rc<Cell<usize>>,
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        self.depth.set(self.depth.get().saturating_sub(1));
    }
}

impl Parser {
    /// Look at the current token without consuming it. If the parser is
    /// positioned past the end of an empty token stream, returns a static
    /// EOF token instead of panicking. The lexer always emits a trailing
    /// EOF, so the fallback only triggers on a deliberately empty input.
    pub fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .or_else(|| self.tokens.last())
            .unwrap_or_else(|| eof_token())
    }

    /// Convenience: peek at the current token's kind.
    pub fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    /// Look ahead n tokens from current position.
    pub fn peek_nth(&self, n: usize) -> &Token {
        self.tokens
            .get(self.pos + n)
            .or_else(|| self.tokens.last())
            .unwrap_or_else(|| eof_token())
    }

    /// Consume the current token and return it.
    pub fn bump(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or_else(|| Token {
            kind: TokenKind::Eof,
            span: self.eof_span(),
        });
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    /// If the current token matches `kind`, consume it and return true.
    pub fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.peek_kind().matches(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Expect the current token to match `kind`, consuming it. Error if not.
    pub fn expect(&mut self, kind: &TokenKind, context: &str) -> Result<Token, ParseError> {
        if self.peek_kind().matches(kind) {
            Ok(self.bump())
        } else {
            let tok = self.peek();
            if matches!(tok.kind, TokenKind::Eof) {
                Err(ParseError::unexpected_eof(
                    &format!("{} in {}", describe_kind(kind), context),
                    tok.span,
                ))
            } else {
                Err(ParseError::unexpected_token(
                    &format!("{} in {}", describe_kind(kind), context),
                    describe_kind(&tok.kind),
                    tok.span,
                ))
            }
        }
    }

    /// Expect an identifier token, consuming it. Returns (name, span).
    pub fn expect_ident(&mut self, context: &str) -> Result<(String, Span), ParseError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                let span = tok.span;
                self.bump();
                Ok((name, span))
            }
            TokenKind::Eof => Err(ParseError::unexpected_eof(
                &format!("identifier in {}", context),
                tok.span,
            )),
            _ => Err(ParseError::unexpected_token(
                &format!("identifier in {}", context),
                describe_kind(&tok.kind),
                tok.span,
            )),
        }
    }

    /// Skip newline and semicolon tokens (statement separators).
    pub fn skip_separators(&mut self) {
        while matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Semi) {
            self.bump();
        }
    }

    /// Are we at end of file?
    pub fn at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    /// Get a span for EOF (last position in source).
    fn eof_span(&self) -> Span {
        if let Some(last) = self.tokens.last() {
            Span::new(last.span.end(), 0)
        } else {
            Span::new(0, 0)
        }
    }

    /// Get the span of the previously consumed token.
    pub fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::new(0, 0)
        }
    }
}
