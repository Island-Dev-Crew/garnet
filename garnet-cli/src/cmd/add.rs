//! `garnet add <path>` — vendor a local Garnet directory as a dependency.
//!
//! ## v0.5.1 scope (honest)
//!
//! This is **not** a package manager. There is no central registry,
//! no version resolution, no dependency graph, and no upgrade path.
//! What `garnet add` does today:
//!
//! 1. Resolves the active project root by searching upward for `Garnet.toml`.
//! 2. Copies the source tree at `<path>` into `.garnet/vendor/<name>/`,
//!    where `<name>` defaults to the basename of `<path>` (override via
//!    `--name <id>`).
//! 3. Updates `Garnet.toml`'s `[dependencies]` table with
//!    `<name> = { path = "<original_path>", vendor = ".garnet/vendor/<name>" }`.
//! 4. Writes `Garnet.lock` with the BLAKE3 hash of every vendored `.garnet`
//!    file so a second `garnet add` (or `garnet verify-deps`, later) can
//!    detect tampering or drift.
//!
//! What it does NOT do (honest partial):
//!
//! - Resolve `use <dep>::<symbol>` at parse/check/run time. The
//!   interpreter does not yet load vendored deps into the symbol table;
//!   that wiring is a separate v0.5.x slice. The vendored bytes sit on
//!   disk and the lockfile records their hash, but they do not affect
//!   `garnet run` output yet.
//! - Recursive vendoring (transitive deps). If `<path>` itself has a
//!   `Garnet.toml` with deps, they are NOT pulled in.
//! - Network fetch (`garnet add https://...` or `garnet add @scope/name`).
//!   Local paths only.
//! - Lockfile reconciliation when `<path>` changes underneath. Rerun
//!   `garnet add` to refresh.
//!
//! See `C_Language_Specification/GARNET_MANIFEST_v0_1.md` for the
//! manifest + lockfile formats.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// CLI arg shape. `path` is the source directory or file. `name` defaults
/// to the basename of `path` and may be overridden by `--name <id>`.
struct AddArgs {
    path: PathBuf,
    name: Option<String>,
}

pub fn run(args: &[String]) -> ExitCode {
    // S13: `garnet add --registry <location> <name>@<version>` is a separate
    // resolution path from the local-path vendor flow below.
    if args.iter().any(|a| a == "--registry") {
        return run_registry_add(args);
    }
    let parsed = match parse_args(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("garnet add: {e}");
            print_help();
            return ExitCode::from(2);
        }
    };

    let project_root = match find_project_root() {
        Some(p) => p,
        None => {
            eprintln!(
                "garnet add: not in a Garnet project (no `Garnet.toml` in this directory or any parent)"
            );
            return ExitCode::from(1);
        }
    };

    let source = &parsed.path;
    if !source.exists() {
        eprintln!(
            "garnet add: source path does not exist: {}",
            source.display()
        );
        return ExitCode::from(1);
    }

    let name = match resolve_name(parsed.name.as_deref(), source) {
        Some(n) => n,
        None => {
            eprintln!(
                "garnet add: could not infer a dependency name from {}; pass --name <id>",
                source.display()
            );
            return ExitCode::from(1);
        }
    };
    if !is_valid_dep_name(&name) {
        eprintln!("garnet add: dependency name `{name}` must match [A-Za-z_][A-Za-z0-9_-]*");
        return ExitCode::from(1);
    }

    let vendor_root = project_root.join(".garnet/vendor").join(&name);
    if let Err(e) = vendor_into(source, &vendor_root) {
        eprintln!("garnet add: failed to copy into vendor: {e}");
        return ExitCode::from(1);
    }

    let hashes = match hash_vendor_tree(&vendor_root) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("garnet add: failed to hash vendor tree: {e}");
            return ExitCode::from(1);
        }
    };

    let original_path_str = display_path(source);
    let vendor_rel = format!(".garnet/vendor/{name}");
    if let Err(e) = update_manifest(&project_root, &name, &original_path_str, &vendor_rel) {
        eprintln!("garnet add: failed to update Garnet.toml: {e}");
        return ExitCode::from(1);
    }
    if let Err(e) = write_lockfile(
        &project_root,
        &name,
        &original_path_str,
        &vendor_rel,
        &hashes,
    ) {
        eprintln!("garnet add: failed to write Garnet.lock: {e}");
        return ExitCode::from(1);
    }

    println!(
        "garnet add: vendored `{name}` from {} into {} ({} file(s) hashed)",
        original_path_str,
        vendor_rel,
        hashes.len()
    );
    println!("  Garnet.toml [dependencies] updated");
    println!("  Garnet.lock updated");
    ExitCode::SUCCESS
}

/// S13: `garnet add --registry <location> <name>@<version>`. Resolves a
/// package from a filesystem-backed registry (an `index.json` + versioned
/// package directories), verifies its BLAKE3 content-address, and vendors it
/// exactly like a local-path add. v0.1 stub: no HTTP, no SemVer ranges, no
/// transitive deps, no signature verification.
fn run_registry_add(args: &[String]) -> ExitCode {
    let mut location: Option<PathBuf> = None;
    let mut spec: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--registry" => {
                if i + 1 >= args.len() {
                    eprintln!("garnet add: --registry requires a <location>");
                    return ExitCode::from(2);
                }
                location = Some(PathBuf::from(strip_file_url(&args[i + 1])));
                i += 2;
            }
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            arg if arg.starts_with("--") => {
                eprintln!("garnet add: unknown flag: {arg}");
                return ExitCode::from(2);
            }
            other => {
                if spec.is_some() {
                    eprintln!("garnet add: unexpected extra argument: {other}");
                    return ExitCode::from(2);
                }
                spec = Some(other.to_string());
                i += 1;
            }
        }
    }

    let Some(location) = location else {
        eprintln!("garnet add: --registry requires a <location>");
        return ExitCode::from(2);
    };
    let Some(spec) = spec else {
        eprintln!("garnet add: expected <name>@<version> with --registry");
        return ExitCode::from(2);
    };
    let (name, version) = match spec.split_once('@') {
        Some((n, v)) if !n.is_empty() && !v.is_empty() => (n.to_string(), v.to_string()),
        _ => {
            eprintln!("garnet add: registry spec must be <name>@<version>, got `{spec}`");
            return ExitCode::from(2);
        }
    };
    if !is_valid_dep_name(&name) {
        eprintln!("garnet add: dependency name `{name}` must match [A-Za-z_][A-Za-z0-9_-]*");
        return ExitCode::from(1);
    }

    let project_root = match find_project_root() {
        Some(p) => p,
        None => {
            eprintln!(
                "garnet add: not in a Garnet project (no `Garnet.toml` in this directory or any parent)"
            );
            return ExitCode::from(1);
        }
    };

    let index = match garnet_registry_stub::load_index(&location) {
        Ok(index) => index,
        Err(e) => {
            eprintln!("garnet add: registry {}: {e}", location.display());
            return ExitCode::from(1);
        }
    };
    let entry = match garnet_registry_stub::resolve(&index, &name, &version) {
        Ok(entry) => entry,
        Err(e) => {
            eprintln!("garnet add: {e}");
            // S45 slopsquatting guard: only when the *name* is unknown (a missing
            // version is not a near-miss). Surface resembling known names so an
            // unknown package that mimics a real one is questioned before trust.
            if !index.packages.contains_key(&name) {
                let suspicions =
                    garnet_registry_stub::slopguard::nearest(&name, index.known_names(), 2);
                if !suspicions.is_empty() {
                    let hints: Vec<String> = suspicions
                        .iter()
                        .take(3)
                        .map(|s| format!("`{}`", s.candidate))
                        .collect();
                    eprintln!(
                        "garnet add: `{name}` is not in this registry — did you mean {}? \
                         Unknown names that closely resemble known packages are a slopsquatting \
                         risk; verify the source before adding.",
                        hints.join(" or ")
                    );
                }
            }
            return ExitCode::from(1);
        }
    };
    let pkg_dir = match garnet_registry_stub::package_dir(&location, &entry) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("garnet add: {e}");
            return ExitCode::from(1);
        }
    };
    if let Err(e) = garnet_registry_stub::verify_package(&pkg_dir, &entry) {
        eprintln!("garnet add: registry integrity check failed for {name}@{version}: {e}");
        return ExitCode::from(1);
    }

    let vendor_root = project_root.join(".garnet/vendor").join(&name);
    if let Err(e) = vendor_into(&pkg_dir, &vendor_root) {
        eprintln!("garnet add: failed to copy into vendor: {e}");
        return ExitCode::from(1);
    }
    let hashes = match hash_vendor_tree(&vendor_root) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("garnet add: failed to hash vendor tree: {e}");
            return ExitCode::from(1);
        }
    };

    let vendor_rel = format!(".garnet/vendor/{name}");
    let registry_str = display_path(&location);
    if let Err(e) =
        update_manifest_registry(&project_root, &name, &registry_str, &version, &vendor_rel)
    {
        eprintln!("garnet add: failed to update Garnet.toml: {e}");
        return ExitCode::from(1);
    }
    let lock_source = format!("registry+{registry_str}#{name}@{version}");
    if let Err(e) = write_lockfile(&project_root, &name, &lock_source, &vendor_rel, &hashes) {
        eprintln!("garnet add: failed to write Garnet.lock: {e}");
        return ExitCode::from(1);
    }

    println!(
        "garnet add: vendored `{name}@{version}` from registry {registry_str} into {vendor_rel} ({} file(s) hashed)",
        hashes.len()
    );
    println!("  Garnet.toml [dependencies] updated");
    println!("  Garnet.lock updated");
    ExitCode::SUCCESS
}

/// Strip a `file://` prefix so `--registry file:///abs/path` and
/// `--registry /abs/path` both work. HTTP(S) transport is deferred.
fn strip_file_url(loc: &str) -> String {
    loc.strip_prefix("file://").unwrap_or(loc).to_string()
}

/// Write a registry-shaped `[dependencies]` entry:
/// `name = { registry = "...", version = "...", vendor = "..." }`.
fn update_manifest_registry(
    project_root: &Path,
    name: &str,
    registry: &str,
    version: &str,
    vendor_rel: &str,
) -> std::io::Result<()> {
    let manifest_path = project_root.join("Garnet.toml");
    let text = fs::read_to_string(&manifest_path)?;
    let entry_line = format!(
        "{name} = {{ registry = \"{}\", version = \"{}\", vendor = \"{}\" }}",
        toml_escape(registry),
        toml_escape(version),
        toml_escape(vendor_rel),
    );
    let updated = upsert_dependency(&text, name, &entry_line);
    fs::write(&manifest_path, updated)?;
    Ok(())
}

fn print_help() {
    eprintln!("usage: garnet add [--name <id>] <path>");
    eprintln!("       garnet add --registry <location> <name>@<version>");
    eprintln!();
    eprintln!("  Vendor a local Garnet directory as a project dependency, or");
    eprintln!("  resolve one from a filesystem-backed registry (index.json).");
    eprintln!("  Updates Garnet.toml [dependencies] and Garnet.lock; copies");
    eprintln!("  source files under .garnet/vendor/<name>/.");
    eprintln!();
    eprintln!("  Registry v0.1 stub: filesystem/file:// only — no HTTP transport,");
    eprintln!("  no SemVer ranges, no transitive deps, no signature verification.");
    eprintln!("  See C_Language_Specification/GARNET_REGISTRY_v0_1.md.");
}

fn parse_args(args: &[String]) -> Result<AddArgs, String> {
    let mut path: Option<String> = None;
    let mut name: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                if i + 1 >= args.len() {
                    return Err("--name requires a value".into());
                }
                name = Some(args[i + 1].clone());
                i += 2;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            arg if arg.starts_with("--") => {
                return Err(format!("unknown flag: {arg}"));
            }
            _ => {
                if path.is_none() {
                    path = Some(args[i].clone());
                    i += 1;
                } else {
                    return Err(format!("unexpected extra argument: {}", args[i]));
                }
            }
        }
    }
    let path = path.ok_or("missing <path> argument")?;
    Ok(AddArgs {
        path: PathBuf::from(path),
        name,
    })
}

/// Walk upward from the current directory looking for `Garnet.toml`.
pub(crate) fn find_project_root() -> Option<PathBuf> {
    let mut cur = std::env::current_dir().ok()?;
    loop {
        if cur.join("Garnet.toml").exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

fn resolve_name(explicit: Option<&str>, source: &Path) -> Option<String> {
    if let Some(n) = explicit {
        return Some(n.to_string());
    }
    let base = if source.is_dir() {
        source.file_name()
    } else {
        source.file_stem()
    }?;
    let s = base.to_string_lossy().to_string();
    // Replace problematic chars: dots, spaces.
    let cleaned: String = s
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => c,
            _ => '-',
        })
        .collect();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn is_valid_dep_name(name: &str) -> bool {
    let mut chars = name.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Recursively copy `source` into `dest`, replacing any prior contents.
fn vendor_into(source: &Path, dest: &Path) -> std::io::Result<()> {
    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;
    if source.is_file() {
        let target_name = source
            .file_name()
            .unwrap_or_else(|| OsStr::new("lib.garnet"));
        fs::copy(source, dest.join(target_name))?;
        return Ok(());
    }
    copy_tree(source, dest)
}

fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        // Skip vendored re-vendoring: never recurse into a nested
        // .garnet/vendor; never copy lockfiles either.
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == ".garnet" || name == "Garnet.lock" {
                continue;
            }
        }
        let target = dst.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            fs::create_dir_all(&target)?;
            copy_tree(&path, &target)?;
        } else if file_type.is_file() {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

/// `(relative_path, blake3_hex)` for every regular file under the vendor dir.
pub(crate) fn hash_vendor_tree(root: &Path) -> std::io::Result<Vec<(String, String)>> {
    let mut out: Vec<(String, String)> = Vec::new();
    visit_files(root, root, &mut out)?;
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn visit_files(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            visit_files(root, &path, out)?;
        } else if file_type.is_file() {
            let bytes = fs::read(&path)?;
            let hash = blake3::hash(&bytes);
            let rel = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/").to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            out.push((rel, hash.to_hex().to_string()));
        }
    }
    Ok(())
}

fn display_path(p: &Path) -> String {
    p.to_string_lossy().to_string()
}

/// Add or replace `<name>` under `[dependencies]` in `Garnet.toml`. Keeps
/// existing entries; preserves comments and unknown sections by
/// line-based rewrite.
fn update_manifest(
    project_root: &Path,
    name: &str,
    original_path: &str,
    vendor_rel: &str,
) -> std::io::Result<()> {
    let manifest_path = project_root.join("Garnet.toml");
    let text = fs::read_to_string(&manifest_path)?;
    let entry_line = format!(
        "{name} = {{ path = \"{}\", vendor = \"{}\" }}",
        toml_escape(original_path),
        toml_escape(vendor_rel),
    );
    let updated = upsert_dependency(&text, name, &entry_line);
    fs::write(&manifest_path, updated)?;
    Ok(())
}

fn upsert_dependency(text: &str, name: &str, new_line: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    // Locate the [dependencies] section (or any subsection like
    // [dependencies.foo]).
    let mut deps_start: Option<usize> = None;
    let mut section_end: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "[dependencies]" {
            deps_start = Some(i);
        } else if deps_start.is_some()
            && trimmed.starts_with('[')
            && trimmed.ends_with(']')
            && !trimmed.starts_with("[dependencies")
        {
            section_end = Some(i);
            break;
        }
    }
    let Some(start) = deps_start else {
        let mut out = text.to_string();
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("\n[dependencies]\n");
        out.push_str(new_line);
        out.push('\n');
        return out;
    };
    let end = section_end.unwrap_or(lines.len());
    let prefix = format!("{name} = ");
    let mut new_lines: Vec<String> = Vec::with_capacity(lines.len() + 1);
    let mut replaced = false;
    for (idx, line) in lines.iter().enumerate() {
        if idx > start && idx < end && line.trim_start().starts_with(&prefix) {
            new_lines.push(new_line.to_string());
            replaced = true;
        } else {
            new_lines.push((*line).to_string());
        }
    }
    if !replaced {
        // Insert the new entry right before section_end (or at end of deps).
        let insert_at = end;
        // Trim trailing blank lines inside the [dependencies] section so
        // the insert lands snugly after the existing entries.
        let mut anchor = insert_at;
        while anchor > start + 1 && new_lines[anchor - 1].trim().is_empty() {
            anchor -= 1;
        }
        new_lines.insert(anchor, new_line.to_string());
    }
    let mut joined = new_lines.join("\n");
    if !joined.ends_with('\n') {
        joined.push('\n');
    }
    joined
}

fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// One vendored dependency as recorded under `[dependencies]` in
/// `Garnet.toml`. Symmetric to `update_manifest`'s writer; used by the S12
/// resolver in `cmd::run` to pre-load vendored sources at `garnet run` time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DependencyEntry {
    pub name: String,
    pub vendor_rel: PathBuf,
}

/// Parse the TOML 1.0 semantic `dependencies` table and return each entry's
/// `(name, vendor_rel)` pair. A real TOML parser is required here: equivalent
/// declarations may use quoted/spaced headers, dotted keys, a top-level inline
/// table, or dependency subtables. Duplicate or malformed keys are rejected by
/// the parser rather than being silently collapsed by a line scanner.
pub(crate) fn read_dependency_table(project_root: &Path) -> std::io::Result<Vec<DependencyEntry>> {
    let manifest_path = project_root.join("Garnet.toml");
    if !manifest_path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&manifest_path)?;
    let document = text.parse::<toml::Value>().map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid Garnet.toml: {error}"),
        )
    })?;
    let Some(dependencies) = document.get("dependencies") else {
        return Ok(Vec::new());
    };
    let table = dependencies.as_table().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Garnet.toml dependencies must be a table",
        )
    })?;
    let mut out: Vec<DependencyEntry> = Vec::new();
    for (name, declaration) in table {
        if !is_valid_dep_name(name) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid dependency name: {name}"),
            ));
        }
        let declaration = declaration.as_table().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("dependency {name} must be a table"),
            )
        })?;
        let vendor = declaration
            .get("vendor")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("dependency {name} must declare a string vendor path"),
                )
            })?;
        let path_shape = declaration.len() == 2
            && declaration
                .get("path")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.is_empty())
            && declaration.contains_key("vendor");
        let registry_shape = declaration.len() == 3
            && declaration
                .get("registry")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.is_empty())
            && declaration
                .get("version")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.is_empty())
            && declaration.contains_key("vendor");
        if !path_shape && !registry_shape {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "dependency {name} must use exactly path+vendor or registry+version+vendor string keys"
                ),
            ));
        }
        let expected_vendor = format!(".garnet/vendor/{name}");
        if vendor != expected_vendor || Path::new(vendor).is_absolute() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("vendor path for {name} must be exactly {expected_vendor}"),
            ));
        }
        out.push(DependencyEntry {
            name: name.clone(),
            vendor_rel: PathBuf::from(vendor),
        });
    }
    Ok(out)
}

/// Write `Garnet.lock` recording every dep + every file hash. Idempotent:
/// re-running `garnet add` for the same dep replaces its block in place.
fn write_lockfile(
    project_root: &Path,
    name: &str,
    original_path: &str,
    vendor_rel: &str,
    hashes: &[(String, String)],
) -> std::io::Result<()> {
    let lock_path = project_root.join("Garnet.lock");
    let existing = match fs::read_to_string(&lock_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };
    let mut entries = parse_lock(&existing);
    let mut files: Vec<(String, String)> = hashes.to_vec();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    entries.insert(
        name.to_string(),
        LockEntry {
            path: original_path.to_string(),
            vendor: vendor_rel.to_string(),
            files,
        },
    );
    fs::write(&lock_path, render_lock(&entries))?;
    Ok(())
}

#[derive(Debug, Clone)]
struct LockEntry {
    path: String,
    vendor: String,
    files: Vec<(String, String)>,
}

fn parse_lock(text: &str) -> std::collections::BTreeMap<String, LockEntry> {
    use std::collections::BTreeMap;
    let mut out: BTreeMap<String, LockEntry> = BTreeMap::new();
    let mut current: Option<(String, LockEntry)> = None;
    let mut in_files = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix("[[dependencies.")
            .and_then(|s| s.strip_suffix("]]"))
        {
            if let Some((n, e)) = current.take() {
                out.insert(n, e);
            }
            current = Some((
                name.to_string(),
                LockEntry {
                    path: String::new(),
                    vendor: String::new(),
                    files: Vec::new(),
                },
            ));
            in_files = false;
        } else if let Some((_, ref mut e)) = current.as_mut() {
            if let Some(rest) = trimmed.strip_prefix("path = \"") {
                e.path = rest.trim_end_matches('"').to_string();
            } else if let Some(rest) = trimmed.strip_prefix("vendor = \"") {
                e.vendor = rest.trim_end_matches('"').to_string();
            } else if trimmed == "files = [" {
                in_files = true;
            } else if in_files {
                if trimmed == "]" {
                    in_files = false;
                } else if let Some(rest) = trimmed.strip_prefix("{") {
                    let body = rest.trim_end_matches(',').trim_end_matches('}');
                    let mut path = String::new();
                    let mut hash = String::new();
                    for kv in body.split(',') {
                        let kv = kv.trim();
                        if let Some(v) = kv.strip_prefix("path = \"") {
                            path = v.trim_end_matches('"').to_string();
                        } else if let Some(v) = kv.strip_prefix("hash = \"") {
                            hash = v.trim_end_matches('"').to_string();
                        }
                    }
                    if !path.is_empty() && !hash.is_empty() {
                        e.files.push((path, hash));
                    }
                }
            }
        }
    }
    if let Some((n, e)) = current {
        out.insert(n, e);
    }
    out
}

fn render_lock(entries: &std::collections::BTreeMap<String, LockEntry>) -> String {
    let mut out = String::new();
    out.push_str("# Garnet.lock — generated by `garnet add`. Do not edit by hand.\n");
    out.push_str("# Per-file BLAKE3 hashes detect tampering and drift. Re-run `garnet add\n");
    out.push_str("# <path>` to refresh after the upstream source changes.\n");
    out.push_str("version = \"0.1\"\n\n");
    for (name, entry) in entries {
        out.push_str(&format!("[[dependencies.{name}]]\n"));
        out.push_str(&format!("path = \"{}\"\n", toml_escape(&entry.path)));
        out.push_str(&format!("vendor = \"{}\"\n", toml_escape(&entry.vendor)));
        out.push_str("files = [\n");
        for (p, h) in &entry.files {
            out.push_str(&format!(
                "  {{ path = \"{}\", hash = \"{}\" }},\n",
                toml_escape(p),
                h
            ));
        }
        out.push_str("]\n\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn new_garnet_project(root: &Path) {
        fs::write(
            root.join("Garnet.toml"),
            "[package]\nname = \"demo\"\n\n[dependencies]\n",
        )
        .unwrap();
    }

    #[test]
    fn vendor_into_copies_files_and_skips_existing_vendor() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src-lib");
        fs::create_dir_all(src.join("nested")).unwrap();
        fs::write(src.join("lib.garnet"), b"def hello() { \"hi\" }\n").unwrap();
        fs::write(src.join("nested/util.garnet"), b"def util() { 1 }\n").unwrap();
        // A pre-existing .garnet directory in the source must NOT be copied.
        fs::create_dir_all(src.join(".garnet/vendor/x")).unwrap();
        fs::write(
            src.join(".garnet/vendor/x/inner.garnet"),
            b"def x() { 0 }\n",
        )
        .unwrap();
        let dest = tmp.path().join("vendor/dest");
        vendor_into(&src, &dest).unwrap();
        assert!(dest.join("lib.garnet").exists());
        assert!(dest.join("nested/util.garnet").exists());
        assert!(
            !dest.join(".garnet").exists(),
            "nested vendor must be skipped"
        );
    }

    #[test]
    fn hash_vendor_tree_is_stable_and_sorted() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("a")).unwrap();
        fs::write(root.join("a/x.garnet"), b"x\n").unwrap();
        fs::write(root.join("z.garnet"), b"z\n").unwrap();
        let h1 = hash_vendor_tree(root).unwrap();
        let h2 = hash_vendor_tree(root).unwrap();
        assert_eq!(h1, h2, "BLAKE3 must be deterministic");
        let paths: Vec<&str> = h1.iter().map(|(p, _)| p.as_str()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted, "output must be path-sorted");
    }

    #[test]
    fn upsert_dependency_inserts_in_existing_section() {
        let manifest = "[package]\nname = \"demo\"\n\n[dependencies]\n\n[caps]\nallowed = []\n";
        let updated = upsert_dependency(manifest, "mylib", "mylib = { path = \"../mylib\" }");
        assert!(updated.contains("[dependencies]"));
        assert!(updated.contains("mylib = { path = \"../mylib\" }"));
        // The [caps] section must remain.
        assert!(updated.contains("[caps]"));
    }

    #[test]
    fn upsert_dependency_replaces_existing_entry() {
        let manifest = "[package]\nname = \"demo\"\n\n[dependencies]\nmylib = { path = \"old\" }\n";
        let updated = upsert_dependency(manifest, "mylib", "mylib = { path = \"new\" }");
        assert!(updated.contains("path = \"new\""));
        assert!(!updated.contains("path = \"old\""));
        assert_eq!(
            updated.matches("mylib = {").count(),
            1,
            "must not duplicate the entry"
        );
    }

    #[test]
    fn render_then_parse_lock_round_trips() {
        use std::collections::BTreeMap;
        let mut entries: BTreeMap<String, LockEntry> = BTreeMap::new();
        entries.insert(
            "mylib".to_string(),
            LockEntry {
                path: "../mylib".to_string(),
                vendor: ".garnet/vendor/mylib".to_string(),
                files: vec![
                    ("a.garnet".to_string(), "deadbeef".to_string()),
                    ("nested/b.garnet".to_string(), "cafebabe".to_string()),
                ],
            },
        );
        let text = render_lock(&entries);
        let parsed = parse_lock(&text);
        assert_eq!(parsed.len(), 1);
        let got = parsed.get("mylib").unwrap();
        assert_eq!(got.path, "../mylib");
        assert_eq!(got.files.len(), 2);
    }

    #[test]
    fn end_to_end_add_writes_lock_and_manifest() {
        let tmp = TempDir::new().unwrap();
        let project_root = tmp.path().join("demo");
        fs::create_dir_all(&project_root).unwrap();
        new_garnet_project(&project_root);
        let src = tmp.path().join("local-lib");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.garnet"), b"def hello() { \"hi\" }\n").unwrap();

        // Mimic what `run()` does internally without the std::env::current_dir
        // dependency.
        let name = "local-lib";
        let vendor_root = project_root.join(".garnet/vendor").join(name);
        vendor_into(&src, &vendor_root).unwrap();
        let hashes = hash_vendor_tree(&vendor_root).unwrap();
        let original = src.to_string_lossy().to_string();
        update_manifest(&project_root, name, &original, ".garnet/vendor/local-lib").unwrap();
        write_lockfile(
            &project_root,
            name,
            &original,
            ".garnet/vendor/local-lib",
            &hashes,
        )
        .unwrap();
        let manifest = fs::read_to_string(project_root.join("Garnet.toml")).unwrap();
        assert!(manifest.contains("local-lib"));
        let lock = fs::read_to_string(project_root.join("Garnet.lock")).unwrap();
        assert!(lock.contains("[[dependencies.local-lib]]"));
        assert!(lock.contains("hash = \""));
    }
}
