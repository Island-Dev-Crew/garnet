//! `garnet test [<dir>]` — discover + run every function whose name starts
//! with `test_` in the project's `tests/*.garnet` files (and, optionally,
//! `src/main.garnet`). Mirrors the Cargo `cargo test` convention: a test
//! fails iff it raises a `RuntimeError::Raised(...)` exception, otherwise
//! passes. Reports a per-test pass/fail line + a summary; exits non-zero
//! if any test fails. Phase 6E (v4.2).

use garnet_interp::{Interpreter, RuntimeError, Value};
use std::path::PathBuf;
use std::process::ExitCode;

pub fn run(args: &[String]) -> ExitCode {
    // Optional positional argument: the project root. Defaults to CWD.
    let mut project_root = PathBuf::from(".");
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
                project_root = PathBuf::from(args[i].clone());
                i += 1;
            }
            other => {
                eprintln!("unknown test flag: {other}");
                return ExitCode::from(2);
            }
        }
    }

    // Discover candidate files: every .garnet under tests/. The project's
    // src/main.garnet is loaded as a HELPER context (so test functions can
    // call helpers defined in main.garnet — the Cargo convention) rather
    // than as a test file itself, unless --no-main is passed.
    let tests_dir = project_root.join("tests");
    let mut files: Vec<PathBuf> = Vec::new();
    if tests_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&tests_dir) {
            let mut sorted: Vec<PathBuf> = entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "garnet"))
                .collect();
            sorted.sort();
            files.extend(sorted);
        }
    }
    // src/main.garnet is loaded as a helper context for test files, not as
    // a test file itself. Test functions named `test_*` defined inside
    // main.garnet still get discovered + run if --no-main is NOT passed.
    let main_path = project_root.join("src/main.garnet");
    let main_src: Option<String> = if include_main && main_path.is_file() {
        std::fs::read_to_string(&main_path).ok()
    } else {
        None
    };
    if main_src.is_some() {
        files.push(main_path.clone());
    }

    if files.is_empty() {
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

    for file in &files {
        let src = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("garnet test: failed to read {}: {e}", file.display());
                return ExitCode::from(1);
            }
        };
        let module = match garnet_parser::parse_source(&src) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("garnet test: parse error in {}: {e:?}", file.display());
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
        let is_main_file = file == &main_path;
        // Firewalled: top-level `let`/`const`/`memory` initializers are EVALUATED
        // during `load_source` (the interpreter's `register_item`), so a load can
        // panic exactly like a test body can (e.g. `const X = i64::MIN.abs()`).
        // A helper-preload panic taints this file's interpreter; fail the file's
        // tests and move on rather than aborting the whole run.
        if let Some(helper_src) = main_src.as_ref() {
            if !is_main_file {
                match crate::panic_firewall::firewalled(|| {
                    interp.load_source_with_entry_caps(helper_src, "main")
                }) {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        eprintln!(
                            "garnet test: failed to preload src/main.garnet for {}: {e}",
                            file.display()
                        );
                    }
                    Err(panic_msg) => {
                        eprintln!(
                            "garnet test: src/main.garnet preload panicked for {}: {panic_msg}",
                            file.display()
                        );
                        total_failed += test_names.len();
                        for n in &test_names {
                            failed_names.push(format!("{}::{}", file.display(), n));
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
        let load_result =
            crate::panic_firewall::firewalled(|| interp.load_source_with_entry_caps(&src, "main"));
        let load_failure = match load_result {
            Ok(Ok(())) => None,
            Ok(Err(e)) => Some(format!("load error in {}: {e}", file.display())),
            Err(panic_msg) => Some(format!("load panicked in {}: {panic_msg}", file.display())),
        };
        if let Some(reason) = load_failure {
            eprintln!("garnet test: {reason}");
            total_failed += test_names.len();
            for n in &test_names {
                failed_names.push(format!("{}::{}", file.display(), n));
            }
            continue;
        }

        println!("running {} test(s) in {}", test_names.len(), file.display());
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
                    failed_names.push(format!("{}::{}", file.display(), name));
                    total_failed += 1;
                }
                Ok(Err(e)) => {
                    println!("  test {name} ... FAILED: {e}");
                    failed_names.push(format!("{}::{}", file.display(), name));
                    total_failed += 1;
                }
                Err(panic_msg) => {
                    println!("  test {name} ... FAILED (panicked): {panic_msg}");
                    failed_names.push(format!("{}::{}", file.display(), name));
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
            files.len()
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
