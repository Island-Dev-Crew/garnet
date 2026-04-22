//! FFIGeiger — dependency-safety audit (v3.5 Security Layer 3 / master plan §Phase 3-SEC item 3).
//!
//! Closes the "unreviewed `unsafe` / FFI in Rust deps" threat class. For a
//! Garnet program that pulls any Rust crate transitively, an invisible
//! `unsafe` block OR a long-chain of FFI calls to a C library changes the
//! program's trust surface — Garnet's `@safe` claim doesn't hold at the
//! module level if the dep graph contains un-audited unsafe.
//!
//! `garnet audit` walks the dep graph and produces a structured report:
//!
//! - count of `unsafe` blocks per crate (wrap `cargo-geiger` when available;
//!   fall back to our own structural counter)
//! - count of `extern "C"` function declarations (raw FFI entry points)
//! - crates with `build.rs` that could execute arbitrary code at build time
//! - the composite hash of all deps, pinned to the manifest
//!
//! The CLI surface is:
//!
//!   $ garnet audit
//!   audit: 3 direct deps, 14 transitive
//!   unsafe block count: blake3=3  rusqlite=27  (see details below)
//!   extern "C": rusqlite=82  (all expected — SQLite C API)
//!   build.rs: rusqlite (bundled-sqlite), ed25519-dalek (zeroize feature)
//!   manifest pin: 9f3a...e1c2
//!
//! A caller MAY promote warnings to errors with `--fail-on-unsafe`.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// One crate's safety profile in the dep graph.
#[derive(Debug, Clone, Default)]
pub struct CrateProfile {
    pub name: String,
    pub version: String,
    pub unsafe_blocks: usize,
    pub extern_c_fns: usize,
    pub has_build_rs: bool,
    pub loc: usize,
}

impl CrateProfile {
    /// Heuristic "risk score" combining the three concerning factors. Used
    /// to sort the audit report so the riskiest deps surface first.
    pub fn risk_score(&self) -> u64 {
        let unsafe_weight = self.unsafe_blocks as u64 * 10;
        let extern_weight = self.extern_c_fns as u64 * 3;
        let build_weight = if self.has_build_rs { 25 } else { 0 };
        unsafe_weight + extern_weight + build_weight
    }
}

impl fmt::Display for CrateProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}: unsafe={} extern\"C\"={} build.rs={} loc={} score={}",
            self.name,
            self.version,
            self.unsafe_blocks,
            self.extern_c_fns,
            self.has_build_rs,
            self.loc,
            self.risk_score()
        )
    }
}

/// Aggregate audit result across a project's dep graph.
#[derive(Debug, Default)]
pub struct AuditReport {
    pub direct_deps: Vec<String>,
    pub profiles: BTreeMap<String, CrateProfile>,
    /// BLAKE3 hash pinning the sorted list of (name, version, risk_score)
    /// to the manifest. Changes if ANY dep's risk profile shifts between
    /// builds — useful for CI "no new unsafe" gates.
    pub manifest_pin: String,
}

impl AuditReport {
    pub fn total_unsafe(&self) -> usize {
        self.profiles.values().map(|p| p.unsafe_blocks).sum()
    }

    pub fn total_extern_c(&self) -> usize {
        self.profiles.values().map(|p| p.extern_c_fns).sum()
    }

    pub fn crates_with_build_rs(&self) -> Vec<&str> {
        self.profiles
            .values()
            .filter(|p| p.has_build_rs)
            .map(|p| p.name.as_str())
            .collect()
    }

    /// Crates sorted by descending risk_score for the human-readable report.
    pub fn risk_sorted(&self) -> Vec<&CrateProfile> {
        let mut v: Vec<&CrateProfile> = self.profiles.values().collect();
        v.sort_by(|a, b| b.risk_score().cmp(&a.risk_score()));
        v
    }

    /// Human-readable CLI output.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "garnet audit: {} direct deps, {} total in graph\n",
            self.direct_deps.len(),
            self.profiles.len()
        ));
        out.push_str(&format!(
            "  total unsafe blocks: {}\n  total extern \"C\" fns: {}\n",
            self.total_unsafe(),
            self.total_extern_c()
        ));
        let build_rs = self.crates_with_build_rs();
        if !build_rs.is_empty() {
            out.push_str(&format!(
                "  crates with build.rs (build-time arbitrary code): {}\n",
                build_rs.join(", ")
            ));
        }
        out.push_str(&format!("  manifest pin: {}\n", self.manifest_pin));
        out.push_str("\nrisk-sorted crate details:\n");
        for p in self.risk_sorted() {
            out.push_str(&format!("  {}\n", p));
        }
        out
    }

    /// Exit code for the `--fail-on-unsafe` mode: non-zero if any unsafe,
    /// build.rs, or extern "C" exceeds caller-supplied thresholds.
    pub fn fail_on_unsafe(
        &self,
        max_unsafe: usize,
        max_extern_c: usize,
        allow_build_rs: bool,
    ) -> i32 {
        if self.total_unsafe() > max_unsafe {
            return 2;
        }
        if self.total_extern_c() > max_extern_c {
            return 3;
        }
        if !allow_build_rs && !self.crates_with_build_rs().is_empty() {
            return 4;
        }
        0
    }
}

/// Best-effort structural scan — counts occurrences of `unsafe {`,
/// `extern "C"`, and the presence of `build.rs` in a crate's directory.
/// When `cargo-geiger` is installed, prefer that tool's richer output
/// (our scan is a fallback + a first-pass signal, not a substitute for
/// the real thing).
pub fn scan_crate(crate_dir: &Path) -> std::io::Result<CrateProfile> {
    let mut profile = CrateProfile {
        name: crate_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string(),
        version: read_version(crate_dir).unwrap_or_else(|| "?.?.?".to_string()),
        ..Default::default()
    };

    profile.has_build_rs = crate_dir.join("build.rs").exists();

    let src = crate_dir.join("src");
    if src.exists() {
        walk_and_count(&src, &mut profile)?;
    }
    Ok(profile)
}

fn walk_and_count(dir: &Path, profile: &mut CrateProfile) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_and_count(&path, profile)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let text = std::fs::read_to_string(&path)?;
            profile.loc += text.lines().count();
            // Count unsafe blocks. Exclude `unsafe fn` declarations from the
            // block count (those are declared-unsafe but without a block).
            for line in text.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("unsafe {") || trimmed.contains(" unsafe {") {
                    profile.unsafe_blocks += 1;
                }
                if trimmed.contains("extern \"C\"") {
                    profile.extern_c_fns += 1;
                }
            }
        }
    }
    Ok(())
}

fn read_version(crate_dir: &Path) -> Option<String> {
    let toml = std::fs::read_to_string(crate_dir.join("Cargo.toml")).ok()?;
    for line in toml.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("version") {
            if let Some(eq) = rest.find('=') {
                let v = rest[eq + 1..].trim();
                let v = v.trim_matches('"').trim_matches('\'');
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Pin the report's crate risk profiles into a stable BLAKE3 hash. A
/// change in ANY dep's risk profile produces a distinct hash; useful
/// for CI gates that refuse PRs which increase unsafe surface.
pub fn compute_manifest_pin(profiles: &BTreeMap<String, CrateProfile>) -> String {
    let mut h = blake3::Hasher::new();
    h.update(b"garnet-audit-v1");
    for (name, p) in profiles {
        h.update(name.as_bytes());
        h.update(p.version.as_bytes());
        h.update(&p.unsafe_blocks.to_le_bytes());
        h.update(&p.extern_c_fns.to_le_bytes());
        h.update(&[p.has_build_rs as u8]);
    }
    let out = *h.finalize().as_bytes();
    out.iter().map(|b| format!("{b:02x}")).collect()
}

/// Scan a workspace — each member becomes a CrateProfile. Entry point
/// used by `garnet audit`.
pub fn audit_workspace(workspace_root: &Path) -> std::io::Result<AuditReport> {
    let mut report = AuditReport::default();
    for entry in std::fs::read_dir(workspace_root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // Skip target directories and hidden dirs
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name == "target" || name.starts_with('.') {
            continue;
        }
        if !path.join("Cargo.toml").exists() {
            continue;
        }
        let profile = scan_crate(&path)?;
        report.direct_deps.push(profile.name.clone());
        report.profiles.insert(profile.name.clone(), profile);
    }
    report.manifest_pin = compute_manifest_pin(&report.profiles);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn profile(name: &str, unsafe_blocks: usize, extern_c: usize, build_rs: bool) -> CrateProfile {
        CrateProfile {
            name: name.into(),
            version: "1.0.0".into(),
            unsafe_blocks,
            extern_c_fns: extern_c,
            has_build_rs: build_rs,
            loc: 1000,
        }
    }

    #[test]
    fn risk_score_orders_expected() {
        let clean = profile("clean", 0, 0, false);
        let some_unsafe = profile("ffi", 5, 0, false);
        let heavy = profile("heavy", 10, 50, true);
        assert!(clean.risk_score() < some_unsafe.risk_score());
        assert!(some_unsafe.risk_score() < heavy.risk_score());
    }

    #[test]
    fn aggregate_counts() {
        let mut report = AuditReport::default();
        report.profiles.insert("a".into(), profile("a", 3, 10, false));
        report.profiles.insert("b".into(), profile("b", 5, 2, true));
        assert_eq!(report.total_unsafe(), 8);
        assert_eq!(report.total_extern_c(), 12);
        assert_eq!(report.crates_with_build_rs(), vec!["b"]);
    }

    #[test]
    fn fail_on_unsafe_thresholds() {
        let mut report = AuditReport::default();
        report.profiles.insert("a".into(), profile("a", 3, 0, false));
        assert_eq!(report.fail_on_unsafe(10, 10, true), 0); // under threshold
        assert_eq!(report.fail_on_unsafe(2, 10, true), 2); // unsafe over
    }

    #[test]
    fn fail_on_extern_c_threshold() {
        let mut report = AuditReport::default();
        report.profiles.insert("a".into(), profile("a", 0, 100, false));
        assert_eq!(report.fail_on_unsafe(10, 10, true), 3); // extern_c over
    }

    #[test]
    fn fail_on_build_rs_when_disallowed() {
        let mut report = AuditReport::default();
        report.profiles.insert("a".into(), profile("a", 0, 0, true));
        assert_eq!(report.fail_on_unsafe(10, 10, false), 4); // build.rs forbidden
        assert_eq!(report.fail_on_unsafe(10, 10, true), 0); // permitted
    }

    #[test]
    fn manifest_pin_is_deterministic() {
        let mut a: BTreeMap<String, CrateProfile> = BTreeMap::new();
        a.insert("x".into(), profile("x", 3, 2, false));
        a.insert("y".into(), profile("y", 0, 0, false));
        let pin_a = compute_manifest_pin(&a);
        let pin_b = compute_manifest_pin(&a);
        assert_eq!(pin_a, pin_b);
        assert_eq!(pin_a.len(), 64); // 32-byte BLAKE3 hex
    }

    #[test]
    fn manifest_pin_sensitive_to_unsafe_increase() {
        let mut a: BTreeMap<String, CrateProfile> = BTreeMap::new();
        a.insert("x".into(), profile("x", 3, 2, false));
        let pin1 = compute_manifest_pin(&a);
        a.insert("x".into(), profile("x", 4, 2, false)); // +1 unsafe
        let pin2 = compute_manifest_pin(&a);
        assert_ne!(pin1, pin2);
    }

    #[test]
    fn render_contains_expected_fields() {
        let mut report = AuditReport::default();
        report.direct_deps.push("a".into());
        report.profiles.insert("a".into(), profile("a", 2, 5, true));
        report.manifest_pin = "deadbeef".into();
        let out = report.render();
        assert!(out.contains("total unsafe blocks: 2"));
        assert!(out.contains("total extern \"C\" fns: 5"));
        assert!(out.contains("build.rs"));
        assert!(out.contains("deadbeef"));
    }

    #[test]
    fn risk_sorted_descending() {
        let mut report = AuditReport::default();
        report.profiles.insert("clean".into(), profile("clean", 0, 0, false));
        report.profiles.insert("heavy".into(), profile("heavy", 10, 50, true));
        report.profiles.insert("medium".into(), profile("medium", 3, 5, false));
        let sorted = report.risk_sorted();
        assert_eq!(sorted[0].name, "heavy");
        assert_eq!(sorted[2].name, "clean");
    }
}
