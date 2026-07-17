//! `garnet test [<dir>]` — discover + run every function whose name starts
//! with `test_` in the project's `tests/*.garnet` files (and, optionally,
//! `src/main.garnet`). Mirrors the Cargo `cargo test` convention: a test
//! fails iff it raises a `RuntimeError::Raised(...)` exception, otherwise
//! passes. Reports a per-test pass/fail line + a summary; exits non-zero
//! if any test fails. Phase 6E (v4.2).

use crate::bound_source::{read_bound_source, BoundSource};
use garnet_interp::{Interpreter, RuntimeError, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run(args: &[String]) -> ExitCode {
    // Optional positional argument: the project root. Defaults to CWD.
    let mut project_root = PathBuf::from(".");
    let mut project_root_seen = false;
    let mut filter: Option<String> = None;
    let mut include_main = true;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--filter" => {
                if i + 1 >= args.len() {
                    eprintln!("--filter requires a substring argument");
                    return ExitCode::from(2);
                }
                filter = Some(args[i + 1].clone());
                i += 2;
            }
            "--no-main" => {
                include_main = false;
                i += 1;
            }
            "--help" | "-h" => {
                println!("usage: garnet test [<project-dir>] [--filter <substr>] [--no-main]");
                println!("  Discovers test_* functions in <dir>/tests/*.garnet (and");
                println!("  src/main.garnet unless --no-main) and runs each. A test");
                println!("  fails iff its body raises an exception.");
                return ExitCode::SUCCESS;
            }
            other if !other.starts_with("--") => {
                if project_root_seen {
                    eprintln!("usage: garnet test [<project-dir>] [--filter <substr>] [--no-main]");
                    return ExitCode::from(2);
                }
                project_root = PathBuf::from(args[i].clone());
                project_root_seen = true;
                i += 1;
            }
            other => {
                eprintln!("unknown test flag: {other}");
                return ExitCode::from(2);
            }
        }
    }

    if let Err(message) = validate_project_root(&project_root) {
        eprintln!("{message}");
        return ExitCode::from(1);
    }

    // Discover candidate files: every .garnet under tests/. The project's
    // src/main.garnet is loaded as a HELPER context (so test functions can
    // call helpers defined in main.garnet — the Cargo convention) rather
    // than as a test file itself, unless --no-main is passed.
    let tests_dir = project_root.join("tests");
    let mut sources = match discover_test_sources(&tests_dir) {
        Ok(sources) => sources,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(1);
        }
    };
    // src/main.garnet is loaded as a helper context for test files, not as
    // a test file itself. Test functions named `test_*` defined inside
    // main.garnet still get discovered + run if --no-main is NOT passed.
    let main_path = project_root.join("src/main.garnet");
    let main_source = match read_optional_main_helper(&main_path, include_main) {
        Ok(source) => source,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(1);
        }
    };
    if let Some(source) = main_source.as_ref() {
        sources.push(source.clone());
    }

    if sources.is_empty() {
        println!(
            "garnet test: no .garnet files found under {}/tests/ or {}/src/main.garnet",
            project_root.display(),
            project_root.display()
        );
        println!("  hint: scaffold a project with `garnet new --template cli <name>`");
        return ExitCode::SUCCESS;
    }

    // Aggregate test_* functions across every file. We load each file into a
    // FRESH interpreter so tests in one file can't leak state into another;
    // matches Cargo's per-file isolation.
    // Count passes directly rather than deriving `passed = run - failed`: a
    // parse/load failure increments `total_failed` for a file without a
    // matching per-test run, so the subtraction could underflow (`usize`) and
    // panic the summary — an exit-101 abort OUTSIDE the firewall. A direct
    // pass tally cannot underflow.
    let mut total_passed = 0usize;
    let mut total_failed = 0usize;
    let mut failed_names: Vec<String> = Vec::new();

    for source in &sources {
        let module = match garnet_parser::parse_source(&source.text) {
            Ok(m) => m,
            Err(e) => {
                eprintln!(
                    "garnet test: parse error in {}: {e:?}",
                    source.path.display()
                );
                total_failed += 1;
                continue;
            }
        };

        let mut test_names: Vec<String> = Vec::new();
        for item in &module.items {
            if let garnet_parser::ast::Item::Fn(f) = item {
                if f.name.starts_with("test_")
                    && f.params.is_empty()
                    && filter.as_deref().is_none_or(|s| f.name.contains(s))
                {
                    test_names.push(f.name.clone());
                }
            }
        }
        if test_names.is_empty() {
            continue;
        }

        let mut interp = Interpreter::new();
        // Pre-load src/main.garnet as a helper context for test files under
        // tests/, so cross-file references (e.g. tests/test_main.garnet
        // calling `timestamp()` from src/main.garnet) resolve correctly.
        // Skip when the file BEING tested IS main.garnet itself.
        let is_main_file = source.path == main_path;
        // Firewalled: top-level `let`/`const`/`memory` initializers are EVALUATED
        // during `load_source` (the interpreter's `register_item`), so a load can
        // panic exactly like a test body can (e.g. `const X = i64::MIN.abs()`).
        // A helper-preload panic taints this file's interpreter; fail the file's
        // tests and move on rather than aborting the whole run.
        if let Some(helper_source) = main_source.as_ref() {
            if !is_main_file {
                match crate::panic_firewall::firewalled(|| {
                    interp.load_source_with_entry_caps(&helper_source.text, "main")
                }) {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        // Fail-closed (S114 acceptance, cond. #5): a helper that
                        // fails to load means the file's tests ran against a
                        // broken/partial helper. Setup failure must not produce
                        // a green run — fail the file's tests like the panic arm.
                        eprintln!(
                            "garnet test: failed to preload src/main.garnet for {}: {e}",
                            source.path.display()
                        );
                        total_failed += test_names.len();
                        for n in &test_names {
                            failed_names.push(format!("{}::{}", source.path.display(), n));
                        }
                        continue;
                    }
                    Err(panic_msg) => {
                        eprintln!(
                            "garnet test: src/main.garnet preload panicked for {}: {panic_msg}",
                            source.path.display()
                        );
                        total_failed += test_names.len();
                        for n in &test_names {
                            failed_names.push(format!("{}::{}", source.path.display(), n));
                        }
                        continue;
                    }
                }
            }
        }
        // Firewalled: a load-time panic in the test file fails that file's tests
        // and continues to the next file — never an exit-101 process abort.
        // S114-FIX-2: frame the test-file load under the file's `main` entry so a
        // top-level `let`/`const` initializer is checked against declared `@caps`
        // (parity with `garnet run`); the prior unframed `load_source` let a
        // top-level host read execute fail-open at test-load time.
        let load_result = crate::panic_firewall::firewalled(|| {
            interp.load_source_with_entry_caps(&source.text, "main")
        });
        let load_failure = match load_result {
            Ok(Ok(())) => None,
            Ok(Err(e)) => Some(format!("load error in {}: {e}", source.path.display())),
            Err(panic_msg) => Some(format!(
                "load panicked in {}: {panic_msg}",
                source.path.display()
            )),
        };
        if let Some(reason) = load_failure {
            eprintln!("garnet test: {reason}");
            total_failed += test_names.len();
            for n in &test_names {
                failed_names.push(format!("{}::{}", source.path.display(), n));
            }
            continue;
        }

        println!(
            "running {} test(s) in {}",
            test_names.len(),
            source.path.display()
        );
        for name in &test_names {
            // PR-2: each test is its own program entry — route through `call_entry`
            // so the test's `@caps(...)` is installed as the entry-authority frame
            // and host-authority is checked exactly as `garnet run` checks `main`.
            // (Previously `interp.call` skipped the entry frame, so a `@caps()` test
            // could exercise undeclared authority that `garnet run` would reject.)
            // Firewalled: a panicking test (e.g. an `i64::MIN.abs()` overflow)
            // is recorded as a FAILED case and the run continues to the next
            // test + prints the summary — a single panic must not abort the
            // whole `garnet test` invocation (matching how a real test harness
            // isolates each test).
            match crate::panic_firewall::firewalled(|| interp.call_entry(name, vec![])) {
                Ok(Ok(Value::Nil) | Ok(_)) => {
                    println!("  test {name} ... ok");
                    total_passed += 1;
                }
                Ok(Err(RuntimeError::Raised(v))) => {
                    println!("  test {name} ... FAILED: {}", v.display());
                    failed_names.push(format!("{}::{}", source.path.display(), name));
                    total_failed += 1;
                }
                Ok(Err(e)) => {
                    println!("  test {name} ... FAILED: {e}");
                    failed_names.push(format!("{}::{}", source.path.display(), name));
                    total_failed += 1;
                }
                Err(panic_msg) => {
                    println!("  test {name} ... FAILED (panicked): {panic_msg}");
                    failed_names.push(format!("{}::{}", source.path.display(), name));
                    total_failed += 1;
                }
            }
        }
    }

    println!();
    let passed = total_passed;
    if total_failed == 0 {
        println!(
            "test result: ok. {passed} passed; 0 failed; in {} file(s)",
            sources.len()
        );
        ExitCode::SUCCESS
    } else {
        println!("test result: FAILED. {passed} passed; {total_failed} failed");
        for n in &failed_names {
            println!("  - {n}");
        }
        ExitCode::from(1)
    }
}

/// An explicit nonexistent project path used to flow through the two optional
/// input probes and become a green "no files" result. Establish the project
/// directory itself before treating absent `tests/` or `src/main.garnet` as a
/// legitimate empty project.
fn validate_project_root(project_root: &Path) -> Result<(), String> {
    let metadata = std::fs::metadata(project_root).map_err(|e| {
        format!(
            "garnet test: failed to inspect project root {}: {e}",
            project_root.display()
        )
    })?;
    if !metadata.is_dir() {
        return Err(format!(
            "garnet test: project root {} is not a directory",
            project_root.display()
        ));
    }
    std::fs::read_dir(project_root).map_err(|e| {
        format!(
            "garnet test: project root {} is not readable: {e}",
            project_root.display()
        )
    })?;
    Ok(())
}

/// Discover regular `*.garnet` test files without collapsing filesystem
/// enumeration failures into an empty, successful suite. An absent `tests/`
/// directory is the only no-input case; an existing non-directory, an
/// unreadable directory/entry, or a non-regular `*.garnet` candidate is a
/// setup error.
fn discover_test_sources(tests_dir: &Path) -> Result<Vec<BoundSource>, String> {
    let metadata = match std::fs::symlink_metadata(tests_dir) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(format!(
                "garnet test: failed to inspect {}: {e}",
                tests_dir.display()
            ));
        }
    };
    if !metadata.file_type().is_dir() {
        return Err(format!(
            "garnet test: {} exists but is not a directory",
            tests_dir.display()
        ));
    }

    let entries = std::fs::read_dir(tests_dir).map_err(|e| {
        format!(
            "garnet test: failed to read test directory {}: {e}",
            tests_dir.display()
        )
    })?;
    let candidates = entries.map(|result| {
        let entry = result.map_err(|e| {
            format!(
                "garnet test: failed to enumerate an entry in {}: {e}",
                tests_dir.display()
            )
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| {
            format!(
                "garnet test: failed to inspect discovered path {}: {e}",
                path.display()
            )
        })?;
        Ok((path, file_type.is_file()))
    });
    collect_test_candidates(candidates)
}

/// Pure collector separated from `read_dir` so an iterator-level enumeration
/// error has a deterministic platform-independent regression test.
fn collect_test_candidates<I>(candidates: I) -> Result<Vec<BoundSource>, String>
where
    I: IntoIterator<Item = Result<(PathBuf, bool), String>>,
{
    let mut sources = Vec::new();
    for candidate in candidates {
        let (path, is_regular_file) = candidate?;
        if path
            .extension()
            .is_some_and(|extension| extension == "garnet")
        {
            if !is_regular_file {
                return Err(format!(
                    "garnet test: discovered Garnet test {} is not a regular file",
                    path.display()
                ));
            }
            let source = read_bound_source(&path).map_err(|error| {
                format!(
                    "garnet test: failed to bind discovered source {}: {error}",
                    path.display()
                )
            })?;
            sources.push(source);
        }
    }
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(sources)
}

/// Load the optional helper without using `is_file()`/`.ok()`, both of which
/// turn metadata or read failures into silent absence. `--no-main` is the one
/// explicit opt-out and intentionally avoids inspecting the helper.
fn read_optional_main_helper(
    main_path: &Path,
    include_main: bool,
) -> Result<Option<BoundSource>, String> {
    if !include_main {
        return Ok(None);
    }
    let metadata = match std::fs::symlink_metadata(main_path) {
        Ok(metadata) => metadata,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(format!(
                "garnet test: failed to inspect helper {}: {e}",
                main_path.display()
            ));
        }
    };
    if !metadata.file_type().is_file() {
        return Err(format!(
            "garnet test: helper {} exists but is not a regular file",
            main_path.display()
        ));
    }
    read_bound_source(main_path).map(Some).map_err(|error| {
        format!(
            "garnet test: failed to bind helper {}: {error}",
            main_path.display()
        )
    })
}

#[cfg(test)]
mod discovery_tests {
    use super::collect_test_candidates;

    #[test]
    fn iterator_level_discovery_error_is_not_an_empty_suite() {
        let candidates = vec![Err("synthetic per-entry discovery failure".to_string())];
        let error = collect_test_candidates(candidates).unwrap_err();
        assert!(error.contains("per-entry discovery failure"));
    }
}
