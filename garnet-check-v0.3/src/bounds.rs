//! S39 — `@bounded` resource-budget extraction.
//!
//! `@bounded(N)` declares a CPU / fuel budget of N Wasmtime-fuel units;
//! enforcement lowers to Wasmtime fuel metering (the wrap). This extracts the
//! DECLARED budgets per function — the surface a fuel-metering backend consumes
//! — sorted by name. Purely syntactic.

use garnet_parser::ast::{Annotation, Item, Module};

/// Functions that declare an `@bounded(N)` fuel budget, as `(name, fuel)`,
/// sorted by function name. If a function carries multiple `@bounded`, the last
/// wins (the checker rejects nonsensical values separately).
pub fn bounded_functions(module: &Module) -> Vec<(String, i64)> {
    let mut out: Vec<(String, i64)> = Vec::new();
    for item in &module.items {
        let Item::Fn(f) = item else { continue };
        let mut budget: Option<i64> = None;
        for ann in &f.annotations {
            if let Annotation::Bounded(n, _) = ann {
                budget = Some(*n);
            }
        }
        if let Some(n) = budget {
            out.push((f.name.clone(), n));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use garnet_parser::parse_source;

    fn bounds(src: &str) -> Vec<(String, i64)> {
        bounded_functions(&parse_source(src).expect("parses"))
    }

    #[test]
    fn extracts_declared_budgets_sorted_by_name() {
        let b = bounds("@bounded(2000)\ndef zebra() { 1 }\n@bounded(500)\ndef alpha() { 1 }\n");
        assert_eq!(
            b,
            vec![("alpha".to_string(), 500), ("zebra".to_string(), 2000)]
        );
    }

    #[test]
    fn functions_without_bounded_are_absent() {
        let b = bounds("def plain() { 1 }\n@bounded(10)\ndef g() { 1 }\n");
        assert_eq!(b, vec![("g".to_string(), 10)]);
    }

    #[test]
    fn positive_bound_parses_and_checks_clean() {
        let module = parse_source("@bounded(1000)\ndef f() { 1 }\n").expect("parses");
        let report = crate::check_module(&module);
        assert!(
            !report.errors.iter().any(|e| matches!(
                e,
                crate::CheckError::AnnotationError(m) if m.contains("@bounded")
            )),
            "a positive @bounded budget must not raise an annotation error"
        );
    }

    #[test]
    fn zero_bound_is_a_check_error() {
        // Zero parses (a valid Int literal) and is caught by the checker.
        let module = parse_source("@bounded(0)\ndef f() { 1 }\n").expect("parses");
        let report = crate::check_module(&module);
        assert!(
            report.errors.iter().any(|e| matches!(
                e,
                crate::CheckError::AnnotationError(m) if m.contains("@bounded")
            )),
            "@bounded(0) must raise an annotation error"
        );
    }

    #[test]
    fn negative_bound_is_rejected_at_parse() {
        // A negative literal is a leading-minus token plus an Int, not a single
        // Int — so the single-int annotation arg rejects it at parse time
        // (consistent with @mailbox / @max_depth).
        assert!(
            parse_source("@bounded(-5)\ndef f() { 1 }\n").is_err(),
            "@bounded(-5) is not a valid integer literal and must be a parse error"
        );
    }
}
