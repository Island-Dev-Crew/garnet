//! Property-based tests for the safe-mode checker.

use garnet_check::{check_module, CheckError};
use garnet_parser::parse_source;
use proptest::prelude::*;

proptest! {
    #[test]
    fn mode_map_total_over_function_items(n in 1usize..15) {
        let body: Vec<String> = (0..n).map(|i| format!("def f{i}() {{ {i} }}")).collect();
        let src = body.join("\n");
        let m = parse_source(&src).unwrap();
        let r = check_module(&m);
        prop_assert_eq!(r.mode_map.len(), n);
    }
}

proptest! {
    #[test]
    fn boundary_call_count_matches_call_expressions(call_count in 0usize..20) {
        let calls: Vec<String> = (0..call_count).map(|i| format!("f{i}()")).collect();
        let helpers: Vec<String> = (0..call_count).map(|i| format!("def f{i}() {{ 0 }}")).collect();
        let body = if calls.is_empty() {
            "0".to_string()
        } else {
            calls.join(" + ")
        };
        let src = format!("{}\ndef caller() {{ {body} }}", helpers.join("\n"));
        let m = parse_source(&src).unwrap();
        let r = check_module(&m);
        prop_assert!(
            r.boundary_call_sites >= call_count,
            "expected at least {call_count} boundary calls, got {}",
            r.boundary_call_sites
        );
    }
}

proptest! {
    #[test]
    fn check_module_is_idempotent(n in 1usize..10) {
        let body: Vec<String> = (0..n).map(|i| format!("def f{i}() {{ {i} }}")).collect();
        let src = body.join("\n");
        let m = parse_source(&src).unwrap();
        let r1 = check_module(&m);
        let r2 = check_module(&m);
        prop_assert_eq!(r1.mode_map.len(), r2.mode_map.len());
        prop_assert_eq!(r1.boundary_call_sites, r2.boundary_call_sites);
        prop_assert_eq!(r1.errors.len(), r2.errors.len());
    }
}

proptest! {
    #[test]
    fn safe_module_var_use_always_flagged(n in 1usize..8) {
        // Every safe-tagged def with `var` should produce one violation per occurrence.
        let mut src = String::from("@safe\n");
        for i in 0..n {
            src.push_str(&format!("def bad{i}() {{ var x{i} = 1\n x{i} }}\n"));
        }
        let m = parse_source(&src).unwrap();
        let r = check_module(&m);
        let var_violations = r.errors.iter().filter(|e| matches!(e, CheckError::SafeModeViolation(m) if m.contains("var"))).count();
        prop_assert!(var_violations >= n, "expected at least {n} var violations, got {var_violations}");
    }
}
