//! The `garnet` binary — the user-facing CLI.
//!
//! A deliberately tiny command dispatcher. All real logic lives in the
//! supporting crates (parser, interp, check, memory). Adding subcommands is
//! as simple as matching another arm here.

use garnet_cli::cache::{self, Episode};
use garnet_cli::knowledge;
use garnet_cli::manifest::Manifest;
use garnet_cli::strategies;
use garnet_cli::{print_help, print_version, read_file};
use std::time::{SystemTime, UNIX_EPOCH};
use garnet_interp::Interpreter;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

/// Helper used by every subcommand at start: surface a one-line note for any
/// prior failures of the same source hash, and consult any strategies
/// matched by AST fingerprint similarity. Silent if nothing relevant.
fn surface_prior(source: &str) {
    let hash = cache::source_hash(source);
    let prior = cache::recall(&hash);
    let failures = prior.iter().filter(|e| e.outcome != "ok").count();
    if failures > 0 {
        eprintln!(
            "note: this source has {failures} prior failure(s) recorded in .garnet-cache/episodes.log"
        );
    }
    // Try to surface relevant strategies — best-effort, never fail.
    if let Ok(module) = garnet_parser::parse_source(source) {
        let target = knowledge::fingerprint(&module);
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        if let Ok(conn) = strategies::open(&cwd) {
            if let Ok(matches) = strategies::consult(&conn, &target, 3) {
                for (dist, s) in matches.iter().filter(|(d, _)| *d <= 32) {
                    eprintln!(
                        "note: strategy '{}' applies (Hamming distance {dist}/256, last triggered ts={})",
                        s.heuristic, s.created_ts
                    );
                }
            }
        }
    }
}

/// Persist a knowledge-graph row + (optionally) synthesize new strategies.
/// Best-effort; failures are silent so they never break the CLI.
fn persist_knowledge(source: &str, source_hash: &str, outcome: &str) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let Ok(module) = garnet_parser::parse_source(source) else {
        return;
    };
    let fp = knowledge::fingerprint(&module);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    if let Ok(conn) = knowledge::open(&cwd) {
        let _ = knowledge::record_context(&conn, source_hash, &fp, outcome, ts);
    }
    // Synthesize strategies from the cumulative episode log.
    if let Ok(strat_conn) = strategies::open(&cwd) {
        let cache_dir = cache::cache_dir_for(&cwd);
        let episodes = cache::read_all_in(&cache_dir);
        let proposed = strategies::synthesize_from_episodes(&episodes, |hash| {
            // Only recover the fingerprint if we just observed this hash.
            if hash == source_hash {
                Some(fp)
            } else {
                None
            }
        });
        for s in proposed {
            let _ = strategies::record_strategy(&strat_conn, &s, ts);
        }
    }
}

/// Helper used by every subcommand at exit: append an episode record AND
/// fold the outcome into the SQLite knowledge + strategies stores.
fn record(
    cmd: &str,
    file: &str,
    source: &str,
    outcome: &str,
    error_kind: Option<String>,
    started: Instant,
    exit_code: i32,
) {
    let duration_ms = started.elapsed().as_millis() as u64;
    let hash = cache::source_hash(source);
    let ep = Episode::now(
        cmd,
        file,
        hash.clone(),
        outcome,
        error_kind,
        duration_ms,
        exit_code,
    );
    let _ = cache::record_episode(&ep);
    persist_knowledge(source, &hash, outcome);
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        return ExitCode::SUCCESS;
    }
    match args[0].as_str() {
        "help" | "-h" | "--help" => {
            print_help();
            ExitCode::SUCCESS
        }
        "version" | "-V" | "--version" => {
            print_version();
            ExitCode::SUCCESS
        }
        "new" => cmd_new(&args[1..]),
        "parse" => {
            if args.len() < 2 {
                eprintln!("usage: garnet parse <file.garnet>");
                return ExitCode::from(2);
            }
            cmd_parse(PathBuf::from(&args[1]))
        }
        "check" => {
            if args.len() < 2 {
                eprintln!("usage: garnet check <file.garnet>");
                return ExitCode::from(2);
            }
            cmd_check(PathBuf::from(&args[1]))
        }
        "run" => {
            if args.len() < 2 {
                eprintln!("usage: garnet run <file.garnet>");
                return ExitCode::from(2);
            }
            cmd_run(PathBuf::from(&args[1]))
        }
        "eval" => {
            if args.len() < 2 {
                eprintln!("usage: garnet eval \"<expr>\"");
                return ExitCode::from(2);
            }
            cmd_eval(&args[1])
        }
        "repl" => {
            let preload = if args.len() >= 2 {
                Some(PathBuf::from(&args[1]))
            } else {
                None
            };
            cmd_repl(preload)
        }
        "build" => {
            // Accept `build <file>`, `build --deterministic <file>`, or
            // `build --deterministic --sign <keyfile> <file>` (v3.4.1).
            let mut deterministic = false;
            let mut sign_keyfile: Option<String> = None;
            let mut file_opt: Option<String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--deterministic" => {
                        deterministic = true;
                        i += 1;
                    }
                    "--sign" => {
                        if i + 1 >= args.len() {
                            eprintln!("--sign requires a keyfile argument");
                            return ExitCode::from(2);
                        }
                        sign_keyfile = Some(args[i + 1].clone());
                        i += 2;
                    }
                    other if !other.starts_with("--") => {
                        file_opt = Some(args[i].clone());
                        i += 1;
                    }
                    other => {
                        eprintln!("unknown build flag: {other}");
                        return ExitCode::from(2);
                    }
                }
            }
            let Some(file) = file_opt else {
                eprintln!("usage: garnet build [--deterministic] [--sign <keyfile>] <file.garnet>");
                return ExitCode::from(2);
            };
            if sign_keyfile.is_some() && !deterministic {
                eprintln!("--sign requires --deterministic (signing only applies to the deterministic manifest)");
                return ExitCode::from(2);
            }
            cmd_build(PathBuf::from(file), deterministic, sign_keyfile)
        }
        "keygen" => {
            // `garnet keygen <keyfile>` — create a fresh Ed25519 signing key
            // and write it to `<keyfile>`. Prints the public key to stdout
            // so the caller can record it as the expected signer.
            if args.len() < 2 {
                eprintln!("usage: garnet keygen <keyfile>");
                return ExitCode::from(2);
            }
            cmd_keygen(PathBuf::from(&args[1]))
        }
        "verify" => {
            if args.len() < 3 {
                eprintln!("usage: garnet verify <file.garnet> <manifest.json> [--signature]");
                return ExitCode::from(2);
            }
            let require_sig = args.iter().any(|a| a == "--signature");
            cmd_verify(PathBuf::from(&args[1]), PathBuf::from(&args[2]), require_sig)
        }
        "convert" => cmd_convert(&args[1..]),
        "test" => cmd_test(&args[1..]),
        other => {
            eprintln!("unknown subcommand: {other}");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn cmd_parse(path: PathBuf) -> ExitCode {
    let started = Instant::now();
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    surface_prior(&src);
    match garnet_parser::parse_source(&src) {
        Ok(module) => {
            println!("parsed {} ({} items, safe={})", path.display(), module.items.len(), module.safe);
            for item in &module.items {
                println!("  - {}", describe_item(item));
            }
            record("parse", &path.display().to_string(), &src, "ok", None, started, 0);
            ExitCode::SUCCESS
        }
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src.clone());
            eprintln!("{report:?}");
            record("parse", &path.display().to_string(), &src, "parse_err", Some("UnexpectedToken".to_string()), started, 1);
            ExitCode::from(1)
        }
    }
}

fn cmd_check(path: PathBuf) -> ExitCode {
    let started = Instant::now();
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    surface_prior(&src);
    let module = match garnet_parser::parse_source(&src) {
        Ok(m) => m,
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src.clone());
            eprintln!("{report:?}");
            record("check", &path.display().to_string(), &src, "parse_err", Some("UnexpectedToken".to_string()), started, 1);
            return ExitCode::from(1);
        }
    };
    let report = garnet_check::check_module(&module);
    for err in &report.errors {
        println!("{err}");
    }
    println!(
        "\n{} functions checked, {} boundary call sites, {} diagnostics",
        report.mode_map.len(),
        report.boundary_call_sites,
        report.errors.len()
    );
    if report.ok() {
        record("check", &path.display().to_string(), &src, "ok", None, started, 0);
        ExitCode::SUCCESS
    } else {
        record("check", &path.display().to_string(), &src, "check_err", Some("safe_violation".to_string()), started, 1);
        ExitCode::from(1)
    }
}

fn cmd_run(path: PathBuf) -> ExitCode {
    let started = Instant::now();
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    surface_prior(&src);
    let mut interp = Interpreter::new();
    if let Err(e) = interp.load_source(&src) {
        eprintln!("load error: {e}");
        record("run", &path.display().to_string(), &src, "parse_err", Some(format!("{e}")), started, 1);
        return ExitCode::from(1);
    }
    // If a `main` function exists, call it; otherwise just exit success.
    if interp.global.get("main").is_some() {
        match interp.call("main", vec![]) {
            Ok(v) => {
                println!("=> {}", v.display());
                record("run", &path.display().to_string(), &src, "ok", None, started, 0);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("runtime error: {e}");
                record("run", &path.display().to_string(), &src, "runtime_err", Some(format!("{e}")), started, 1);
                ExitCode::from(1)
            }
        }
    } else {
        record("run", &path.display().to_string(), &src, "ok", None, started, 0);
        ExitCode::SUCCESS
    }
}

fn cmd_eval(src: &str) -> ExitCode {
    let started = Instant::now();
    let interp = Interpreter::new();
    match interp.eval_expr_src(src) {
        Ok(v) => {
            println!("{}", v.display());
            record("eval", "<inline>", src, "ok", None, started, 0);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            record("eval", "<inline>", src, "runtime_err", Some(format!("{e}")), started, 1);
            ExitCode::from(1)
        }
    }
}

fn cmd_repl(preload: Option<PathBuf>) -> ExitCode {
    let mut repl = garnet_interp::repl::Repl::new();
    if let Some(p) = preload {
        match read_file(&p) {
            Ok(src) => {
                if let Err(e) = repl.preload(&src) {
                    eprintln!("preload error: {e}");
                    return ExitCode::from(1);
                }
                println!("preloaded {}", p.display());
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(1);
            }
        }
    }
    if let Err(e) = repl.run_stdio() {
        eprintln!("REPL IO error: {e}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn cmd_build(path: PathBuf, deterministic: bool, sign_keyfile: Option<String>) -> ExitCode {
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source(&src) {
        Ok(m) => m,
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src);
            eprintln!("{report:?}");
            return ExitCode::from(1);
        }
    };
    if deterministic {
        let mut manifest = Manifest::build(&src, &module);

        // v3.4.1 ManifestSig: if --sign <keyfile> was passed, load the key
        // and sign the manifest in place. The signature covers every field
        // except signer_pubkey + signature themselves.
        if let Some(keyfile) = sign_keyfile.as_deref() {
            let key_hex = match std::fs::read_to_string(keyfile) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("failed to read keyfile {keyfile}: {e}");
                    return ExitCode::from(1);
                }
            };
            let signing_key = match garnet_cli::manifest::signing_key_from_hex(&key_hex) {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("bad keyfile {keyfile}: {e}");
                    return ExitCode::from(1);
                }
            };
            let _payload = manifest.sign(&signing_key);
        }

        let json = manifest.to_canonical_json();
        let manifest_path = path.with_extension(
            path.extension()
                .map(|e| format!("{}.manifest.json", e.to_string_lossy()))
                .unwrap_or_else(|| "manifest.json".to_string()),
        );
        if let Err(e) = std::fs::write(&manifest_path, &json) {
            eprintln!("failed to write manifest: {e}");
            return ExitCode::from(1);
        }
        println!("built {} ({} items)", path.display(), module.items.len());
        println!("  source_hash = {}", manifest.source_hash);
        println!("  ast_hash    = {}", manifest.ast_hash);
        println!("  manifest    = {}", manifest_path.display());
        if manifest.is_signed() {
            println!("  signed_by   = {}", manifest.signer_pubkey);
            println!("  signature   = {}", manifest.signature);
        }
    } else {
        println!("built {} ({} items)", path.display(), module.items.len());
        println!("  hint: pass --deterministic to emit a provenance manifest");
    }
    ExitCode::SUCCESS
}

/// `garnet keygen <keyfile>` — create an Ed25519 signing keypair; write the
/// hex-encoded 32-byte signing key to `<keyfile>`, print the corresponding
/// hex-encoded public key to stdout. The caller is responsible for
/// protecting the keyfile (e.g., `chmod 0600` on UNIX).
fn cmd_keygen(keyfile: PathBuf) -> ExitCode {
    let (signing_key, pubkey_hex) = garnet_cli::manifest::generate_signing_key();
    let key_hex = garnet_cli::manifest::signing_key_to_hex(&signing_key);
    // Write with a trailing newline — POSIX-friendly.
    let body = format!("{key_hex}\n");
    if let Err(e) = std::fs::write(&keyfile, body) {
        eprintln!("failed to write keyfile {}: {e}", keyfile.display());
        return ExitCode::from(1);
    }
    // Best-effort UNIX permission tightening. On Windows, this is a no-op —
    // caller should use an ACL or keep the file in a protected directory.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        if let Err(e) = std::fs::set_permissions(&keyfile, perms) {
            eprintln!("warning: could not chmod 0600 {}: {e}", keyfile.display());
        }
    }
    println!("generated Ed25519 signing keypair");
    println!("  keyfile = {} (keep private; chmod 0600)", keyfile.display());
    println!("  pubkey  = {pubkey_hex}");
    ExitCode::SUCCESS
}

/// `garnet test [<dir>]` — discover + run every function whose name starts
/// with `test_` in the project's `tests/*.garnet` files (and, optionally,
/// `src/main.garnet`). Mirrors the Cargo `cargo test` convention: a test
/// fails iff it raises a `RuntimeError::Raised(...)` exception, otherwise
/// passes. Reports a per-test pass/fail line + a summary; exits non-zero
/// if any test fails. Phase 6E (v4.2).
fn cmd_test(args: &[String]) -> ExitCode {
    use garnet_interp::{Interpreter, RuntimeError, Value};

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
            "--no-main" => { include_main = false; i += 1; }
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
        println!("garnet test: no .garnet files found under {}/tests/ or {}/src/main.garnet",
                 project_root.display(), project_root.display());
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
                    eprintln!("garnet test: failed to preload src/main.garnet for {}: {e}",
                              file.display());
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
        println!("test result: ok. {passed} passed; 0 failed; in {} file(s)", files.len());
        ExitCode::SUCCESS
    } else {
        println!("test result: FAILED. {passed} passed; {total_failed} failed");
        for n in &failed_names {
            println!("  - {n}");
        }
        ExitCode::from(1)
    }
}

/// `garnet convert <lang> <file>` — run the v4.1 converter. Writes
/// `<file>.garnet` + `.lineage.json` + `.migrate_todo.md` + `.metrics.json`
/// beside the source, or under `--out <dir>` if supplied.
fn cmd_convert(args: &[String]) -> ExitCode {
    use garnet_cli::convert_cmd::{self, ConvertArgs};

    // `new` subcommand pattern — accept flags in any order.
    let mut source_lang: Option<String> = None;
    let mut source_path: Option<String> = None;
    let mut out_dir: Option<String> = None;
    let mut strict = false;
    let mut fail_on_todo = false;
    let mut fail_on_untranslatable = false;
    let mut quiet = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--lang" => {
                if i + 1 >= args.len() {
                    eprintln!("--lang requires an argument (rust/ruby/python/go)");
                    return ExitCode::from(2);
                }
                source_lang = Some(args[i + 1].clone());
                i += 2;
            }
            "--out" => {
                if i + 1 >= args.len() {
                    eprintln!("--out requires a directory argument");
                    return ExitCode::from(2);
                }
                out_dir = Some(args[i + 1].clone());
                i += 2;
            }
            "--strict"                  => { strict = true; i += 1; }
            "--fail-on-todo"            => { fail_on_todo = true; i += 1; }
            "--fail-on-untranslatable"  => { fail_on_untranslatable = true; i += 1; }
            "--quiet"                   => { quiet = true; i += 1; }
            "--help" | "-h" => {
                println!("usage: garnet convert [--lang <lang>] [--out <dir>] [--strict] [--fail-on-todo] [--fail-on-untranslatable] [--quiet] <lang> <file>");
                println!("  langs: rust, ruby, python, go (also inferred from file extension)");
                return ExitCode::SUCCESS;
            }
            other if !other.starts_with("--") => {
                // Positional: first is lang (unless --lang flag already set), second is file.
                if source_lang.is_none() && source_path.is_none() {
                    source_lang = Some(args[i].clone());
                } else if source_path.is_none() {
                    source_path = Some(args[i].clone());
                } else {
                    eprintln!("unexpected extra positional argument: {other}");
                    return ExitCode::from(2);
                }
                i += 1;
            }
            other => {
                eprintln!("unknown convert flag: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let Some(file) = source_path else {
        eprintln!("usage: garnet convert <lang> <file>");
        return ExitCode::from(2);
    };
    let lang = source_lang.unwrap_or_default();

    let convert_args = ConvertArgs {
        source_lang: lang,
        source_path: PathBuf::from(&file),
        strict,
        fail_on_todo,
        fail_on_untranslatable,
        out_dir: out_dir.map(PathBuf::from),
        quiet,
    };

    match convert_cmd::run(convert_args) {
        Ok(outcome) => {
            if !quiet {
                println!("converted {}", file);
                println!("  source_lang   = {}", outcome_source_lang_label(&outcome));
                println!("  target        = {}", outcome.target_path.display());
                println!("  lineage       = {}", outcome.lineage_path.display());
                println!("  migrate_todo  = {}", outcome.migrate_todo_path.display());
                println!("  metrics       = {}", outcome.metrics_path.display());
                println!("  total_nodes   = {}", outcome.total_nodes);
                println!(
                    "  migrate_todo  = {} ({:.1}% clean-translate)",
                    outcome.migrate_todo_count, outcome.clean_percent,
                );
                println!("  untranslatable= {}", outcome.untranslatable_count);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("garnet convert failed: {e}");
            ExitCode::from(1)
        }
    }
}

/// Best-effort label recovered from the output target path (convert_cmd
/// doesn't carry the selected language on the outcome, but it's embedded
/// in the `lineage.json`; for the console summary we just surface "ok"
/// since the CLI already echoed the user's argument).
fn outcome_source_lang_label(_outcome: &garnet_cli::convert_cmd::ConvertOutcome) -> &'static str {
    "ok"
}

/// `garnet new --template <name> <dir>` — scaffold a new project from one
/// of the three bundled templates. Phase 6B (v4.2).
fn cmd_new(args: &[String]) -> ExitCode {
    // Accept `new --template <name> <dir>` or the shorter `new <dir>`
    // (defaults to `cli`). `-t` is accepted as a `--template` alias.
    let mut template: Option<String> = None;
    let mut dir_opt: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--template" | "-t" => {
                if i + 1 >= args.len() {
                    eprintln!("--template requires a template name");
                    return ExitCode::from(2);
                }
                template = Some(args[i + 1].clone());
                i += 2;
            }
            "--help" | "-h" => {
                println!("usage: garnet new [--template <name>] <dir>");
                println!("  templates:");
                for (key, desc) in garnet_cli::new_cmd::template_descriptions() {
                    println!("    {key:<20} {desc}");
                }
                return ExitCode::SUCCESS;
            }
            other if !other.starts_with("--") => {
                dir_opt = Some(args[i].clone());
                i += 1;
            }
            other => {
                eprintln!("unknown `new` flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(dir) = dir_opt else {
        eprintln!("usage: garnet new [--template <name>] <dir>");
        return ExitCode::from(2);
    };
    let template_key = template.unwrap_or_else(|| "cli".to_string());
    let target = PathBuf::from(dir);

    match garnet_cli::new_cmd::create_project(&template_key, &target) {
        Ok(report) => {
            // Colored-free output — Phase 6C adds terminal coloring via
            // `is-terminal` detection; the wordmark itself is emitted by
            // `print_help` / `print_version` and intentionally NOT repeated
            // per scaffolded project.
            println!(
                "Created `{}` from template `{}` ({} files)",
                report.root.display(),
                report.template,
                report.files_written.len(),
            );
            for f in &report.files_written {
                println!("  + {f}");
            }
            print!("{}", garnet_cli::new_cmd::next_steps_hint(&report));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("garnet new failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_verify(file: PathBuf, manifest_path: PathBuf, require_signature: bool) -> ExitCode {
    let src = match read_file(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let module = match garnet_parser::parse_source(&src) {
        Ok(m) => m,
        Err(e) => {
            let report = miette::Report::new(e).with_source_code(src);
            eprintln!("{report:?}");
            return ExitCode::from(1);
        }
    };
    let on_disk = match read_file(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let stored = match Manifest::from_canonical_json(&on_disk) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("manifest parse error: {e}");
            return ExitCode::from(1);
        }
    };
    let recomputed = Manifest::build(&src, &module);

    if stored.source_hash != recomputed.source_hash {
        eprintln!(
            "FAIL source_hash mismatch:\n  stored:     {}\n  recomputed: {}",
            stored.source_hash, recomputed.source_hash
        );
        return ExitCode::from(2);
    }
    if stored.ast_hash != recomputed.ast_hash {
        eprintln!(
            "FAIL ast_hash mismatch:\n  stored:     {}\n  recomputed: {}",
            stored.ast_hash, recomputed.ast_hash
        );
        return ExitCode::from(2);
    }
    if stored.schema != recomputed.schema {
        eprintln!(
            "FAIL schema mismatch: stored={}, expected={}",
            stored.schema, recomputed.schema
        );
        return ExitCode::from(2);
    }

    // v3.4.1 ManifestSig — signature verification.
    //
    // `--signature` flag forces a signed manifest + valid signature. Without
    // the flag, an unsigned manifest passes (backwards compat) but a signed
    // manifest whose signature is invalid is ALWAYS rejected — a tampered
    // signed manifest cannot quietly pass by dropping the flag.
    if stored.is_signed() {
        match stored.verify_signature() {
            Ok(()) => {
                println!("OK {} matches manifest + signature valid", file.display());
                println!("  source_hash = {}", stored.source_hash);
                println!("  ast_hash    = {}", stored.ast_hash);
                println!("  signed_by   = {}", stored.signer_pubkey);
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("FAIL signature verification: {e}");
                return ExitCode::from(2);
            }
        }
    } else if require_signature {
        eprintln!("FAIL manifest is unsigned but --signature was required");
        return ExitCode::from(2);
    }

    println!("OK {} matches manifest (unsigned)", file.display());
    println!("  source_hash = {}", stored.source_hash);
    println!("  ast_hash    = {}", stored.ast_hash);
    ExitCode::SUCCESS
}

fn describe_item(item: &garnet_parser::ast::Item) -> String {
    use garnet_parser::ast::Item;
    match item {
        Item::Fn(f) => format!("fn/def {} ({} args)", f.name, f.params.len()),
        Item::Struct(s) => format!("struct {}", s.name),
        Item::Enum(e) => format!("enum {}", e.name),
        Item::Trait(t) => format!("trait {}", t.name),
        Item::Impl(_) => "impl block".to_string(),
        Item::Actor(a) => format!("actor {} ({} items)", a.name, a.items.len()),
        Item::Memory(m) => format!("memory {:?} {}", m.kind, m.name),
        Item::Module(m) => format!("module {}", m.name),
        Item::Use(u) => format!("use {}", u.path.join("::")),
        Item::Const(c) => format!("const {}", c.name),
        Item::Let(l) => format!("let {}", l.name),
    }
}
