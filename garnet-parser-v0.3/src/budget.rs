//! Parser resource budget — Security hardening #3 (v3.3).
//!
//! Garnet's lexer and recursive-descent parser grow allocations as they
//! consume input. Without a budget, an adversary can pin a CPU for seconds
//! to minutes with a trivially-small file:
//!
//! - **ParensBomb:** `((((...` × N creates a depth-N expression tree, each
//!   level adding a stack frame and heap allocation. A 100 MB file pins
//!   several seconds of CPU and hundreds of MB of memory.
//! - **StringBlimp:** a single 1 GB string literal forces the lexer to
//!   allocate a GB-scale buffer before any error surfaces.
//! - **CommentFlood:** a multi-MB comment block gets walked byte-by-byte.
//! - **UnicodeNestingZipper:** Unicode whitespace between nested constructs
//!   evades naive depth counters (not that we had any).
//!
//! `ParseBudget` caps each axis independently. The defaults are
//! *generous* — enough for any realistic program but small enough that
//! pathological inputs fail in milliseconds with a clear error.
//!
//! Budgets can be relaxed via `parse_source_with_budget()` for test
//! harnesses that generate intentionally-large fuzz inputs.

use crate::error::ParseError;
use crate::token::Span;

/// Triple-axis parser resource budget. All limits are inclusive upper
/// bounds — values equal to the limit are accepted, values greater are
/// rejected with `ParseError::BudgetExceeded`.
#[derive(Debug, Clone, Copy)]
pub struct ParseBudget {
    /// Maximum input size in bytes. Checked at `parse_source` entry
    /// before any allocation. Default: 64 MiB (2^26).
    pub max_source_bytes: usize,

    /// Maximum total tokens the lexer may produce. Checked after each
    /// token push. Default: ~1M tokens (2^20). A real Garnet module
    /// rarely exceeds 100k tokens.
    pub max_tokens: usize,

    /// Maximum recursion depth for nested parser productions
    /// (parens, braces, brackets, blocks, match arms, closures, etc.).
    /// Default: 256 — any plausible Garnet program uses < 30.
    pub max_depth: usize,

    /// Maximum byte length of a single string / symbol / numeric /
    /// comment / identifier literal. Default: 16 MiB (2^24).
    pub max_literal_bytes: usize,
}

impl Default for ParseBudget {
    fn default() -> Self {
        Self {
            max_source_bytes: 64 * 1024 * 1024,
            max_tokens: 1 << 20,
            max_depth: 256,
            max_literal_bytes: 16 * 1024 * 1024,
        }
    }
}

impl ParseBudget {
    /// All limits set to `usize::MAX`. Use only in test harnesses that
    /// need to produce adversarial fuzz inputs.
    pub fn unlimited() -> Self {
        Self {
            max_source_bytes: usize::MAX,
            max_tokens: usize::MAX,
            max_depth: usize::MAX,
            max_literal_bytes: usize::MAX,
        }
    }

    /// Check source-bytes budget. Called once at `parse_source` entry.
    pub fn check_source_bytes(&self, actual: usize) -> Result<(), ParseError> {
        if actual > self.max_source_bytes {
            return Err(ParseError::budget_exceeded(
                "source_bytes",
                self.max_source_bytes,
                actual,
                Span::new(0, 0),
            ));
        }
        Ok(())
    }

    /// Check token-count budget. Called after each lexer push.
    pub fn check_tokens(&self, actual: usize, span: Span) -> Result<(), ParseError> {
        if actual > self.max_tokens {
            return Err(ParseError::budget_exceeded(
                "tokens",
                self.max_tokens,
                actual,
                span,
            ));
        }
        Ok(())
    }

    /// Check literal-size budget. Called inside lex_string/lex_number/
    /// lex_ident_or_keyword/lex_symbol/comment-skip.
    pub fn check_literal_bytes(&self, actual: usize, span: Span) -> Result<(), ParseError> {
        if actual > self.max_literal_bytes {
            return Err(ParseError::budget_exceeded(
                "literal_bytes",
                self.max_literal_bytes,
                actual,
                span,
            ));
        }
        Ok(())
    }

    /// Check depth budget. Called by the depth-guard RAII helper.
    pub fn check_depth(&self, actual: usize, span: Span) -> Result<(), ParseError> {
        if actual > self.max_depth {
            return Err(ParseError::budget_exceeded(
                "depth",
                self.max_depth,
                actual,
                span,
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_budget_is_sensible_for_real_code() {
        let b = ParseBudget::default();
        assert_eq!(b.max_source_bytes, 64 * 1024 * 1024);
        assert_eq!(b.max_tokens, 1 << 20);
        assert_eq!(b.max_depth, 256);
        assert_eq!(b.max_literal_bytes, 16 * 1024 * 1024);
    }

    #[test]
    fn source_bytes_check_accepts_at_limit_rejects_above() {
        let b = ParseBudget {
            max_source_bytes: 100,
            ..ParseBudget::default()
        };
        assert!(b.check_source_bytes(100).is_ok());
        assert!(b.check_source_bytes(101).is_err());
    }

    #[test]
    fn depth_check_accepts_at_limit_rejects_above() {
        let b = ParseBudget {
            max_depth: 10,
            ..ParseBudget::default()
        };
        assert!(b.check_depth(10, Span::new(0, 0)).is_ok());
        assert!(b.check_depth(11, Span::new(0, 0)).is_err());
    }

    #[test]
    fn unlimited_never_rejects() {
        let b = ParseBudget::unlimited();
        assert!(b.check_source_bytes(usize::MAX - 1).is_ok());
        assert!(b.check_tokens(usize::MAX - 1, Span::new(0, 0)).is_ok());
        assert!(b.check_depth(usize::MAX - 1, Span::new(0, 0)).is_ok());
        assert!(b.check_literal_bytes(usize::MAX - 1, Span::new(0, 0)).is_ok());
    }
}
