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
    let mut total_run = 0usize;
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
        if let Some(helper_src) = main_src.as_ref() {
            if !is_main_file {
                if let Err(e) = interp.load_source(helper_src) {
                    eprintln!(
                        "garnet test: failed to preload src/main.garnet for {}: {e}",
                        file.display()
                    );
                }
            }
        }
        if let Err(e) = interp.load_source(&src) {
            eprintln!("garnet test: load error in {}: {e}", file.display());
            total_failed += test_names.len();
            for n in &test_names {
                failed_names.push(format!("{}::{}", file.display(), n));
            }
            continue;
        }

        println!("running {} test(s) in {}", test_names.len(), file.display());
        for name in &test_names {
            total_run += 1;
            match interp.call(name, vec![]) {
                Ok(Value::Nil) | Ok(_) => {
                    println!("  test {name} ... ok");
                }
                Err(RuntimeError::Raised(v)) => {
                    println!("  test {name} ... FAILED: {}", v.display());
                    failed_names.push(format!("{}::{}", file.display(), name));
                    total_failed += 1;
                }
                Err(e) => {
                    println!("  test {name} ... FAILED: {e}");
                    failed_names.push(format!("{}::{}", file.display(), name));
                    total_failed += 1;
                }
            }
        }
    }

    println!();
    let passed = total_run - total_failed;
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
