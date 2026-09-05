//! `garnet verify <path>` — the S33 acceptance gate.
//!
//! Runs edition-aware parse + safe-mode check over the target(s), emits a fused
//! merge-confidence band, and exits non-zero iff any target fails fatally.
//! (Distinct from `garnet verify <file> <manifest.json>`, the 2-arg
//! deterministic-manifest verify in `cmd/verify.rs`; the dispatcher routes on
//! positional-arg count.)

use crate::verify_gate::{fuse, Band, CapabilitySignal, GateTally};
use crate::{edition_manifest, read_file};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Parsed arguments for the acceptance gate.
pub struct GateArgs {
    pub path: PathBuf,
    /// Optional external-reviewer band (1..=5); in CI/PR this is Greptile.
    pub external_band: Option<u8>,
    /// Optional baseline path (an older revision of the same tree) for the S37
    /// capability signal: `diff-caps(baseline, current)` feeds the fuse.
    pub caps_baseline: Option<PathBuf>,
}

pub fn run(args: GateArgs) -> ExitCode {
    let tally = match gate_tally(&args.path) {
        Ok(tally) => tally,
        Err(message) => {
            eprintln!("garnet verify: {message}");
            return ExitCode::from(2);
        }
    };

    let internal = tally.internal_band();
    let external = args.external_band.map(Band::new);
    let capability = resolve_capability_signal(&args);
    let fused = fuse(internal, external, capability);

    println!();
    println!(
        "Verified {} target(s): {} failing, {} advisory diagnostic(s).",
        tally.targets, tally.failing, tally.advisories
    );
    println!("Merge confidence (fused): {}/5", fused.get());
    println!(
        "  internal (local parse + safe-mode check): {}/5",
        internal.get()
    );
    match external {
        Some(b) => println!("  external reviewer: {}/5", b.get()),
        None => println!("  external reviewer: not supplied (Greptile wires in at PR time)"),
    }
    match capability {
        CapabilitySignal::Surface(b) => {
            println!("  capability signal (diff-caps vs baseline): {}/5", b.get())
        }
        CapabilitySignal::Pending => {
            println!("  capability signal: pending (pass --caps-baseline <old> for diff-caps)")
        }
    }
    println!("  fusion rule: min of the present signals");

    if tally.passes() {
        println!("\ngate: PASS");
        ExitCode::SUCCESS
    } else {
        println!(
            "\ngate: FAIL ({} target(s) with fatal diagnostics)",
            tally.failing
        );
        ExitCode::from(1)
    }
}

/// Compute the S37 capability signal. With a `--caps-baseline`, run
/// `diff-caps(baseline, current)` and map an authority change to a band;
/// otherwise the slot stays pending (back-compat with S33).
fn resolve_capability_signal(args: &GateArgs) -> CapabilitySignal {
    let Some(baseline) = &args.caps_baseline else {
        return CapabilitySignal::Pending;
    };
    match (
        crate::cap_manifest::surface_for_path(baseline),
        crate::cap_manifest::surface_for_path(&args.path),
    ) {
        (Ok(base), Ok(current)) => {
            let diff = garnet_check::diff_caps(&base, &current);
            CapabilitySignal::Surface(capability_band(&diff))
        }
        _ => {
            eprintln!(
                "garnet verify: could not build capability surfaces for --caps-baseline; \
                 capability signal left pending"
            );
            CapabilitySignal::Pending
        }
    }
}

/// Map a capability diff to the capability signal band: `5` when the program did
/// not gain authority, `2` when it did (so the fused `min` flags the change for
/// review).
pub fn capability_band(diff: &garnet_check::CapsDiff) -> Band {
    if diff.authority_expanded() {
        Band::new(2)
    } else {
        Band::new(5)
    }
}

/// Parse + check a single target, folding the outcome into `tally`.
fn verify_one(target: &Path, tally: &mut GateTally) {
    let src = match read_file(target) {
        Ok(s) => s,
        Err(e) => {
            println!("  ✗ {} : read error: {e}", target.display());
            tally.failing += 1;
            return;
        }
    };
    let edition = match edition_manifest::resolve_edition_for(target) {
        Ok(resolved) => {
            if let Some(warning) = resolved.warning {
                eprintln!("{warning}");
            }
            resolved.edition
        }
        Err(message) => {
            println!("  ✗ {} : {message}", target.display());
            tally.failing += 1;
            return;
        }
    };
    match garnet_parser::parse_source_with_edition(&src, edition) {
        Ok(module) => {
            let report = garnet_check::check_module(&module);
            if report.ok() {
                let advisories = report.errors.len();
                if advisories > 0 {
                    tally.advisories += advisories;
                    println!(
                        "  ~ {} : ok ({advisories} advisor{})",
                        target.display(),
                        if advisories == 1 { "y" } else { "ies" }
                    );
                } else {
                    println!("  ✓ {} : clean", target.display());
                }
            } else {
                tally.failing += 1;
                println!(
                    "  ✗ {} : {} diagnostic(s)",
                    target.display(),
                    report.errors.len()
                );
                for err in &report.errors {
                    println!("      {err}");
                }
            }
        }
        Err(e) => {
            tally.failing += 1;
            println!("  ✗ {} : parse error: {e}", target.display());
        }
    }
}

/// Collect the targets under `path`, run edition-aware parse + safe-mode check
/// over each (printing a per-target line), and return the aggregate tally.
/// `Err` signals a *usage* problem (unreadable path / no `.garnet` files) — not
/// a gate failure. Exposed for integration tests so the accept/reject verdict is
/// asserted without scraping stdout or process exit codes.
pub fn gate_tally(path: &Path) -> Result<GateTally, String> {
    let targets = collect_targets(path).map_err(|e| e.to_string())?;
    if targets.is_empty() {
        return Err(format!("no .garnet files found under {}", path.display()));
    }
    println!(
        "garnet verify: acceptance gate over {} target(s)",
        targets.len()
    );
    let mut tally = GateTally::default();
    for target in &targets {
        tally.targets += 1;
        verify_one(target, &mut tally);
    }
    Ok(tally)
}

/// Rule names for the directories the walk refuses to read. These strings are
/// contract text: they appear verbatim in the `garnet.diff-caps.machine/1`
/// disclosure, so a reviewer or agent can tell WHY a path went unread.
pub const RULE_BUILD_OUTPUT: &str = "build-output";
/// VCS internals (`.git`) — managed by git, not authored source.
pub const RULE_VCS_METADATA: &str = "vcs-metadata";
/// This tool's own cache (`.garnet-cache`) — machine-generated.
pub const RULE_TOOL_CACHE: &str = "tool-cache";
/// The one documented vendored-dependency path, `<root>/.garnet/vendor`.
pub const RULE_VENDORED_DEPENDENCIES: &str = "vendored-dependencies";
/// A directory reached through a symbolic link. Not followed — a link loop
/// must terminate — so whatever it holds is unread, and said so.
pub const RULE_SYMLINKED_DIRECTORY: &str = "symlinked-directory";

/// What a walk did NOT read, tallied by rule.
///
/// Crown C B-1: a skipped `.garnet` file is DECLARED authority the capability
/// surface never contains, so a silent skip lets a widening merge pass the
/// gate. This tally makes the omission visible. It carries counts and rule
/// names only — never paths, absolute or otherwise.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanOmissions {
    counts: BTreeMap<&'static str, usize>,
}

impl ScanOmissions {
    fn record(&mut self, rule: &'static str) {
        *self.counts.entry(rule).or_insert(0) += 1;
    }

    /// Directories declined. `0` means every directory the walk reached was
    /// either read or tallied here — not that the filesystem holds nothing
    /// else: a directory symlink met BELOW the supplied root is not followed
    /// and is tallied under [`RULE_SYMLINKED_DIRECTORY`]; a link with no
    /// target holds nothing to read and is not tallied; a link the walk
    /// cannot resolve (permission denied, a loop) is an ERROR, never a zero.
    /// The supplied root itself is resolved by the OS and walked.
    pub fn total(&self) -> usize {
        self.counts.values().copied().sum()
    }

    /// `(rule, count)` pairs, sorted by rule name for deterministic output.
    pub fn by_rule(&self) -> Vec<(&'static str, usize)> {
        self.counts.iter().map(|(r, c)| (*r, *c)).collect()
    }

    /// Fold another walk's omissions in — `diff-caps` walks two trees and the
    /// verdict covers both.
    pub fn merge(&mut self, other: &ScanOmissions) {
        for (rule, count) in &other.counts {
            *self.counts.entry(rule).or_insert(0) += count;
        }
    }
}

/// Resolve the target list: a single `.garnet` file, or every `.garnet` file
/// under a directory (skipping the name-matched trees of [`omission_rule`]).
/// Returned sorted for deterministic output. `pub(crate)` so `garnet caps`
/// (S36) reuses the walk.
///
/// Callers that GATE on the result should prefer
/// [`collect_targets_with_omissions`] and disclose what went unread.
pub(crate) fn collect_targets(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    Ok(collect_targets_with_omissions(path)?.0)
}

/// [`collect_targets`] plus the tally of directories the walk refused to read.
pub fn collect_targets_with_omissions(
    path: &Path,
) -> std::io::Result<(Vec<PathBuf>, ScanOmissions)> {
    let mut omissions = ScanOmissions::default();
    if path.is_file() {
        return Ok((vec![path.to_path_buf()], omissions));
    }
    let mut out = Vec::new();
    walk(path, path, &mut out, &mut omissions)?;
    out.sort();
    Ok((out, omissions))
}

/// The rule under which `dir` is omitted from the walk, or `None` to walk it.
///
/// Crown C B-1: this used to match a bare directory NAME at any depth, so a
/// `.garnet` file under any `vendor/` or `node_modules/` — names an author or
/// an attacker picks freely — was invisible to the authority gate while
/// `diff-caps` still reported `no-authority-expansion`, band 5/5, exit 0.
///
/// Now `vendor` is recognized only as the ROOT-RELATIVE `<root>/.garnet/vendor`
/// — the one path a dependency may bind to (garnet-cli/AGENTS.md) and one that
/// lives inside the tool-owned `.garnet` directory. `node_modules` has no
/// meaning in Garnet's build model at all, so it is no longer a skip: it is
/// ordinary source and is walked.
///
/// `target`, `.git`, and `.garnet-cache` remain skipped at any depth. Each
/// names a tree a tool conventionally generates — Cargo/Garnet build output,
/// git internals, this tool's own cache. That is a NAME match, not a verified
/// ownership fact: nothing checks who wrote a `target/`. Every skip this rule
/// makes, and every directory symlink [`walk`] declines, is disclosed by
/// [`ScanOmissions`]; a link [`walk`] cannot resolve is an error.
fn omission_rule(root: &Path, dir: &Path) -> Option<&'static str> {
    let name = dir.file_name()?.to_string_lossy().into_owned();
    match name.as_str() {
        "target" => Some(RULE_BUILD_OUTPUT),
        ".git" => Some(RULE_VCS_METADATA),
        ".garnet-cache" => Some(RULE_TOOL_CACHE),
        "vendor" => {
            let relative = dir.strip_prefix(root).ok()?;
            (relative == Path::new(".garnet").join("vendor")).then_some(RULE_VENDORED_DEPENDENCIES)
        }
        _ => None,
    }
}

fn walk(
    root: &Path,
    dir: &Path,
    out: &mut Vec<PathBuf>,
    omissions: &mut ScanOmissions,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            // Resolve the link with an error-PRESERVING call: `is_dir()`
            // folds EACCES / ELOOP into `false`, which is how review B1-v2
            // found an existing source-bearing directory behind an
            // unsearchable parent dropped with neither a tally nor an error.
            match std::fs::metadata(&path) {
                // A linked directory is not FOLLOWED — a link loop must
                // terminate — but not silent either (review B1): `.garnet`
                // files behind it are declared authority this walk never
                // read, so the refusal is tallied like every other one.
                Ok(meta) if meta.is_dir() => {
                    omissions.record(RULE_SYMLINKED_DIRECTORY);
                    continue;
                }
                // A linked FILE takes the extension branch below and is read
                // through the link as before.
                Ok(_) => {}
                // Nothing behind the link: nothing to read, so nothing to
                // tally. A dangling `.garnet` NAME still reaches the read
                // below and fails there, as it always has.
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                // Anything else — permission denied, a loop, an I/O fault —
                // means the walk cannot say what is behind the link, and a
                // gate must not print a green it could not earn.
                Err(e) => {
                    return Err(std::io::Error::new(
                        e.kind(),
                        format!("{}: cannot resolve symlink target: {e}", path.display()),
                    ));
                }
            }
        }
        if file_type.is_dir() {
            if let Some(rule) = omission_rule(root, &path) {
                omissions.record(rule);
                continue;
            }
            walk(root, &path, out, omissions)?;
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("garnet"))
        {
            // Case-insensitive so Windows' case-insensitive filesystem does not
            // silently skip an uppercase `.GARNET` file (WIN-S33/36/37/46). This
            // shared collector is reused by `cap_manifest::surface_for_path`, so
            // the one fix covers verify / caps / diff-caps / sandbox-policy walks.
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::collect_targets;
    use std::fs;
    use tempfile::TempDir;

    /// The shared collector must discover an uppercase `.GARNET` file. macOS
    /// preserves filename case, so this reproduces the Windows-only *skip*
    /// (WIN-S33/36/37/46) on Mac: pre-fix the case-sensitive `== "garnet"`
    /// dropped `BAD.GARNET`; post-fix the case-insensitive compare keeps it.
    #[test]
    fn collect_targets_discovers_uppercase_garnet_extension() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("main.garnet"),
            "@caps()\ndef main() { 1 }\n",
        )
        .unwrap();
        fs::write(tmp.path().join("BAD.GARNET"), "def main( { 1 }\n").unwrap();

        let found = collect_targets(tmp.path()).unwrap();
        let names: Vec<String> = found
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(
            names.iter().any(|n| n == "BAD.GARNET"),
            "uppercase .GARNET must be discovered, got {names:?}"
        );
        assert!(names.iter().any(|n| n == "main.garnet"), "got {names:?}");
        assert_eq!(found.len(), 2, "both targets discovered, got {names:?}");
    }

    /// A single uppercase `.GARNET` *file* target also resolves (the file-path
    /// branch is already case-agnostic; this pins it against regressions).
    #[test]
    fn collect_targets_accepts_single_uppercase_file() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("LIB.GARNET");
        fs::write(&f, "def f() { 1 }\n").unwrap();
        assert_eq!(collect_targets(&f).unwrap(), vec![f]);
    }

    /// Write a `.garnet` file at `rel` under `root`, creating parents.
    fn write_nested(root: &std::path::Path, rel: &str, body: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, body).unwrap();
    }

    fn names(root: &std::path::Path) -> Vec<String> {
        collect_targets(root)
            .unwrap()
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect()
    }

    /// crown C B-1: a bare `vendor/` is NOT the documented vendored path
    /// (`.garnet/vendor/<name>`), so the collector must walk it — otherwise
    /// declared authority under it is invisible to `diff-caps`.
    #[test]
    fn collect_targets_walks_a_bare_vendor_directory() {
        let tmp = TempDir::new().unwrap();
        write_nested(tmp.path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
        write_nested(
            tmp.path(),
            "vendor/evil.garnet",
            "@caps(net, fs)\ndef reach() { 1 }\n",
        );
        let found = names(tmp.path());
        assert!(
            found.iter().any(|n| n == "evil.garnet"),
            "bare vendor/ must be walked, got {found:?}"
        );
    }

    /// `node_modules` is an equally arbitrary name with no Garnet meaning; a
    /// `.garnet` file there was just as invisible to the authority gate.
    #[test]
    fn collect_targets_walks_node_modules() {
        let tmp = TempDir::new().unwrap();
        write_nested(tmp.path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
        write_nested(
            tmp.path(),
            "node_modules/evil.garnet",
            "@caps(net, fs)\ndef reach() { 1 }\n",
        );
        let found = names(tmp.path());
        assert!(
            found.iter().any(|n| n == "evil.garnet"),
            "node_modules/ must be walked, got {found:?}"
        );
    }

    /// The old skip matched the bare directory NAME at any depth.
    #[test]
    fn collect_targets_walks_a_nested_vendor_directory() {
        let tmp = TempDir::new().unwrap();
        write_nested(tmp.path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
        write_nested(
            tmp.path(),
            "a/b/vendor/evil.garnet",
            "@caps(net, fs)\ndef reach() { 1 }\n",
        );
        let found = names(tmp.path());
        assert!(
            found.iter().any(|n| n == "evil.garnet"),
            "a/b/vendor/ must be walked, got {found:?}"
        );
    }

    /// Legitimate skips survive: build output, VCS internals, tool cache.
    #[test]
    fn collect_targets_skips_build_output_vcs_and_cache() {
        let tmp = TempDir::new().unwrap();
        write_nested(tmp.path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
        for dir in ["target", ".git", ".garnet-cache"] {
            write_nested(
                tmp.path(),
                &format!("{dir}/build.garnet"),
                "@caps(net)\ndef g() { 1 }\n",
            );
        }
        assert_eq!(names(tmp.path()), vec!["tool.garnet".to_string()]);
    }

    /// The one documented vendored path stays skipped — and only at exactly
    /// `<root>/.garnet/vendor`, never at an arbitrary depth.
    #[test]
    fn collect_targets_skips_only_the_root_relative_dot_garnet_vendor() {
        let tmp = TempDir::new().unwrap();
        write_nested(tmp.path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
        write_nested(
            tmp.path(),
            ".garnet/vendor/dep/lib.garnet",
            "@caps(net)\ndef d() { 1 }\n",
        );
        write_nested(
            tmp.path(),
            "sub/.garnet/vendor/dep/lib.garnet",
            "@caps(net)\ndef d() { 1 }\n",
        );
        let found = names(tmp.path());
        assert_eq!(
            found.iter().filter(|n| n.as_str() == "lib.garnet").count(),
            1,
            "only <root>/.garnet/vendor is skipped, got {found:?}"
        );
    }

    /// The tally names every omission so a consumer can see the walk was not
    /// total, without leaking absolute paths.
    #[test]
    fn collect_targets_reports_the_omissions_it_made() {
        let tmp = TempDir::new().unwrap();
        write_nested(tmp.path(), "tool.garnet", "@caps()\ndef main() { 1 }\n");
        for rel in [
            "target/a.garnet",
            ".git/b.garnet",
            ".garnet-cache/c.garnet",
            ".garnet/vendor/dep/d.garnet",
        ] {
            write_nested(tmp.path(), rel, "@caps(net)\ndef g() { 1 }\n");
        }
        let (_, omissions) = super::collect_targets_with_omissions(tmp.path()).unwrap();
        assert_eq!(omissions.total(), 4);
        assert_eq!(
            omissions.by_rule(),
            vec![
                ("build-output", 1),
                ("tool-cache", 1),
                ("vcs-metadata", 1),
                ("vendored-dependencies", 1),
            ]
        );
    }

    /// A total walk reports nothing skipped.
    #[test]
    fn a_total_walk_records_no_omissions() {
        let tmp = TempDir::new().unwrap();
        write_nested(tmp.path(), "a/b/tool.garnet", "@caps()\ndef main() { 1 }\n");
        let (_, omissions) = super::collect_targets_with_omissions(tmp.path()).unwrap();
        assert_eq!(omissions.total(), 0);
        assert!(omissions.by_rule().is_empty());
    }

    /// Cross-family review B1: a directory reached through a symlink is not
    /// followed — and must be tallied, not silently dropped.
    #[cfg(unix)]
    #[test]
    fn collect_targets_declines_a_symlinked_directory_and_says_so() {
        let dir = TempDir::new().unwrap();
        let external = TempDir::new().unwrap();
        write_nested(dir.path(), "a.garnet", "@caps()\ndef main() { 1 }\n");
        write_nested(external.path(), "b.garnet", "@caps(net)\ndef g() { 1 }\n");
        std::os::unix::fs::symlink(external.path(), dir.path().join("link")).unwrap();
        let (found, omissions) = super::collect_targets_with_omissions(dir.path()).unwrap();
        let names: Vec<String> = found
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["a.garnet".to_string()]);
        assert_eq!(omissions.total(), 1);
        assert_eq!(
            omissions.by_rule(),
            vec![(super::RULE_SYMLINKED_DIRECTORY, 1)]
        );
    }

    /// Review B1-v2: a link the walk cannot resolve is an error, not a silent
    /// zero. (Skipped under root, which bypasses directory permissions.)
    #[cfg(unix)]
    #[test]
    fn collect_targets_errors_on_an_unresolvable_link() {
        use std::os::unix::fs::PermissionsExt;
        let uid = std::process::Command::new("id")
            .arg("-u")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if uid == "0" {
            eprintln!("skipped: root bypasses directory permissions");
            return;
        }
        let dir = TempDir::new().unwrap();
        let locked = TempDir::new().unwrap();
        let actual = locked.path().join("actual");
        std::fs::create_dir_all(&actual).unwrap();
        write_nested(dir.path(), "a.garnet", "@caps()\ndef main() { 1 }\n");
        write_nested(&actual, "b.garnet", "@caps(net)\ndef g() { 1 }\n");
        std::os::unix::fs::symlink(&actual, dir.path().join("src")).unwrap();
        std::fs::set_permissions(locked.path(), std::fs::Permissions::from_mode(0o000)).unwrap();
        let result = super::collect_targets_with_omissions(dir.path());
        std::fs::set_permissions(locked.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let err = result.expect_err("an unresolvable link must not be a silent omission");
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(err.to_string().contains("src"), "{err}");
    }
}
