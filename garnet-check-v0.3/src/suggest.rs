//! Rules-based advisory suggestions (S10).
//!
//! Deterministic, no-LLM suggestions emitted by the checker when invoked
//! through `garnet check --suggest`. Every suggestion is structured as a
//! `Suggestion` value and rendered with the literal prefix
//! "compiler suggested:" so downstream tooling (and the contract's dogfood
//! block) can grep for advisories.
//!
//! Out of scope for S10:
//! - LLM-derived suggestions (pending Paper VI Exp 1 infrastructure).
//! - Auto-apply / quick-fix wiring into LSP code-actions.
//! - Cross-module suggestions (current rules are intra-module only).
//!
//! Adding a new rule:
//! 1. Add a `Rule` variant + its description here.
//! 2. Implement an `inspect_*` function that walks the AST and appends
//!    `Suggestion` values when it finds a match.
//! 3. Call it from `suggest_for_module`.
//! 4. Add a fixture file under `tests/suggest_corpus/` and assert the rule
//!    fires.

use garnet_parser::ast::{Annotation, Block, FnDef, FnMode, Item, Module};

/// One rules-based advisory suggestion attached to a span of source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub rule: Rule,
    pub function: String,
    pub message: String,
}

/// Catalog of S10 rules. Add new rules here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rule {
    /// A managed-mode `def` function is missing an `@caps(...)` annotation.
    /// Even purely-computational functions benefit from an explicit `@caps()`
    /// so the CapCaps propagator can reason about the call graph and so
    /// readers see the intended capability surface up front.
    ManagedFnMissingCaps,
    /// A function declares 4 or more parameters. Grouping related parameters
    /// into a struct typically improves call-site readability and lets the
    /// type system catch swapped-argument bugs.
    LongParameterList,
    /// A function body is empty (no statements, no tail expression). The
    /// function compiles and returns `Nil`, but readers will rarely
    /// recognize that as intentional.
    EmptyFunctionBody,
}

impl Rule {
    pub fn id(self) -> &'static str {
        match self {
            Rule::ManagedFnMissingCaps => "managed-fn-missing-caps",
            Rule::LongParameterList => "long-parameter-list",
            Rule::EmptyFunctionBody => "empty-function-body",
        }
    }
}

/// Threshold: parameter count at or above which the long-parameter-list rule
/// fires. Picked conservatively. Lowering this is a separate, reviewed change
/// — the contract names "3 patterns produce suggestions"; lowering the bar
/// changes the corpus impact and must be reviewed accordingly.
pub const LONG_PARAMETER_THRESHOLD: usize = 4;

/// Run all S10 rules against a parsed module and return the resulting
/// `Suggestion` list. Stable, deterministic, no I/O.
pub fn suggest_for_module(module: &Module) -> Vec<Suggestion> {
    let mut out = Vec::new();
    for item in &module.items {
        if let Item::Fn(f) = item {
            inspect_fn(f, &mut out);
        }
    }
    out
}

fn inspect_fn(f: &FnDef, out: &mut Vec<Suggestion>) {
    rule_managed_fn_missing_caps(f, out);
    rule_long_parameter_list(f, out);
    rule_empty_function_body(f, out);
}

fn rule_managed_fn_missing_caps(f: &FnDef, out: &mut Vec<Suggestion>) {
    if !matches!(f.mode, FnMode::Managed) {
        return;
    }
    let has_caps = f
        .annotations
        .iter()
        .any(|ann| matches!(ann, Annotation::Caps(_, _)));
    if has_caps {
        return;
    }
    out.push(Suggestion {
        rule: Rule::ManagedFnMissingCaps,
        function: f.name.clone(),
        message: format!(
            "compiler suggested: `def {}` has no `@caps(...)` annotation. \
            Adding `@caps()` (empty) makes purely-computational intent explicit; \
            adding `@caps(fs)` / `@caps(net)` / etc. declares OS authority up front. \
            Either is reviewable; an unannotated managed function is silent on its \
            capability surface.",
            f.name
        ),
    });
}

fn rule_long_parameter_list(f: &FnDef, out: &mut Vec<Suggestion>) {
    if f.params.len() < LONG_PARAMETER_THRESHOLD {
        return;
    }
    out.push(Suggestion {
        rule: Rule::LongParameterList,
        function: f.name.clone(),
        message: format!(
            "compiler suggested: `{}` declares {} parameters (threshold {}). \
            Grouping related parameters into a struct usually improves \
            call-site readability and lets the type system catch swapped \
            arguments.",
            f.name,
            f.params.len(),
            LONG_PARAMETER_THRESHOLD,
        ),
    });
}

fn rule_empty_function_body(f: &FnDef, out: &mut Vec<Suggestion>) {
    if !is_block_empty(&f.body) {
        return;
    }
    out.push(Suggestion {
        rule: Rule::EmptyFunctionBody,
        function: f.name.clone(),
        message: format!(
            "compiler suggested: `{}` has an empty body. \
            The function compiles and returns `Nil`, but readers rarely recognize that as \
            intentional. Add a comment or a placeholder return value to make the intent explicit.",
            f.name,
        ),
    });
}

fn is_block_empty(b: &Block) -> bool {
    b.stmts.is_empty() && b.tail_expr.is_none()
}

/// Render a `Suggestion` for terminal output (single line).
pub fn render(s: &Suggestion) -> String {
    format!("[{}] {}", s.rule.id(), s.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Module {
        garnet_parser::parse_source(src).expect("source should parse")
    }

    #[test]
    fn managed_fn_without_caps_triggers_suggestion() {
        let src = "def helper() { 1 + 1 }\n";
        let suggestions = suggest_for_module(&parse(src));
        assert_eq!(1, suggestions.len(), "{suggestions:?}");
        assert_eq!(Rule::ManagedFnMissingCaps, suggestions[0].rule);
        assert!(suggestions[0].message.contains("compiler suggested"));
    }

    #[test]
    fn managed_fn_with_caps_does_not_trigger() {
        let src = "@caps()\ndef helper() { 1 + 1 }\n";
        let suggestions = suggest_for_module(&parse(src));
        assert!(
            suggestions.is_empty(),
            "expected no suggestions, got {suggestions:?}"
        );
    }

    #[test]
    fn long_parameter_list_triggers_suggestion() {
        let src = "@caps()\ndef big(a, b, c, d, e) { a }\n";
        let suggestions = suggest_for_module(&parse(src));
        let long: Vec<&Suggestion> = suggestions
            .iter()
            .filter(|s| s.rule == Rule::LongParameterList)
            .collect();
        assert_eq!(1, long.len(), "{suggestions:?}");
        assert!(long[0].message.contains("5 parameters"));
    }

    #[test]
    fn empty_body_triggers_suggestion() {
        let src = "@caps()\ndef nothing() { }\n";
        let suggestions = suggest_for_module(&parse(src));
        let empty: Vec<&Suggestion> = suggestions
            .iter()
            .filter(|s| s.rule == Rule::EmptyFunctionBody)
            .collect();
        assert_eq!(1, empty.len(), "{suggestions:?}");
    }

    #[test]
    fn safe_mode_fn_does_not_trigger_managed_caps_rule() {
        // Safe-mode `fn` has its own caps-coverage error path; this rule only
        // applies to managed `def` to avoid double-firing.
        let src = "@safe\nfn helper() -> i64 { 1 }\n";
        let suggestions = suggest_for_module(&parse(src));
        for s in &suggestions {
            assert_ne!(s.rule, Rule::ManagedFnMissingCaps, "{s:?}");
        }
    }
}
