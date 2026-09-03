//! U-91 — the entry point's declared budget bounds every gated primitive.
//!
//! The defect these tests pin: `garnet check` builds callee edges only from
//! NAMED calls, so a `def` reached through a function value, a closure, an actor
//! handler, a top-level initializer, or a map of functions produces no edge and
//! the caller's transitive caps stay empty. At run time the interpreter's
//! `call_fn` pushed the CALLEE's own `@caps`, and `require_capability` passed if
//! ANY active frame declared the capability. So a program whose entry declares
//! `@caps()` reached a `@caps(fs)` helper and WROTE THE FILE, with `garnet check`
//! reporting 0 diagnostics.
//!
//! The cure extends the S92 program-entry gate (`require_entry_capability`) from
//! the three subprocess-launch surfaces to the whole gated surface, so the entry
//! point's budget binds regardless of which call edge the checker missed. That is
//! the semantics `garnet check` already enforces for named chains — it rejects
//! `@caps() main` → `@caps(fs) helper` with `caps coverage: ... transitively
//! calls '(via helper)'` — and the semantics the VM's natively-lowered path
//! already had, since VM frames carry no per-callee caps guard.
//!
//! Each case asserts BOTH the trap and the ABSENCE of the side effect: a trap
//! that fires after the file is written would not be a cure.

use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

/// The program-entry trap for `cap`, raised when the entry point's declared
/// budget does not cover a gated primitive reached anywhere beneath it.
fn entry_trap(cap: &str) -> String {
    format!("requires program entry @caps({cap})")
}

/// Run `program` with the process CWD set to a fresh temp dir, so a relative
/// `leak.txt` written by the program lands there and is observable.
struct Ran {
    out: std::process::Output,
    leaked: bool,
}

fn run_in_sandbox(program: &str, backend: &str) -> Ran {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("prog.garnet");
    std::fs::write(&path, program).unwrap();
    let out = garnet()
        .args(["run", backend])
        .arg("prog.garnet")
        .current_dir(dir.path())
        .output()
        .unwrap();
    let leaked = dir.path().join("leak.txt").exists();
    Ran { out, leaked }
}

/// The core U-91 assertion for one laundering shape.
///
/// `--interp` must raise the program-entry trap by name (it is the backend whose
/// per-callee caps frame created the hole). `--vm` must also refuse: its message
/// is whichever gate fires first, which depends on whether the helper lowered
/// natively (no per-callee frame → the call-chain gate fires) or fell back to the
/// tree-walk interpreter (per-callee frame → the entry gate fires). Both forms
/// are a capability refusal; neither may write the file.
fn entry_budget_binds(program: &str, cap: &str) {
    let interp = run_in_sandbox(program, "--interp");
    assert!(
        !interp.out.status.success(),
        "--interp must refuse: the entry declares @caps() but the program reaches a {cap} \
         primitive\nprogram:\n{program}"
    );
    let interp_err = String::from_utf8_lossy(&interp.out.stderr);
    assert!(
        interp_err.contains(&entry_trap(cap)),
        "--interp must name the program-entry budget; got: {interp_err}"
    );
    assert!(
        !interp.leaked,
        "--interp trapped but the side effect still happened: leak.txt was written"
    );

    let vm = run_in_sandbox(program, "--vm");
    assert!(
        !vm.out.status.success(),
        "--vm must refuse the same program\nprogram:\n{program}"
    );
    let vm_err = String::from_utf8_lossy(&vm.out.stderr);
    // Two refusal forms are accepted on the VM, and only two. Either it reached
    // the capability gate, or it hit its own pre-existing inability to invoke a
    // function value held in a local (`unknown function 'f'`) — a lowering
    // limitation that predates this cure and fails CLOSED. Anything else, and
    // especially a success, would mean the VM found a third path.
    let capability_refusal = vm_err.contains(&format!("@caps({cap})"));
    let cannot_invoke_fn_value = vm_err.contains("unknown function");
    assert!(
        capability_refusal || cannot_invoke_fn_value,
        "--vm must refuse with either a {cap} capability trap or its documented \
         function-value limitation; got: {vm_err}"
    );
    assert!(
        !vm.leaked,
        "--vm trapped but the side effect still happened: leak.txt was written"
    );
}

/// A `@caps(fs)` helper that writes the observable file, plus a `@caps()` entry.
fn fs_leak_program(main_body: &str) -> String {
    format!(
        "@caps(fs)\ndef leak() {{\n  write_file(\"leak.txt\", \"u91\")\n  true\n}}\n\n\
         @caps()\ndef main() {{\n{main_body}\n}}\n"
    )
}

// ---------------------------------------------------------------------------
// The nine laundering shapes the checker cannot see. Each one wrote the file
// before the cure, on at least one backend, with `garnet check` clean.
// ---------------------------------------------------------------------------

/// L1 — a bare alias binding the `def` to a local, then calling the local.
#[test]
fn alias_to_declaring_helper_is_bound_by_the_entry_budget() {
    entry_budget_binds(&fs_leak_program("  let f = leak\n  f()"), "fs");
}

/// L2 — a closure body calling the helper. `walk_expr_for_callees`'s
/// `Expr::Closure` arm is a no-op, so the checker sees no edge. Leaked on BOTH
/// backends before the cure (the closure forces the tree-walk fallback).
#[test]
fn closure_to_declaring_helper_is_bound_by_the_entry_budget() {
    entry_budget_binds(&fs_leak_program("  let g = || leak()\n  g()"), "fs");
}

/// L3 — the helper passed as a higher-order argument.
#[test]
fn higher_order_argument_is_bound_by_the_entry_budget() {
    let program = "@caps(fs)\ndef leak() {\n  write_file(\"leak.txt\", \"u91\")\n  true\n}\n\n\
                   def apply(g) {\n  g()\n}\n\n\
                   @caps()\ndef main() {\n  apply(leak)\n}\n";
    entry_budget_binds(program, "fs");
}

/// L4 — the helper reached from a string-interpolation part. The `Expr::Str`
/// arm of `walk_expr_for_callees` is a no-op, so interpolated calls are invisible.
#[test]
fn string_interpolation_is_bound_by_the_entry_budget() {
    entry_budget_binds(&fs_leak_program("  let s = \"#{leak()}\"\n  s"), "fs");
}

/// L5 — an actor handler calling the helper. `Item::Actor` is never walked by
/// `collect_fn_callees`, so handler bodies contribute no edges at all.
#[test]
fn actor_handler_is_bound_by_the_entry_budget() {
    let program = "@caps(fs)\ndef leak() {\n  write_file(\"leak.txt\", \"u91\")\n  true\n}\n\n\
                   actor A {\n  protocol go() -> Bool\n  on go() {\n    leak()\n    true\n  }\n}\n\n\
                   @caps()\ndef main() {\n  spawn A.go()\n}\n";
    entry_budget_binds(program, "fs");
}

/// L6 — a top-level `let` initializer calling the helper. `Item::Let` is not
/// walked either, and the initializer runs at LOAD time, before `main`'s body.
/// `garnet run` installs the entry frame from the entry annotations across the
/// load precisely so load-time authority is checked against the same budget.
#[test]
fn top_level_initializer_is_bound_by_the_entry_budget() {
    let program = "@caps(fs)\ndef leak() {\n  write_file(\"leak.txt\", \"u91\")\n  true\n}\n\n\
                   let z = leak()\n\n\
                   @caps()\ndef main() {\n  z\n}\n";
    entry_budget_binds(program, "fs");
}

/// c7c — the helper stored in a map and invoked through the subscript. Leaked on
/// BOTH backends before the cure.
#[test]
fn map_of_functions_is_bound_by_the_entry_budget() {
    entry_budget_binds(
        &fs_leak_program("  let tbl = {\"go\" => leak}\n  tbl[\"go\"]()"),
        "fs",
    );
}

/// L11 — the same shape against host ENVIRONMENT authority rather than fs. Before
/// the cure this returned the real `$HOME` to a program whose entry declared
/// nothing.
#[test]
fn env_read_through_declaring_helper_is_bound_by_the_entry_budget() {
    let program = "@caps(env)\ndef leak() {\n  std::env::get(\"HOME\")\n}\n\n\
                   @caps()\ndef main() {\n  let f = leak\n  f()\n}\n";
    entry_budget_binds(program, "env");
}

/// L12 — network authority. Before the cure the caps gate PASSED here and only
/// the strict default `NetPolicy` (which denies loopback) stopped the socket, so
/// the trap message was a network denial, not a capability refusal. After the
/// cure the capability gate refuses first, before any address is evaluated.
#[test]
fn net_connect_through_declaring_helper_is_bound_by_the_entry_budget() {
    let program = "@caps(net)\ndef leak() {\n  tcp_connect(\"127.0.0.1\", 9)\n}\n\n\
                   @caps()\ndef main() {\n  let f = leak\n  f()\n}\n";
    let out = run_in_sandbox(program, "--interp").out;
    assert!(!out.status.success(), "--interp must refuse");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(&entry_trap("net")),
        "the capability gate must refuse BEFORE the net policy evaluates the address; got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Positive controls — the cure must bind under-declared entries only. A program
// whose entry declares the capability, or the wildcard, is unaffected.
// ---------------------------------------------------------------------------

/// The same laundering shape with `@caps(fs)` on the entry runs normally and
/// writes the file. This is the anti-over-rejection control: the cure changes
/// which programs are refused, not what a correctly-declared program can do.
///
/// The CLOSURE shape is used rather than the bare alias because the VM cannot
/// invoke a function value held in a local at all (`unknown function 'f'`) — a
/// pre-existing backend limitation, unrelated to capabilities, that would make an
/// alias-shaped positive control fail on `--vm` for the wrong reason.
#[test]
fn declared_entry_budget_still_permits_the_helper() {
    let program = "@caps(fs)\ndef leak() {\n  write_file(\"leak.txt\", \"u91\")\n  true\n}\n\n\
                   @caps(fs)\ndef main() {\n  let g = || leak()\n  g()\n}\n";
    for backend in ["--interp", "--vm"] {
        let ran = run_in_sandbox(program, backend);
        assert!(
            ran.out.status.success(),
            "{backend} must still run a program whose entry declares fs: {}",
            String::from_utf8_lossy(&ran.out.stderr)
        );
        assert!(
            ran.leaked,
            "{backend}: the declared program must actually perform its effect"
        );
    }
}

/// A wildcard entry budget covers everything, exactly as it does for the three
/// subprocess surfaces today (`has(\"*\")` in `require_entry_capability`).
#[test]
fn wildcard_entry_budget_still_permits_the_helper() {
    let program = "@caps(fs)\ndef leak() {\n  write_file(\"leak.txt\", \"u91\")\n  true\n}\n\n\
                   @caps(*)\ndef main() {\n  let f = leak\n  f()\n}\n";
    let ran = run_in_sandbox(program, "--interp");
    assert!(
        ran.out.status.success(),
        "@caps(*) entry must still be the documented escape hatch: {}",
        String::from_utf8_lossy(&ran.out.stderr)
    );
    assert!(ran.leaked, "the wildcard program must perform its effect");
}

/// Pure computation under `@caps()` is untouched — no gated primitive, no gate.
#[test]
fn pure_computation_is_unaffected_by_the_entry_gate() {
    let ran = run_in_sandbox("@caps()\ndef main() {\n  1 + 2 * 3\n}\n", "--interp");
    let stdout = String::from_utf8_lossy(&ran.out.stdout);
    assert!(ran.out.status.success(), "pure code must run: {stdout}");
    assert!(stdout.contains("=> 7"), "got {stdout}");
}

// ---------------------------------------------------------------------------
// The `test` and `doctest` lanes inherit the same defect, and the same cure.
// Both already refuse the three entry-gated subprocess surfaces reached through
// a declaring helper; these pin that the other gated primitives now match.
// ---------------------------------------------------------------------------

/// TST3 — a `@caps()` test function reaching a `@caps(fs)` helper by alias.
/// `garnet test` routes each test through `call_entry`, so the TEST FUNCTION is
/// the program entry and its budget is what binds.
#[test]
fn test_lane_binds_the_test_function_budget() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("tests")).unwrap();
    std::fs::write(
        dir.path().join("tests/probe.garnet"),
        "@caps(fs)\ndef leak() {\n  write_file(\"leak.txt\", \"u91\")\n  true\n}\n\n\
         @caps()\ndef test_leak() {\n  let f = leak\n  f()\n}\n",
    )
    .unwrap();
    let out = garnet()
        .arg("test")
        .arg(".")
        .current_dir(dir.path())
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out.status.success(),
        "garnet test must FAIL a @caps() test that launders fs through a helper:\n{combined}"
    );
    assert!(
        combined.contains(&entry_trap("fs")),
        "expected the program-entry fs trap on test_leak; got:\n{combined}"
    );
    assert!(
        !dir.path().join("leak.txt").exists(),
        "the test lane trapped but the file was still written"
    );
}

/// A `@caps(fs)` test declares its own budget and still passes — the test lane's
/// published "test-runner entry authority" claim is preserved.
#[test]
fn test_lane_still_allows_a_declared_test_budget() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("tests")).unwrap();
    std::fs::write(
        dir.path().join("tests/probe.garnet"),
        "@caps(fs)\ndef leak() {\n  write_file(\"leak.txt\", \"u91\")\n  true\n}\n\n\
         @caps(fs)\ndef test_declared() {\n  let f = leak\n  f()\n}\n",
    )
    .unwrap();
    let out = garnet()
        .arg("test")
        .arg(".")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "a @caps(fs) test must still pass:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// DOC2 — a documentation fence calling a documented `@caps(fs)` helper. Fences
/// evaluate with NO program-entry frame, so under deny-by-default the entry gate
/// refuses them — exactly as it already refuses a fence that reaches
/// `std::process::output` through a `@caps(proc)` helper.
#[test]
fn doctest_lane_refuses_a_fence_that_reaches_gated_authority() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("prog.garnet"),
        "/// Leaks via the documented helper.\n///\n/// ```garnet\n/// leak()\n/// ```\n\
         @caps(fs)\ndef leak() {\n  write_file(\"leak.txt\", \"u91\")\n  true\n}\n\n\
         @caps()\ndef main() { true }\n",
    )
    .unwrap();
    let out = garnet()
        .arg("doctest")
        .arg("prog.garnet")
        .current_dir(dir.path())
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("0 passed, 1 failed"),
        "the fence must FAIL rather than exercise fs authority; got:\n{combined}"
    );
    assert!(
        combined.contains(&entry_trap("fs")),
        "expected the program-entry fs trap in the doctest lane; got:\n{combined}"
    );
    assert!(
        !dir.path().join("leak.txt").exists(),
        "the doctest lane reported a failure but the file was still written"
    );
}
