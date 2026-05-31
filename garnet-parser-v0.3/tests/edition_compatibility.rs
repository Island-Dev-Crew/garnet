//! S32 — edition compatibility (Layer 1).
//!
//! Proves the two load-bearing properties of the edition model at the parser
//! boundary:
//!   1. exactly one demonstrable parse-time surface difference between editions
//!      (an edition-gated reserved word), and
//!   2. the one-canonical-IR invariant — source valid in *both* editions parses
//!      to a byte-identical AST, so editions never change what a program means
//!      (and, downstream, never change its capability surface).

use garnet_parser::{parse_source, parse_source_with_edition, Edition};

/// `let async = 1` parses under v1.0 (where `async` is a free identifier) but is
/// rejected under v2.0 (where `async` is reserved). This is the single
/// edition-gated surface difference, confined to the lexer.
#[test]
fn reserved_word_is_edition_gated() {
    let src = "def main() { let async = 1 }";

    // v1.0 (default): `async` is a free identifier — parses.
    assert!(
        parse_source(src).is_ok(),
        "v1.0 should parse `async` as ident"
    );
    assert!(parse_source_with_edition(src, Edition::V1_0).is_ok());

    // v2.0: `async` is reserved — rejected at lex time with a clear message.
    let err = parse_source_with_edition(src, Edition::Next).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("async") && msg.contains("reserved"),
        "expected a reserved-word error mentioning `async`, got: {msg}"
    );
}

/// One-canonical-IR invariant: source that uses no edition-gated identifier is
/// valid in both editions and parses to a byte-identical AST. `Module` does not
/// derive `PartialEq`, so we compare its canonical pretty-`Debug` rendering — a
/// deterministic structural serialization of the entire tree (including spans).
#[test]
fn ast_is_identical_across_editions_for_shared_source() {
    let src = r#"
        def add(a, b) { a + b }
        def main() {
            let total = add(2, 3)
            for x in [1, 2, 3] { total = total + x }
            total
        }
    "#;

    let v1 = parse_source_with_edition(src, Edition::V1_0).expect("parses under v1.0");
    let next = parse_source_with_edition(src, Edition::Next).expect("parses under v2.0");

    assert_eq!(
        format!("{v1:#?}"),
        format!("{next:#?}"),
        "AST must be byte-identical across editions for shared-valid source"
    );
}

/// The default entry points are exactly edition v1.0, so every existing caller,
/// example, and test is unaffected by the edition mechanism.
#[test]
fn default_entry_points_are_v1_0() {
    let src = "def main() { 42 }";
    let default = parse_source(src).expect("default parses");
    let explicit = parse_source_with_edition(src, Edition::V1_0).expect("v1.0 parses");
    assert_eq!(format!("{default:#?}"), format!("{explicit:#?}"));
}
