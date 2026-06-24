//! RB-4 — VM ⇄ interpreter parity for block-local / loop-local variable
//! shadowing.
//!
//! ## The bug this guards
//!
//! `garnet-vm`'s compiler used ONE bytecode slot per variable NAME (a flat
//! `local_slots` map with no lexical scope). A nested block that re-bound a
//! name already bound in an enclosing block overwrote the outer slot and could
//! not restore it on block exit. So for
//!
//! ```text
//! @caps()
//! def main() -> int {
//!   let x = 1
//!   if true { let x = 2  x }
//!   x
//! }
//! ```
//!
//! `--interp` returns `1` (the inner `let x = 2` is block-local; the outer `x`
//! is still `1`) while `--vm` returned `2` (the inner binding leaked). The fix
//! detects enclosing-scope shadowing at compile time and forces such functions
//! onto the existing tree-walk fallback, which is the reference. This test file
//! is the acceptance gate:
//!
//! * `probe_program_matches_interpreter` — the exact divergent program above
//!   now agrees (both `1`), and lowers to FALLBACK (not native).
//! * `non_shadowing_program_stays_native_and_matches` — a plain program still
//!   compiles to VM native bytecode (the fix does not over-fallback the common
//!   case) and agrees with the interpreter.
//! * `prop_vm_matches_interp_on_random_shadowing_programs` — a bounded random
//!   generator emits valid nested-block `let`-rebinding programs (shadowing and
//!   non-shadowing) and asserts the VM output EQUALS the interpreter output for
//!   every one. If the detector ever misses a shadow shape, this goes RED.
//!
//! Both lanes are driven in-process (VM via `run_function_with_options`,
//! interpreter via `garnet_interp::Interpreter`) — no subprocess — so the test
//! is fast and deterministic.

use garnet_interp::Interpreter;
use garnet_vm::{compile_source, run_function_with_options, RunOptions};
use proptest::prelude::*;

fn quiet() -> RunOptions {
    RunOptions { emit_stdout: false }
}

/// Run `main` under the tree-walk interpreter and return its displayed value.
fn interp_main(src: &str) -> Result<String, String> {
    let mut interp = Interpreter::new();
    interp
        .load_source(src)
        .map_err(|e| format!("interp load: {e}"))?;
    interp
        .call("main", vec![])
        .map(|v| v.display())
        .map_err(|e| format!("interp call: {e}"))
}

/// Run `main` under the bytecode VM and return its displayed value.
fn vm_main(src: &str) -> Result<String, String> {
    run_function_with_options(src, "main", vec![], quiet())
        .map(|r| r.value.display())
        .map_err(|e| format!("vm run: {e}"))
}

/// Whether `main` lowered to native bytecode (`true`) or fell back to the
/// tree-walk interpreter (`false`).
fn main_is_native(src: &str) -> bool {
    let artifact = compile_source(src).expect("compile");
    artifact
        .program
        .functions
        .iter()
        .find(|f| f.name == "main")
        .map(|f| f.native)
        .expect("main present")
}

const PROBE: &str = "@caps()\n\
def main() -> int {\n\
\x20 let x = 1\n\
\x20 if true { let x = 2  x }\n\
\x20 x\n\
}\n";

#[test]
fn probe_program_matches_interpreter() {
    let interp = interp_main(PROBE).expect("interp");
    let vm = vm_main(PROBE).expect("vm");
    assert_eq!(
        interp, "1",
        "interpreter is the reference: inner let is block-local"
    );
    assert_eq!(
        vm, interp,
        "vm must agree with interp on the shadowing probe"
    );
    assert!(
        !main_is_native(PROBE),
        "the shadowing program must fall back to tree-walk (not native)"
    );
}

#[test]
fn non_shadowing_program_stays_native_and_matches() {
    let src = "@caps()\n\
def main() -> int {\n\
\x20 let a = 1\n\
\x20 let b = 2\n\
\x20 a + b\n\
}\n";
    let interp = interp_main(src).expect("interp");
    let vm = vm_main(src).expect("vm");
    assert_eq!(interp, "3");
    assert_eq!(
        vm, interp,
        "vm and interp must agree on the non-shadowing program"
    );
    assert!(
        main_is_native(src),
        "a plain non-shadowing program must still lower to VM native (no over-fallback)"
    );
}

#[test]
fn same_scope_rebind_stays_native_and_matches() {
    // Two `let x` in the SAME block is a rebind, NOT enclosing-scope shadowing —
    // it must still compile to VM native bytecode.
    let src = "@caps()\n\
def main() -> int {\n\
\x20 let x = 1\n\
\x20 let x = 2\n\
\x20 x\n\
}\n";
    let interp = interp_main(src).expect("interp");
    let vm = vm_main(src).expect("vm");
    assert_eq!(interp, "2");
    assert_eq!(vm, interp);
    assert!(
        main_is_native(src),
        "same-scope rebind must stay native, not fall back"
    );
}

// ───────────────────────── proptest backstop ─────────────────────────

/// A tiny expression over a fixed name pool. Only `+ - *` and int literals so
/// generated programs never trap (no div-by-zero) and stay deterministic.
#[derive(Clone, Debug)]
enum GExpr {
    Lit(i64),
    Var(usize),
    Add(Box<GExpr>, Box<GExpr>),
    Sub(Box<GExpr>, Box<GExpr>),
    Mul(Box<GExpr>, Box<GExpr>),
}

const NAMES: [&str; 3] = ["a", "b", "c"];

/// Generate an expression that references ONLY names whose index is in
/// `in_scope` (so the program always type/scope-checks). `in_scope` is never
/// empty in our generator (the body seeds at least one binding before any
/// expression that may reference a var).
fn gexpr(in_scope: Vec<usize>) -> impl Strategy<Value = GExpr> {
    let leaf = prop_oneof![
        (-20i64..20).prop_map(GExpr::Lit),
        prop::sample::select(in_scope).prop_map(GExpr::Var),
    ];
    leaf.prop_recursive(3, 12, 2, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(l, r)| GExpr::Add(Box::new(l), Box::new(r))),
            (inner.clone(), inner.clone()).prop_map(|(l, r)| GExpr::Sub(Box::new(l), Box::new(r))),
            (inner.clone(), inner).prop_map(|(l, r)| GExpr::Mul(Box::new(l), Box::new(r))),
        ]
    })
}

fn render_expr(e: &GExpr, out: &mut String) {
    match e {
        GExpr::Lit(n) => out.push_str(&n.to_string()),
        GExpr::Var(i) => out.push_str(NAMES[*i]),
        GExpr::Add(l, r) => render_bin(l, "+", r, out),
        GExpr::Sub(l, r) => render_bin(l, "-", r, out),
        GExpr::Mul(l, r) => render_bin(l, "*", r, out),
    }
}

fn render_bin(l: &GExpr, op: &str, r: &GExpr, out: &mut String) {
    out.push('(');
    render_expr(l, out);
    out.push(' ');
    out.push_str(op);
    out.push(' ');
    render_expr(r, out);
    out.push(')');
}

/// A statement in a generated block: either a `let <name> = <expr>` binder
/// (possibly shadowing an enclosing name) or a nested `if true { ... }` block.
#[derive(Clone, Debug)]
enum GStmt {
    Let { name: usize, value: GExpr },
    NestedIf { body: Vec<GStmt>, tail: GExpr },
}

/// Bounded program generator: a function body is a sequence of `let`s and nested
/// `if true { ... }` blocks, ending in a tail expression. Names are drawn from a
/// 3-name pool so nested blocks frequently re-bind an enclosing name (exercising
/// the shadowing path) but also frequently do not (exercising native lowering).
///
/// To keep every program valid we thread the set of in-scope names through
/// generation: a `let` adds its name to the current scope, a nested block starts
/// from the parent scope, and every expression only references in-scope names.
/// At least one binding is always introduced before any var-referencing
/// expression.
fn gen_block(
    seed_scope: Vec<usize>,
    depth: u32,
) -> impl Strategy<Value = (Vec<GStmt>, GExpr, Vec<usize>)> {
    // Number of statements in this block.
    prop::collection::vec(0usize..NAMES.len(), 0..3).prop_flat_map(move |let_names| {
        let seed_scope = seed_scope.clone();
        // Decide, for each slot, let vs nested-if via a parallel bool vec.
        prop::collection::vec(any::<bool>(), let_names.len()).prop_flat_map(move |is_nested| {
            let seed_scope = seed_scope.clone();
            let let_names = let_names.clone();
            build_stmts(seed_scope, let_names, is_nested, depth)
        })
    })
}

fn build_stmts(
    seed_scope: Vec<usize>,
    let_names: Vec<usize>,
    is_nested: Vec<bool>,
    depth: u32,
) -> BoxedStrategy<(Vec<GStmt>, GExpr, Vec<usize>)> {
    // Always start with a `let a = <lit>` so the scope is non-empty for any
    // expression that follows. This keeps generated programs trivially valid.
    let mut base: BoxedStrategy<(Vec<GStmt>, Vec<usize>)> = {
        let scope0 = vec![0usize];
        Just((
            vec![GStmt::Let {
                name: 0,
                value: GExpr::Lit(1),
            }],
            // seed_scope ∪ {0}
            merge_scope(&seed_scope, 0),
        ))
        .prop_map(move |(stmts, _)| (stmts, merge_scope(&scope0, 0)))
        .boxed()
    };
    // Compose each subsequent statement, threading the live scope.
    for (idx, &name) in let_names.iter().enumerate() {
        let nested = is_nested.get(idx).copied().unwrap_or(false);
        let next_depth = depth;
        base = base
            .prop_flat_map(move |(stmts, scope)| {
                let scope_for_expr = scope.clone();
                if nested && next_depth > 0 {
                    // Nested `if true { ... }` block: its body starts from the
                    // current scope (so inner `let`s can shadow it).
                    gen_block(scope.clone(), next_depth - 1)
                        .prop_map(move |(inner_stmts, inner_tail, _inner_scope)| {
                            let mut s = stmts.clone();
                            s.push(GStmt::NestedIf {
                                body: inner_stmts,
                                tail: inner_tail,
                            });
                            // A nested block introduces no bindings into the
                            // enclosing scope.
                            (s, scope.clone())
                        })
                        .boxed()
                } else {
                    // `let <name> = <expr-over-current-scope>`
                    gexpr(scope_for_expr)
                        .prop_map(move |value| {
                            let mut s = stmts.clone();
                            s.push(GStmt::Let { name, value });
                            (s, merge_scope(&scope, name))
                        })
                        .boxed()
                }
            })
            .boxed();
    }
    // Final tail expression over whatever is in scope.
    base.prop_flat_map(|(stmts, scope)| {
        let stmts2 = stmts.clone();
        let scope2 = scope.clone();
        gexpr(scope.clone()).prop_map(move |tail| (stmts2.clone(), tail, scope2.clone()))
    })
    .boxed()
}

fn merge_scope(scope: &[usize], name: usize) -> Vec<usize> {
    let mut s = scope.to_vec();
    if !s.contains(&name) {
        s.push(name);
    }
    s
}

fn render_stmts(stmts: &[GStmt], indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    for stmt in stmts {
        match stmt {
            GStmt::Let { name, value } => {
                out.push_str(&pad);
                out.push_str("let ");
                out.push_str(NAMES[*name]);
                out.push_str(" = ");
                render_expr(value, out);
                out.push('\n');
            }
            GStmt::NestedIf { body, tail } => {
                out.push_str(&pad);
                out.push_str("if true {\n");
                render_stmts(body, indent + 1, out);
                out.push_str(&"  ".repeat(indent + 1));
                render_expr(tail, out);
                out.push('\n');
                out.push_str(&pad);
                out.push_str("}\n");
            }
        }
    }
}

/// Render a full `@caps() def main() -> int { ... }` program.
fn render_program(stmts: &[GStmt], tail: &GExpr) -> String {
    let mut out = String::from("@caps()\ndef main() -> int {\n");
    render_stmts(stmts, 1, &mut out);
    out.push_str("  ");
    render_expr(tail, &mut out);
    out.push('\n');
    out.push_str("}\n");
    out
}

proptest! {
    // Bounded: a few hundred shrink-friendly cases. The objective gate — if the
    // shadowing detector ever misses a shape, the VM output diverges from the
    // interpreter and this fails with the offending (shrunk) program.
    #![proptest_config(ProptestConfig::with_cases(300))]
    #[test]
    fn prop_vm_matches_interp_on_random_shadowing_programs(
        (stmts, tail, _scope) in gen_block(vec![], 3)
    ) {
        let src = render_program(&stmts, &tail);

        let interp = interp_main(&src);
        let vm = vm_main(&src);

        // Both lanes must reach the same terminal state: same Ok value, or both
        // Err (e.g. an i64 overflow trap, which both sides report identically).
        match (interp, vm) {
            (Ok(i), Ok(v)) => prop_assert_eq!(
                &i, &v,
                "VM disagreed with interpreter\n--- program ---\n{}\ninterp={} vm={}",
                src, i, v
            ),
            (Err(_), Err(_)) => { /* both trapped — parity holds */ }
            (i, v) => prop_assert!(
                false,
                "VM/interp Ok-vs-Err mismatch\n--- program ---\n{}\ninterp={:?} vm={:?}",
                src, i, v
            ),
        }
    }
}

#[cfg(test)]
mod generator_coverage {
    use super::*;
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;

    // Meta-guard: prove the generator yields BOTH fallback (enclosing-scope
    // shadowing) and native (non-shadowing) programs, so the parity proptest
    // above truly exercises the shadowing path instead of degenerating into
    // all-trivial programs. If a future generator tweak stops producing
    // shadowing cases, this fails loudly rather than letting the gate go quiet.
    #[test]
    fn generator_produces_both_shadowing_and_native() {
        let mut runner = TestRunner::deterministic();
        let strat = gen_block(vec![], 3);
        let mut native = 0u32;
        let mut fallback = 0u32;
        for _ in 0..400 {
            let tree = strat.new_tree(&mut runner).expect("tree");
            let (stmts, tail, _scope) = tree.current();
            let src = render_program(&stmts, &tail);
            if main_is_native(&src) {
                native += 1;
            } else {
                fallback += 1;
            }
        }
        assert!(
            fallback > 0,
            "generator never produced a shadowing/fallback program"
        );
        assert!(native > 0, "generator never produced a native program");
    }
}
