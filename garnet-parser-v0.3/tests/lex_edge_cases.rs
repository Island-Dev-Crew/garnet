//! Lexer edge cases — every operator at every adjacency, all separators,
//! all comment placements. Catches off-by-one boundary bugs and tokenizer
//! greedy-match regressions.

use garnet_parser::lex_source;
use garnet_parser::token::{StrPart, TokenKind};

fn kinds(src: &str) -> Vec<TokenKind> {
    lex_source(src)
        .unwrap()
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

fn first_kind(src: &str) -> TokenKind {
    kinds(src).into_iter().next().unwrap()
}

// ── Boundary tests for two-char operators ──

#[test]
fn dot_alone() {
    assert!(matches!(first_kind("."), TokenKind::Dot));
}

#[test]
fn dot_dot_alone() {
    assert!(matches!(first_kind(".."), TokenKind::DotDot));
}

#[test]
fn dot_dot_dot_alone() {
    assert!(matches!(first_kind("..."), TokenKind::DotDotDot));
}

#[test]
fn arrow_after_minus() {
    assert!(matches!(first_kind("->"), TokenKind::Arrow));
}

#[test]
fn fat_arrow_after_eq() {
    assert!(matches!(first_kind("=>"), TokenKind::FatArrow));
}

#[test]
fn pipeline_after_pipe() {
    assert!(matches!(first_kind("|>"), TokenKind::PipeGt));
}

#[test]
fn lone_pipe() {
    assert!(matches!(first_kind("|"), TokenKind::Pipe));
}

#[test]
fn equality_eq_eq() {
    assert!(matches!(first_kind("=="), TokenKind::EqEq));
}

#[test]
fn bang_eq_disambiguates_bang_alone() {
    let toks = kinds("!=");
    assert!(matches!(toks[0], TokenKind::BangEq));
}

#[test]
fn lone_bang_keeps_kind() {
    let toks = kinds("!");
    assert!(matches!(toks[0], TokenKind::Bang));
}

#[test]
fn lt_eq_two_chars() {
    assert!(matches!(first_kind("<="), TokenKind::LtEq));
}

#[test]
fn gt_eq_two_chars() {
    assert!(matches!(first_kind(">="), TokenKind::GtEq));
}

#[test]
fn colon_colon_two_chars() {
    assert!(matches!(first_kind("::"), TokenKind::ColonCol));
}

#[test]
fn lone_colon_when_no_following_alpha() {
    assert!(matches!(first_kind(": "), TokenKind::Colon));
}

#[test]
fn plus_eq_two_chars() {
    assert!(matches!(first_kind("+="), TokenKind::PlusEq));
}

#[test]
fn minus_eq_two_chars() {
    assert!(matches!(first_kind("-="), TokenKind::MinusEq));
}

#[test]
fn star_eq_two_chars() {
    assert!(matches!(first_kind("*="), TokenKind::StarEq));
}

#[test]
fn slash_eq_two_chars() {
    assert!(matches!(first_kind("/="), TokenKind::SlashEq));
}

#[test]
fn percent_eq_two_chars() {
    assert!(matches!(first_kind("%="), TokenKind::PercentEq));
}

// ── Range disambiguation: 1..2 vs 1.0..2 vs 1...2 ──

#[test]
fn int_dot_dot_int() {
    let toks = kinds("1..5");
    assert!(matches!(toks[0], TokenKind::Int(1)));
    assert!(matches!(toks[1], TokenKind::DotDot));
    assert!(matches!(toks[2], TokenKind::Int(5)));
}

#[test]
fn int_dot_dot_dot_int() {
    let toks = kinds("1...5");
    assert!(matches!(toks[0], TokenKind::Int(1)));
    assert!(matches!(toks[1], TokenKind::DotDotDot));
    assert!(matches!(toks[2], TokenKind::Int(5)));
}

#[test]
fn int_dot_int_is_float() {
    let toks = kinds("1.5");
    assert!(matches!(toks[0], TokenKind::Float(_)));
}

#[test]
fn float_dot_dot_int_is_float_then_range() {
    let toks = kinds("1.5..3");
    assert!(matches!(toks[0], TokenKind::Float(_)));
    assert!(matches!(toks[1], TokenKind::DotDot));
    assert!(matches!(toks[2], TokenKind::Int(3)));
}

// ── Identifiers vs keywords ──

#[test]
fn keyword_def_classified() {
    assert!(matches!(first_kind("def"), TokenKind::KwDef));
}

#[test]
fn keyword_fn_classified() {
    assert!(matches!(first_kind("fn"), TokenKind::KwFn));
}

#[test]
fn ident_starting_like_keyword() {
    let toks = kinds("define");
    assert!(matches!(toks[0], TokenKind::Ident(ref n) if n == "define"));
}

#[test]
fn ident_with_keyword_inside() {
    let toks = kinds("undefine");
    assert!(matches!(toks[0], TokenKind::Ident(ref n) if n == "undefine"));
}

#[test]
fn ident_with_underscore() {
    let toks = kinds("snake_case_var");
    assert!(matches!(toks[0], TokenKind::Ident(ref n) if n == "snake_case_var"));
}

#[test]
fn ident_starting_with_underscore() {
    let toks = kinds("_private");
    assert!(matches!(toks[0], TokenKind::Ident(ref n) if n == "_private"));
}

#[test]
fn ident_with_digits() {
    let toks = kinds("var1");
    assert!(matches!(toks[0], TokenKind::Ident(ref n) if n == "var1"));
}

// ── String literal edge cases ──

#[test]
fn empty_string_literal() {
    let toks = kinds(r#""""#);
    assert!(
        matches!(toks[0], TokenKind::Str(ref parts) if parts.is_empty() || matches!(&parts[0], StrPart::Lit(s) if s.is_empty()))
    );
}

#[test]
fn string_with_only_interpolation() {
    let toks = kinds(r##""#{x}""##);
    if let TokenKind::Str(parts) = &toks[0] {
        assert!(parts.iter().any(|p| matches!(p, StrPart::Interp(_))));
    } else {
        panic!("expected string");
    }
}

#[test]
fn string_with_lit_then_interp_then_lit() {
    let toks = kinds(r##""hello, #{name}!""##);
    if let TokenKind::Str(parts) = &toks[0] {
        assert!(parts.len() >= 2);
    } else {
        panic!("expected string");
    }
}

#[test]
fn raw_string_keeps_backslash() {
    let toks = kinds(r#"r"a\nb""#);
    assert!(matches!(toks[0], TokenKind::RawStr(ref s) if s == "a\\nb"));
}

// ── Comment placement ──

#[test]
fn comment_at_eof_no_newline() {
    let toks = kinds("# comment with no trailing newline");
    assert!(matches!(toks[0], TokenKind::Eof));
}

#[test]
fn comment_after_token_on_same_line() {
    let toks = kinds("42 # answer");
    assert!(matches!(toks[0], TokenKind::Int(42)));
    assert!(matches!(toks[1], TokenKind::Eof));
}

#[test]
fn comment_does_not_eat_following_lines() {
    let toks = kinds("# c1\n42");
    // Should be: Newline, Int, Eof
    assert!(toks.len() >= 2);
    assert!(toks.iter().any(|k| matches!(k, TokenKind::Int(42))));
}

// ── Whitespace handling ──

#[test]
fn tabs_treated_as_spaces() {
    let toks = kinds("a\tb");
    assert!(matches!(toks[0], TokenKind::Ident(_)));
    assert!(matches!(toks[1], TokenKind::Ident(_)));
}

#[test]
fn crlf_newline_is_one_token() {
    let toks = kinds("a\r\nb");
    let nl_count = toks
        .iter()
        .filter(|k| matches!(k, TokenKind::Newline))
        .count();
    assert_eq!(nl_count, 1);
}

// ── Symbols ──

#[test]
fn symbol_takes_following_ident() {
    let toks = kinds(":symbol_name");
    assert!(matches!(toks[0], TokenKind::Symbol(ref s) if s == "symbol_name"));
}

#[test]
fn symbol_with_underscore_start() {
    let toks = kinds(":_priv");
    assert!(matches!(toks[0], TokenKind::Symbol(ref s) if s == "_priv"));
}

// ── Number parsing ──

#[test]
fn underscores_stripped_from_int() {
    let toks = kinds("1_234_567");
    assert!(matches!(toks[0], TokenKind::Int(1234567)));
}

#[test]
fn underscores_stripped_from_float() {
    let toks = kinds("1_000.5");
    assert!(matches!(toks[0], TokenKind::Float(_)));
}

#[test]
fn negative_exponent_in_float() {
    let toks = kinds("1.5e-10");
    if let TokenKind::Float(f) = toks[0] {
        assert!((f - 1.5e-10).abs() < 1e-15);
    }
}

#[test]
fn positive_exponent_in_float() {
    let toks = kinds("1.5e+5");
    if let TokenKind::Float(f) = toks[0] {
        assert!((f - 1.5e5).abs() < 1.0);
    }
}

// ── Punctuation & adjacency ──

#[test]
fn paren_brace_bracket() {
    let toks = kinds("({[]})");
    assert!(matches!(toks[0], TokenKind::LParen));
    assert!(matches!(toks[1], TokenKind::LBrace));
    assert!(matches!(toks[2], TokenKind::LBracket));
    assert!(matches!(toks[3], TokenKind::RBracket));
    assert!(matches!(toks[4], TokenKind::RBrace));
    assert!(matches!(toks[5], TokenKind::RParen));
}

#[test]
fn arithmetic_operators_distinct() {
    let toks = kinds("+ - * / %");
    let kinds_only: Vec<_> = toks.into_iter().take(5).collect();
    assert!(matches!(kinds_only[0], TokenKind::Plus));
    assert!(matches!(kinds_only[1], TokenKind::Minus));
    assert!(matches!(kinds_only[2], TokenKind::Star));
    assert!(matches!(kinds_only[3], TokenKind::Slash));
    assert!(matches!(kinds_only[4], TokenKind::Percent));
}

#[test]
fn ampersand_alone() {
    assert!(matches!(first_kind("&"), TokenKind::Amp));
}

#[test]
fn at_alone() {
    assert!(matches!(first_kind("@"), TokenKind::At));
}

#[test]
fn question_alone() {
    assert!(matches!(first_kind("?"), TokenKind::Question));
}

// ── Eof always terminates ──

#[test]
fn eof_token_always_emitted() {
    let toks = kinds("x");
    assert!(matches!(toks.last().unwrap(), TokenKind::Eof));
}

#[test]
fn empty_source_emits_only_eof() {
    let toks = kinds("");
    assert_eq!(toks.len(), 1);
    assert!(matches!(toks[0], TokenKind::Eof));
}
