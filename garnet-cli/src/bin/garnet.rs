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
        "parse" => {
            if args.len() < 2 {
                eprintln!("usage: garnet parse <file.garnet>");
                return ExitCode::from(2);
            }
            cmd::parse::run(PathBuf::from(&args[1]))
        }
        "check" => {
            if args.len() < 2 {
                eprintln!("usage: garnet check <file.garnet>");
                return ExitCode::from(2);
            }
            cmd::check::run(PathBuf::from(&args[1]))
        }
        "run" => {
            if args.len() < 2 {
                eprintln!("usage: garnet run <file.garnet>");
                return ExitCode::from(2);
            }
            cmd::run::run(PathBuf::from(&args[1]))
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
            if args.len() < 3 {
                eprintln!("usage: garnet verify <file.garnet> <manifest.json> [--signature]");
                return ExitCode::from(2);
            }
            let require_sig = args.iter().any(|a| a == "--signature");
            cmd::verify::run(
                PathBuf::from(&args[1]),
                PathBuf::from(&args[2]),
                require_sig,
            )
        }
        "convert" => cmd::convert::run(&args[1..]),
        "test" => cmd::test::run(&args[1..]),
        other => {
            eprintln!("unknown subcommand: {other}");
            print_help();
            ExitCode::from(2)
        }
    }
}
