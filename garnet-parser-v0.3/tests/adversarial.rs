//! Adversarial parser inputs — pathological nesting, weird whitespace,
//! tokenizer pressure tests. Each test must not panic; it should either
//! parse cleanly or return a `ParseError`.

use garnet_parser::{lex_source, parse_source};

// ── Empty + whitespace ───────────────────────────────────────────────

#[test]
fn empty_token_stream_does_not_crash_lexer() {
    let toks = lex_source("").unwrap();
    assert_eq!(toks.len(), 1, "expected EOF only");
}

#[test]
fn empty_source_parses_to_empty_module() {
    let m = parse_source("").unwrap();
    assert!(m.items.is_empty());
}

#[test]
fn nul_byte_in_comment_does_not_crash() {
    let src = "# comment with \u{0000} null byte\n";
    let _ = parse_source(src); // either Ok or Err is fine; must not panic.
}

#[test]
fn crlf_then_blank_then_crlf() {
    let src = "\r\n\r\n\r\n";
    let m = parse_source(src).unwrap();
    assert!(m.items.is_empty());
}

// ── Deep nesting ─────────────────────────────────────────────────────

#[test]
fn deep_paren_nesting_runs_in_dedicated_thread() {
    // The recursive-descent parser uses one stack frame per nesting level
    // for each Pratt-precedence call. The default 2 MiB Windows test thread
    // stack overflows around ~100 paren levels (≈11-deep Pratt × 100 = 1100
    // frames × ~2 KiB each). Run on a fat thread so the test reflects the
    // real grammar limit, not the harness's miserly default stack.
    let result = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let depth = 100;
            let src = format!(
                "def main() {{ {open}1{close} }}",
                open = "(".repeat(depth),
                close = ")".repeat(depth)
            );
            parse_source(&src).expect("100-level paren nesting must parse")
        })
        .unwrap()
        .join();
    assert!(result.is_ok(), "thread panicked: {result:?}");
}

#[test]
fn fifty_level_array_nesting() {
    let depth = 50;
    let src = format!(
        "def main() {{ {open}1{close} }}",
        open = "[".repeat(depth),
        close = "]".repeat(depth)
    );
    parse_source(&src).expect("50-level array nesting must parse");
}

#[test]
fn long_arithmetic_chain() {
    let chain: String = (0..1000)
        .map(|i| format!("{i}"))
        .collect::<Vec<_>>()
        .join(" + ");
    let src = format!("def main() {{ {chain} }}");
    parse_source(&src).expect("1000-term sum must parse");
}

// ── Tokenizer pressure ───────────────────────────────────────────────

#[test]
fn one_megabyte_random_ascii_does_not_panic() {
    // Deterministic pseudo-random ASCII so the test is reproducible.
    let mut s = String::with_capacity(1 << 20);
    let mut x: u32 = 12345;
    for _ in 0..(1 << 20) {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        let c = ((x >> 16) % 95 + 32) as u8 as char; // printable ASCII
        s.push(c);
    }
    // Lexer must not panic on arbitrary printable ASCII; parse may fail.
    let _ = lex_source(&s);
}

#[test]
fn unicode_in_string_literal_parses() {
    parse_source(r#"def main() { "hello, café 🎉 — world" }"#)
        .expect("UTF-8 inside string literal must round-trip");
}

#[test]
fn unicode_in_comment_parses() {
    parse_source("# comment: café 🎉 — résumé\ndef main() { 1 }")
        .expect("UTF-8 in comments must be skipped cleanly");
}

#[test]
fn mixed_separators_around_items() {
    let src = "\n\n; ;\n;def a() { 1 }\n; ;\n;def b() { 2 }\n";
    let m = parse_source(src).unwrap();
    assert_eq!(m.items.len(), 2);
}

// ── Operator soup ────────────────────────────────────────────────────

#[test]
fn fifty_consecutive_unary_minus() {
    let src = format!("def main() {{ {} 1 }}", "-".repeat(50));
    parse_source(&src).expect("50 stacked unary minuses must parse");
}

#[test]
fn fifty_postfix_questions_does_not_overflow() {
    let src = format!("def main() {{ x{} }}", "?".repeat(50));
    parse_source(&src).expect("50 postfix ? must parse");
}

// ── Comment edge cases ───────────────────────────────────────────────

#[test]
fn comment_at_eof_without_newline_terminates_cleanly() {
    let src = "def main() { 1 } # trailing no newline";
    parse_source(src).expect("trailing comment without newline must parse");
}

#[test]
fn comment_with_special_characters() {
    let src = "# ()[]{}<>=>->|>=:! != == :: .. ...\ndef main() { 1 }";
    parse_source(src).unwrap();
}

// ── Pathological string literals ─────────────────────────────────────

#[test]
fn many_escape_sequences_in_string() {
    let src = r#"def main() { "\n\t\r\\\"\#" }"#;
    parse_source(src).expect("multiple escape sequences must parse");
}

#[test]
fn unterminated_string_returns_error_not_panic() {
    let src = r#"def main() { "no closing quote"#;
    assert!(parse_source(src).is_err());
}

#[test]
fn string_with_newline_in_middle_returns_error_not_panic() {
    let src = "def main() { \"line1\nline2\" }";
    assert!(parse_source(src).is_err());
}

// ── Number literal edge cases ────────────────────────────────────────

#[test]
fn integer_overflow_returns_error_not_panic() {
    let src = "def main() { 99999999999999999999999999999 }";
    assert!(parse_source(src).is_err());
}

#[test]
fn float_with_only_underscores_in_fraction() {
    parse_source("def main() { 1.5_5_5 }").expect("underscores in float fraction must parse");
}

// ── Trailing-comma resilience (regression for v3.1 fix) ──────────────

#[test]
fn trailing_comma_in_array_literal() {
    parse_source("def main() { [1, 2, 3,] }").expect("trailing comma should parse");
}

#[test]
fn trailing_comma_in_call() {
    parse_source("def main() { foo(1, 2,) }").expect("trailing comma in call should parse");
}

#[test]
fn trailing_comma_in_map_literal() {
    parse_source(r#"def main() { { "a" => 1, "b" => 2, } }"#)
        .expect("trailing comma in map should parse");
}
