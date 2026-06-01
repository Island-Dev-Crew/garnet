//! S34 — structured diagnostics (machine + human).
//!
//! The reusable structured form behind `garnet check`'s diagnostics: a stable
//! `code`, a `Severity`, the human `message`, and an optional source `span`.
//! `garnet check --format human` prints today's miette/Display output; `--format
//! json` serializes these records as **deterministic, hand-rolled JSON** (no
//! `serde`, matching `manifest.rs`'s determinism stance). This is the type a
//! future MCP/LSP server serves.
//!
//! ## Authoritative `garnet check` exit codes (S34)
//! * [`EXIT_OK`] `0` — no fatal diagnostics.
//! * [`EXIT_DIAGNOSTICS`] `1` — ≥1 fatal diagnostic, or a parse / IO error.
//! * [`EXIT_USAGE`] `2` — bad invocation.
//!
//! Severity is independent of `code`; only `Error`-severity diagnostics are
//! fatal (they mirror `CheckReport::ok()` — `SafeModeViolation`,
//! `AnnotationError`, `CapsCoverage`, `StabilityError`, `BoundedLoop`).
//! `BoundaryNote` and `StabilityAdvice` are advisory and never change the exit
//! code.

use garnet_check::{CheckError, CheckReport};
use garnet_parser::error::ParseError;

/// Authoritative exit code: clean (no fatal diagnostics).
pub const EXIT_OK: u8 = 0;
/// Authoritative exit code: ≥1 fatal diagnostic, or a parse / IO error.
pub const EXIT_DIAGNOSTICS: u8 = 1;
/// Authoritative exit code: bad invocation / usage error.
pub const EXIT_USAGE: u8 = 2;

/// Diagnostic severity. `Error` is fatal (drives the exit code); `Warning` and
/// `Info` are advisory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    /// Stable lowercase wire name used in the JSON form.
    pub fn wire(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

impl From<garnet_check::Severity> for Severity {
    /// Adopt the checker's canonical severity (S44: single source of truth in
    /// `garnet-check`, shared by this CLI and the LSP).
    fn from(s: garnet_check::Severity) -> Self {
        match s {
            garnet_check::Severity::Error => Severity::Error,
            garnet_check::Severity::Warning => Severity::Warning,
            garnet_check::Severity::Info => Severity::Info,
        }
    }
}

/// A single structured diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    /// A stable machine code, e.g. `check.caps_coverage` or `parse.reserved_word`.
    pub code: &'static str,
    pub message: String,
    /// `(start, len)` byte span when known. Parse diagnostics carry one; check
    /// diagnostics are message-only today (an honest partial — the `CheckError`
    /// variants do not yet carry spans).
    pub span: Option<(usize, usize)>,
}

/// Map a single checker diagnostic to its structured form. Severity and code
/// come from `garnet-check`'s canonical accessors (S44), so the CLI, the JSON
/// wire form, and the LSP cannot drift apart.
pub fn from_check_error(err: &CheckError) -> Diagnostic {
    Diagnostic {
        severity: err.severity().into(),
        code: err.code(),
        message: err.to_string(),
        span: None,
    }
}

/// Structured diagnostics for an entire check report, preserving order.
pub fn from_check_report(report: &CheckReport) -> Vec<Diagnostic> {
    report.errors.iter().map(from_check_error).collect()
}

/// Map a parse error to a structured diagnostic (always `Error` severity, with
/// the variant's source span).
pub fn from_parse_error(err: &ParseError) -> Diagnostic {
    let (code, span) = match err {
        ParseError::UnexpectedChar { span, .. } => ("parse.unexpected_char", *span),
        ParseError::UnterminatedString { span } => ("parse.unterminated_string", *span),
        ParseError::InvalidInt { span } => ("parse.invalid_int", *span),
        ParseError::InvalidFloat { span } => ("parse.invalid_float", *span),
        ParseError::UnexpectedToken { span, .. } => ("parse.unexpected_token", *span),
        ParseError::UnexpectedEof { span, .. } => ("parse.unexpected_eof", *span),
        ParseError::BudgetExceeded { span, .. } => ("parse.budget_exceeded", *span),
        ParseError::ReservedWord { span, .. } => ("parse.reserved_word", *span),
    };
    Diagnostic {
        severity: Severity::Error,
        code,
        message: err.to_string(),
        span: Some((span.start, span.len)),
    }
}

/// Serialize diagnostics as deterministic JSON: a `diagnostics` array (in order)
/// plus a `summary`. Field order is fixed; the only variability is the data.
pub fn to_json(diagnostics: &[Diagnostic]) -> String {
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut infos = 0usize;
    let mut items = Vec::with_capacity(diagnostics.len());
    for d in diagnostics {
        match d.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
            Severity::Info => infos += 1,
        }
        let span = match d.span {
            Some((start, len)) => format!("{{\"start\":{start},\"len\":{len}}}"),
            None => "null".to_string(),
        };
        items.push(format!(
            "{{\"severity\":\"{}\",\"code\":\"{}\",\"message\":\"{}\",\"span\":{}}}",
            d.severity.wire(),
            d.code,
            json_escape(&d.message),
            span
        ));
    }
    let ok = errors == 0;
    format!(
        "{{\"diagnostics\":[{}],\"summary\":{{\"errors\":{errors},\"warnings\":{warnings},\"infos\":{infos},\"ok\":{ok}}}}}",
        items.join(",")
    )
}

/// Escape a string for embedding in a JSON double-quoted value.
pub(crate) fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use garnet_parser::token::Span;

    #[test]
    fn check_error_severity_and_code_mapping() {
        assert_eq!(
            from_check_error(&CheckError::AnnotationError("x".into())).code,
            "check.annotation_error"
        );
        assert_eq!(
            from_check_error(&CheckError::AnnotationError("x".into())).severity,
            Severity::Error
        );
        assert_eq!(
            from_check_error(&CheckError::BoundaryNote("x".into())).severity,
            Severity::Warning
        );
        assert_eq!(
            from_check_error(&CheckError::StabilityAdvice("x".into())).severity,
            Severity::Info
        );
        let caps = CheckError::CapsCoverage {
            fn_name: "f".into(),
            missing: "fs".into(),
            via: "g".into(),
        };
        let d = from_check_error(&caps);
        assert_eq!(d.code, "check.caps_coverage");
        assert_eq!(d.severity, Severity::Error);
        assert!(d.span.is_none());
    }

    #[test]
    fn parse_error_carries_code_and_span() {
        let err = ParseError::reserved_word("async", "v2.0", Span::new(13, 5));
        let d = from_parse_error(&err);
        assert_eq!(d.code, "parse.reserved_word");
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.span, Some((13, 5)));
    }

    #[test]
    fn json_is_valid_and_escapes_messages() {
        let d = Diagnostic {
            severity: Severity::Error,
            code: "check.safe_mode_violation",
            message: "needs a \"quote\" and a\nnewline".to_string(),
            span: None,
        };
        let json = to_json(std::slice::from_ref(&d));
        assert!(json.contains(r#""severity":"error""#));
        assert!(json.contains(r#""code":"check.safe_mode_violation""#));
        assert!(
            json.contains(r#"\"quote\""#),
            "quotes must be escaped: {json}"
        );
        assert!(json.contains(r#"\n"#), "newlines must be escaped: {json}");
        assert!(json.contains(r#""errors":1"#));
        assert!(json.contains(r#""ok":false"#));
    }

    #[test]
    fn json_is_deterministic() {
        let diags = vec![
            from_check_error(&CheckError::AnnotationError("a".into())),
            from_check_error(&CheckError::BoundaryNote("b".into())),
        ];
        assert_eq!(to_json(&diags), to_json(&diags));
    }

    #[test]
    fn empty_report_is_ok() {
        let json = to_json(&[]);
        assert!(json.contains(r#""diagnostics":[]"#));
        assert!(json.contains(r#""ok":true"#));
    }
}
