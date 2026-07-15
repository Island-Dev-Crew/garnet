//! Shared helpers between the `garnet` binary and potential future binaries
//! (e.g. a future `garnet-lsp`).

// RB-2 crash-surface sweep: user-facing crates must not unwrap/expect on
// reachable paths. Sanctioned escapes are in-line `// INVARIANT:` allows
// (provably-cannot-fail) and the one documented `// FAIL-CLOSED:` abort
// (machine_key). Test code is exempt via the cfg_attr below.
#![deny(clippy::unwrap_used, clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod audit_deps;
pub mod cache;
pub mod cap_manifest;
pub mod cmd;
pub mod convert_cmd;
pub mod diagnostics;
pub mod doctest;
pub mod edition_manifest;
pub mod knowledge;
pub mod machine_key;
pub mod manifest;
pub mod mcp;
pub mod mcp_schema;
pub mod new_cmd;
pub mod panic_firewall;
pub mod provenance;
pub mod runtime_settings;
pub mod sandbox;
pub mod seal;
pub mod strategies;
pub mod verify_gate;

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
const ANSI_RESET: &str = "\x1b[0m";

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

/// Print the public `--version` banner — ASCII wordmark + component
/// versions + the Rung identification for each crate.
///
/// The wordmark is tinted in the Garnet accent color when stdout is a
/// real terminal; plain ASCII when piped or redirected (so CI logs stay
/// escape-free).
pub fn print_version() {
    print!("{}", wordmark_for_stdout());
    println!("  Rust Rigor. Ruby Velocity. One Coherent Language.");
    println!();
    println!(
        "garnet {} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION")
    );
    println!("  parser    garnet-parser 0.3.0 (Mini-Spec v1.0)");
    println!("  interp    garnet-interp 0.3.0 (tree-walk, Rung 3)");
    println!(
        "  vm        garnet-vm     0.5.0 (bytecode VM; @max_depth + @caps trap-parity with interp, S99–S101)"
    );
    println!("  check     garnet-check  0.3.0 (safe-mode + borrow + CapCaps v3.4.1, Rung 4)");
    println!(
        "  memory    garnet-memory 0.3.0 (Mnemos — Memory Core reference impl; production roadmap in MEMORY_CORE_ROADMAP.md, Rung 5)"
    );
    println!(
        "  actor-rt  garnet-actor-runtime 0.4.0 (Result-returning ask, hot-reloadable + signed reload, Rung 6)"
    );
    println!(
        "  stdlib    garnet-stdlib 0.4.0 ({} registry primitives, dispatch derived from the registry)",
        garnet_stdlib::registry::all_prims().len()
    );
    println!(
        "  convert   garnet-convert 0.4.0 (migration assistant: Rust / Ruby / Python / Go → Garnet)"
    );
}

pub fn print_help() {
    print!("{}", wordmark_for_stdout());
    println!("  Rust Rigor. Ruby Velocity. One Coherent Language.");
    println!();
    println!("USAGE:");
    println!("    garnet <SUBCOMMAND> [ARGS]\n");
    println!("SUBCOMMANDS:");
    println!("    new    --template <T> <dir>      Scaffold a new project (T=cli|web-api|agent-orchestrator)");
    println!(
        "    add    <path> [--name <id>]      Vendor a local Garnet dir into .garnet/vendor (not a registry)"
    );
    println!("    parse  [--mode ast|cst] <file>   Parse a file and print a structural summary");
    println!("    check  [--suggest] [--format human|json] <file.garnet>");
    println!(
        "                                     Safe-mode checker (CapCaps); --format json emits structured diagnostics"
    );
    println!(
        "    caps   [--standard-profile] <path> Emit the capability manifest or S98 draft standard profile"
    );
    println!(
        "    diff-caps [--machine] <old> <new>  Diff the capability surface; nonzero exit if authority expanded"
    );
    println!(
        "                                     (--machine: deterministic single-line JSON verdict for agent reviewers)"
    );
    println!(
        "    seal   <file.garnet>             Emit an in-toto seal attestation (cosign-signable; SBOM-equivalent)"
    );
    println!(
        "    agent-loop --baseline <o> --proposal <n>  Accept agent-authored code ONLY on diff-caps + enforced-kernel evidence, then seal (S102)"
    );
    println!(
        "    bounds <file.garnet>             Report declared @bounded(N) fuel budgets (Wasmtime-fuel target)"
    );
    println!(
        "    ceilings <file.garnet>           Identify explosive ops (loop/spawn) + default-ceiling policy"
    );
    println!(
        "    concurrency <file.garnet>        Report the actor concurrency contract (ask/tell protocols)"
    );
    println!(
        "    trust-report <file.garnet>       Structural trust report (actor/thread count + capability surface)"
    );
    println!("    run    <file.garnet>             Parse, load, and invoke `main` if it exists");
    println!(
        "    test   [<dir>]                   Discover + run test_* functions in tests/*.garnet"
    );
    println!("    eval   \"<expr>\"                  Evaluate a single expression");
    println!(
        "    repl   [file.garnet]             Interactive REPL (optionally preloading a file)"
    );
    println!("    build  [--deterministic] [--sign <key>] <file>");
    println!(
        "                                     Emit a (deterministic, optionally signed) manifest"
    );
    println!(
        "    verify <path>                    Acceptance gate: edition-aware parse + safe-mode"
    );
    println!(
        "           [--external-band <1-5>]   check; emits a fused merge-confidence band (min of"
    );
    println!("           [--caps-baseline <old>]   internal/external/diff-caps; nonzero on fatal)");
    println!("    verify <file> <manifest.json>    Verify the manifest matches the source");
    println!("           [--signature]             Require a valid Ed25519 signature");
    println!("    keygen <keyfile>                 Generate an Ed25519 signing keypair");
    println!(
        "    convert <lang> <file>            Migration assistant — lift Rust/Ruby/Python/Go source"
    );
    println!(
        "                                     into Garnet (sandbox-on; emits MigrateTodo checklist)"
    );
    println!("    fmt    [--check|--stdout] <file> Whitespace-normalize a Garnet source file");
    println!("    doc    [--stdout|--out P] <file> Extract /// doc comments to a markdown summary");
    println!("    doctest [--format human|json] <file> Run ```garnet examples in /// doc comments");
    println!("    sandbox [--format human|json] <file> seccomp/WASI/egress policy from @caps");
    println!("    mcp-caps [--format human|json] <file> Capability surface of an MCP tool-set");
    println!(
        "    caps-log <file> [--log P] | --verify <log> Append-only capability transparency log"
    );
    println!("    version                          Print toolchain versions + wordmark");
    println!("    help                             This message");
}
