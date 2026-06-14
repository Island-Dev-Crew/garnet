//! RB-4b.2 — `SyntaxError` carries a span (a range over the offending
//! token), not just a start offset. This is the foundation a future LSP
//! single-parse needs to render range diagnostics matching `parse_source`'s
//! labeled-span quality. (The single-parse cutover itself is deferred:
//! `parse_cst`'s error recovery produces cascades — e.g. 9 errors for
//! `@@@ def` vs `parse_source`'s 1 — so dropping `parse_source` from the LSP
//! would degrade diagnostics until parser error-recovery is improved.)

use garnet_cst::parse_cst;

#[test]
fn grammar_error_span_covers_the_offending_token() {
    // `(` opens a paren the grammar can't complete: the recorded error is
    // anchored to the `{` token (a real, non-zero-width range), not offset-only.
    let src = "def broken( {\n";
    let parse = parse_cst(src);
    assert!(!parse.errors.is_empty());
    let e = &parse.errors[0];
    // span is a range over a real token (len > 0), and the slice it covers
    // is the actual source text at that span.
    assert!(e.span.len > 0, "span must cover a token, got {:?}", e.span);
    let covered = &src[e.span.start..e.span.end()];
    assert!(
        !covered.trim().is_empty(),
        "span covers real text: {covered:?}"
    );
    // offset() back-compat = span start.
    assert_eq!(e.offset(), e.span.start);
}

#[test]
fn budget_error_span_points_at_the_offending_depth() {
    // Over-depth: the budget error's span is the token where the depth
    // ceiling was crossed, not a bare offset 0.
    let deep = format!(
        "@caps()\ndef main() {{ {}1{} }}\n",
        "(".repeat(400),
        ")".repeat(400)
    );
    let parse = parse_cst(&deep);
    let budget = parse
        .errors
        .iter()
        .find(|e| e.message.contains("budget"))
        .expect("a budget error is recorded");
    assert!(
        budget.span.start > 0,
        "budget span is a real position, not 0"
    );
}

#[test]
fn lex_error_span_is_the_bad_input() {
    // An unterminated string fails to lex; the recorded error span is a real
    // position into the source.
    let src = "@caps()\ndef main() { \"unterminated\n}\n";
    let parse = parse_cst(src);
    assert!(parse
        .errors
        .iter()
        .any(|e| e.span.len > 0 || e.span.start > 0));
}
