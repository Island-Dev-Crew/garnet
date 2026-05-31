//! The `garnet` binary — the user-facing CLI.
//!
//! A deliberately tiny command dispatcher. All real logic lives in the
//! `garnet_cli::cmd::*` submodules (one per subcommand) and the
//! supporting crates (parser, interp, check, memory). Adding a
//! subcommand is two steps: create `garnet-cli/src/cmd/<name>.rs` with
//! a `pub fn run(...)`, then add an arm to the `match` in `main()`
//! below.

use garnet_cli::cmd;
use garnet_cli::{print_help, print_version};
use std::path::PathBuf;
use std::process::ExitCode;

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
        "new" => cmd::new::run(&args[1..]),
        "add" => cmd::add::run(&args[1..]),
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
            if args.len() < 2 {
                eprintln!("usage: garnet caps <path>");
                return ExitCode::from(2);
            }
            cmd::caps::run(PathBuf::from(&args[1]))
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
            if args.len() < 3 {
                eprintln!("usage: garnet diff-caps <old-path> <new-path>");
                return ExitCode::from(2);
            }
            cmd::diff_caps::run(PathBuf::from(&args[1]), PathBuf::from(&args[2]))
        }
        "seal" => {
            if args.len() < 2 {
                eprintln!("usage: garnet seal <file.garnet>");
                return ExitCode::from(2);
            }
            cmd::seal::run(PathBuf::from(&args[1]))
        }
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
        other => {
            eprintln!("unknown subcommand: {other}");
            print_help();
            ExitCode::from(2)
        }
    }
}
