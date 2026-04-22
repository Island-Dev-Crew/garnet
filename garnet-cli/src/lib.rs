//! Shared helpers between the `garnet` binary and potential future binaries
//! (e.g. a future `garnet-lsp`).

pub mod audit_deps;
pub mod cache;
pub mod convert_cmd;
pub mod knowledge;
pub mod machine_key;
pub mod manifest;
pub mod new_cmd;
pub mod provenance;
pub mod strategies;

use std::fs;
use std::io::IsTerminal;
use std::path::Path;

/// Small ASCII-art wordmark shown by `--version` and by `garnet new`
/// project-creation success messages. Deliberately compact (7 lines) so it
/// fits a 24×80 terminal without scrolling.
pub const GARNET_WORDMARK: &str = concat!(
    "                                                  \n",
    "   ####   ###  ####  #   # ####### ##### ######   \n",
    "  #    # #   # #   # ##  # #         #     #      \n",
    "  #      ##### ####  # # # #####     #     #      \n",
    "  #  ### #   # #  #  #  ## #         #     #      \n",
    "  #    # #   # #   # #   # #         #     #      \n",
    "   ####  #   # #   # #   # #######   #     #      \n",
);

/// ANSI truecolor sequence for the Garnet accent color (#9C2B2E). Used by
/// `colored_wordmark` when stdout is a TTY. Falls back to plain ASCII when
/// output is piped, redirected, or captured by CI — so `garnet --version >
/// file` never embeds escape sequences in the file.
const ANSI_GARNET: &str = "\x1b[38;2;156;43;46m";
const ANSI_RESET:  &str = "\x1b[0m";

/// Return the wordmark, wrapped in the Garnet accent color if `is_tty` is
/// true (typically `io::stdout().is_terminal()` at the call site). Falls
/// back to plain ASCII otherwise — deterministic output for pipes / CI.
pub fn colored_wordmark(is_tty: bool) -> String {
    if is_tty {
        format!("{ANSI_GARNET}{GARNET_WORDMARK}{ANSI_RESET}")
    } else {
        GARNET_WORDMARK.to_string()
    }
}

/// Convenience: colored-or-not wordmark based on whether stdout is a TTY.
pub fn wordmark_for_stdout() -> String {
    colored_wordmark(std::io::stdout().is_terminal())
}

pub fn read_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("failed to read {:?}: {e}", path))
}

/// Print the v4.2 `--version` banner — ASCII wordmark + component
/// versions + the Rung identification for each crate. Phase 6C.
///
/// The wordmark is tinted in the Garnet accent color when stdout is a
/// real terminal; plain ASCII when piped or redirected (so CI logs stay
/// escape-free).
pub fn print_version() {
    print!("{}", wordmark_for_stdout());
    println!("  Rust Rigor. Ruby Velocity. One Coherent Language.");
    println!();
    println!("garnet 0.4.2 ({})", env!("CARGO_PKG_DESCRIPTION"));
    println!("  parser    garnet-parser 0.3.0 (Mini-Spec v1.0)");
    println!("  interp    garnet-interp 0.3.0 (tree-walk, Rung 3)");
    println!("  check     garnet-check  0.3.0 (safe-mode + borrow + CapCaps v3.4.1, Rung 4)");
    println!("  memory    garnet-memory 0.3.0 (reference stores, Rung 5)");
    println!("  actor-rt  garnet-actor-runtime 0.3.1 (hot-reloadable + signed reload, Rung 6)");
    println!("  stdlib    garnet-stdlib 0.4.0 (22 bridged primitives)");
    println!("  convert   garnet-convert 0.4.0 (Rust / Ruby / Python / Go → Garnet)");
}

pub fn print_help() {
    print!("{}", wordmark_for_stdout());
    println!("  Rust Rigor. Ruby Velocity. One Coherent Language.");
    println!();
    println!("USAGE:");
    println!("    garnet <SUBCOMMAND> [ARGS]\n");
    println!("SUBCOMMANDS:");
    println!("    new    --template <T> <dir>      Scaffold a new project (T=cli|web-api|agent-orchestrator)");
    println!("    parse  <file.garnet>             Parse a file and print a structural summary");
    println!("    check  <file.garnet>             Run the safe-mode checker (incl. CapCaps propagator)");
    println!("    run    <file.garnet>             Parse, load, and invoke `main` if it exists");
    println!("    test   [<dir>]                   Discover + run test_* functions in tests/*.garnet");
    println!("    eval   \"<expr>\"                  Evaluate a single expression");
    println!("    repl   [file.garnet]             Interactive REPL (optionally preloading a file)");
    println!("    build  [--deterministic] [--sign <key>] <file>");
    println!("                                     Emit a (deterministic, optionally signed) manifest");
    println!("    verify <file> <manifest.json>    Verify the manifest matches the source");
    println!("           [--signature]             Require a valid Ed25519 signature");
    println!("    keygen <keyfile>                 Generate an Ed25519 signing keypair");
    println!("    convert <lang> <file>            Convert Rust/Ruby/Python/Go source to Garnet");
    println!("    version                          Print toolchain versions + wordmark");
    println!("    help                             This message");
}
