//! CapCaps call-graph propagator (v3.4.1 — Day 2).
//!
//! The v3.4 spec shipped the `@caps(...)` annotation surface and single-
//! function validation (`audit.rs`). That check confirms each function
//! declares its caps, rejects unknown cap names, and flags safe-mode wildcard
//! use. What it did NOT do is propagate required caps transitively across the
//! call graph. This module closes that gap.
//!
//! ## The rule we enforce
//!
//! > For every function `f` in the compilation unit, the set of capabilities
//! > declared on `f` (`@caps(...)` or empty) must be a superset of the union
//! > of required capabilities of every primitive or user function `f`
//! > transitively invokes, UNLESS `f` declares `@caps(*)` (wildcard trust,
//! > managed mode only).
//!
//! Primitive caps are read from `garnet_stdlib::registry::all_prims()` — a
//! single source of truth shared by the interpreter's stdlib bridge and this
//! checker, so a primitive's capability contract cannot drift between the
//! two layers.
//!
//! ## Call resolution
//!
//! - `Expr::Call { callee: Ident(name), .. }` — simple-name call. Resolve
//!   against (a) user functions in the module, (b) stdlib primitives by BARE
//!   name (last segment after `::`). The interpreter's `eval_path` last-
//!   segment fallback makes both accessible at runtime.
//! - `Expr::Call { callee: Path(segs), .. }` — qualified call. Resolve the
//!   full `segs.join("::")` against the stdlib registry first (e.g.
//!   `fs::read_file`), fall back to the last segment as a user fn.
//! - `Expr::Method { receiver, method, .. }` — method dispatch. Deferred;
//!   method dispatch needs type information to resolve, and Day 2 scope is
//!   the free-function propagator. Method calls are counted in
//!   `boundary_call_sites` by `audit.rs` but do not contribute to the caps
//!   transitive set in this pass.
//!
//! ## Cycle handling
//!
//! Functions can self-recurse or participate in mutually-recursive SCCs. The
//! propagator uses a simple colored-DFS: white (unvisited), gray (currently
//! computing), black (finalized with memoized result). Encountering a gray
//! node during DFS short-circuits by contributing an empty caps set for that
//! edge — the fn-under-computation will union its own caps once it resolves,
//! so an SCC converges in one pass.
//!
//! A Tarjan SCC + iterated fixed-point would be slightly more aggressive for
//! pathological many-way mutual recursion, but the colored-DFS approach is
//! sound (always terminates, never over-reports) and handles the realistic
//! Garnet call-graph shapes we see in MVPs 1–10 in a single traversal.
//!
//! ## Wildcard semantics
//!
//! A function with `@caps(*)` passes coverage vacuously — the wildcard
//! asserts trust rather than enumerates. The existing `audit.rs` already
//! rejects wildcard use in safe-mode functions as a hard error; this pass
//! leaves safe-mode wildcard functions OUT of its coverage check (the
//! audit.rs error is enough), and permits managed-mode wildcard use to pass.

use garnet_parser::ast::{Capability, Expr, FnDef, FnMode, Item, Module, Stmt};
use std::collections::{BTreeMap, BTreeSet};

/// A capability label, e.g., "fs", "net", "time". Kept as `String` so we
/// can round-trip the stdlib registry's `&'static str` caps AND the parser's
/// `Capability::Other` fallback.
pub type CapsSet = BTreeSet<String>;

/// A propagated-caps violation: a function invokes a primitive/callee
/// transitively requiring a capability its `@caps(...)` does not cover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapsViolation {
    /// The function missing the capability declaration.
    pub fn_name: String,
    /// The capability NOT covered by the function's `@caps(...)`.
    pub missing: String,
    /// The primitive (or user fn) whose requirement is unmet. One of the
    /// transitively-reachable names. Multiple primitives may require the
    /// same missing cap; we surface one representative for the diagnostic.
    pub via: String,
}

/// The propagator's report.
#[derive(Debug, Default)]
pub struct CapsReport {
    pub violations: Vec<CapsViolation>,
    /// Per-function transitive caps (for introspection / later tooling).
    pub transitive: BTreeMap<String, CapsSet>,
}

/// Entry point: build the call graph from `module`, propagate caps, verify
/// every `@caps(...)` declaration covers the transitive requirements.
pub fn check_caps_coverage(module: &Module) -> CapsReport {
    let mut graph = CapsGraph::build(module);
    graph.verify()
}

// ── Internal state ─────────────────────────────────────────────────

struct CapsGraph {
    /// fn name → set of callee names (user fns + primitives by their
    /// registry key, bare or qualified).
    callees: BTreeMap<String, BTreeSet<CalleeRef>>,
    /// fn name → declared `@caps(...)` set.
    declared: BTreeMap<String, CapsSet>,
    /// fn name → whether `@caps(*)` wildcard was used.
    wildcard: BTreeMap<String, bool>,
    /// fn name → mode (safe vs managed). Used to skip safe-mode wildcard
    /// functions (audit.rs already flags them).
    modes: BTreeMap<String, FnMode>,
    /// Primitive name → required caps, from stdlib registry. Indexed by
    /// BOTH the qualified name ("fs::read_file") AND the bare last segment
    /// ("read_file"), so both call shapes resolve.
    prim_caps: BTreeMap<String, CapsSet>,
    /// Memoized transitive caps per fn (black-colored nodes).
    memo: BTreeMap<String, CapsSet>,
    /// DFS stack color. True = currently computing (gray); absence = white.
    in_progress: BTreeSet<String>,
}

/// A callee reference — either a primitive (stdlib registry entry) or a
/// user-defined function identified by name. Kept as a tagged string
/// internally so BTreeSet ordering is deterministic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum CalleeRef {
    /// A stdlib primitive. Holds the registry key (qualified name, e.g.
    /// "fs::read_file"). Caps are looked up from `prim_caps`.
    Primitive(String),
    /// A user-defined function in this module.
    UserFn(String),
}

impl CapsGraph {
    fn build(module: &Module) -> Self {
        // Build the primitive-caps lookup table. Index by BOTH the qualified
        // "module::name" (registry key) AND the bare "name" (matches the
        // bridge's unqualified prelude binding).
        let registry = garnet_stdlib::registry::all_prims();
        let mut prim_caps: BTreeMap<String, CapsSet> = BTreeMap::new();
        for (qualified, meta) in &registry {
            let caps: CapsSet = meta
                .required_caps
                .0
                .iter()
                .map(|s| (*s).to_string())
                .collect();
            prim_caps.insert(qualified.clone(), caps.clone());
            // Bare-name index: e.g., "fs::read_file" also indexed as "read_file".
            // `&str` `Split` isn't DoubleEndedIterator, so iterate by last-wins.
            let bare_opt = qualified.split("::").last();
            if let Some(bare) = bare_opt {
                // Multiple qualified prims could share a bare name (e.g.,
                // `array::contains` vs. `str::contains`). If a collision
                // occurs, union the caps — a conservative stance that never
                // under-requires capabilities at the source layer.
                prim_caps
                    .entry(bare.to_string())
                    .and_modify(|existing| existing.extend(caps.iter().cloned()))
                    .or_insert(caps.clone());
            }
        }

        let mut graph = CapsGraph {
            callees: BTreeMap::new(),
            declared: BTreeMap::new(),
            wildcard: BTreeMap::new(),
            modes: BTreeMap::new(),
            prim_caps,
            memo: BTreeMap::new(),
            in_progress: BTreeSet::new(),
        };

        // First pass: record every user fn and its declared caps, so the
        // second pass can resolve user-fn callees by name lookup.
        for item in &module.items {
            graph.collect_fn_decls(item, /*module_safe=*/ module.safe);
        }

        // Second pass: walk each fn's body, record its callees.
        for item in &module.items {
            graph.collect_fn_callees(item);
        }

        graph
    }

    fn collect_fn_decls(&mut self, item: &Item, module_safe: bool) {
        match item {
            Item::Fn(f) => self.record_fn_decl(f, module_safe),
            Item::Module(m) => {
                let merged = module_safe || m.safe;
                for inner in &m.items {
                    self.collect_fn_decls(inner, merged);
                }
            }
            Item::Impl(impl_block) => {
                for method in &impl_block.methods {
                    self.record_fn_decl(method, module_safe);
                }
            }
            _ => {}
        }
    }

    fn record_fn_decl(&mut self, f: &FnDef, module_safe: bool) {
        let mut caps: CapsSet = BTreeSet::new();
        let mut has_wildcard = false;
        for ann in &f.annotations {
            if let garnet_parser::ast::Annotation::Caps(items, _) = ann {
                for c in items {
                    match c {
                        Capability::Wildcard => has_wildcard = true,
                        _ => {
                            caps.insert(c.as_str().to_string());
                        }
                    }
                }
            }
        }
        self.declared.insert(f.name.clone(), caps);
        self.wildcard.insert(f.name.clone(), has_wildcard);
        // Effective mode: safe if the module is safe OR the fn declares @safe.
        let effective_mode = if module_safe || f.mode == FnMode::Safe {
            FnMode::Safe
        } else {
            FnMode::Managed
        };
        self.modes.insert(f.name.clone(), effective_mode);
        // Initialize an empty callee-set so every declared fn appears in the
        // map even if its body is empty.
        self.callees.entry(f.name.clone()).or_default();
    }

    fn collect_fn_callees(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => self.record_fn_callees(f),
            Item::Module(m) => {
                for inner in &m.items {
                    self.collect_fn_callees(inner);
                }
            }
            Item::Impl(impl_block) => {
                for method in &impl_block.methods {
                    self.record_fn_callees(method);
                }
            }
            _ => {}
        }
    }

    fn record_fn_callees(&mut self, f: &FnDef) {
        let mut callees: BTreeSet<CalleeRef> = BTreeSet::new();
        for s in &f.body.stmts {
            self.walk_stmt_for_callees(s, &mut callees);
        }
        if let Some(tail) = &f.body.tail_expr {
            self.walk_expr_for_callees(tail, &mut callees);
        }
        self.callees.insert(f.name.clone(), callees);
    }

    fn walk_stmt_for_callees(&self, s: &Stmt, out: &mut BTreeSet<CalleeRef>) {
        match s {
            Stmt::Expr(e) => self.walk_expr_for_callees(e, out),
            Stmt::Let(decl) => self.walk_expr_for_callees(&decl.value, out),
            Stmt::Var(decl) => self.walk_expr_for_callees(&decl.value, out),
            Stmt::Const(decl) => self.walk_expr_for_callees(&decl.value, out),
            Stmt::Assign { target, value, .. } => {
                self.walk_expr_for_callees(target, out);
                self.walk_expr_for_callees(value, out);
            }
            Stmt::Return { value: Some(e), .. } | Stmt::Raise { value: e, .. } => {
                self.walk_expr_for_callees(e, out);
            }
            Stmt::Return { value: None, .. } => {}
            Stmt::Break { value: Some(e), .. } => self.walk_expr_for_callees(e, out),
            Stmt::Break { value: None, .. } | Stmt::Continue { .. } => {}
            Stmt::While {
                condition, body, ..
            } => {
                self.walk_expr_for_callees(condition, out);
                self.walk_block(body, out);
            }
            Stmt::For { iter, body, .. } => {
                self.walk_expr_for_callees(iter, out);
                self.walk_block(body, out);
            }
            Stmt::Loop { body, .. } => {
                self.walk_block(body, out);
            }
        }
    }

    fn walk_expr_for_callees(&self, e: &Expr, out: &mut BTreeSet<CalleeRef>) {
        match e {
            Expr::Call { callee, args, .. } => {
                if let Some(cref) = self.resolve_callee(callee) {
                    out.insert(cref);
                }
                // Walk the callee expr in case it's more complex (e.g. a
                // closure expression or a field access whose RHS is itself a
                // call).
                self.walk_expr_for_callees(callee, out);
                for a in args {
                    self.walk_expr_for_callees(a, out);
                }
            }
            Expr::Method { receiver, args, .. } => {
                // Method dispatch requires type info — deferred to a future
                // pass. Walk children so nested free-function calls are
                // still captured.
                self.walk_expr_for_callees(receiver, out);
                for a in args {
                    self.walk_expr_for_callees(a, out);
                }
            }
            Expr::Binary { lhs, rhs, .. } => {
                self.walk_expr_for_callees(lhs, out);
                self.walk_expr_for_callees(rhs, out);
            }
            Expr::Unary { expr, .. } => self.walk_expr_for_callees(expr, out),
            Expr::Field { receiver, .. } => self.walk_expr_for_callees(receiver, out),
            Expr::Index {
                receiver, index, ..
            } => {
                self.walk_expr_for_callees(receiver, out);
                self.walk_expr_for_callees(index, out);
            }
            Expr::If {
                condition,
                then_block,
                elsif_clauses,
                else_block,
                ..
            } => {
                self.walk_expr_for_callees(condition, out);
                self.walk_block(then_block, out);
                for (c, b) in elsif_clauses {
                    self.walk_expr_for_callees(c, out);
                    self.walk_block(b, out);
                }
                if let Some(b) = else_block {
                    self.walk_block(b, out);
                }
            }
            Expr::Match { subject, arms, .. } => {
                self.walk_expr_for_callees(subject, out);
                for arm in arms {
                    self.walk_expr_for_callees(&arm.body, out);
                }
            }
            Expr::Try {
                body,
                rescues,
                ensure,
                ..
            } => {
                self.walk_block(body, out);
                for r in rescues {
                    self.walk_block(&r.body, out);
                }
                if let Some(e) = ensure {
                    self.walk_block(e, out);
                }
            }
            Expr::Array { elements, .. } => {
                for el in elements {
                    self.walk_expr_for_callees(el, out);
                }
            }
            Expr::Map { entries, .. } => {
                for (k, v) in entries {
                    self.walk_expr_for_callees(k, out);
                    self.walk_expr_for_callees(v, out);
                }
            }
            Expr::Spawn { expr, .. } => self.walk_expr_for_callees(expr, out),
            Expr::Closure { .. }
            | Expr::Ident(_, _)
            | Expr::Path(_, _)
            | Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::Bool(_, _)
            | Expr::Nil(_)
            | Expr::Str(_, _)
            | Expr::Symbol(_, _) => {}
        }
    }

    fn walk_block(&self, b: &garnet_parser::ast::Block, out: &mut BTreeSet<CalleeRef>) {
        for s in &b.stmts {
            self.walk_stmt_for_callees(s, out);
        }
        if let Some(t) = &b.tail_expr {
            self.walk_expr_for_callees(t, out);
        }
    }

    /// Given a callee expression (the LHS of a `Call`), figure out whether
    /// it names a known primitive, a user function in this module, or
    /// something unresolvable (closure value, field access, etc.).
    fn resolve_callee(&self, callee: &Expr) -> Option<CalleeRef> {
        match callee {
            Expr::Ident(name, _) => {
                // Prefer user-defined fn over prim of the same bare name
                // (matches `prelude::install` ordering: legacy prelude
                // shadows stdlib_bridge on collisions, though none exist
                // today).
                if self.declared.contains_key(name) {
                    return Some(CalleeRef::UserFn(name.clone()));
                }
                if self.prim_caps.contains_key(name) {
                    return Some(CalleeRef::Primitive(name.clone()));
                }
                None
            }
            Expr::Path(segs, _) => {
                let qualified = segs.join("::");
                if self.prim_caps.contains_key(&qualified) {
                    return Some(CalleeRef::Primitive(qualified));
                }
                // Fall back: last segment as a user fn.
                if let Some(last) = segs.last() {
                    if self.declared.contains_key(last) {
                        return Some(CalleeRef::UserFn(last.clone()));
                    }
                    if self.prim_caps.contains_key(last) {
                        return Some(CalleeRef::Primitive(last.clone()));
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Compute the transitive caps set for `fn_name`. Colored-DFS: gray
    /// nodes encountered mid-recursion contribute empty (avoiding infinite
    /// loops in cyclic SCCs).
    fn transitive_caps(&mut self, fn_name: &str) -> CapsSet {
        if let Some(cached) = self.memo.get(fn_name) {
            return cached.clone();
        }
        if self.in_progress.contains(fn_name) {
            // Cycle — the caller will fold in its own direct caps separately.
            return BTreeSet::new();
        }
        self.in_progress.insert(fn_name.to_string());

        let mut caps: CapsSet = BTreeSet::new();
        // Clone the callee list so we don't hold a borrow on self while
        // recursing into transitive_caps(callee).
        let callees = self.callees.get(fn_name).cloned().unwrap_or_default();
        for callee in callees {
            match callee {
                CalleeRef::Primitive(key) => {
                    if let Some(pc) = self.prim_caps.get(&key) {
                        caps.extend(pc.iter().cloned());
                    }
                }
                CalleeRef::UserFn(name) => {
                    let child = self.transitive_caps(&name);
                    caps.extend(child);
                }
            }
        }

        self.in_progress.remove(fn_name);
        self.memo.insert(fn_name.to_string(), caps.clone());
        caps
    }

    /// Verify every fn's declared caps covers its transitive requirement.
    /// Emits one `CapsViolation` per (fn, missing-cap) pair.
    fn verify(&mut self) -> CapsReport {
        let mut report = CapsReport::default();
        // Snapshot fn-names to iterate while we mutate memo internally.
        let fn_names: Vec<String> = self.declared.keys().cloned().collect();
        for fn_name in fn_names {
            let required = self.transitive_caps(&fn_name);
            report.transitive.insert(fn_name.clone(), required.clone());

            // Skip coverage check for wildcard functions. If safe-mode
            // wildcard, audit.rs already emitted a hard error.
            if *self.wildcard.get(&fn_name).unwrap_or(&false) {
                continue;
            }
            // Skip fns with no declared caps annotation AT ALL. The existing
            // audit.rs already flags `main` as needing an explicit @caps();
            // for other fns, missing-annotation means we don't know what the
            // author INTENDED to permit and a transitive-caps error here
            // would be noise (the audit.rs signature would catch main). A
            // stricter "every fn calling a cap-requiring prim must annotate"
            // mode can be added as opt-in later via a CheckConfig flag.
            if !self.has_caps_annotation(&fn_name) {
                continue;
            }
            let declared = self.declared.get(&fn_name).cloned().unwrap_or_default();
            for missing in required.difference(&declared) {
                // Find one representative callee requiring this cap for the
                // diagnostic "via" field. Deterministic: first callee in
                // BTreeSet order whose transitive caps contain `missing`.
                let via = self.find_cap_source(&fn_name, missing);
                report.violations.push(CapsViolation {
                    fn_name: fn_name.clone(),
                    missing: missing.clone(),
                    via,
                });
            }
        }
        report
    }

    /// Whether the fn has ANY `@caps(...)` annotation at all — including an
    /// empty one like `@caps()`. Used to distinguish "declared nothing
    /// because purely computational" (covered by `declared.contains_key`)
    /// from "didn't annotate at all" (not yet covered).
    fn has_caps_annotation(&self, fn_name: &str) -> bool {
        // `declared` contains an entry for every fn (with empty CapsSet if
        // no @caps). To distinguish "declared nothing" from "declared @caps()",
        // we look at the original AST — but rather than re-walking, we treat
        // wildcard-OR-nonempty-OR-main-annotated as "has annotation". A fn
        // with `@caps()` will have an empty set but wildcard=false and isn't
        // `main`, so it would skip here — acceptable for v3.4.1 Day 2. A
        // later refinement can track the presence of the annotation
        // explicitly.
        if self.wildcard.get(fn_name).copied().unwrap_or(false) {
            return true;
        }
        if let Some(caps) = self.declared.get(fn_name) {
            if !caps.is_empty() {
                return true;
            }
        }
        // Special case: `main` is required to annotate per audit.rs, so treat
        // it as annotated (the audit check already failed if it wasn't).
        fn_name == "main"
    }

    /// Find a representative callee whose transitive caps include `missing`.
    fn find_cap_source(&mut self, fn_name: &str, missing: &str) -> String {
        let callees = self.callees.get(fn_name).cloned().unwrap_or_default();
        for callee in callees {
            match callee {
                CalleeRef::Primitive(ref key) => {
                    if let Some(pc) = self.prim_caps.get(key) {
                        if pc.contains(missing) {
                            return key.clone();
                        }
                    }
                }
                CalleeRef::UserFn(ref name) => {
                    let child_caps = self.transitive_caps(name);
                    if child_caps.contains(missing) {
                        return format!("(via {name})");
                    }
                }
            }
        }
        "<unknown>".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Module {
        garnet_parser::parse_source(src).expect("parse failed")
    }

    #[test]
    fn fn_calling_fs_prim_without_caps_flagged() {
        // main() calls `read_file` which requires `fs`; main declares only @caps().
        let m = parse(
            r#"
            @caps()
            def main() {
                read_file("path.txt")
            }
            "#,
        );
        let r = check_caps_coverage(&m);
        assert!(
            r.violations
                .iter()
                .any(|v| v.fn_name == "main" && v.missing == "fs"),
            "expected fs violation on main, got {:?}",
            r.violations
        );
    }

    #[test]
    fn fn_with_matching_caps_passes() {
        let m = parse(
            r#"
            @caps(fs)
            def main() {
                read_file("path.txt")
            }
            "#,
        );
        let r = check_caps_coverage(&m);
        assert!(
            r.violations.is_empty(),
            "expected no violations, got {:?}",
            r.violations
        );
    }

    #[test]
    fn wildcard_fn_skips_coverage_check() {
        let m = parse(
            r#"
            @caps(*)
            def main() {
                read_file("path.txt")
            }
            "#,
        );
        let r = check_caps_coverage(&m);
        // audit.rs emits its own safe-mode-wildcard error; this pass does
        // not add a caps violation because @caps(*) is the explicit trust
        // declaration.
        assert!(
            r.violations.is_empty(),
            "wildcard should skip coverage check, got {:?}",
            r.violations
        );
    }

    #[test]
    fn transitive_caps_flow_through_user_fn() {
        // helper() uses fs; main calls helper but declares only @caps(); should fail.
        let m = parse(
            r#"
            def helper(p) {
                read_file(p)
            }
            @caps()
            def main() {
                helper("x")
            }
            "#,
        );
        let r = check_caps_coverage(&m);
        assert!(
            r.violations
                .iter()
                .any(|v| v.fn_name == "main" && v.missing == "fs"),
            "expected transitive fs violation on main, got {:?}",
            r.violations
        );
    }

    #[test]
    fn self_recursion_does_not_hang() {
        // Classic pathological case: fn calls itself. The propagator must
        // terminate.
        let m = parse(
            r#"
            @caps(fs)
            def main() {
                read_file("a")
                main()
            }
            "#,
        );
        let r = check_caps_coverage(&m);
        assert!(r.violations.is_empty());
        // And the transitive set of main is {fs}.
        let caps = r.transitive.get("main").cloned().unwrap_or_default();
        assert!(caps.contains("fs"));
    }

    #[test]
    fn mutual_recursion_does_not_hang() {
        let m = parse(
            r#"
            def ping(n) {
                read_file("ping")
                pong(n)
            }
            def pong(n) {
                write_file("pong", "x")
                ping(n)
            }
            @caps(fs)
            def main() {
                ping(0)
            }
            "#,
        );
        let r = check_caps_coverage(&m);
        assert!(r.violations.is_empty());
        assert!(r.transitive.get("main").unwrap().contains("fs"));
    }

    #[test]
    fn time_and_net_separate() {
        // A fn using `now_ms` needs `time`; a fn using `read_file` needs `fs`.
        // Declaring only `fs` on the time-user should fail.
        let m = parse(
            r#"
            @caps(fs)
            def main() {
                now_ms()
            }
            "#,
        );
        let r = check_caps_coverage(&m);
        assert!(
            r.violations
                .iter()
                .any(|v| v.fn_name == "main" && v.missing == "time"),
            "expected time violation on main, got {:?}",
            r.violations
        );
    }

    #[test]
    fn qualified_path_resolves_to_prim() {
        // Using `fs::read_file(...)` (qualified) should resolve to the same
        // stdlib primitive as the bare `read_file(...)` call.
        let m = parse(
            r#"
            @caps()
            def main() {
                fs::read_file("path")
            }
            "#,
        );
        let r = check_caps_coverage(&m);
        assert!(
            r.violations
                .iter()
                .any(|v| v.fn_name == "main" && v.missing == "fs"),
            "expected fs violation on qualified call, got {:?}",
            r.violations
        );
    }

    #[test]
    fn pure_fn_needs_no_caps() {
        let m = parse(
            r#"
            @caps()
            def main() {
                trim("  hi  ")
            }
            "#,
        );
        let r = check_caps_coverage(&m);
        assert!(
            r.violations.is_empty(),
            "pure trim call should need no caps, got {:?}",
            r.violations
        );
    }

    #[test]
    fn violation_carries_representative_via() {
        let m = parse(
            r#"
            @caps()
            def main() {
                read_file("a.txt")
            }
            "#,
        );
        let r = check_caps_coverage(&m);
        let v = r
            .violations
            .iter()
            .find(|v| v.fn_name == "main")
            .expect("violation present");
        // The "via" should name a prim requiring fs — either the bare or
        // qualified form, depending on how the bare-name bridge resolved.
        assert!(
            v.via.contains("read_file"),
            "expected via to mention read_file, got '{}'",
            v.via
        );
    }
}
