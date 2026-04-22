//! Lexer tests — covers all token categories, float/range disambiguation,
//! string interpolation, and error paths.

use garnet_parser::lex_source;
use garnet_parser::token::{StrPart, TokenKind};

fn kinds(src: &str) -> Vec<TokenKind> {
    lex_source(src)
        .unwrap()
        .into_iter()
        .map(|t| t.kind)
        .filter(|k| !matches!(k, TokenKind::Newline))
        .collect()
}

#[test]
fn lexes_integer_literals() {
    let toks = kinds("42 0 1_000_000");
    assert!(matches!(toks[0], TokenKind::Int(42)));
    assert!(matches!(toks[1], TokenKind::Int(0)));
    assert!(matches!(toks[2], TokenKind::Int(1_000_000)));
}

#[test]
fn lexes_float_literals() {
    let toks = kinds("2.5 0.5 1.0e10");
    assert!(matches!(toks[0], TokenKind::Float(f) if (f - 2.5).abs() < 1e-9));
    assert!(matches!(toks[1], TokenKind::Float(f) if (f - 0.5).abs() < 1e-9));
    assert!(matches!(toks[2], TokenKind::Float(f) if (f - 1.0e10).abs() < 1.0));
}

#[test]
fn disambiguates_float_vs_range() {
    // 1..5 must lex as Int(1), DotDot, Int(5) — NOT Float(1.0)
    let toks = kinds("1..5");
    assert!(matches!(toks[0], TokenKind::Int(1)));
    assert!(matches!(toks[1], TokenKind::DotDot));
    assert!(matches!(toks[2], TokenKind::Int(5)));
}

#[test]
fn disambiguates_inclusive_range() {
    let toks = kinds("1...10");
    assert!(matches!(toks[0], TokenKind::Int(1)));
    assert!(matches!(toks[1], TokenKind::DotDotDot));
    assert!(matches!(toks[2], TokenKind::Int(10)));
}

#[test]
fn lexes_string_literals() {
    let toks = kinds(r#""hello world""#);
    match &toks[0] {
        TokenKind::Str(parts) => {
            assert_eq!(parts.len(), 1);
            assert!(matches!(&parts[0], StrPart::Lit(s) if s == "hello world"));
        }
        _ => panic!("expected string"),
    }
}

#[test]
fn lexes_string_interpolation() {
    let toks = kinds(r#""hello, #{name}!""#);
    match &toks[0] {
        TokenKind::Str(parts) => {
            assert_eq!(parts.len(), 3);
            assert!(matches!(&parts[0], StrPart::Lit(s) if s == "hello, "));
            assert!(matches!(&parts[1], StrPart::Interp(s) if s == "name"));
            assert!(matches!(&parts[2], StrPart::Lit(s) if s == "!"));
        }
        _ => panic!("expected interpolated string"),
    }
}

#[test]
fn lexes_raw_string() {
    let toks = kinds(r#"r"no #{interp} here""#);
    match &toks[0] {
        TokenKind::RawStr(s) => assert_eq!(s, "no #{interp} here"),
        _ => panic!("expected raw string"),
    }
}

#[test]
fn lexes_symbols() {
    let toks = kinds(":ok :not_found :error123");
    assert!(matches!(&toks[0], TokenKind::Symbol(s) if s == "ok"));
    assert!(matches!(&toks[1], TokenKind::Symbol(s) if s == "not_found"));
    assert!(matches!(&toks[2], TokenKind::Symbol(s) if s == "error123"));
}

#[test]
fn lexes_all_arithmetic_operators() {
    let toks = kinds("+ - * / %");
    assert!(matches!(toks[0], TokenKind::Plus));
    assert!(matches!(toks[1], TokenKind::Minus));
    assert!(matches!(toks[2], TokenKind::Star));
    assert!(matches!(toks[3], TokenKind::Slash));
    assert!(matches!(toks[4], TokenKind::Percent));
}

#[test]
fn lexes_comparison_operators() {
    let toks = kinds("== != < > <= >=");
    assert!(matches!(toks[0], TokenKind::EqEq));
    assert!(matches!(toks[1], TokenKind::BangEq));
    assert!(matches!(toks[2], TokenKind::Lt));
    assert!(matches!(toks[3], TokenKind::Gt));
    assert!(matches!(toks[4], TokenKind::LtEq));
    assert!(matches!(toks[5], TokenKind::GtEq));
}

#[test]
fn lexes_assignment_operators() {
    let toks = kinds("= += -= *= /= %=");
    assert!(matches!(toks[0], TokenKind::Eq));
    assert!(matches!(toks[1], TokenKind::PlusEq));
    assert!(matches!(toks[2], TokenKind::MinusEq));
    assert!(matches!(toks[3], TokenKind::StarEq));
    assert!(matches!(toks[4], TokenKind::SlashEq));
    assert!(matches!(toks[5], TokenKind::PercentEq));
}

#[test]
fn lexes_pipeline_vs_pipe() {
    let toks = kinds("|> |");
    assert!(matches!(toks[0], TokenKind::PipeGt));
    assert!(matches!(toks[1], TokenKind::Pipe));
}

#[test]
fn lexes_arrows() {
    let toks = kinds("-> =>");
    assert!(matches!(toks[0], TokenKind::Arrow));
    assert!(matches!(toks[1], TokenKind::FatArrow));
}

#[test]
fn lexes_path_separator() {
    let toks = kinds("Foo::Bar");
    assert!(matches!(&toks[0], TokenKind::Ident(s) if s == "Foo"));
    assert!(matches!(toks[1], TokenKind::ColonCol));
    assert!(matches!(&toks[2], TokenKind::Ident(s) if s == "Bar"));
}

#[test]
fn lexes_keywords() {
    let toks = kinds("def fn let mut if else match");
    assert!(matches!(toks[0], TokenKind::KwDef));
    assert!(matches!(toks[1], TokenKind::KwFn));
    assert!(matches!(toks[2], TokenKind::KwLet));
    assert!(matches!(toks[3], TokenKind::KwMut));
    assert!(matches!(toks[4], TokenKind::KwIf));
    assert!(matches!(toks[5], TokenKind::KwElse));
    assert!(matches!(toks[6], TokenKind::KwMatch));
}

#[test]
fn lexes_annotation_marker() {
    let toks = kinds("@safe @max_depth");
    assert!(matches!(toks[0], TokenKind::At));
    assert!(matches!(&toks[1], TokenKind::Ident(s) if s == "safe"));
    assert!(matches!(toks[2], TokenKind::At));
    assert!(matches!(&toks[3], TokenKind::Ident(s) if s == "max_depth"));
}

#[test]
fn skips_comments() {
    let toks = kinds("42 # this is a comment\n7");
    assert!(matches!(toks[0], TokenKind::Int(42)));
    assert!(matches!(toks[1], TokenKind::Int(7)));
}

// ── Error paths ──

#[test]
fn errors_on_unterminated_string() {
    let result = lex_source(r#""unterminated"#);
    assert!(result.is_err());
}

#[test]
fn errors_on_unterminated_string_across_newline() {
    let result = lex_source("\"line1\nline2\"");
    assert!(result.is_err());
}

#[test]
fn errors_on_unexpected_character() {
    let result = lex_source("let x = `backtick");
    assert!(result.is_err());
}

#[test]
fn errors_on_integer_overflow() {
    let result = lex_source("99999999999999999999");
    assert!(result.is_err());
}

#[test]
fn errors_on_malformed_float() {
    // A run of digits and e/E with no exponent digits — but still lex-valid
    // because we parse with Rust's f64 parser. Test that truly malformed floats fail.
    // "1.0e" — nothing after 'e' — f64 parse should fail.
    let result = lex_source("1.0e");
    assert!(result.is_err());
}
