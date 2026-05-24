//! PR-1 round-trip + `SyntaxKind` invariants for the rowan CST stub.
//!
//! The stub treats the whole source as one trivia leaf, so round-trip is
//! byte-exact for *any* input. PR-2 keeps these same assertions green once the
//! real recursive-descent builder lands.

use garnet_cst::{cst_to_source, parse_cst, SyntaxKind};

#[test]
fn roundtrips_simple_source() {
    let src = "def greet(name) {\n  \"Hello, #{name}!\"\n}\n";
    let parse = parse_cst(src);
    assert!(parse.ok());
    assert_eq!(parse.syntax().kind(), SyntaxKind::Root);
    assert_eq!(cst_to_source(parse.syntax()), src);
}

#[test]
fn roundtrips_empty_input() {
    let parse = parse_cst("");
    assert!(parse.ok());
    assert_eq!(parse.to_source(), "");
    assert_eq!(parse.syntax().kind(), SyntaxKind::Root);
}

#[test]
fn roundtrips_comment_and_whitespace_trivia() {
    let src = "  # leading comment\n\n  def f() { 1 }  # trailing\n";
    let parse = parse_cst(src);
    assert_eq!(parse.to_source(), src);
}

#[test]
fn syntax_kind_u16_roundtrip_is_total_and_bounded() {
    for raw in 0..SyntaxKind::COUNT {
        let kind = SyntaxKind::from_u16(raw).expect("every discriminant in range maps to a kind");
        assert_eq!(kind.to_u16(), raw, "u16 round-trip must be identity");
    }
    assert!(
        SyntaxKind::from_u16(SyntaxKind::COUNT).is_none(),
        "one past the last discriminant must be out of range"
    );
}

#[test]
fn trivia_classification() {
    assert!(SyntaxKind::Whitespace.is_trivia());
    assert!(SyntaxKind::Comment.is_trivia());
    assert!(!SyntaxKind::Ident.is_trivia());
    assert!(!SyntaxKind::Root.is_trivia());
}

proptest::proptest! {
    /// Round-trip holds for ANY UTF-8 input (newlines included), regardless of
    /// grammatical validity: tokens that lex are emitted in order, and input
    /// that fails to lex is preserved under an `Error` leaf. `parse.ok()` is
    /// NOT asserted — arbitrary text is usually not a valid program, but it
    /// must still reconstruct byte-for-byte.
    #[test]
    fn roundtrips_any_utf8_input(s in "(?s).*") {
        let parse = parse_cst(&s);
        proptest::prop_assert_eq!(cst_to_source(parse.syntax()), s);
    }
}
