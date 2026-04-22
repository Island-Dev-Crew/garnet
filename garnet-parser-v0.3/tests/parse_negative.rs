//! Negative parser tests — every error path must produce a diagnostic, not a
//! panic. Locking these in keeps the language friendly to humans even when
//! they hand it malformed source.

use garnet_parser::parse_source;

fn parse_err(src: &str) {
    assert!(
        parse_source(src).is_err(),
        "expected parse error for: {src}"
    );
}

// ── Top-level item errors ──

#[test]
fn rejects_random_keyword_at_top_level() {
    parse_err("not_a_keyword");
}

#[test]
fn rejects_lone_open_brace_at_top_level() {
    parse_err("{");
}

#[test]
fn rejects_lone_arrow() {
    parse_err("->");
}

// ── Function errors ──

#[test]
fn rejects_def_without_name() {
    parse_err("def () { 1 }");
}

#[test]
fn rejects_def_without_paren() {
    parse_err("def f { 1 }");
}

#[test]
fn rejects_def_without_body() {
    parse_err("def f()");
}

// `fn f(x) -> Int { x }` parses cleanly — the missing parameter type is a
// safe-mode discipline violation caught by garnet-check, not a parse error.

#[test]
fn rejects_safe_fn_without_return_type() {
    parse_err("fn f(x: Int) { x }");
}

#[test]
fn rejects_unclosed_param_list() {
    parse_err("def f(x, y { x }");
}

#[test]
fn rejects_orphan_comma_in_params() {
    parse_err("def f(,) { 1 }");
}

// ── Let / var / const errors ──

#[test]
fn rejects_let_without_eq() {
    parse_err("let x");
}

#[test]
fn rejects_let_without_value() {
    parse_err("let x =");
}

#[test]
fn rejects_var_without_value() {
    parse_err("var x =");
}

#[test]
fn rejects_const_without_value() {
    parse_err("const C =");
}

// ── Expression errors ──

#[test]
fn rejects_binary_with_missing_rhs() {
    parse_err("def f() { 1 + }");
}

#[test]
fn rejects_unclosed_paren() {
    parse_err("def f() { (1 + 2 }");
}

#[test]
fn rejects_unclosed_array() {
    parse_err("def f() { [1, 2, 3 }");
}

#[test]
fn rejects_unclosed_string() {
    parse_err(r#"def f() { "unterminated }"#);
}

#[test]
fn rejects_string_with_newline() {
    parse_err("def f() { \"line1\nline2\" }");
}

#[test]
fn rejects_chained_minus_minus_as_decrement() {
    // Garnet has no `--` operator; `--x` should be `-(-x)` (parses) but `x--`
    // at expression position should fail when not followed by an operand.
    parse_err("def f() { x-- + 1 }");
}

// ── Match errors ──

#[test]
fn rejects_match_arm_missing_arrow() {
    parse_err("def f() { match x { 1 2 } }");
}

// Empty match { } parses but is unreachable at runtime — the checker /
// interpreter raises NoMatch then. Not a parser-level error.

// ── Try errors ──

#[test]
fn rejects_try_without_block() {
    parse_err("def f() { try 42 }");
}

#[test]
fn rejects_rescue_without_block() {
    parse_err("def f() { try { 1 } rescue e }");
}

// ── Struct errors ──

#[test]
fn rejects_struct_with_field_missing_type() {
    parse_err("struct S { x }");
}

#[test]
fn rejects_struct_with_unclosed_body() {
    parse_err("struct S { x: Int");
}

// ── Enum errors ──

#[test]
fn rejects_enum_with_unclosed_variant_payload() {
    parse_err("enum E { Variant(Int }");
}

// ── Trait / impl errors ──

#[test]
fn rejects_trait_without_body() {
    parse_err("trait T");
}

#[test]
fn rejects_impl_without_body() {
    parse_err("impl T for S");
}

// ── Memory unit errors ──

#[test]
fn rejects_memory_without_kind() {
    parse_err("memory unit : Store<T>");
}

#[test]
fn rejects_memory_without_type() {
    parse_err("memory working unit");
}

#[test]
fn rejects_memory_without_colon() {
    parse_err("memory working unit Store<T>");
}

// ── Use errors ──

#[test]
fn rejects_use_without_path() {
    parse_err("use");
}

#[test]
fn rejects_use_with_unclosed_brace() {
    parse_err("use Foo::{A, B");
}

// ── Module errors ──

#[test]
fn rejects_module_without_name() {
    parse_err("module { }");
}

#[test]
fn rejects_module_with_unclosed_body() {
    parse_err("module M { def f() { 1 }");
}

// ── Annotation errors ──

#[test]
fn rejects_unknown_annotation() {
    parse_err("@unknown_ann def f() { 1 }");
}

#[test]
fn rejects_max_depth_without_argument() {
    parse_err("@max_depth def f() { 1 }");
}

// ── Lexer errors ──

#[test]
fn rejects_invalid_character() {
    parse_err("def f() { ` }");
}

#[test]
fn rejects_oversized_int_literal() {
    // Garnet uses i64; anything beyond 2^63-1 should trigger an InvalidInt.
    parse_err("def f() { 99999999999999999999999999999 }");
}

// ── Pipeline errors (don't allow trailing |>) ──

#[test]
fn rejects_pipeline_without_rhs() {
    parse_err("def f() { x |> }");
}
