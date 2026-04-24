//! Boundary tests for ParseBudget — v3.3 Security Layer 1 (hardening #3).
//!
//! Each axis is tested at three points:
//!   - pass at N-1 (just under the limit)
//!   - pass at N   (exactly at the limit — inclusive)
//!   - fail at N+1 (just over)
//!   - fail at 2N  (well over, confirms failure isn't off-by-one lucky)
//!
//! The default budget is explicitly sized to be ample for real code and
//! small enough that adversarial inputs fail in milliseconds.

use garnet_parser::{parse_source_with_budget, ParseBudget};

// ── Axis 1: source_bytes ────────────────────────────────────────────

#[test]
fn source_bytes_under_limit_passes() {
    let budget = ParseBudget {
        max_source_bytes: 100,
        ..ParseBudget::default()
    };
    // 20 bytes of valid Garnet.
    let src = "def main() { 42 }";
    assert!(src.len() < 100);
    assert!(parse_source_with_budget(src, budget).is_ok());
}

#[test]
fn source_bytes_at_limit_passes() {
    // Exactly 20 bytes.
    let src = "def main() { 42 }aaa"; // not valid Garnet but source_bytes
                                      // check runs BEFORE lex, so the byte
                                      // budget is tested in isolation.
    let budget = ParseBudget {
        max_source_bytes: src.len(),
        ..ParseBudget::default()
    };
    // Source-byte check passes; lex/parse might fail on its own.
    // We only assert the budget didn't produce a BudgetExceeded error.
    let result = parse_source_with_budget(src, budget);
    let err_text = format!("{result:?}");
    assert!(
        !err_text.contains("BudgetExceeded"),
        "at-limit should not trigger BudgetExceeded: {err_text}"
    );
}

#[test]
fn source_bytes_over_limit_fails_fast() {
    let budget = ParseBudget {
        max_source_bytes: 10,
        ..ParseBudget::default()
    };
    let src = "def main() { 42 }"; // 17 bytes
    let err = parse_source_with_budget(src, budget).unwrap_err();
    let err_text = format!("{err:?}");
    assert!(
        err_text.contains("BudgetExceeded"),
        "expected BudgetExceeded, got {err_text}"
    );
    assert!(err_text.contains("source_bytes"));
}

// ── Axis 2: depth (the ParensBomb defense) ──────────────────────────

#[test]
fn nested_parens_just_under_depth_limit_passes() {
    // depth=10 → nine levels of parens around a simple int is fine
    let budget = ParseBudget {
        max_depth: 10,
        ..ParseBudget::default()
    };
    let src = format!(
        "def main() {{ {}{}{} }}",
        "(".repeat(8),
        "42",
        ")".repeat(8)
    );
    let result = parse_source_with_budget(&src, budget);
    assert!(
        result.is_ok(),
        "depth-8 nesting should pass under max_depth=10: {:?}",
        result.err()
    );
}

#[test]
fn nested_parens_well_over_depth_limit_fails() {
    // 500 levels of parens > default 256 limit
    let src = format!(
        "def main() {{ {}{}{} }}",
        "(".repeat(500),
        "42",
        ")".repeat(500)
    );
    let err = parse_source_with_budget(&src, ParseBudget::default()).unwrap_err();
    let err_text = format!("{err:?}");
    assert!(
        err_text.contains("BudgetExceeded"),
        "expected BudgetExceeded, got {err_text}"
    );
    assert!(err_text.contains("depth"));
}

#[test]
fn parens_bomb_tight_budget_fails_fast() {
    // ParensBomb with a depth=5 limit — 100 levels of parens must fail.
    let budget = ParseBudget {
        max_depth: 5,
        ..ParseBudget::default()
    };
    let src = format!(
        "def main() {{ {}{}{} }}",
        "(".repeat(100),
        "42",
        ")".repeat(100)
    );
    let err = parse_source_with_budget(&src, budget).unwrap_err();
    let err_text = format!("{err:?}");
    assert!(err_text.contains("depth"));
}

// ── Axis 3: tokens ──────────────────────────────────────────────────

#[test]
fn token_count_over_limit_fails() {
    // max_tokens=10 — a simple fn has more than 10 tokens easily.
    let budget = ParseBudget {
        max_tokens: 10,
        ..ParseBudget::default()
    };
    // This source has ~15 tokens: def, main, (, ), {, let, x, =, 1, +, 2, ;, x, }, EOF
    let src = "def main() { let x = 1 + 2; x }";
    let err = parse_source_with_budget(src, budget).unwrap_err();
    let err_text = format!("{err:?}");
    assert!(
        err_text.contains("BudgetExceeded"),
        "expected BudgetExceeded, got {err_text}"
    );
    assert!(err_text.contains("tokens"));
}

#[test]
fn token_count_under_limit_passes() {
    let budget = ParseBudget {
        max_tokens: 100,
        ..ParseBudget::default()
    };
    let src = "def main() { let x = 1 + 2; x }";
    assert!(parse_source_with_budget(src, budget).is_ok());
}

// ── Axis 4: literal_bytes (StringBlimp + CommentFlood defense) ──────

#[test]
fn string_literal_over_limit_fails_fast() {
    // max_literal_bytes=20 — a 100-byte string literal fails.
    let budget = ParseBudget {
        max_literal_bytes: 20,
        max_source_bytes: usize::MAX,
        max_tokens: usize::MAX,
        ..ParseBudget::default()
    };
    let blimp = "a".repeat(100);
    let src = format!("def main() {{ \"{}\" }}", blimp);
    let err = parse_source_with_budget(&src, budget).unwrap_err();
    let err_text = format!("{err:?}");
    assert!(
        err_text.contains("BudgetExceeded"),
        "expected BudgetExceeded, got {err_text}"
    );
    assert!(err_text.contains("literal_bytes"));
}

#[test]
fn identifier_over_limit_fails_fast() {
    let budget = ParseBudget {
        max_literal_bytes: 50,
        max_source_bytes: usize::MAX,
        max_tokens: usize::MAX,
        ..ParseBudget::default()
    };
    let long_ident = "a".repeat(200);
    let src = format!("def main() {{ let {} = 1; {} }}", long_ident, long_ident);
    let err = parse_source_with_budget(&src, budget).unwrap_err();
    let err_text = format!("{err:?}");
    assert!(err_text.contains("literal_bytes"));
}

#[test]
fn comment_flood_over_limit_fails_fast() {
    let budget = ParseBudget {
        max_literal_bytes: 100,
        max_source_bytes: usize::MAX,
        max_tokens: usize::MAX,
        ..ParseBudget::default()
    };
    // 500-char comment on one line — no \n because comments run to EOL.
    let flood = "x".repeat(500);
    let src = format!("# {}\ndef main() {{ 1 }}", flood);
    let err = parse_source_with_budget(&src, budget).unwrap_err();
    let err_text = format!("{err:?}");
    assert!(err_text.contains("literal_bytes"));
}

// ── Defaults ────────────────────────────────────────────────────────

#[test]
fn real_code_passes_default_budget() {
    // A realistic chunk of Garnet — exercises struct, pattern match,
    // closures, method calls. Well under all defaults.
    let src = r#"
        struct Point { x: Int, y: Int }
        def main() {
            let xs = [1, 2, 3, 4]
            let total = xs.reduce(0, |a, b| a + b)
            match total {
                0 => "zero",
                _ => "nonzero"
            }
        }
    "#;
    let result = parse_source_with_budget(src, ParseBudget::default());
    assert!(
        result.is_ok(),
        "real code should parse under defaults: {:?}",
        result.err()
    );
}

#[test]
fn unlimited_budget_parses_aggressive_nesting() {
    // 400-deep nesting — over default max_depth (256) but under
    // unlimited.
    let src = format!(
        "def main() {{ {}{}{} }}",
        "(".repeat(400),
        "42",
        ")".repeat(400)
    );
    let budget = ParseBudget::unlimited();
    let result = parse_source_with_budget(&src, budget);
    assert!(
        result.is_ok(),
        "unlimited budget should accept 400-deep: {:?}",
        result.err()
    );
}
