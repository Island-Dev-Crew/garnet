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
//! Call resolution mirrors `caps_graph`: qualified path first
//! (`std::json::parse`), then the bare last segment (`parse`), and a
//! user-defined function of the same name shadows a primitive.
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

/// Public entry point: advise on `@stability` for every primitive call site
/// in `module`, reading tiers from the live stdlib registry.
pub fn check_stability(module: &Module) -> Vec<CheckError> {
    advise(module, &registry_stability_map())
}

/// Build the primitive→stability map from the stdlib registry. On a bare-name
/// collision (e.g. `core::iter::map` vs `core::result::map`) keep the most
/// volatile tier so the advisory never under-reports.
fn registry_stability_map() -> StabilityMap {
    let mut map: StabilityMap = BTreeMap::new();
    for (qualified, meta) in garnet_stdlib::registry::all_prims() {
        map.insert(qualified.clone(), meta.stability);
        if let Some(bare) = qualified.split("::").last() {
            map.entry(bare.to_string())
                .and_modify(|existing| {
                    if volatility(meta.stability) > volatility(*existing) {
                        *existing = meta.stability;
                    }
                })
                .or_insert(meta.stability);
        }
    }
    map
}

/// Higher = more deserving of a louder advisory on a bare-name collision.
fn volatility(s: Stability) -> u8 {
    match s {
        Stability::Stable => 0,
        Stability::Frozen => 1,
        Stability::Experimental => 2,
        Stability::Deprecated => 3,
    }
}

/// Core policy: walk every function in `module` and, for each call into a
/// primitive present in `stability` with a non-`stable` tier, push one
/// advisory (deduped per `(fn, primitive)` pair, deterministic order).
/// Separated from the registry so it is unit-testable with synthetic maps.
fn advise(module: &Module, stability: &StabilityMap) -> Vec<CheckError> {
    let user_fns = collect_user_fns(module);
    let mut audit = Audit {
        stability,
        user_fns: &user_fns,
        seen: BTreeSet::new(),
        out: Vec::new(),
    };
    audit.walk_items(&module.items);
    audit.out
}

fn collect_user_fns(module: &Module) -> BTreeSet<String> {
    fn go(items: &[Item], out: &mut BTreeSet<String>) {
        for item in items {
            match item {
                Item::Fn(f) => {
                    out.insert(f.name.clone());
                }
                Item::Module(m) => go(&m.items, out),
                Item::Impl(b) => {
                    for m in &b.methods {
                        out.insert(m.name.clone());
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = BTreeSet::new();
    go(&module.items, &mut out);
    out
}

struct Audit<'a> {
    stability: &'a StabilityMap,
    user_fns: &'a BTreeSet<String>,
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
                    self.out.push(advisory(&f.name, &prim, tier));
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
            Expr::Ident(name, _) => {
                if self.user_fns.contains(name) {
                    return None;
                }
                self.stability.contains_key(name).then(|| name.clone())
            }
            Expr::Path(segs, _) => {
                let qualified = segs.join("::");
                if self.stability.contains_key(&qualified) {
                    return Some(qualified);
                }
                let last = segs.last()?;
                if self.user_fns.contains(last) {
                    return None;
                }
                self.stability.contains_key(last).then(|| last.clone())
            }
            _ => None,
        }
    }
}

/// Build the advisory message + severity for a (caller, primitive, tier).
fn advisory(fn_name: &str, prim: &str, tier: Stability) -> CheckError {
    let msg = match tier {
        Stability::Experimental => format!(
            "stability warning: function `{fn_name}` calls experimental primitive `{prim}`; \
             its API may change between minor releases"
        ),
        Stability::Deprecated => format!(
            "stability warning: function `{fn_name}` calls deprecated primitive `{prim}`; \
             it is scheduled for removal — migrate to its replacement"
        ),
        Stability::Frozen => format!(
            "stability info: function `{fn_name}` calls frozen primitive `{prim}`; \
             it is supported but will not grow"
        ),
        // `stable` never produces an advisory; included for exhaustiveness.
        Stability::Stable => format!("stability: `{fn_name}` calls `{prim}`"),
    };
    CheckError::StabilityAdvice(msg)
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
    fn user_fn_shadowing_a_bare_prim_name_is_not_flagged() {
        // A user `map` shadows any bare `map` primitive.
        let m = parse(
            r#"
            def map(x) { x }
            @caps()
            def main() {
                map(1)
            }
            "#,
        );
        assert!(
            msgs(&check_stability(&m)).is_empty(),
            "user-defined `map` must shadow the primitive, got {:?}",
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
                gone()
                kept()
            }
            "#,
        );
        let map = map_of(&[("gone", Stability::Deprecated), ("kept", Stability::Frozen)]);
        let ms = msgs(&advise(&m, &map));
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
                exp()
                exp()
                exp()
            }
            "#,
        );
        let map = map_of(&[("exp", Stability::Experimental)]);
        assert_eq!(
            msgs(&advise(&m, &map)).len(),
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
        let ms = msgs(&advise(&m, &map));
        assert!(
            ms.iter().any(|m| m.contains("std::json::parse")),
            "qualified call should resolve to the qualified key, got {ms:?}"
        );
    }
}
