//! Garnet edition / compatibility-epoch model (S32, Layer 1).
//!
//! Editions are **named compatibility epochs** (Rust 2015 / 2018 / 2021 style),
//! decoupled from the rolling compiler version. The current and default edition
//! is `v1.0` — the spec-canonical form (Mini-Spec §16.3). A second edition
//! `v2.0` is **registered only to prove the edition mechanism** in v0.8: it is
//! *not* a shipped language version. It exists so the compiler can demonstrate
//! one edition-gated surface difference (a single reserved word) and the
//! one-canonical-IR invariant below.
//!
//! ## One-canonical-IR invariant
//!
//! Editions gate **only** the front-end surface (lexing). The AST, the checker,
//! the interpreter, and the capability manifest are **edition-invariant by
//! construction**: source that is valid in two editions produces a
//! byte-identical AST, and therefore an identical capability surface. An
//! edition may change *what spelling is legal* but never *what authority a
//! program holds*. (S32 load-bearing invariant; proven by the
//! `parse_source_with_edition` AST-equality test and the CLI manifest-invariance
//! test.)

use std::fmt;

/// A named Garnet compatibility epoch.
///
/// Ordering follows declaration order (`V1_0 < Next`), so future code may write
/// edition-threshold checks such as `edition >= Edition::Next`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Edition {
    /// `v1.0` — the current, default edition (Mini-Spec §16.3 canonical form).
    /// Every existing source file, example, and test parses under this edition
    /// unchanged.
    #[default]
    V1_0,
    /// `v2.0` — a **registered future edition that exists only to prove the
    /// edition mechanism** in v0.8. Not a shipped language version. Under this
    /// edition exactly one identifier (`async`) is reserved that is free under
    /// `v1.0`.
    Next,
}

/// Error returned when a manifest names a canonical edition the compiler does
/// not recognize. (The legacy-alias mapping and its deprecation warning live in
/// the manifest layer, `garnet-cli`, which calls [`Edition::parse`] only with
/// canonical strings.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditionError {
    /// The unrecognized edition string exactly as written in the manifest.
    pub value: String,
}

impl fmt::Display for EditionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown edition `{}` (known editions: {})",
            self.value,
            Edition::known_names().join(", ")
        )
    }
}

impl std::error::Error for EditionError {}

impl Edition {
    /// The edition shipped as the default for `garnet new` projects and used by
    /// every caller that does not pin one explicitly.
    pub const fn current() -> Self {
        Edition::V1_0
    }

    /// The spec-canonical name of this edition — the value written in
    /// `[project].edition`.
    pub const fn name(self) -> &'static str {
        match self {
            Edition::V1_0 => "v1.0",
            Edition::Next => "v2.0",
        }
    }

    /// All known canonical edition names, for diagnostics.
    pub fn known_names() -> Vec<&'static str> {
        vec![Edition::V1_0.name(), Edition::Next.name()]
    }

    /// Parse a **canonical** edition string (spec form, e.g. `"v1.0"`).
    ///
    /// Legacy aliases (e.g. the old template's `"garnet-0.3"`) are intentionally
    /// *not* accepted here: the manifest layer maps a legacy form to a canonical
    /// edition and emits a one-line deprecation warning before calling this.
    pub fn parse(s: &str) -> Result<Self, EditionError> {
        match s {
            "v1.0" => Ok(Edition::V1_0),
            "v2.0" => Ok(Edition::Next),
            other => Err(EditionError {
                value: other.to_string(),
            }),
        }
    }

    /// Whether `ident` is a **reserved word** (rejected by the lexer) under this
    /// edition. This is the one demonstrable parse-time surface difference
    /// between editions — deliberately confined to the keyword layer so the
    /// grammar and AST are untouched.
    ///
    /// `async` is a free identifier under `v1.0` (so `let async = 1` parses) and
    /// a reserved word under `v2.0` (so the identical source is rejected at lex
    /// time). No other identifier is gated.
    pub fn is_reserved_ident(self, ident: &str) -> bool {
        match self {
            Edition::V1_0 => false,
            Edition::Next => matches!(ident, "async"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_current_are_v1_0() {
        assert_eq!(Edition::default(), Edition::V1_0);
        assert_eq!(Edition::current(), Edition::V1_0);
    }

    #[test]
    fn canonical_names_round_trip() {
        assert_eq!(Edition::parse("v1.0"), Ok(Edition::V1_0));
        assert_eq!(Edition::parse("v2.0"), Ok(Edition::Next));
        assert_eq!(Edition::V1_0.name(), "v1.0");
        assert_eq!(Edition::Next.name(), "v2.0");
    }

    #[test]
    fn unknown_edition_is_an_error_listing_known_names() {
        let err = Edition::parse("garnet-0.3").unwrap_err();
        assert_eq!(err.value, "garnet-0.3");
        let msg = err.to_string();
        assert!(msg.contains("v1.0") && msg.contains("v2.0"), "got: {msg}");
    }

    #[test]
    fn async_is_reserved_only_under_next() {
        assert!(!Edition::V1_0.is_reserved_ident("async"));
        assert!(Edition::Next.is_reserved_ident("async"));
        // A non-gated identifier is free in both editions.
        assert!(!Edition::V1_0.is_reserved_ident("widget"));
        assert!(!Edition::Next.is_reserved_ident("widget"));
    }

    #[test]
    fn editions_are_ordered() {
        assert!(Edition::V1_0 < Edition::Next);
    }
}
