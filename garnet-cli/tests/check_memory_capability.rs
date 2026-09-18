//! D-04 (2026-09-18, ADR 0011) — `mem` is a capability the checker knows.
//!
//! Before this cure the four `memory::*` constructors had no registry row, so
//! `garnet check` said nothing about a program that built every memory tier
//! under `@caps()`, and `@caps(mem)` was rejected as an unknown capability.
//! These tests pin the static side of the cure: an undeclared tier is a
//! `caps coverage` diagnostic, a declared one is accepted, and `garnet sandbox`
//! no longer calls `mem` unknown. `caps_manifest_reports_the_memory_tier` and
//! `gaining_mem_is_authority_expansion` already held before the cure — the
//! manifest and `diff-caps` keep full string fidelity for any declared name —
//! and are kept so the ADR's "adding a tier moves a `diff-caps` verdict"
//! promise stays pinned.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn fresh(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "garnet-check-mem-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    p
}

fn check(program: &str) -> (Option<i32>, String) {
    let dir = fresh("check");
    let p = write(&dir, "m.garnet", program);
    let out = garnet().arg("check").arg(&p).output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code(), text)
}

const MEMORY_TIERS: [&str; 4] = [
    "memory::working",
    "memory::episodic",
    "memory::semantic",
    "memory::procedural",
];

#[test]
fn check_rejects_an_undeclared_memory_tier() {
    for tier in MEMORY_TIERS {
        let (code, text) = check(&format!(
            "@caps()\ndef main() {{\n  let store = {tier}(\"scratch\")\n  0\n}}\n"
        ));
        assert_eq!(
            code,
            Some(1),
            "{tier}: undeclared memory must fail check: {text}"
        );
        assert!(
            text.contains(&format!(
                "caps coverage: function `main` does not declare `mem` but transitively calls `{tier}` which requires it"
            )),
            "{tier}: expected a caps coverage diagnostic naming `mem`, got: {text}"
        );
    }
}

#[test]
fn check_accepts_a_declared_memory_tier() {
    let (code, text) = check(
        "@caps(mem)\ndef main() {\n  let store = memory::working(\"scratch\")\n  store.push(\"a\")\n  store.len()\n}\n",
    );
    assert_eq!(code, Some(0), "declared @caps(mem) must pass check: {text}");
    assert!(
        !text.contains("unknown capability"),
        "`mem` must be a known capability: {text}"
    );
    assert!(
        !text.contains("caps coverage"),
        "declared `mem` must satisfy caps coverage: {text}"
    );
    // The memory rows are `Stability::Experimental` like their S22 siblings,
    // so the layer policy's non-fatal "calls experimental primitive" warning
    // is expected here; only capability diagnostics are asserted absent.
    assert!(
        !text.contains("error"),
        "declared `mem` must produce no error-level diagnostic: {text}"
    );
}

#[test]
fn check_rejects_a_memory_tier_reached_through_a_helper() {
    let (code, text) = check(
        "@caps(mem)\ndef helper() {\n  memory::episodic(\"trace\")\n}\n\n@caps()\ndef main() {\n  helper()\n  0\n}\n",
    );
    assert_eq!(code, Some(1), "{text}");
    assert!(
        text.contains("does not declare `mem`") && text.contains("(via helper)"),
        "expected the transitive caps coverage diagnostic, got: {text}"
    );
}

#[test]
fn fs_does_not_cover_memory_at_check_time() {
    // ADR 0011 rejected mapping memory onto `fs`.
    let (code, text) =
        check("@caps(fs)\ndef main() {\n  let store = memory::semantic(\"facts\")\n  0\n}\n");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("does not declare `mem`"), "{text}");
}

#[test]
fn caps_manifest_reports_the_memory_tier() {
    let dir = fresh("caps");
    let p = write(
        &dir,
        "m.garnet",
        "@caps(mem)\ndef main() {\n  let store = memory::working(\"scratch\")\n  0\n}\n",
    );
    let out = garnet().arg("caps").arg(&p).output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains(r#""aggregate":["mem"]"#), "{s}");
}

#[test]
fn gaining_mem_is_authority_expansion() {
    let dir = fresh("diff");
    let old = write(&dir, "old.garnet", "@caps(fs)\ndef main() { 1 }\n");
    let new = write(&dir, "new.garnet", "@caps(fs, mem)\ndef main() { 1 }\n");
    let out = garnet()
        .arg("diff-caps")
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "gaining mem must exit 1");
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("AUTHORITY EXPANDED"), "{s}");
    assert!(s.contains("caps GAINED") && s.contains("mem"), "{s}");
}

#[test]
fn sandbox_policy_does_not_call_mem_unknown() {
    // `garnet sandbox` maps the closed capability set to WASI facilities and
    // warns on names outside it. `mem` is in the closed set after D-04, so
    // it must not be reported as an unknown capability.
    let dir = fresh("sandbox");
    let p = write(
        &dir,
        "m.garnet",
        "@caps(mem)\ndef main() {\n  let store = memory::working(\"scratch\")\n  0\n}\n",
    );
    let out = garnet().arg("sandbox").arg(&p).output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !text.contains("unknown capability `mem`"),
        "mem is a canonical capability, not an unknown one: {text}"
    );
}

// D-04b (2026-09-18, cross-family review of 4ce90eb6) — the `memory`
// declaration is charged to the program entry the way its constructor call
// is. Before this cure `garnet check` walked only function bodies, so a
// top-level or actor `memory ...` declaration under `@caps()` produced no
// diagnostic and `garnet caps` reported an empty set for a program that owned
// four stores.

#[test]
fn check_rejects_an_undeclared_memory_declaration() {
    for (decl, tier) in [
        ("memory working scratch : String", "memory::working"),
        ("memory episodic scratch : EpisodeStore<String>", "memory::episodic"),
        ("memory semantic scratch : VectorIndex<String>", "memory::semantic"),
        ("memory procedural scratch : WorkflowStore<String>", "memory::procedural"),
    ] {
        let (code, text) = check(&format!("{decl}\n\n@caps()\ndef main() {{\n  1\n}}\n"));
        assert_eq!(
            code,
            Some(1),
            "`{decl}`: an undeclared memory declaration must fail check: {text}"
        );
        assert!(
            text.contains(&format!(
                "caps coverage: function `main` does not declare `mem` but transitively calls `{tier}` which requires it"
            )),
            "`{decl}`: expected a caps coverage diagnostic naming `mem` via `{tier}`, got: {text}"
        );
    }
}

#[test]
fn check_accepts_a_declared_memory_declaration() {
    let (code, text) = check(
        "memory working scratch : String\n\n@caps(mem)\ndef main() {\n  scratch.push(\"a\")\n  scratch.len()\n}\n",
    );
    assert_eq!(code, Some(0), "declared @caps(mem) must pass check: {text}");
    assert!(
        !text.contains("caps coverage"),
        "declared `mem` must satisfy caps coverage: {text}"
    );
}

#[test]
fn check_rejects_an_undeclared_actor_memory_declaration() {
    let (code, text) = check(
        "actor Recorder {\n  memory episodic log : EpisodeStore<String>\n\n  protocol note(x: String) -> Int\n\n  on note(x) {\n    log.append(x)\n    1\n  }\n}\n\n@caps()\ndef main() {\n  let r = spawn Recorder.note(\"a\")\n  r\n}\n",
    );
    assert_eq!(code, Some(1), "{text}");
    assert!(
        text.contains("does not declare `mem`") && text.contains("memory::episodic"),
        "expected the caps coverage diagnostic for the actor store, got: {text}"
    );
}

#[test]
fn check_rejects_an_undeclared_module_memory_declaration() {
    let (code, text) = check(
        "module Store {\n  memory semantic facts : VectorIndex<String>\n}\n\n@caps()\ndef main() {\n  1\n}\n",
    );
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("does not declare `mem`"), "{text}");
}

#[test]
fn fs_does_not_cover_a_memory_declaration_at_check_time() {
    let (code, text) =
        check("memory semantic facts : VectorIndex<String>\n\n@caps(fs)\ndef main() {\n  1\n}\n");
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("does not declare `mem`"), "{text}");
}
