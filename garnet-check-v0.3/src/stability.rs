//! S17 — registry-driven `@stability` enforcement.
//!
//! Reads each primitive's [`Stability`] tier from `garnet_stdlib::registry`
//! and emits a **non-fatal** advisory at every call site into a non-`stable`
//! primitive (Layer Policy §4 enforcement table):
//!
//! | callee tier  | advisory  |
//! |--------------|-----------|
//! | `stable`     | (none)    |
//! | `experimental` | warning |
//! | `deprecated` | warning   |
//! | `frozen`     | info      |
//!
//! Call resolution is **qualified-only**: a call is advised only when it
//! names a primitive by its full path (`std::json::parse`). Bare-name calls
//! (`parse`, `ok`, `map`, `replace`) are deliberately NOT advised — a bare
//! name is ambiguous with Garnet's built-in `Ok`/`Err`/`Some` builders, user
//! functions, and stable primitives sharing the name, so flagging it would
//! false-positive on existing code. This is intentionally more conservative
//! than `caps_graph` (which unions caps across bare-name collisions): an
//! advisory must never fire on a program that did nothing experimental.
//!
//! ## Scope (calibrated honesty)
//!
//! This pass enforces stability for **primitives**, whose tier is registry
//! metadata. Source-level `@stability(...)`/`@uses(experimental)`/
//! `@migration(...)` on **user-defined** functions require parser annotation
//! variants that do not exist yet (win-opus → mac-opus Handoff Request); they
//! are NOT consulted here and land in a follow-up. Until then the
//! experimental/deprecated advisories are un-suppressable, which is the safe
//! default.

use crate::CheckError;
use garnet_parser::ast::{Block, Expr, FnDef, Item, Module, Stmt};
use garnet_stdlib::registry::Stability;
use std::collections::{BTreeMap, BTreeSet};

/// A primitive name → its stability tier. Indexed by BOTH the qualified
/// registry key (`std::json::parse`) and the bare last segment (`parse`),
/// matching how `caps_graph` resolves both call shapes.
pub type StabilityMap = BTreeMap<String, Stability>;

/// S29 — whether opt-in **error-level** `@stability` enforcement is enabled.
/// Reads the `GARNET_STABILITY_ERRORS` env var (`1` or `true`). Default: off,
/// i.e. warning-level advisories (the v0.7 behavior, kept for backward compat).
/// This is the Layer Policy §4 "error-level enforcement is v0.8" line, shipped
/// as an opt-in so existing programs and CI stay green by default.
pub fn stability_error_mode() -> bool {
    matches!(
        std::env::var("GARNET_STABILITY_ERRORS").as_deref(),
        Ok("1") | Ok("true")
    )
}

/// Public entry point: advise on `@stability` for every primitive call site
/// in `module`, reading tiers from the live stdlib registry. When the opt-in
/// error mode is enabled, experimental/deprecated call sites become FATAL
/// [`CheckError::StabilityError`]s instead of non-fatal advisories.
pub fn check_stability(module: &Module) -> Vec<CheckError> {
    advise(module, &registry_stability_map(), stability_error_mode())
}

/// Build the primitive→stability map from the stdlib registry, indexed by the
/// QUALIFIED key only (e.g. `std::regex::replace`). Bare names are deliberately
/// not indexed: a bare call like `ok`, `err`, `map`, or `replace` is ambiguous
/// with Garnet's built-in `Ok`/`Err`/`Some` builders, user-defined functions,
/// and stable primitives that share the name — flagging it would false-positive
/// on existing code (e.g. `safe_io_layer.garnet`'s `ok(...)`/`err(...)`). Only
/// an unambiguous qualified reference is advised.
fn registry_stability_map() -> StabilityMap {
    garnet_stdlib::registry::all_prims()
        .into_iter()
        .map(|(qualified, meta)| (qualified, meta.stability))
        .collect()
}

/// Core policy: walk every function in `module` and, for each call into a
/// primitive present in `stability` with a non-`stable` tier, push one
/// advisory (deduped per `(fn, primitive)` pair, deterministic order).
/// Separated from the registry so it is unit-testable with synthetic maps.
fn advise(module: &Module, stability: &StabilityMap, error_mode: bool) -> Vec<CheckError> {
    let mut audit = Audit {
        stability,
        error_mode,
        seen: BTreeSet::new(),
        out: Vec::new(),
    };
    audit.walk_items(&module.items);
    audit.out
}

struct Audit<'a> {
    stability: &'a StabilityMap,
    /// When true, experimental/deprecated call sites are fatal errors (S29).
    error_mode: bool,
    seen: BTreeSet<(String, String)>,
    out: Vec<CheckError>,
}

impl Audit<'_> {
    fn walk_items(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::Fn(f) => self.walk_fn(f),
                Item::Module(m) => self.walk_items(&m.items),
                Item::Impl(b) => {
                    for m in &b.methods {
                        self.walk_fn(m);
                    }
                }
                _ => {}
            }
        }
    }

    fn walk_fn(&mut self, f: &FnDef) {
        let mut callees: Vec<String> = Vec::new();
        self.collect_block(&f.body, &mut callees);
        for prim in callees {
            if let Some(tier) = self.stability.get(&prim).copied() {
                if matches!(tier, Stability::Stable) {
                    continue;
                }
                if self.seen.insert((f.name.clone(), prim.clone())) {
                    self.out
                        .push(advisory(&f.name, &prim, tier, self.error_mode));
                }
            }
        }
    }

    fn collect_block(&self, b: &Block, out: &mut Vec<String>) {
        for s in &b.stmts {
            self.collect_stmt(s, out);
        }
        if let Some(t) = &b.tail_expr {
            self.collect_expr(t, out);
        }
    }

    fn collect_stmt(&self, s: &Stmt, out: &mut Vec<String>) {
        match s {
            Stmt::Expr(e) => self.collect_expr(e, out),
            Stmt::Let(d) => self.collect_expr(&d.value, out),
            Stmt::Var(d) => self.collect_expr(&d.value, out),
            Stmt::Const(d) => self.collect_expr(&d.value, out),
            Stmt::Assign { target, value, .. } => {
                self.collect_expr(target, out);
                self.collect_expr(value, out);
            }
            Stmt::Return { value: Some(e), .. }
            | Stmt::Yield { value: Some(e), .. }
            | Stmt::Next { value: Some(e), .. }
            | Stmt::Raise { value: e, .. }
            | Stmt::Break { value: Some(e), .. } => self.collect_expr(e, out),
            Stmt::While {
                condition, body, ..
            } => {
                self.collect_expr(condition, out);
                self.collect_block(body, out);
            }
            Stmt::For { iter, body, .. } => {
                self.collect_expr(iter, out);
                self.collect_block(body, out);
            }
            Stmt::Loop { body, .. } => self.collect_block(body, out),
            _ => {}
        }
    }

    fn collect_expr(&self, e: &Expr, out: &mut Vec<String>) {
        match e {
            Expr::Call { callee, args, .. } => {
                if let Some(prim) = self.resolve_prim(callee) {
                    out.push(prim);
                }
                self.collect_expr(callee, out);
                for a in args {
                    self.collect_expr(a, out);
                }
            }
            Expr::Method { receiver, args, .. } => {
                self.collect_expr(receiver, out);
                for a in args {
                    self.collect_expr(a, out);
                }
            }
            Expr::Binary { lhs, rhs, .. } => {
                self.collect_expr(lhs, out);
                self.collect_expr(rhs, out);
            }
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } => self.collect_expr(expr, out),
            Expr::Field { receiver, .. } => self.collect_expr(receiver, out),
            Expr::Index {
                receiver, index, ..
            } => {
                self.collect_expr(receiver, out);
                self.collect_expr(index, out);
            }
            Expr::If {
                condition,
                then_block,
                elsif_clauses,
                else_block,
                ..
            } => {
                self.collect_expr(condition, out);
                self.collect_block(then_block, out);
                for (c, b) in elsif_clauses {
                    self.collect_expr(c, out);
                    self.collect_block(b, out);
                }
                if let Some(b) = else_block {
                    self.collect_block(b, out);
                }
            }
            Expr::Match { subject, arms, .. } => {
                self.collect_expr(subject, out);
                for arm in arms {
                    self.collect_block(&arm.body, out);
                }
            }
            Expr::Try {
                body,
                rescues,
                ensure,
                ..
            } => {
                self.collect_block(body, out);
                for r in rescues {
                    self.collect_block(&r.body, out);
                }
                if let Some(e) = ensure {
                    self.collect_block(e, out);
                }
            }
            Expr::Array { elements, .. } => {
                for el in elements {
                    self.collect_expr(el, out);
                }
            }
            Expr::Map { entries, .. } => {
                for (k, v) in entries {
                    self.collect_expr(k, out);
                    self.collect_expr(v, out);
                }
            }
            Expr::Spawn { expr, .. } => self.collect_expr(expr, out),
            _ => {}
        }
    }

    /// Resolve a call's callee to a primitive registry key, or `None` if it
    /// is a user function (which shadows a primitive) or unresolvable.
    fn resolve_prim(&self, callee: &Expr) -> Option<String> {
        match callee {
            // Only an unambiguous QUALIFIED path is flagged. Bare-name calls
            // (`Expr::Ident`) are skipped: they collide with built-in builders
            // (`ok`/`err`/`some`), user functions, and stable primitives, so
            // flagging them would false-positive on existing code.
            Expr::Path(segs, _) => {
                let qualified = segs.join("::");
                self.stability.contains_key(&qualified).then_some(qualified)
            }
            _ => None,
        }
    }
}

/// Build the diagnostic for a (caller, primitive, tier). Under `error_mode`
/// (S29), experimental/deprecated call sites become FATAL `StabilityError`s
/// (severity word "error"); frozen stays informational and `stable` is silent.
/// In the default mode every non-stable tier is a non-fatal `StabilityAdvice`,
/// byte-for-byte the v0.7 wording.
fn advisory(fn_name: &str, prim: &str, tier: Stability, error_mode: bool) -> CheckError {
    // Frozen is "supported but won't grow" — informational, never an error.
    let is_error = error_mode && matches!(tier, Stability::Experimental | Stability::Deprecated);
    let severity = if is_error {
        "error"
    } else if matches!(tier, Stability::Frozen) {
        "info"
    } else {
        "warning"
    };
    let detail = match tier {
        Stability::Experimental => format!(
            "calls experimental primitive `{prim}`; its API may change between minor releases"
        ),
        Stability::Deprecated => format!(
            "calls deprecated primitive `{prim}`; it is scheduled for removal — migrate to its \
             replacement"
        ),
        Stability::Frozen => {
            format!("calls frozen primitive `{prim}`; it is supported but will not grow")
        }
        // `stable` never produces a diagnostic; included for exhaustiveness.
        Stability::Stable => format!("calls `{prim}`"),
    };
    let msg = format!("stability {severity}: function `{fn_name}` {detail}");
    if is_error {
        CheckError::StabilityError(msg)
    } else {
        CheckError::StabilityAdvice(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Module {
        garnet_parser::parse_source(src).expect("parse failed")
    }

    fn msgs(errs: &[CheckError]) -> Vec<String> {
        errs.iter()
            .filter_map(|e| match e {
                CheckError::StabilityAdvice(m) => Some(m.clone()),
                _ => None,
            })
            .collect()
    }

    // ── End-to-end against the real registry ────────────────────────────

    #[test]
    fn calling_experimental_prim_warns_via_registry() {
        // `core::iter::map` ships @stability(experimental) in v0.7.
        let m = parse(
            r#"
            @caps()
            def main() {
                core::iter::map(xs, f)
            }
            "#,
        );
        let out = check_stability(&m);
        let ms = msgs(&out);
        assert!(
            ms.iter().any(|m| m.contains("experimental")
                && m.contains("core::iter::map")
                && m.contains("warning")),
            "expected experimental warning for core::iter::map, got {ms:?}"
        );
    }

    #[test]
    fn calling_only_stable_prims_is_silent() {
        // `trim` and `read_file` are @stability(stable); mirrors the kind of
        // call existing examples make. No stability advisory expected.
        let m = parse(
            r#"
            @caps(fs)
            def main() {
                trim("  hi  ")
                read_file("p.txt")
            }
            "#,
        );
        assert!(
            msgs(&check_stability(&m)).is_empty(),
            "stable-only calls must not produce stability advisories"
        );
    }

    #[test]
    fn bare_call_to_experimental_prim_name_is_not_flagged() {
        // Regression (safe_io_layer.garnet): bare `ok(...)`/`err(...)` are
        // Garnet's built-in Result builders. `core::result::ok`/`err` are
        // experimental in the registry, but a BARE call must NOT be flagged —
        // only a qualified `core::result::ok(...)` would. Bare names are too
        // ambiguous (builders, user fns, stable prims) to advise on.
        let m = parse(
            r#"
            @caps()
            def main() {
                ok(1)
                err(2)
                map(xs, f)
                replace("a", "b", "c")
            }
            "#,
        );
        assert!(
            msgs(&check_stability(&m)).is_empty(),
            "bare ok/err/map/replace must not be flagged (ambiguous), got {:?}",
            msgs(&check_stability(&m))
        );
    }

    // ── Core policy against synthetic maps (all four tiers) ─────────────

    fn map_of(pairs: &[(&str, Stability)]) -> StabilityMap {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn deprecated_emits_warning_frozen_emits_info() {
        let m = parse(
            r#"
            @caps()
            def main() {
                pkg::gone(x)
                pkg::kept(x)
            }
            "#,
        );
        let map = map_of(&[
            ("pkg::gone", Stability::Deprecated),
            ("pkg::kept", Stability::Frozen),
        ]);
        let ms = msgs(&advise(&m, &map, false));
        assert!(
            ms.iter()
                .any(|m| m.contains("deprecated") && m.contains("warning")),
            "expected deprecated warning, got {ms:?}"
        );
        assert!(
            ms.iter()
                .any(|m| m.contains("frozen") && m.contains("info")),
            "expected frozen info, got {ms:?}"
        );
    }

    #[test]
    fn advisory_is_deduped_per_fn_prim_pair() {
        let m = parse(
            r#"
            @caps()
            def main() {
                pkg::exp(x)
                pkg::exp(x)
                pkg::exp(x)
            }
            "#,
        );
        let map = map_of(&[("pkg::exp", Stability::Experimental)]);
        assert_eq!(
            msgs(&advise(&m, &map, false)).len(),
            1,
            "three calls to the same experimental prim should warn once"
        );
    }

    #[test]
    fn qualified_path_resolves_before_bare() {
        let m = parse(
            r#"
            @caps()
            def main() {
                std::json::parse(s)
            }
            "#,
        );
        let map = map_of(&[("std::json::parse", Stability::Experimental)]);
        let ms = msgs(&advise(&m, &map, false));
        assert!(
            ms.iter().any(|m| m.contains("std::json::parse")),
            "qualified call should resolve to the qualified key, got {ms:?}"
        );
    }

    // ── S29: opt-in error-level enforcement ─────────────────────────────

    /// Collect FATAL `StabilityError` messages (vs the non-fatal advisories).
    fn errors(errs: &[CheckError]) -> Vec<String> {
        errs.iter()
            .filter_map(|e| match e {
                CheckError::StabilityError(m) => Some(m.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn s29_error_mode_promotes_experimental_and_deprecated_to_fatal() {
        let m = parse(
            r#"
            @caps()
            def main() {
                pkg::exp(x)
                pkg::gone(x)
            }
            "#,
        );
        let map = map_of(&[
            ("pkg::exp", Stability::Experimental),
            ("pkg::gone", Stability::Deprecated),
        ]);
        let out = advise(&m, &map, true);
        // Both become fatal StabilityErrors carrying the "error" severity word…
        let errs = errors(&out);
        assert_eq!(
            errs.len(),
            2,
            "both non-stable calls become errors, got {errs:?}"
        );
        assert!(errs.iter().all(|m| m.starts_with("stability error:")));
        assert!(errs
            .iter()
            .any(|m| m.contains("experimental") && m.contains("pkg::exp")));
        assert!(errs
            .iter()
            .any(|m| m.contains("deprecated") && m.contains("pkg::gone")));
        // …and NONE remain as non-fatal advisories.
        assert!(
            msgs(&out).is_empty(),
            "error mode must not also emit advisories, got {:?}",
            msgs(&out)
        );
    }

    #[test]
    fn s29_error_mode_keeps_frozen_informational() {
        // Frozen is "supported but won't grow" — it must stay a non-fatal info
        // advisory even under error mode.
        let m = parse(
            r#"
            @caps()
            def main() {
                pkg::kept(x)
            }
            "#,
        );
        let map = map_of(&[("pkg::kept", Stability::Frozen)]);
        let out = advise(&m, &map, true);
        assert!(errors(&out).is_empty(), "frozen must never be an error");
        assert!(
            msgs(&out)
                .iter()
                .any(|m| m.contains("frozen") && m.contains("info")),
            "frozen stays an info advisory, got {:?}",
            msgs(&out)
        );
    }

    #[test]
    fn s29_default_mode_stays_warning_level_non_fatal() {
        // Without error mode the v0.7 behavior is unchanged: experimental →
        // non-fatal warning advisory, byte-for-byte the prior wording.
        let m = parse(
            r#"
            @caps()
            def main() {
                core::iter::map(xs, f)
            }
            "#,
        );
        let out = advise(
            &m,
            &map_of(&[("core::iter::map", Stability::Experimental)]),
            false,
        );
        assert!(
            errors(&out).is_empty(),
            "default mode emits no fatal errors"
        );
        assert_eq!(
            msgs(&out),
            vec![
                "stability warning: function `main` calls experimental primitive \
                 `core::iter::map`; its API may change between minor releases"
                    .to_string()
            ]
        );
    }
}
