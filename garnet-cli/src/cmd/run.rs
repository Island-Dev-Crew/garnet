//! `garnet run <file>` — parse, load, and invoke `main` if it exists.
//!
//! ## S12 resolver pre-load (`--interp` only)
//!
//! When `<file>` lives inside a Garnet project (i.e. a `Garnet.toml`
//! exists in the file's parent chain), `garnet run --interp` reads the
//! project's `[dependencies]` table and pre-loads every `.garnet` source
//! under each declared vendor directory into the interpreter's global
//! environment **before** the user source is loaded. The interpreter's
//! `Item::Use(_)` handling stays a no-op; the vendored symbols are
//! already in scope by the time `use <dep>::*` is reached.
//!
//! Honest partials (S12):
//! - Pre-load is `--interp` only. The `--vm` path uses
//!   `run_source_with_options` from `garnet-vm` and has no shared env;
//!   S14 will harmonize this when the VM grows its own load_source.
//! - Qualified-path resolution (`local_lib::hello()`) is NOT in S12;
//!   only `use local_lib::*` plus an unqualified call is honored.
//! - Vendor parse / read errors are surfaced on stderr but do not abort
//!   the run; main may still resolve.
//! - Lockfile BLAKE3 hashes are NOT verified at run time (separate
//!   `garnet verify-deps` slice).
//! - Two deps declaring the same top-level symbol shadow last-wins.
//! - A vendored dep's own `main` is intentionally skipped during pre-load
//!   so it cannot shadow the user's entry point.

use super::{cache_file_label, record, surface_prior};
use crate::read_file;
use garnet_interp::Interpreter;
use garnet_vm::{compile_source, run_source_with_options, CompileSummary, RunOptions};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Interp,
    Vm,
}

pub fn run(args: &[String]) -> ExitCode {
    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    let RunArgs {
        mode,
        dump_lowering,
        path,
    } = parsed;
    let started = Instant::now();
    let src = match read_file(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
    };
    let file_label = cache_file_label(&path);
    surface_prior(&src);
    match mode {
        RunMode::Interp => {
            if dump_lowering {
                eprintln!("garnet run: --dump-lowering applies to --vm only; ignoring");
            }
            run_interpreter(&file_label, &src, &path, started)
        }
        RunMode::Vm => run_vm(&file_label, &src, dump_lowering, started),
    }
}

struct RunArgs {
    mode: RunMode,
    dump_lowering: bool,
    path: PathBuf,
}

fn parse_args(args: &[String]) -> Result<RunArgs, String> {
    const USAGE: &str = "usage: garnet run [--interp|--vm] [--dump-lowering] <file.garnet>";
    let mut mode = RunMode::Interp;
    let mut dump_lowering = false;
    let mut path: Option<PathBuf> = None;
    for arg in args {
        match arg.as_str() {
            "--interp" => mode = RunMode::Interp,
            "--vm" => mode = RunMode::Vm,
            "--dump-lowering" => dump_lowering = true,
            "--help" | "-h" => return Err(USAGE.to_string()),
            other if other.starts_with("--") => return Err(format!("unknown run flag: {other}")),
            other => {
                if path.is_some() {
                    return Err(USAGE.to_string());
                }
                path = Some(PathBuf::from(other));
            }
        }
    }
    path.map(|path| RunArgs {
        mode,
        dump_lowering,
        path,
    })
    .ok_or_else(|| USAGE.to_string())
}

/// Run the tree-walking interpreter for `path`. The evaluation runs on a thread
/// with a large explicit stack so deep (but finite) recursion does not overflow
/// the default OS thread stack — notably ~1 MiB on Windows, where the VM lane
/// succeeded but `--interp` aborted with a stack overflow (WIN-S73-001). Inputs
/// are owned-cloned across the boundary and the `Interpreter` is created *inside*
/// the thread, so nothing non-`Send` crosses it. (A truly *unbounded* recursion
/// still exhausts even this stack — that is the `@bounded` enforcement story, S89,
/// not a stack-size question.)
fn run_interpreter(file_label: &str, src: &str, path: &Path, started: Instant) -> ExitCode {
    // 256 MiB is virtual reservation (not committed until touched); it covers the
    // deep-but-finite recursion fixtures the VM ran while the default stack did not.
    const INTERP_STACK_BYTES: usize = 256 * 1024 * 1024;
    let file_label = file_label.to_string();
    let src = src.to_string();
    let path = path.to_path_buf();
    let code = std::thread::Builder::new()
        .name("garnet-interp".to_string())
        .stack_size(INTERP_STACK_BYTES)
        .spawn(move || run_interpreter_inner(&file_label, &src, &path, started))
        .expect("spawn garnet-interp thread")
        .join()
        .unwrap_or_else(|_| {
            eprintln!("runtime error: interpreter thread panicked");
            1
        });
    ExitCode::from(code)
}

/// The interpreter body. Returns a process exit code (`0` = success). Runs on the
/// large-stack `garnet-interp` thread spawned by [`run_interpreter`].
fn run_interpreter_inner(file_label: &str, src: &str, path: &Path, started: Instant) -> u8 {
    let edition = match crate::edition_manifest::resolve_edition_for(path) {
        Ok(resolved) => {
            if let Some(warning) = resolved.warning {
                eprintln!("{warning}");
            }
            resolved.edition
        }
        Err(message) => {
            eprintln!("{message}");
            record(
                "run",
                file_label,
                src,
                "parse_err",
                Some("bad_edition".to_string()),
                started,
                1,
            );
            return 1;
        }
    };
    let mut interp = Interpreter::new();
    if let Some(project_root) = find_project_root_for(path) {
        preload_dependencies(&mut interp, &project_root);
    }
    if let Err(e) = interp.load_source_with_edition(src, edition) {
        eprintln!("load error: {e}");
        record(
            "run",
            file_label,
            src,
            "parse_err",
            Some(format!("{e}")),
            started,
            1,
        );
        return 1;
    }
    // If a `main` function exists, call it; otherwise just exit success.
    if interp.global.get("main").is_some() {
        match interp.call_entry("main", vec![]) {
            Ok(v) => {
                println!("=> {}", v.display());
                record("run", file_label, src, "ok", None, started, 0);
                0
            }
            Err(e) => {
                eprintln!("runtime error: {e}");
                record(
                    "run",
                    file_label,
                    src,
                    "runtime_err",
                    Some(format!("{e}")),
                    started,
                    1,
                );
                1
            }
        }
    } else {
        record("run", file_label, src, "ok", None, started, 0);
        0
    }
}

/// Walk upward from `file`'s parent looking for `Garnet.toml`. Returns
/// `None` for bare-file runs outside any project so `garnet run /tmp/...`
/// keeps working.
fn find_project_root_for(file: &Path) -> Option<PathBuf> {
    let mut cur = file
        .canonicalize()
        .ok()
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .or_else(|| file.parent().map(Path::to_path_buf))?;
    loop {
        if cur.join("Garnet.toml").exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// S12 pre-loader: read `[dependencies]` from `Garnet.toml`, walk each
/// declared vendor directory, and load every `.garnet` source into the
/// interpreter's global environment before the user source. Errors are
/// surfaced on stderr but never abort the run; the user's `main` may
/// still resolve, and a noisy dep should not stop a working program.
fn preload_dependencies(interp: &mut Interpreter, project_root: &Path) {
    let deps = match crate::cmd::add::read_dependency_table(project_root) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("garnet run: could not read Garnet.toml: {e}");
            return;
        }
    };
    for dep in deps {
        let vendor_root = project_root.join(&dep.vendor_rel);
        if !vendor_root.exists() {
            eprintln!(
                "garnet run: dep {}: vendor path {} not found; skipping",
                dep.name,
                vendor_root.display()
            );
            continue;
        }
        let mut files: Vec<PathBuf> = Vec::new();
        if let Err(e) = collect_garnet_files(&vendor_root, &mut files) {
            eprintln!(
                "garnet run: dep {}: could not walk vendor {}: {e}",
                dep.name,
                vendor_root.display()
            );
            continue;
        }
        files.sort();
        for file in files {
            let src = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "garnet run: dep {}: could not read {}: {e}",
                        dep.name,
                        file.display()
                    );
                    continue;
                }
            };
            // Strip a top-level `def main` so a vendored library does not
            // shadow the user's entry point. Crude but predictable; a
            // future slice can do this in the AST.
            let safe_src = strip_top_level_main(&src);
            if let Err(e) = interp.load_source(&safe_src) {
                eprintln!(
                    "garnet run: dep {}: parse error in {}: {e}",
                    dep.name,
                    file.display()
                );
            }
        }
    }
}

fn collect_garnet_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_garnet_files(&path, out)?;
        } else if file_type.is_file() && path.extension().and_then(|s| s.to_str()) == Some("garnet")
        {
            out.push(path);
        }
    }
    Ok(())
}

/// Drop a `def main(...) { ... }` (or `@caps()\ndef main(...) { ... }`)
/// from the top level of `src` so a vendored dep cannot shadow the user
/// program's entry point. Crude line-and-brace scanner — full AST-based
/// stripping is a future slice.
fn strip_top_level_main(src: &str) -> String {
    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    while i < len {
        let rest = &src[i..];
        let trimmed_idx = rest
            .find(|c: char| !c.is_whitespace())
            .map(|p| i + p)
            .unwrap_or(len);
        // Optional `@caps(...)` annotation on the same or preceding line.
        let start = trimmed_idx;
        if src[start..].starts_with("@caps") {
            // Skip until newline after `@caps(...)`.
            if let Some(eol) = src[start..].find('\n') {
                let next = start + eol + 1;
                // Look at next non-whitespace token.
                let after = src[next..]
                    .find(|c: char| !c.is_whitespace())
                    .map(|p| next + p)
                    .unwrap_or(len);
                if starts_with_def_main(&src[after..]) {
                    // Skip the `def main ... { ... }` block by brace count.
                    let drop_end = end_of_def_block(src, after);
                    out.push_str(&src[i..start]);
                    i = drop_end;
                    continue;
                }
            }
            // `@caps` did not introduce a `def main` — fall through to copy.
            out.push_str(&src[i..start + 1]);
            i = start + 1;
            continue;
        }
        if starts_with_def_main(&src[start..]) {
            let drop_end = end_of_def_block(src, start);
            out.push_str(&src[i..start]);
            i = drop_end;
            continue;
        }
        // Copy whatever we found and advance to the next character.
        // Defensive: ensure forward progress on degenerate inputs.
        let next = if start > i { start } else { i + 1 };
        out.push_str(&src[i..next.min(len)]);
        i = next.min(len);
    }
    out
}

/// Word-boundary check: `src` starts with the literal token `def main`
/// followed by `(`, space, tab, or end-of-input. This avoids matching
/// identifiers like `def main_helper` or `def maintenance`.
fn starts_with_def_main(src: &str) -> bool {
    const NEEDLE: &str = "def main";
    if !src.starts_with(NEEDLE) {
        return false;
    }
    match src.as_bytes().get(NEEDLE.len()) {
        None => true,
        Some(c) => matches!(*c, b'(' | b' ' | b'\t' | b'\n' | b'\r'),
    }
}

fn end_of_def_block(src: &str, start: usize) -> usize {
    // Find the first `{`, then match braces.
    let bytes = src.as_bytes();
    let mut i = start;
    let len = bytes.len();
    while i < len && bytes[i] != b'{' {
        i += 1;
    }
    if i >= len {
        return len;
    }
    let mut depth: i32 = 0;
    while i < len {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    len
}

/// Print the S14 lowering summary for `garnet run --vm --dump-lowering`.
/// The `lowered: N%` line is the headline the bytecode-VM dogfood greps.
fn print_lowering_summary(summary: &CompileSummary) {
    let total = summary.native_functions + summary.fallback_functions;
    // 100% when there are no functions; checked_div sidesteps divide-by-zero.
    let pct = (summary.native_functions * 100)
        .checked_div(total)
        .unwrap_or(100);
    println!(
        "lowering: {} native / {} fallback functions ({} native instructions)",
        summary.native_functions, summary.fallback_functions, summary.native_instruction_count
    );
    println!("lowered: {pct}%");
    for reason in &summary.fallback_reasons {
        println!("  fallback {reason}");
    }
}

fn run_vm(file_label: &str, src: &str, dump_lowering: bool, started: Instant) -> ExitCode {
    if dump_lowering {
        match compile_source(src) {
            Ok(artifact) => print_lowering_summary(&artifact.summary),
            Err(error) => {
                eprintln!("vm error: {error}");
                record(
                    "run",
                    file_label,
                    src,
                    "compile_err",
                    Some(format!("{error}")),
                    started,
                    1,
                );
                return ExitCode::from(1);
            }
        }
    }
    match run_source_with_options(src, RunOptions { emit_stdout: true }) {
        Ok(result) => {
            if result.called_entry {
                println!("=> {}", result.value.display());
            }
            record("run", file_label, src, "ok", None, started, 0);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("vm error: {error}");
            record(
                "run",
                file_label,
                src,
                "runtime_err",
                Some(format!("{error}")),
                started,
                1,
            );
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::strip_top_level_main;

    #[test]
    fn strips_plain_def_main() {
        let src = "def main() { 42 }\ndef hello() { \"hi\" }\n";
        let out = strip_top_level_main(src);
        assert!(!out.contains("def main"));
        assert!(out.contains("def hello"));
    }

    #[test]
    fn strips_caps_decorated_main() {
        let src = "@caps()\ndef main() {\n  println(\"hi\")\n}\ndef helper() { 1 }\n";
        let out = strip_top_level_main(src);
        assert!(!out.contains("def main"));
        assert!(!out.contains("@caps"));
        assert!(out.contains("def helper"));
    }

    #[test]
    fn preserves_main_in_unrelated_identifier() {
        // `def main_helper` is a different name; must NOT be stripped.
        let src = "def main_helper() { 1 }\n";
        let out = strip_top_level_main(src);
        assert!(out.contains("def main_helper"));
    }

    #[test]
    fn passes_through_source_with_no_main() {
        let src = "def hello() { \"hi\" }\ndef goodbye() { \"bye\" }\n";
        let out = strip_top_level_main(src);
        assert_eq!(out, src);
    }
}
