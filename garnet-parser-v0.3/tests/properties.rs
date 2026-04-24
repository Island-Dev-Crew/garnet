//! Property-based tests for the parser. Each property either holds for
//! every input proptest generates, or the property is wrong. Cases default
//! to 256 per property — adjustable via `PROPTEST_CASES`.

use garnet_parser::token::TokenKind;
use garnet_parser::{lex_source, parse_source};
use proptest::prelude::*;

// ── Lexer: arbitrary printable ASCII never panics ───────────────────

proptest! {
    #[test]
    fn lex_never_panics_on_printable_ascii(s in r"[ -~]{0,2000}") {
        let _ = lex_source(&s);
    }
}

proptest! {
    #[test]
    fn lex_then_eof_is_always_terminating(s in r"[ -~]{0,500}") {
        if let Ok(toks) = lex_source(&s) {
            prop_assert!(!toks.is_empty());
            prop_assert!(matches!(toks.last().unwrap().kind, TokenKind::Eof));
        }
    }
}

proptest! {
    #[test]
    fn token_spans_are_monotonic(s in r"[a-zA-Z0-9 \n\t+\-\*/\=\(\)\{\},;.]{0,500}") {
        if let Ok(toks) = lex_source(&s) {
            for w in toks.windows(2) {
                let a = &w[0];
                let b = &w[1];
                if matches!(b.kind, TokenKind::Eof) {
                    continue;
                }
                prop_assert!(a.span.start <= b.span.start);
            }
        }
    }
}

// ── Parser: well-formed shapes always parse ─────────────────────────

proptest! {
    #[test]
    fn def_with_int_body_always_parses(n in 0i64..1_000_000) {
        let src = format!("def main() {{ {n} }}");
        let m = parse_source(&src).unwrap();
        prop_assert_eq!(m.items.len(), 1);
    }
}

proptest! {
    #[test]
    fn def_with_addition_body_parses(a in 0i64..10_000, b in 0i64..10_000) {
        let src = format!("def main() {{ {a} + {b} }}");
        prop_assert!(parse_source(&src).is_ok());
    }
}

proptest! {
    #[test]
    fn array_literal_always_parses(elements in proptest::collection::vec(0i64..1000, 0..50)) {
        let body = elements
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let src = format!("def main() {{ [{body}] }}");
        prop_assert!(parse_source(&src).is_ok());
    }
}

proptest! {
    #[test]
    fn nested_paren_up_to_50_parses(depth in 1usize..50) {
        let open = "(".repeat(depth);
        let close = ")".repeat(depth);
        let src = format!("def main() {{ {open}1{close} }}");
        prop_assert!(parse_source(&src).is_ok());
    }
}

proptest! {
    #[test]
    fn random_identifier_parses_as_def_name(name in "[a-z][a-z0-9_]{0,16}") {
        // Reject reserved words; we only want identifiers.
        let reserved = [
            "def","fn","let","var","const","if","elsif","else","while","for","in","loop",
            "break","continue","return","match","when","try","rescue","ensure","raise",
            "own","borrow","ref","mut","move","and","or","not","true","false","nil","self",
            "super","module","use","pub","end","type","trait","impl","struct","enum",
            "memory","working","episodic","semantic","procedural","actor","protocol",
            "on","spawn","send",
        ];
        if reserved.contains(&name.as_str()) {
            return Ok(());
        }
        let src = format!("def {name}() {{ 0 }}");
        prop_assert!(parse_source(&src).is_ok(), "failed to parse: {src}");
    }
}

proptest! {
    #[test]
    fn match_with_n_int_arms_parses(arms in 1usize..20) {
        let body: Vec<String> = (0..arms).map(|i| format!("{i} => {i}")).collect();
        let src = format!("def main() {{ match x {{ {} _ => 0 }} }}", body.join(", "));
        prop_assert!(parse_source(&src).is_ok());
    }
}

proptest! {
    #[test]
    fn struct_with_fields_parses(field_count in 0usize..30) {
        let fields: Vec<String> = (0..field_count).map(|i| format!("f{i}: Int")).collect();
        let src = format!("struct S {{ {} }}", fields.join(", "));
        prop_assert!(parse_source(&src).is_ok());
    }
}

proptest! {
    #[test]
    fn enum_with_variants_parses(variants in 1usize..15) {
        let body: Vec<String> = (0..variants).map(|i| format!("V{i}")).collect();
        let src = format!("enum E {{ {} }}", body.join(", "));
        prop_assert!(parse_source(&src).is_ok());
    }
}
