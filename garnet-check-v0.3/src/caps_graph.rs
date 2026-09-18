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
//! - `Expr::Method { receiver, method, .. }` — method dispatch. The pass has
//!   no receiver-type information, so it cannot perform precise type-directed
//!   dispatch. Instead it resolves `receiver.m(..)` to the UNION of declared
//!   caps of every impl method named `m`, across all types
//!   (via the `CalleeRef::MethodByName` callee ref). This is a sound
//!   OVER-approximation: it
//!   never under-attributes authority (so it cannot let a cap-requiring method
//!   call slip past coverage), but it may over-attribute when two types share
//!   a method name. Precise, type-directed method resolution is a future
//!   slice. (Previously this edge was fully deferred — method calls
//!   contributed ZERO caps, which under-attributed authority.)
//!
//! ## Impl-method identity
//!
//! Impl methods are keyed `Owner::name` (the impl block's target type's last
//! path segment, e.g. `A::go`), exactly mirroring the S114 capability surface
//! (the `capability_surface` module) so the graph and the surface AGREE on every
//! impl-method name. Free (non-impl) functions keep their bare name. Without
//! this, `impl A { def go() }` and `impl B { def go() }` collided under the
//! bare key `"go"` and the second overwrote the first — authority was
//! mis-attributed.
//!
//! ## Cycle handling
//!
//! Functions can self-recurse or participate in mutually-recursive SCCs. The
//! transitive caps of a function is the union of the direct caps of every
//! function reachable from it, so every member of a strongly connected
//! component has the same transitive set. The propagator computes exactly
//! that with the DeRemer–Pennello digraph algorithm: a Tarjan SCC traversal
//! that accumulates caps up the DFS tree and, when it closes a component,
//! assigns the accumulated union to every member at once. Each edge is
//! visited once; the per-node bookkeeping lives in `BTreeMap`s, so the pass
//! is O((V + E) log V) in the size of the call graph rather than strictly
//! linear. The verdict does not depend on which member of a cycle the caller
//! asks about first.
//!
//! The traversal is iterative — an explicit frame stack rather than
//! recursion — so a long chain of functions cannot overflow the thread stack
//! (U-117 also covered a deep-chain abort).
//!
//! Before U-117 (2026-09) this pass was a colored DFS that returned an empty
//! set for an edge back into a function still being computed and memoised
//! that partial answer. A primitive reached only through a cycle was then
//! attributed to some members and not others, depending on visit order.
//!
//! ## Wildcard semantics
//!
//! A function with `@caps(*)` passes coverage vacuously — the wildcard
//! asserts trust rather than enumerates. The existing `audit.rs` already
//! rejects wildcard use in safe-mode functions as a hard error; this pass
//! leaves safe-mode wildcard functions OUT of its coverage check (the
//! audit.rs error is enough), and permits managed-mode wildcard use to pass.

use crate::capset::CapSet;
use garnet_parser::ast::{
    ActorItem, Capability, Expr, FnDef, FnMode, Item, Module, Stmt, TypeExpr,
};
use std::collections::{BTreeMap, BTreeSet};

/// The graph key for a function. Free functions key on their bare name; impl
/// methods key on `Owner::name`. This MUST match the `Owner::method` naming
/// the S114 capability surface uses ([`crate::capability_surface`]) so the
/// graph and the surface agree on impl-method identity.
/// Collect the constructor row (`memory::<kind>`) of every `memory` declaration
/// under `items`, recursing into nested modules and into actor bodies. The walk
/// mirrors the interpreter's `require_module_memory_capabilities`.
fn collect_memory_declaration_rows(items: &[Item], rows: &mut Vec<String>) {
    for item in items {
        match item {
            Item::Memory(decl) => rows.push(format!("memory::{}", decl.kind.as_str())),
            Item::Actor(actor) => {
                for actor_item in &actor.items {
                    if let ActorItem::Memory(decl) = actor_item {
                        rows.push(format!("memory::{}", decl.kind.as_str()));
                    }
                }
            }
            Item::Module(m) => collect_memory_declaration_rows(&m.items, rows),
            _ => {}
        }
    }
}

fn fn_key(owner: Option<&str>, name: &str) -> String {
    match owner {
        Some(owner) => format!("{owner}::{name}"),
        None => name.to_string(),
    }
}

/// A short label for an impl block's owning type — the last path segment of
/// the target type. Mirrors `capability_surface::type_label` exactly so the
/// graph and the surface derive identical owner labels.
fn type_label(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named { path, .. } => path.last().cloned().unwrap_or_else(|| "impl".to_string()),
        _ => "impl".to_string(),
    }
}

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
    /// RB-1: a `Copy` [`CapSet`] bitset; canonical names are recoverable via
    /// [`CapSet::names`] in the same order the old `BTreeSet<String>` gave.
    pub transitive: BTreeMap<String, CapSet>,
}

/// Entry point: build the call graph from `module`, propagate caps, verify
/// every `@caps(...)` declaration covers the transitive requirements.
pub fn check_caps_coverage(module: &Module) -> CapsReport {
    let mut graph = CapsGraph::build(module);
    graph.verify()
}

// ── Internal state ─────────────────────────────────────────────────

struct CapsGraph {
    /// fn key → set of callee names (user fns + primitives by their
    /// registry key, bare or qualified). Keyed by [`fn_key`]: free fns by bare
    /// name, impl methods by `Owner::name`.
    callees: BTreeMap<String, BTreeSet<CalleeRef>>,
    /// fn key → declared `@caps(...)` set. Unknown (user-defined) names
    /// set [`CapSet::OTHER`] so annotation *presence* survives — see the
    /// claim boundary in `capset.rs`. Keyed by [`fn_key`].
    declared: BTreeMap<String, CapSet>,
    /// fn key → whether `@caps(*)` wildcard was used.
    wildcard: BTreeMap<String, bool>,
    /// fn key → mode (safe vs managed). Used to skip safe-mode wildcard
    /// functions (audit.rs already flags them).
    modes: BTreeMap<String, FnMode>,
    /// fn keys that carried ANY `@caps(...)` annotation — including the empty
    /// `@caps()` form. This distinguishes "declared `@caps()` explicitly"
    /// (a deliberate empty grant) from "no `@caps` annotation at all"; the
    /// `declared` bitset is empty in both cases, so presence cannot be read
    /// off it. Keyed by [`fn_key`].
    declares_caps: BTreeSet<String>,
    /// Bare method-name → set of impl-method keys (`Owner::name`) declaring
    /// that name, across every type. Built during decl collection and used to
    /// resolve [`CalleeRef::MethodByName`] to the union of all matching impl
    /// methods (the sound name-based over-approximation for method dispatch).
    method_index: BTreeMap<String, BTreeSet<String>>,
    /// Primitive name → required caps, from stdlib registry. Indexed by
    /// BOTH the qualified name ("fs::read_file") AND the bare last segment
    /// ("read_file"), so both call shapes resolve. Registry cap strings are
    /// all canonical (`capset.rs` registry-drift trap), so these bitsets
    /// never carry `OTHER`.
    prim_caps: BTreeMap<String, CapSet>,
    /// Finalized transitive caps per fn. A fn is inserted only when the SCC
    /// it belongs to has been closed, so an entry is never partial.
    memo: BTreeMap<String, CapSet>,
    /// Tarjan node stack: fns whose SCC is still open, in visit order. Empty
    /// between top-level [`Self::transitive_caps`] calls.
    stack: Vec<String>,
    /// fn key → its position in `stack`, for every fn currently on it.
    on_stack: BTreeMap<String, usize>,
}

/// One open frame of the iterative SCC traversal in
/// [`CapsGraph::transitive_caps`].
struct DfsFrame {
    /// Position of this frame's fn in [`CapsGraph::stack`] (its Tarjan
    /// index).
    pos: usize,
    /// Lowest stack position reachable from this fn so far (Tarjan
    /// low-link). `low == pos` on exit means this fn is the root of its SCC.
    low: usize,
    /// Caps gathered so far: direct primitive caps, finalized callee caps,
    /// and everything passed up from child frames in the same SCC.
    caps: CapSet,
    /// User-fn targets not yet visited (method calls already expanded to
    /// their impl-method keys). Consumed from the back.
    targets: Vec<String>,
}

/// A callee reference — either a primitive (stdlib registry entry) or a
/// user-defined function identified by name. Kept as a tagged string
/// internally so BTreeSet ordering is deterministic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum CalleeRef {
    /// A stdlib primitive. Holds the registry key (qualified name, e.g.
    /// "fs::read_file"). Caps are looked up from `prim_caps`.
    Primitive(String),
    /// A user-defined function in this module. Holds the graph key — a bare
    /// name for a free fn, or `Owner::name` for an impl method.
    UserFn(String),
    /// A method call `receiver.m(..)` resolved by NAME only (no receiver-type
    /// info). At propagation time this resolves to the UNION of declared caps
    /// of every impl method named `m`. Holds the bare method name `m`. This is
    /// a sound over-approximation: it never under-attributes authority, but
    /// may over-attribute when a method name collides across types — precise
    /// type-directed dispatch is a future slice.
    MethodByName(String),
}

impl CapsGraph {
    fn build(module: &Module) -> Self {
        // Build the primitive-caps lookup table. Index by BOTH the qualified
        // "module::name" (registry key) AND the bare "name" (matches the
        // bridge's unqualified prelude binding).
        let registry = garnet_stdlib::registry::all_prims();
        let mut prim_caps: BTreeMap<String, CapSet> = BTreeMap::new();
        for (qualified, meta) in registry {
            let caps = meta
                .required_caps
                .0
                .iter()
                .fold(CapSet::EMPTY, |acc, s| acc | CapSet::from_name_or_other(s));
            // Bare-name index: e.g., "fs::read_file" also indexed as "read_file".
            // `&str` `Split` isn't DoubleEndedIterator, so iterate by last-wins.
            // Indexed before `qualified` moves into the map below; bare names
            // never collide with qualified keys (those contain "::").
            if let Some(bare) = qualified.split("::").last() {
                // Multiple qualified prims could share a bare name (e.g.,
                // `array::contains` vs. `str::contains`). If a collision
                // occurs, union the caps — a conservative stance that never
                // under-requires capabilities at the source layer.
                prim_caps
                    .entry(bare.to_string())
                    .and_modify(|existing| *existing |= caps)
                    .or_insert(caps);
            }
            prim_caps.insert(qualified, caps);
        }

        let mut graph = CapsGraph {
            callees: BTreeMap::new(),
            declared: BTreeMap::new(),
            wildcard: BTreeMap::new(),
            modes: BTreeMap::new(),
            declares_caps: BTreeSet::new(),
            method_index: BTreeMap::new(),
            prim_caps,
            memo: BTreeMap::new(),
            stack: Vec::new(),
            on_stack: BTreeMap::new(),
        };

        // First pass: record every user fn and its declared caps (and build
        // the method-name index over impl methods), so the second pass can
        // resolve user-fn callees by key and method calls by name.
        for item in &module.items {
            graph.collect_fn_decls(item, /*module_safe=*/ module.safe);
        }

        // Second pass: walk each fn's body, record its callees.
        for item in &module.items {
            graph.collect_fn_callees(item);
        }

        // Third pass (D-04b, ADR 0011): every `memory <kind> <name> : <type>`
        // declaration — top level, inside a nested `module`, or inside an
        // `actor` — is the same store construction as the `memory::<kind>`
        // constructor row and is charged to the program entry as that row.
        // The runtime builds these stores at load time under the entry frame
        // (`garnet-interp-v0.3/src/lib.rs` `load_module`), so `main` is the
        // fn whose declared caps must cover them; a module without `main` is
        // a library and has no entry to charge. Before this pass the
        // declaration form was caps-invisible while the constructor rows were
        // gated (cross-family review of 4ce90eb6, blocker 1).
        if graph.declared.contains_key("main") {
            let mut rows = Vec::new();
            collect_memory_declaration_rows(&module.items, &mut rows);
            graph
                .callees
                .entry("main".to_string())
                .or_default()
                .extend(rows.into_iter().map(CalleeRef::Primitive));
        }

        graph
    }

    fn collect_fn_decls(&mut self, item: &Item, module_safe: bool) {
        match item {
            Item::Fn(f) => self.record_fn_decl(f, /*owner=*/ None, module_safe),
            Item::Module(m) => {
                let merged = module_safe || m.safe;
                for inner in &m.items {
                    self.collect_fn_decls(inner, merged);
                }
            }
            Item::Impl(impl_block) => {
                let owner = type_label(&impl_block.target);
                for method in &impl_block.methods {
                    self.record_fn_decl(method, Some(&owner), module_safe);
                }
            }
            _ => {}
        }
    }

    fn record_fn_decl(&mut self, f: &FnDef, owner: Option<&str>, module_safe: bool) {
        let key = fn_key(owner, &f.name);
        let mut caps = CapSet::EMPTY;
        let mut has_wildcard = false;
        let mut has_caps_annotation = false;
        for ann in &f.annotations {
            if let garnet_parser::ast::Annotation::Caps(items, _) = ann {
                // Presence of ANY `@caps(...)` — even empty `@caps()` — marks
                // the fn as explicitly annotated. Tracked separately because
                // the `declared` bitset is empty both for `@caps()` and for
                // no annotation at all.
                has_caps_annotation = true;
                for c in items {
                    match c {
                        Capability::Wildcard => has_wildcard = true,
                        _ => caps |= CapSet::from_name_or_other(c.as_str()),
                    }
                }
            }
        }
        self.declared.insert(key.clone(), caps);
        self.wildcard.insert(key.clone(), has_wildcard);
        if has_caps_annotation {
            self.declares_caps.insert(key.clone());
        }
        // Effective mode: safe if the module is safe OR the fn declares @safe.
        let effective_mode = if module_safe || f.mode == FnMode::Safe {
            FnMode::Safe
        } else {
            FnMode::Managed
        };
        self.modes.insert(key.clone(), effective_mode);
        // Index impl methods by their bare name so name-based method dispatch
        // can resolve `receiver.m()` to every impl method named `m`.
        if owner.is_some() {
            self.method_index
                .entry(f.name.clone())
                .or_default()
                .insert(key.clone());
        }
        // Initialize an empty callee-set so every declared fn appears in the
        // map even if its body is empty.
        self.callees.entry(key).or_default();
    }

    fn collect_fn_callees(&mut self, item: &Item) {
        match item {
            Item::Fn(f) => self.record_fn_callees(f, /*owner=*/ None),
            Item::Module(m) => {
                for inner in &m.items {
                    self.collect_fn_callees(inner);
                }
            }
            Item::Impl(impl_block) => {
                let owner = type_label(&impl_block.target);
                for method in &impl_block.methods {
                    self.record_fn_callees(method, Some(&owner));
                }
            }
            _ => {}
        }
    }

    fn record_fn_callees(&mut self, f: &FnDef, owner: Option<&str>) {
        let key = fn_key(owner, &f.name);
        let mut callees: BTreeSet<CalleeRef> = BTreeSet::new();
        for s in &f.body.stmts {
            self.walk_stmt_for_callees(s, &mut callees);
        }
        if let Some(tail) = &f.body.tail_expr {
            self.walk_expr_for_callees(tail, &mut callees);
        }
        self.callees.insert(key, callees);
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
            Stmt::Return { value: Some(e), .. }
            | Stmt::Yield { value: Some(e), .. }
            | Stmt::Next { value: Some(e), .. }
            | Stmt::Raise { value: e, .. } => {
                self.walk_expr_for_callees(e, out);
            }
            Stmt::Return { value: None, .. }
            | Stmt::Yield { value: None, .. }
            | Stmt::Next { value: None, .. } => {}
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
            Expr::Method {
                receiver,
                method,
                args,
                ..
            } => {
                // No receiver-type info here, so we cannot do precise type-
                // directed dispatch. Emit a name-based reference; the
                // propagator resolves it to the UNION of declared caps of all
                // impl methods named `method` (sound over-approximation — see
                // the module docs and `CalleeRef::MethodByName`). Keep
                // recursing into receiver + args so nested free-function calls
                // are still captured.
                out.insert(CalleeRef::MethodByName(method.clone()));
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
            Expr::Cast { expr, .. } => self.walk_expr_for_callees(expr, out),
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
                    self.walk_block(&arm.body, out);
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

    /// Compute the transitive caps set for `fn_name`: the union of the direct
    /// caps of every fn reachable from it.
    ///
    /// Iterative DeRemer–Pennello digraph traversal (see the module doc,
    /// "Cycle handling"). Every fn visited on the way — the whole SCC forest
    /// below `fn_name` — is finalized into `memo`, so later queries are hits.
    /// A fn whose SCC is still open is never read as "empty"; the edge into
    /// it only lowers the current frame's low-link, and the shared union is
    /// written to every member when the SCC root closes.
    fn transitive_caps(&mut self, fn_name: &str) -> CapSet {
        if let Some(&cached) = self.memo.get(fn_name) {
            return cached;
        }
        debug_assert!(
            self.stack.is_empty() && self.on_stack.is_empty(),
            "transitive_caps is not re-entrant; the SCC stack must be empty between calls"
        );

        let mut frames: Vec<DfsFrame> = vec![self.open_frame(fn_name)];
        while let Some(frame) = frames.last_mut() {
            if let Some(target) = frame.targets.pop() {
                if let Some(&done) = self.memo.get(&target) {
                    frame.caps |= done;
                } else if let Some(&pos) = self.on_stack.get(&target) {
                    // Back or cross edge into an open SCC: `target` and this
                    // fn are mutually reachable, so they share one answer.
                    frame.low = frame.low.min(pos);
                } else {
                    let child = self.open_frame(&target);
                    frames.push(child);
                }
                continue;
            }

            // Every target of this frame has been visited.
            let frame = match frames.pop() {
                Some(frame) => frame,
                None => break,
            };
            if frame.low == frame.pos {
                // SCC root: everything on the stack from `pos` up is exactly
                // this component. All members get the same union.
                for member in self.stack.drain(frame.pos..) {
                    self.on_stack.remove(&member);
                    self.memo.insert(member, frame.caps);
                }
                if let Some(parent) = frames.last_mut() {
                    parent.caps |= frame.caps;
                }
            } else if let Some(parent) = frames.last_mut() {
                // Not a root: this fn stays on the stack until its SCC root
                // closes. Its caps and low-link flow to the DFS parent, which
                // is in the same SCC.
                parent.caps |= frame.caps;
                parent.low = parent.low.min(frame.low);
            }
        }

        debug_assert!(self.stack.is_empty() && self.on_stack.is_empty());
        self.memo.get(fn_name).copied().unwrap_or(CapSet::EMPTY)
    }

    /// Push `fn_name` onto the Tarjan stack and build its traversal frame:
    /// direct primitive caps folded in, user-fn and method targets queued.
    fn open_frame(&mut self, fn_name: &str) -> DfsFrame {
        let pos = self.stack.len();
        self.stack.push(fn_name.to_string());
        self.on_stack.insert(fn_name.to_string(), pos);

        let mut caps = CapSet::EMPTY;
        let mut targets = Vec::new();
        for callee in self.callees.get(fn_name).into_iter().flatten() {
            match callee {
                CalleeRef::Primitive(key) => {
                    if let Some(&pc) = self.prim_caps.get(key) {
                        caps |= pc;
                    }
                }
                CalleeRef::UserFn(name) => targets.push(name.clone()),
                CalleeRef::MethodByName(method) => {
                    // Sound over-approximation: union the transitive caps of
                    // EVERY impl method named `method`. No receiver-type info
                    // is available, so we cannot pick the one true target;
                    // unioning never under-attributes authority (precise
                    // type-directed dispatch is a future slice).
                    if let Some(impls) = self.method_index.get(method) {
                        targets.extend(impls.iter().cloned());
                    }
                }
            }
        }
        DfsFrame {
            pos,
            low: pos,
            caps,
            targets,
        }
    }

    /// Verify every fn's declared caps covers its transitive requirement.
    /// Emits one `CapsViolation` per (fn, missing-cap) pair.
    fn verify(&mut self) -> CapsReport {
        let mut report = CapsReport::default();
        // Snapshot fn-names to iterate while we mutate memo internally.
        let fn_names: Vec<String> = self.declared.keys().cloned().collect();
        for fn_name in fn_names {
            let required = self.transitive_caps(&fn_name);
            report.transitive.insert(fn_name.clone(), required);

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
            let declared = self.declared.get(&fn_name).copied().unwrap_or_default();
            for missing in required.difference(declared).iter_names() {
                // Find one representative callee requiring this cap for the
                // diagnostic "via" field. Deterministic: first callee in
                // BTreeSet order whose transitive caps contain `missing`.
                let via = self.find_cap_source(&fn_name, missing);
                report.violations.push(CapsViolation {
                    fn_name: fn_name.clone(),
                    missing: missing.to_string(),
                    via,
                });
            }
        }
        report
    }

    /// Whether the fn has ANY `@caps(...)` annotation at all — including an
    /// empty one like `@caps()`. Used to distinguish "declared `@caps()`
    /// explicitly" (a deliberate empty grant) from "didn't annotate at all".
    /// `fn_name` is the graph key (bare name, or `Owner::name` for impl
    /// methods).
    fn has_caps_annotation(&self, fn_name: &str) -> bool {
        // `declares_caps` records the presence of the annotation directly
        // (set during decl collection), so an explicit empty `@caps()` is now
        // correctly recognized as "annotated" rather than treated as unknown.
        if self.declares_caps.contains(fn_name) {
            return true;
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
                    if let Some(&pc) = self.prim_caps.get(key) {
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
                CalleeRef::MethodByName(ref method) => {
                    // Report the first impl method named `method` (in key
                    // order) whose transitive caps include `missing`.
                    let targets = self.method_index.get(method).cloned().unwrap_or_default();
                    for target in targets {
                        if self.transitive_caps(&target).contains(missing) {
                            return format!("(via .{method}() → {target})");
                        }
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

    /// D-04b — a `memory` declaration (top-level, in a module, or inside an
    /// actor) is charged to `main` as the tier's constructor row, so the same
    /// `mem` coverage rule the `memory::*` calls obey applies to declarations.
    #[test]
    fn memory_declarations_charge_main_with_mem() {
        for (src, tier) in [
            (
                "memory working scratch : String\n@caps()\ndef main() { 1 }\n",
                "memory::working",
            ),
            (
                "module Store {\n  memory semantic facts : VectorIndex<String>\n}\n@caps()\ndef main() { 1 }\n",
                "memory::semantic",
            ),
            (
                "actor Recorder {\n  memory episodic log : EpisodeStore<String>\n  protocol note(x: String) -> Int\n  on note(x) { 1 }\n}\n@caps()\ndef main() { 1 }\n",
                "memory::episodic",
            ),
        ] {
            let r = check_caps_coverage(&parse(src));
            assert!(
                r.violations
                    .iter()
                    .any(|v| v.fn_name == "main" && v.missing == "mem" && v.via == tier),
                "expected `main` charged with `mem` via `{tier}` for:\n{src}\ngot {:?}",
                r.violations
            );
        }
        let r = check_caps_coverage(&parse(
            "memory working scratch : String\n@caps(mem)\ndef main() { 1 }\n",
        ));
        assert!(
            r.violations.is_empty(),
            "a declared `mem` covers the declaration, got {:?}",
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

#[cfg(test)]
#[path = "caps_graph_cycle_tests.rs"]
mod cycle_tests;
