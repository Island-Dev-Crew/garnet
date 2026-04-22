//! ModeAuditLog — every fn↔def boundary crossing emitted into a
//! structured audit log (v3.5 Security Layer 3).
//!
//! Closes the "hidden safe→managed escalation" threat class. When a
//! reviewer sees a Garnet codebase, they can read *one file* — the
//! audit log shipped with the manifest — and enumerate every trust
//! boundary crossing in the program. No more grepping for "fn " vs
//! "def " across hundreds of modules.
//!
//! The log entries are:
//!
//!   <source-span> <caller-mode> -> <callee-mode> <callee-name>
//!
//! Example:
//!
//!   examples/mvp_02_relational_db.garnet:32 managed -> safe BTree::compare
//!   examples/mvp_02_relational_db.garnet:47 safe -> managed RelDb::trim
//!
//! ## Lint
//!
//! `warn_if_audit_log_grows_faster_than_source` watches the ratio of
//! audit-entries per LOC. If the program is growing its boundary
//! crossings faster than its code size, that's a design smell —
//! suggests the dual-mode boundary is being forgotten.

use garnet_parser::ast::FnMode;
use garnet_parser::token::Span;
use std::fmt;

/// One audit-log entry representing a single boundary-crossing call site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryCall {
    pub caller_name: String,
    pub caller_mode: FnMode,
    pub callee_name: String,
    pub callee_mode: FnMode,
    pub span: Span,
}

impl BoundaryCall {
    pub fn direction(&self) -> BoundaryDirection {
        match (self.caller_mode, self.callee_mode) {
            (FnMode::Managed, FnMode::Safe) => BoundaryDirection::ManagedToSafe,
            (FnMode::Safe, FnMode::Managed) => BoundaryDirection::SafeToManaged,
            (FnMode::Managed, FnMode::Managed) => BoundaryDirection::ManagedInternal,
            (FnMode::Safe, FnMode::Safe) => BoundaryDirection::SafeInternal,
        }
    }

    /// Structured line for the shipped audit log.
    pub fn audit_line(&self) -> String {
        format!(
            "{}:{}:{} {} -> {} {}",
            "<source>", // filled in by the writer when it has the path
            self.span.start,
            self.span.end(),
            mode_tag(self.caller_mode),
            mode_tag(self.callee_mode),
            self.callee_name,
        )
    }
}

fn mode_tag(m: FnMode) -> &'static str {
    match m {
        FnMode::Managed => "managed",
        FnMode::Safe => "safe",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryDirection {
    /// A `def` calling a `fn` — the most common and the most
    /// security-interesting direction because managed code is invoking
    /// the ownership-disciplined world.
    ManagedToSafe,
    /// A `fn` calling a `def` — a managed escape hatch from safe code.
    /// The v3.3 slop reverification caught this as Paper VI C5's
    /// bridge-direction gap; ModeAuditLog makes it visible forever.
    SafeToManaged,
    /// Internal-to-managed call. Not interesting for boundary audit
    /// purposes but logged for completeness when requested.
    ManagedInternal,
    /// Internal-to-safe call. Same rationale.
    SafeInternal,
}

impl fmt::Display for BoundaryDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            BoundaryDirection::ManagedToSafe => "managed→safe",
            BoundaryDirection::SafeToManaged => "safe→managed",
            BoundaryDirection::ManagedInternal => "managed→managed",
            BoundaryDirection::SafeInternal => "safe→safe",
        })
    }
}

/// Audit log aggregated across an entire compilation. Intended to be
/// emitted alongside the manifest at `garnet build --deterministic`.
#[derive(Debug, Default)]
pub struct AuditLog {
    pub entries: Vec<BoundaryCall>,
    /// Source-line count of the compilation unit — used by the
    /// "growing faster than source" lint.
    pub source_lines: usize,
}

impl AuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, entry: BoundaryCall) {
        self.entries.push(entry);
    }

    /// Count boundary crossings by direction. Returns (m→s, s→m, m→m, s→s).
    pub fn direction_counts(&self) -> (usize, usize, usize, usize) {
        let mut m2s = 0;
        let mut s2m = 0;
        let mut mm = 0;
        let mut ss = 0;
        for e in &self.entries {
            match e.direction() {
                BoundaryDirection::ManagedToSafe => m2s += 1,
                BoundaryDirection::SafeToManaged => s2m += 1,
                BoundaryDirection::ManagedInternal => mm += 1,
                BoundaryDirection::SafeInternal => ss += 1,
            }
        }
        (m2s, s2m, mm, ss)
    }

    /// The "grows faster than source" lint. Returns `Some(reason)` if
    /// boundary crossings per LOC exceeds `max_ratio`.
    pub fn warn_if_growing_faster_than_source(&self, max_ratio: f64) -> Option<String> {
        if self.source_lines == 0 {
            return None;
        }
        let boundary = (self.entries.len() as f64) / (self.source_lines as f64);
        if boundary > max_ratio {
            Some(format!(
                "audit log has {} boundary crossings in {} LOC \
                 (ratio {:.3} > max {:.3}) — consider consolidating",
                self.entries.len(),
                self.source_lines,
                boundary,
                max_ratio
            ))
        } else {
            None
        }
    }

    /// Dump the log in the shipped `.audit` format.
    pub fn to_audit_format(&self, source_path: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "# Garnet ModeAuditLog v1\n# source: {}\n# total: {} crossings across {} LOC\n",
            source_path,
            self.entries.len(),
            self.source_lines
        ));
        let (m2s, s2m, mm, ss) = self.direction_counts();
        out.push_str(&format!(
            "# managed->safe: {m2s}\n# safe->managed: {s2m}\n# managed->managed: {mm}\n# safe->safe: {ss}\n\n"
        ));
        for e in &self.entries {
            out.push_str(&format!(
                "{}:{}:{} {} -> {} {}\n",
                source_path,
                e.span.start,
                e.span.end(),
                mode_tag(e.caller_mode),
                mode_tag(e.callee_mode),
                e.callee_name,
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(a: usize, b: usize) -> Span {
        Span { start: a, len: b.saturating_sub(a) }
    }

    fn call(caller: &str, cm: FnMode, callee: &str, dm: FnMode) -> BoundaryCall {
        BoundaryCall {
            caller_name: caller.into(),
            caller_mode: cm,
            callee_name: callee.into(),
            callee_mode: dm,
            span: span(0, 0),
        }
    }

    #[test]
    fn direction_classification() {
        assert_eq!(
            call("a", FnMode::Managed, "b", FnMode::Safe).direction(),
            BoundaryDirection::ManagedToSafe
        );
        assert_eq!(
            call("a", FnMode::Safe, "b", FnMode::Managed).direction(),
            BoundaryDirection::SafeToManaged
        );
    }

    #[test]
    fn direction_counts_accumulate() {
        let mut log = AuditLog::new();
        log.push(call("x", FnMode::Managed, "y", FnMode::Safe));
        log.push(call("x", FnMode::Managed, "z", FnMode::Safe));
        log.push(call("y", FnMode::Safe, "w", FnMode::Managed));
        let (m2s, s2m, _, _) = log.direction_counts();
        assert_eq!(m2s, 2);
        assert_eq!(s2m, 1);
    }

    #[test]
    fn grow_lint_fires_when_ratio_too_high() {
        let mut log = AuditLog::new();
        log.source_lines = 10;
        for i in 0..6 {
            log.push(call("a", FnMode::Managed, &format!("b_{i}"), FnMode::Safe));
        }
        let warn = log.warn_if_growing_faster_than_source(0.5);
        assert!(warn.is_some());
    }

    #[test]
    fn grow_lint_quiet_under_ratio() {
        let mut log = AuditLog::new();
        log.source_lines = 100;
        for i in 0..5 {
            log.push(call("a", FnMode::Managed, &format!("b_{i}"), FnMode::Safe));
        }
        assert!(log.warn_if_growing_faster_than_source(0.5).is_none());
    }

    #[test]
    fn audit_format_includes_header_and_entries() {
        let mut log = AuditLog::new();
        log.source_lines = 50;
        log.push(BoundaryCall {
            caller_name: "main".into(),
            caller_mode: FnMode::Managed,
            callee_name: "hash".into(),
            callee_mode: FnMode::Safe,
            span: span(10, 20),
        });
        let out = log.to_audit_format("src/main.garnet");
        assert!(out.contains("# Garnet ModeAuditLog v1"));
        assert!(out.contains("src/main.garnet"));
        assert!(out.contains("managed -> safe hash"));
        assert!(out.contains("managed->safe: 1"));
    }
}
