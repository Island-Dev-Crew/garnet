//! Garnet registry stub v0.1 — filesystem-backed index + content-addressed
//! package resolution.
//!
//! This is **not** a production registry. v0.1 is deliberately filesystem-only:
//! a registry is a directory containing an `index.json` and `<name>/<version>/`
//! package directories. The substance is the resolution loop (index lookup →
//! BLAKE3 content-address verify → copy), not the transport. HTTP(S) transport,
//! tarball packaging, auth, a publish flow, SemVer ranges, and signature
//! verification are all deferred.
//!
//! See `C_Language_Specification/GARNET_REGISTRY_v0_1.md`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The on-disk `index.json` schema (v0.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryIndex {
    /// Format version. `"0.1"` today.
    pub registry_version: String,
    /// `name -> { versions: { version -> entry } }`, deterministically ordered.
    pub packages: BTreeMap<String, PackageEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEntry {
    pub versions: BTreeMap<String, VersionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionEntry {
    /// Package directory relative to the registry root, e.g. `hello_lib/0.1.0`.
    pub path: String,
    /// `(relative file path, lowercase BLAKE3 hex)` for every regular file in
    /// the package, lexicographically sorted by path.
    pub files: Vec<FileHash>,
    /// Reserved for a future signing slice. v0.1 does NOT verify it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileHash {
    pub path: String,
    pub blake3: String,
}

#[derive(Debug)]
pub enum RegistryError {
    Io(String),
    Json(String),
    NotFound(String),
    Integrity(String),
    Traversal(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::Io(m) => write!(f, "io error: {m}"),
            RegistryError::Json(m) => write!(f, "index json error: {m}"),
            RegistryError::NotFound(m) => write!(f, "not found: {m}"),
            RegistryError::Integrity(m) => write!(f, "integrity error: {m}"),
            RegistryError::Traversal(m) => write!(f, "path traversal refused: {m}"),
        }
    }
}

impl std::error::Error for RegistryError {}

impl From<std::io::Error> for RegistryError {
    fn from(e: std::io::Error) -> Self {
        RegistryError::Io(e.to_string())
    }
}

/// `(relative path, lowercase BLAKE3 hex)` for every regular file under `root`,
/// lexicographically sorted by relative path. POSIX-style `/` separators.
pub fn hash_tree(root: &Path) -> Result<Vec<FileHash>, RegistryError> {
    let mut out = Vec::new();
    visit(root, root, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn visit(root: &Path, dir: &Path, out: &mut Vec<FileHash>) -> Result<(), RegistryError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            visit(root, &path, out)?;
        } else if file_type.is_file() {
            let bytes = fs::read(&path)?;
            let rel = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            out.push(FileHash {
                path: rel,
                blake3: blake3::hash(&bytes).to_hex().to_string(),
            });
        }
    }
    Ok(())
}

/// Build a `RegistryIndex` by scanning `registry_root` for `<name>/<version>/`
/// package directories and hashing each one. A directory is treated as a
/// package version when it is exactly two levels deep (`<name>/<version>`).
pub fn build_index(registry_root: &Path) -> Result<RegistryIndex, RegistryError> {
    let mut packages: BTreeMap<String, PackageEntry> = BTreeMap::new();
    for name_entry in fs::read_dir(registry_root)? {
        let name_entry = name_entry?;
        if !name_entry.file_type()?.is_dir() {
            continue;
        }
        let name = name_entry.file_name().to_string_lossy().to_string();
        let mut versions: BTreeMap<String, VersionEntry> = BTreeMap::new();
        for ver_entry in fs::read_dir(name_entry.path())? {
            let ver_entry = ver_entry?;
            if !ver_entry.file_type()?.is_dir() {
                continue;
            }
            let version = ver_entry.file_name().to_string_lossy().to_string();
            let files = hash_tree(&ver_entry.path())?;
            versions.insert(
                version,
                VersionEntry {
                    path: format!("{name}/{}", ver_entry.file_name().to_string_lossy()),
                    files,
                    signature: None,
                },
            );
        }
        if !versions.is_empty() {
            packages.insert(name, PackageEntry { versions });
        }
    }
    Ok(RegistryIndex {
        registry_version: "0.1".to_string(),
        packages,
    })
}

/// Serialize an index to deterministic, pretty JSON (BTreeMaps keep key order
/// stable so two builds of the same registry produce identical bytes).
pub fn index_to_json(index: &RegistryIndex) -> Result<String, RegistryError> {
    serde_json::to_string_pretty(index).map_err(|e| RegistryError::Json(e.to_string()))
}

/// Write `index.json` into `registry_root`.
pub fn write_index(registry_root: &Path, index: &RegistryIndex) -> Result<(), RegistryError> {
    let json = index_to_json(index)?;
    fs::write(registry_root.join("index.json"), json)?;
    Ok(())
}

/// Load and parse `<registry_root>/index.json`.
pub fn load_index(registry_root: &Path) -> Result<RegistryIndex, RegistryError> {
    let path = registry_root.join("index.json");
    let text = fs::read_to_string(&path)
        .map_err(|e| RegistryError::Io(format!("{}: {e}", path.display())))?;
    serde_json::from_str(&text).map_err(|e| RegistryError::Json(e.to_string()))
}

/// Resolve `name@version` against an already-loaded index. Returns a clone of
/// the matching `VersionEntry`.
pub fn resolve(
    index: &RegistryIndex,
    name: &str,
    version: &str,
) -> Result<VersionEntry, RegistryError> {
    let package = index
        .packages
        .get(name)
        .ok_or_else(|| RegistryError::NotFound(format!("package `{name}`")))?;
    package
        .versions
        .get(version)
        .cloned()
        .ok_or_else(|| RegistryError::NotFound(format!("`{name}@{version}`")))
}

/// Resolve the on-disk package directory for a version entry, refusing any
/// `path` that escapes the registry root (path-traversal guard).
pub fn package_dir(registry_root: &Path, entry: &VersionEntry) -> Result<PathBuf, RegistryError> {
    let candidate = registry_root.join(&entry.path);
    let root_canon = registry_root
        .canonicalize()
        .map_err(|e| RegistryError::Io(format!("registry root: {e}")))?;
    let candidate_canon = candidate
        .canonicalize()
        .map_err(|e| RegistryError::Io(format!("{}: {e}", candidate.display())))?;
    if !candidate_canon.starts_with(&root_canon) {
        return Err(RegistryError::Traversal(format!(
            "`{}` resolves outside the registry root",
            entry.path
        )));
    }
    Ok(candidate_canon)
}

/// Verify every file under `package_dir` matches the hashes recorded in
/// `entry.files` (and that no extra/missing files exist).
pub fn verify_package(package_dir: &Path, entry: &VersionEntry) -> Result<(), RegistryError> {
    let actual = hash_tree(package_dir)?;
    if actual.len() != entry.files.len() {
        return Err(RegistryError::Integrity(format!(
            "{} files on disk, {} in index",
            actual.len(),
            entry.files.len()
        )));
    }
    for (got, want) in actual.iter().zip(entry.files.iter()) {
        if got.path != want.path || got.blake3 != want.blake3 {
            return Err(RegistryError::Integrity(format!(
                "mismatch at `{}` (index `{}`)",
                got.path, want.path
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(p: &Path, body: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    fn fixture() -> TempDir {
        let tmp = TempDir::new().unwrap();
        write(
            &tmp.path().join("hello_lib/0.1.0/lib.garnet"),
            "def vendored_hello() { \"hi from registry\" }\n",
        );
        write(
            &tmp.path().join("hello_lib/0.2.0/lib.garnet"),
            "def vendored_hello() { \"hi v2\" }\n",
        );
        tmp
    }

    #[test]
    fn build_then_load_round_trips() {
        let tmp = fixture();
        let index = build_index(tmp.path()).unwrap();
        write_index(tmp.path(), &index).unwrap();
        let loaded = load_index(tmp.path()).unwrap();
        assert_eq!(index, loaded);
        assert_eq!(index.registry_version, "0.1");
        assert_eq!(index.packages["hello_lib"].versions.len(), 2);
    }

    #[test]
    fn build_is_deterministic() {
        let tmp = fixture();
        let a = index_to_json(&build_index(tmp.path()).unwrap()).unwrap();
        let b = index_to_json(&build_index(tmp.path()).unwrap()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn resolve_and_verify_succeeds() {
        let tmp = fixture();
        let index = build_index(tmp.path()).unwrap();
        let entry = resolve(&index, "hello_lib", "0.1.0").unwrap();
        let dir = package_dir(tmp.path(), &entry).unwrap();
        verify_package(&dir, &entry).unwrap();
    }

    #[test]
    fn resolve_missing_version_errors() {
        let tmp = fixture();
        let index = build_index(tmp.path()).unwrap();
        assert!(resolve(&index, "hello_lib", "9.9.9").is_err());
        assert!(resolve(&index, "ghost", "0.1.0").is_err());
    }

    #[test]
    fn verify_detects_tampering() {
        let tmp = fixture();
        let index = build_index(tmp.path()).unwrap();
        let entry = resolve(&index, "hello_lib", "0.1.0").unwrap();
        // Tamper with the on-disk file after the index was built.
        fs::write(
            tmp.path().join("hello_lib/0.1.0/lib.garnet"),
            "def vendored_hello() { \"TAMPERED\" }\n",
        )
        .unwrap();
        let dir = package_dir(tmp.path(), &entry).unwrap();
        assert!(verify_package(&dir, &entry).is_err());
    }

    #[test]
    fn package_dir_refuses_traversal() {
        let tmp = fixture();
        let index = build_index(tmp.path()).unwrap();
        let mut entry = resolve(&index, "hello_lib", "0.1.0").unwrap();
        entry.path = "../../etc".to_string();
        // Either the path doesn't exist (Io) or it resolves outside the root
        // (Traversal); both are errors — the copy must never proceed.
        assert!(package_dir(tmp.path(), &entry).is_err());
    }
}
