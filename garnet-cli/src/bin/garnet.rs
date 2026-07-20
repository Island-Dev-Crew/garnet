//! The `garnet` binary — the user-facing CLI.
//!
//! A deliberately tiny command dispatcher. All real logic lives in the
//! `garnet_cli::cmd::*` submodules (one per subcommand) and the
//! supporting crates (parser, interp, check, memory). Adding a
//! subcommand is two steps: create `garnet-cli/src/cmd/<name>.rs` with
//! a `pub fn run(...)`, then add an arm to the `match` in `main()`
//! below.

// RB-2 crash-surface sweep: user-facing crates must not unwrap/expect on
// reachable paths. Sanctioned escapes are in-line `// INVARIANT:` allows
// (provably-cannot-fail) and the one documented `// FAIL-CLOSED:` abort
// (machine_key). Test code is exempt via the cfg_attr below.
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use garnet_cli::cmd;
use garnet_cli::{print_help, print_version};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    // S114-FIX-2: deny-by-default capability mediation for the binary. A
    // host-authority primitive reached with no active `@caps` frame is REFUSED
    // (complete mediation / fail-safe default) on every lane — `run`, `eval`,
    // `test`, `doctest`, `repl`, and dependency preload alike — closing the
    // fail-open lanes the load-time-frame fix (4994867) did not wire. Library
    // and embedder callers using `Interpreter::new()`'s high-level methods are
    // also strict by default; `new_permissive()` is their explicit legacy
    // opt-out, and this process-global latch still dominates that opt-out
    // inside the CLI.
    garnet_interp::eval::set_strict_no_frame(true);

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
        "new" => cmd::new::run(&args[1..]),
        "add" => cmd::add::run(&args[1..]),
        "agent-loop" => cmd::agent_loop::run(&args[1..]),
        "parse" => cmd::parse::run(&args[1..]),
        "check" => {
            let mut suggest = false;
            let mut format = cmd::check::CheckFormat::Human;
            let mut file_arg: Option<&String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--suggest" => {
                        suggest = true;
                        i += 1;
                    }
                    "--format" => {
                        match args.get(i + 1).map(String::as_str) {
                            Some("human") => format = cmd::check::CheckFormat::Human,
                            Some("json") => format = cmd::check::CheckFormat::Json,
                            _ => {
                                eprintln!("--format requires 'human' or 'json'");
                                return ExitCode::from(2);
                            }
                        }
                        i += 2;
                    }
                    arg if !arg.starts_with("--") && file_arg.is_none() => {
                        file_arg = Some(&args[i]);
                        i += 1;
                    }
                    other => {
                        eprintln!("garnet check: unexpected argument: {other}");
                        return ExitCode::from(2);
                    }
                }
            }
            let Some(file) = file_arg else {
                eprintln!("usage: garnet check [--suggest] [--format human|json] <file.garnet>");
                return ExitCode::from(2);
            };
            cmd::check::run(PathBuf::from(file), suggest, format)
        }
        "caps" => {
            let mut standard_profile = false;
            let mut path_arg: Option<&String> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--standard-profile" => {
                        standard_profile = true;
                        i += 1;
                    }
                    arg if !arg.starts_with("--") && path_arg.is_none() => {
                        path_arg = Some(&args[i]);
                        i += 1;
                    }
                    other => {
                        eprintln!("garnet caps: unexpected argument: {other}");
                        return ExitCode::from(2);
                    }
                }
            }
            let Some(path) = path_arg else {
                eprintln!("usage: garnet caps [--standard-profile] <path>");
                return ExitCode::from(2);
            };
            let format = if standard_profile {
                cmd::caps::CapsFormat::StandardProfile
            } else {
                cmd::caps::CapsFormat::Internal
            };
            cmd::caps::run(PathBuf::from(path), format)
        }
        "bounds" => {
            if args.len() < 2 {
                eprintln!("usage: garnet bounds <file.garnet>");
                return ExitCode::from(2);
            }
            cmd::bounds::run(PathBuf::from(&args[1]))
        }
        "ceilings" => {
            if args.len() < 2 {
                eprintln!("usage: garnet ceilings <file.garnet>");
                return ExitCode::from(2);
            }
            cmd::ceilings::run(PathBuf::from(&args[1]))
        }
        "concurrency" => {
            if args.len() < 2 {
                eprintln!("usage: garnet concurrency <file.garnet>");
                return ExitCode::from(2);
            }
            cmd::concurrency::run(PathBuf::from(&args[1]))
        }
        "run" => {
            if args.len() < 2 {
                eprintln!("usage: garnet run [--interp|--vm] <file.garnet>");
                return ExitCode::from(2);
            }
            cmd::run::run(&args[1..])
        }
        "eval" => {
            if args.len() < 2 {
                eprintln!("usage: garnet eval \"<expr>\"");
                return ExitCode::from(2);
            }
            cmd::eval::run(&args[1])
        }
        "repl" => {
            let preload = if args.len() >= 2 {
                Some(PathBuf::from(&args[1]))
            } else {
                None
            };
            cmd::repl::run(preload)
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
            cmd::build::run(PathBuf::from(file), deterministic, sign_keyfile)
        }
        "keygen" => {
            // `garnet keygen <keyfile>` — create a fresh Ed25519 signing key
            // and write it to `<keyfile>`. Prints the public key to stdout
            // so the caller can record it as the expected signer.
            if args.len() < 2 {
                eprintln!("usage: garnet keygen <keyfile>");
                return ExitCode::from(2);
            }
            cmd::keygen::run(PathBuf::from(&args[1]))
        }
        "verify" => {
            // Routed by positional-arg count:
            //   garnet verify <path>                  -> S33 acceptance gate
            //   garnet verify <file> <manifest.json>  -> deterministic-manifest verify
            let mut positionals: Vec<String> = Vec::new();
            let mut require_sig = false;
            let mut external_band: Option<u8> = None;
            let mut caps_baseline: Option<PathBuf> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--signature" => {
                        require_sig = true;
                        i += 1;
                    }
                    "--external-band" => {
                        match args.get(i + 1).and_then(|v| v.parse::<u8>().ok()) {
                            Some(n) if (1..=5).contains(&n) => external_band = Some(n),
                            _ => {
                                eprintln!("--external-band requires an integer 1-5");
                                return ExitCode::from(2);
                            }
                        }
                        i += 2;
                    }
                    "--caps-baseline" => {
                        match args.get(i + 1) {
                            Some(p) => caps_baseline = Some(PathBuf::from(p)),
                            None => {
                                eprintln!("--caps-baseline requires a path");
                                return ExitCode::from(2);
                            }
                        }
                        i += 2;
                    }
                    other if other.starts_with("--") => {
                        eprintln!("unknown verify flag: {other}");
                        return ExitCode::from(2);
                    }
                    _ => {
                        positionals.push(args[i].clone());
                        i += 1;
                    }
                }
            }
            match positionals.len() {
                1 => cmd::verify_gate::run(cmd::verify_gate::GateArgs {
                    path: PathBuf::from(&positionals[0]),
                    external_band,
                    caps_baseline,
                }),
                2 => cmd::verify::run(
                    PathBuf::from(&positionals[0]),
                    PathBuf::from(&positionals[1]),
                    require_sig,
                ),
                _ => {
                    eprintln!(
                        "usage: garnet verify <path>                          (acceptance gate)"
                    );
                    eprintln!("       garnet verify <file> <manifest.json> [--signature]  (manifest verify)");
                    ExitCode::from(2)
                }
            }
        }
        "diff-caps" => {
            let mut machine = false;
            let mut positionals: Vec<&String> = Vec::new();
            for arg in args.iter().skip(1) {
                if arg == "--machine" {
                    machine = true;
                } else if arg.starts_with("--") {
                    eprintln!("unknown diff-caps flag: {arg}");
                    return ExitCode::from(2);
                } else {
                    positionals.push(arg);
                }
            }
            if positionals.len() < 2 {
                eprintln!("usage: garnet diff-caps [--machine] <old-path> <new-path>");
                return ExitCode::from(2);
            }
            cmd::diff_caps::run(
                PathBuf::from(positionals[0]),
                PathBuf::from(positionals[1]),
                machine,
            )
        }
        "seal" => cmd::seal::run(&args[1..]),
        "trust-report" => {
            if args.len() < 2 {
                eprintln!("usage: garnet trust-report <file.garnet>");
                return ExitCode::from(2);
            }
            cmd::trust_report::run(PathBuf::from(&args[1]))
        }
        "convert" => cmd::convert::run(&args[1..]),
        "test" => cmd::test::run(&args[1..]),
        "fmt" => cmd::fmt::run(&args[1..]),
        "doc" => cmd::doc::run(&args[1..]),
        "doctest" => cmd::doctest::run(&args[1..]),
        "sandbox" => cmd::sandbox::run(&args[1..]),
        "mcp-caps" => cmd::mcp_caps::run(&args[1..]),
        "mcp-serve" => cmd::mcp_serve::run(&args[1..]),
        "caps-log" => cmd::caps_log::run(&args[1..]),
        other => {
            eprintln!("unknown subcommand: {other}");
            print_help();
            ExitCode::from(2)
        }
    }
}
