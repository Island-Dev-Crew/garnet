//! `xtask truth` — machine-truth generator + public-surface drift guard
//! (W-REBUILD RB-0a, punchlist Part C).
//!
//! Emits `docs/truth.json` from machine-derivable sources and stamps the
//! values between `<!-- truth:KEY -->VALUE<!-- /truth -->` markers on the
//! public surfaces (`README.md`, `FAQ.md`). `truth --check` exits non-zero
//! on any mismatch between machine truth, `docs/truth.json`, and the
//! stamped surfaces — so public-number drift becomes a failing command
//! instead of a manual audit.
//!
//! Field provenance (each field is derived, never hand-entered):
//! - `version` — `[workspace.package].version` in the root `Cargo.toml`.
//! - `primitive_count` / `primitives_by_layer` — `garnet_stdlib::registry::all_prims()`.
//! - `tracked_slices` / `readiness_pct` — the readiness reporters
//!   (`scripts/garnet_readiness_status.py`, `scripts/garnet_mit_readiness_status.py`).
//! - `latest_tag` — `git tag --list 'v*' --sort=-v:refname`, first entry.
//! - `workspace_tests` — measured by actually running
//!   `cargo test --workspace --no-fail-fast` (the same source CI trusts),
//!   recorded with the commit it was measured at. `--skip-tests` carries the
//!   previous measurement forward unchanged (provenance preserved).
//!
//! Deliberate omission: `security_test_count`. The historical public
//! "136 security tests" figure has no recoverable derivation (it entered the
//! site undocumented). Re-stamping an unverifiable number would automate
//! drift behind a green checkmark; the field is omitted and the omission is
//! recorded inside `docs/truth.json` itself.
//!
//! Design note (queued, NOT part of RB-0a): the truth-guard family extends
//! from numbers to semantic claims via caps-claims-as-doctests — every
//! `@caps`/`@bounded` claim in docs becomes a compiled, trap-tested example.
//! Tracked in `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md` (RB-0a
//! design note).
//!
//! Wiring `truth --check` into CI is a gate change and stays Jon-gated; this
//! module only provides the command.

use std::collections::BTreeMap;
use std::process::Command;

pub const TRUTH_JSON_PATH: &str = "docs/truth.json";
pub const STAMPED_FILES: &[&str] = &["README.md", "FAQ.md"];

const MARKER_OPEN: &str = "<!-- truth:";
const MARKER_OPEN_END: &str = " -->";
const MARKER_CLOSE: &str = "<!-- /truth -->";

/// One stamped-surface or truth.json disagreement.
#[derive(Debug, PartialEq, Eq)]
pub struct Mismatch {
    pub location: String,
    pub key: String,
    pub expected: String,
    pub found: String,
}

impl std::fmt::Display for Mismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: truth:{} expected `{}`, found `{}`",
            self.location, self.key, self.expected, self.found
        )
    }
}

/// Entry point for `cargo run -p xtask -- truth [...]`. Returns process exit code.
pub fn run(args: &[String]) -> i32 {
    let check = args.iter().any(|a| a == "--check");
    let skip_tests = args.iter().any(|a| a == "--skip-tests");
    let with_tests = args.iter().any(|a| a == "--with-tests");
    for a in args {
        if !matches!(a.as_str(), "--check" | "--skip-tests" | "--with-tests") {
            eprintln!("unknown truth flag: {a}");
            return 2;
        }
    }
    let result = if check {
        run_check(with_tests)
    } else {
        run_generate(skip_tests)
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("xtask truth: error: {e}");
            2
        }
    }
}

// ---------------------------------------------------------------------------
// generate
// ---------------------------------------------------------------------------

fn run_generate(skip_tests: bool) -> Result<i32, String> {
    let truth = derive_truth(if skip_tests {
        TestCounts::CarryOver(load_previous_tests()?)
    } else {
        TestCounts::Measure
    })?;

    let json = truth_to_json(&truth);
    let rendered = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())? + "\n";
    std::fs::write(TRUTH_JSON_PATH, &rendered)
        .map_err(|e| format!("writing {TRUTH_JSON_PATH}: {e}"))?;
    println!("wrote {TRUTH_JSON_PATH}");

    let values = marker_values(&truth);
    for path in STAMPED_FILES {
        let text = std::fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
        let (stamped, keys) = stamp_text(&text, &values).map_err(|e| format!("{path}: {e}"))?;
        if stamped != text {
            std::fs::write(path, &stamped).map_err(|e| format!("writing {path}: {e}"))?;
            println!("stamped {path}: {}", keys.join(", "));
        } else {
            println!("{path}: already current ({} markers)", keys.len());
        }
    }
    Ok(0)
}

// ---------------------------------------------------------------------------
// check
// ---------------------------------------------------------------------------

fn run_check(with_tests: bool) -> Result<i32, String> {
    let committed: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(TRUTH_JSON_PATH)
            .map_err(|e| format!("reading {TRUTH_JSON_PATH}: {e} (run `xtask truth` first)"))?,
    )
    .map_err(|e| format!("parsing {TRUTH_JSON_PATH}: {e}"))?;

    let live = derive_truth(if with_tests {
        TestCounts::Measure
    } else {
        TestCounts::CarryOver(load_previous_tests()?)
    })?;
    let mut mismatches: Vec<Mismatch> = Vec::new();

    // 1. truth.json vs live machine truth (the file itself can drift).
    let live_json = truth_to_json(&live);
    let check_keys: &[&str] = if with_tests {
        &[
            "version",
            "primitive_count",
            "primitives_by_layer",
            "tracked_slices",
            "readiness_pct",
            "latest_tag",
            "workspace_tests",
        ]
    } else {
        &[
            "version",
            "primitive_count",
            "primitives_by_layer",
            "tracked_slices",
            "readiness_pct",
            "latest_tag",
        ]
    };
    for key in check_keys {
        let exp = live_json.get(key);
        let got = committed.get(key);
        // `workspace_tests` provenance fields legitimately differ between
        // measurements; compare only the counts.
        let (exp_cmp, got_cmp) = if *key == "workspace_tests" {
            (
                exp.map(|v| (v.get("passed").cloned(), v.get("failed").cloned())),
                got.map(|v| (v.get("passed").cloned(), v.get("failed").cloned())),
            )
        } else {
            (
                exp.map(|v| (Some(v.clone()), None)),
                got.map(|v| (Some(v.clone()), None)),
            )
        };
        if exp_cmp != got_cmp {
            mismatches.push(Mismatch {
                location: TRUTH_JSON_PATH.to_string(),
                key: (*key).to_string(),
                expected: exp
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "<absent>".into()),
                found: got
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "<absent>".into()),
            });
        }
    }

    // 2. Stamped surfaces vs truth.json values.
    let values = marker_values_from_json(&committed)?;
    for path in STAMPED_FILES {
        let text = std::fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
        mismatches.extend(check_text(path, &text, &values).map_err(|e| format!("{path}: {e}"))?);
    }

    if mismatches.is_empty() {
        println!(
            "truth --check: ok ({} fields vs machine truth, {} stamped surfaces)",
            check_keys.len(),
            STAMPED_FILES.len()
        );
        Ok(0)
    } else {
        eprintln!("truth --check: {} mismatch(es):", mismatches.len());
        for m in &mismatches {
            eprintln!("  {m}");
        }
        Ok(1)
    }
}

// ---------------------------------------------------------------------------
// derivation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WorkspaceTests {
    pub passed: u64,
    pub failed: u64,
    pub measured_at_commit: String,
}

enum TestCounts {
    Measure,
    CarryOver(Option<WorkspaceTests>),
}

#[derive(Debug)]
pub struct Truth {
    pub version: String,
    pub primitive_count: u64,
    pub primitives_by_layer: BTreeMap<String, u64>,
    pub tracked_slices: String,
    pub readiness_pct: f64,
    pub latest_tag: String,
    pub workspace_tests: Option<WorkspaceTests>,
    pub generated_at_commit: String,
}

fn derive_truth(tests: TestCounts) -> Result<Truth, String> {
    let version = parse_workspace_version(
        &std::fs::read_to_string("Cargo.toml").map_err(|e| format!("reading Cargo.toml: {e}"))?,
    )?;

    let prims = garnet_stdlib::registry::all_prims();
    let primitive_count = prims.len() as u64;
    let mut primitives_by_layer: BTreeMap<String, u64> = BTreeMap::new();
    for meta in prims.values() {
        *primitives_by_layer
            .entry(meta.layer.as_str().to_string())
            .or_insert(0) += 1;
    }

    let readiness = run_json(
        "python3",
        &["scripts/garnet_readiness_status.py", "--format", "json"],
    )?;
    let tracked_slices = format!(
        "{}/{}",
        readiness
            .get("completed_slices")
            .and_then(|v| v.as_u64())
            .ok_or("readiness: no completed_slices")?,
        readiness
            .get("total_slices")
            .and_then(|v| v.as_u64())
            .ok_or("readiness: no total_slices")?,
    );
    let mit = run_json(
        "python3",
        &["scripts/garnet_mit_readiness_status.py", "--format", "json"],
    )?;
    let readiness_pct = mit
        .get("completion_percent")
        .and_then(|v| v.as_f64())
        .ok_or("mit readiness: no completion_percent")?;

    let latest_tag = run_capture("git", &["tag", "--list", "v*", "--sort=-v:refname"])?
        .lines()
        .next()
        .ok_or("no v* tags found")?
        .trim()
        .to_string();

    let commit = current_commit()?;
    let workspace_tests = match tests {
        TestCounts::Measure => {
            println!("measuring workspace tests (cargo test --workspace --no-fail-fast)…");
            let out = Command::new("cargo")
                .args(["test", "--workspace", "--no-fail-fast"])
                .output()
                .map_err(|e| format!("spawning cargo test: {e}"))?;
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let counts = parse_counts(&stdout, &stderr);
            if counts.passed == 0 {
                return Err("cargo test reported 0 passed — refusing to record".into());
            }
            Some(WorkspaceTests {
                passed: counts.passed,
                failed: counts.failed,
                measured_at_commit: commit.clone(),
            })
        }
        TestCounts::CarryOver(prev) => prev,
    };

    Ok(Truth {
        version,
        primitive_count,
        primitives_by_layer,
        tracked_slices,
        readiness_pct,
        latest_tag,
        workspace_tests,
        generated_at_commit: commit,
    })
}

/// Parse `[workspace.package] version = "…"` from the root Cargo.toml text.
pub fn parse_workspace_version(toml: &str) -> Result<String, String> {
    let mut in_section = false;
    for line in toml.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_section = t == "[workspace.package]";
            continue;
        }
        if in_section {
            if let Some(rest) = t.strip_prefix("version") {
                let rest = rest.trim_start();
                if let Some(rest) = rest.strip_prefix('=') {
                    let v = rest.trim().trim_matches('"');
                    if !v.is_empty() {
                        return Ok(v.to_string());
                    }
                }
            }
        }
    }
    Err("no [workspace.package] version in Cargo.toml".into())
}

fn load_previous_tests() -> Result<Option<WorkspaceTests>, String> {
    let Ok(text) = std::fs::read_to_string(TRUTH_JSON_PATH) else {
        return Ok(None);
    };
    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("parsing {TRUTH_JSON_PATH}: {e}"))?;
    let Some(wt) = json.get("workspace_tests") else {
        return Ok(None);
    };
    Ok(Some(WorkspaceTests {
        passed: wt
            .get("passed")
            .and_then(|v| v.as_u64())
            .ok_or("workspace_tests.passed missing")?,
        failed: wt
            .get("failed")
            .and_then(|v| v.as_u64())
            .ok_or("workspace_tests.failed missing")?,
        measured_at_commit: wt
            .get("measured_at_commit")
            .and_then(|v| v.as_str())
            .ok_or("workspace_tests.measured_at_commit missing")?
            .to_string(),
    }))
}

fn truth_to_json(t: &Truth) -> serde_json::Value {
    let mut root = serde_json::Map::new();
    root.insert(
        "_generated_by".into(),
        "cargo run -p xtask -- truth — do not edit by hand; `xtask truth --check` fails on drift"
            .into(),
    );
    root.insert(
        "generated_at_commit".into(),
        t.generated_at_commit.clone().into(),
    );
    root.insert("latest_tag".into(), t.latest_tag.clone().into());
    root.insert(
        "omissions".into(),
        serde_json::json!({
            "security_test_count":
                "No trusted derivation exists for the historical public '136 security tests' \
                 figure (it entered the site undocumented). Re-stamping an unverifiable number \
                 would automate drift; the public row is removed/replaced by RB-0d instead."
        }),
    );
    root.insert("primitive_count".into(), t.primitive_count.into());
    root.insert(
        "primitives_by_layer".into(),
        serde_json::Value::Object(
            t.primitives_by_layer
                .iter()
                .map(|(k, v)| (k.clone(), (*v).into()))
                .collect(),
        ),
    );
    root.insert("readiness_pct".into(), t.readiness_pct.into());
    root.insert("tracked_slices".into(), t.tracked_slices.clone().into());
    root.insert("version".into(), t.version.clone().into());
    if let Some(wt) = &t.workspace_tests {
        root.insert(
            "workspace_tests".into(),
            serde_json::json!({
                "passed": wt.passed,
                "failed": wt.failed,
                "measured_at_commit": wt.measured_at_commit,
                "note": "Measured by `cargo test --workspace --no-fail-fast` during truth \
                         generation; refreshed only when xtask truth runs without --skip-tests."
            }),
        );
    }
    serde_json::Value::Object(root)
}

/// The key→value map the markers stamp/check against.
fn marker_values(t: &Truth) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert("version".into(), t.version.clone());
    m.insert("primitive_count".into(), t.primitive_count.to_string());
    m.insert("tracked_slices".into(), t.tracked_slices.clone());
    m.insert("readiness_pct".into(), format!("{}", t.readiness_pct));
    m.insert("latest_tag".into(), t.latest_tag.clone());
    if let Some(wt) = &t.workspace_tests {
        m.insert("workspace_test_count".into(), wt.passed.to_string());
    }
    m
}

fn marker_values_from_json(json: &serde_json::Value) -> Result<BTreeMap<String, String>, String> {
    let mut m = BTreeMap::new();
    let s = |v: &serde_json::Value| -> String {
        match v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    };
    for key in [
        "version",
        "primitive_count",
        "tracked_slices",
        "readiness_pct",
        "latest_tag",
    ] {
        let v = json
            .get(key)
            .ok_or_else(|| format!("{TRUTH_JSON_PATH}: missing `{key}`"))?;
        m.insert(key.to_string(), s(v));
    }
    if let Some(wt) = json.get("workspace_tests") {
        if let Some(p) = wt.get("passed") {
            m.insert("workspace_test_count".into(), s(p));
        }
    }
    Ok(m)
}

// ---------------------------------------------------------------------------
// marker engine (pure)
// ---------------------------------------------------------------------------

/// Replace every `<!-- truth:KEY -->…<!-- /truth -->` span's interior with
/// the current value for KEY. Returns the new text plus the keys stamped.
/// Unknown keys and unterminated markers are errors, never silently skipped.
pub fn stamp_text(
    text: &str,
    values: &BTreeMap<String, String>,
) -> Result<(String, Vec<String>), String> {
    let mut out = String::with_capacity(text.len());
    let mut keys = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(MARKER_OPEN) {
        let after_open = &rest[start + MARKER_OPEN.len()..];
        let key_end = after_open
            .find(MARKER_OPEN_END)
            .ok_or_else(|| format!("unterminated `{MARKER_OPEN}` marker"))?;
        let key = after_open[..key_end].trim().to_string();
        let value = values
            .get(&key)
            .ok_or_else(|| format!("unknown truth key `{key}` (known: {:?})", values.keys()))?;
        let body = &after_open[key_end + MARKER_OPEN_END.len()..];
        let close = body
            .find(MARKER_CLOSE)
            .ok_or_else(|| format!("truth:{key}: missing `{MARKER_CLOSE}`"))?;
        out.push_str(&rest[..start]);
        out.push_str(MARKER_OPEN);
        out.push_str(&key);
        out.push_str(MARKER_OPEN_END);
        out.push_str(value);
        out.push_str(MARKER_CLOSE);
        keys.push(key);
        rest = &body[close + MARKER_CLOSE.len()..];
    }
    out.push_str(rest);
    Ok((out, keys))
}

/// Collect mismatches between marker interiors and the expected values.
pub fn check_text(
    location: &str,
    text: &str,
    values: &BTreeMap<String, String>,
) -> Result<Vec<Mismatch>, String> {
    let mut mismatches = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(MARKER_OPEN) {
        let after_open = &rest[start + MARKER_OPEN.len()..];
        let key_end = after_open
            .find(MARKER_OPEN_END)
            .ok_or_else(|| format!("unterminated `{MARKER_OPEN}` marker"))?;
        let key = after_open[..key_end].trim().to_string();
        let expected = values
            .get(&key)
            .ok_or_else(|| format!("unknown truth key `{key}` (known: {:?})", values.keys()))?;
        let body = &after_open[key_end + MARKER_OPEN_END.len()..];
        let close = body
            .find(MARKER_CLOSE)
            .ok_or_else(|| format!("truth:{key}: missing `{MARKER_CLOSE}`"))?;
        let found = &body[..close];
        if found != expected {
            mismatches.push(Mismatch {
                location: location.to_string(),
                key,
                expected: expected.clone(),
                found: found.to_string(),
            });
        }
        rest = &body[close + MARKER_CLOSE.len()..];
    }
    Ok(mismatches)
}

// ---------------------------------------------------------------------------
// process helpers
// ---------------------------------------------------------------------------

fn run_capture(prog: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(prog)
        .args(args)
        .output()
        .map_err(|e| format!("spawning {prog}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{prog} {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn run_json(prog: &str, args: &[&str]) -> Result<serde_json::Value, String> {
    let raw = run_capture(prog, args)?;
    serde_json::from_str(&raw).map_err(|e| format!("{prog} {}: bad JSON: {e}", args.join(" ")))
}

fn current_commit() -> Result<String, String> {
    let sha = run_capture("git", &["rev-parse", "--short", "HEAD"])?
        .trim()
        .to_string();
    let dirty = !run_capture("git", &["status", "--porcelain"])?
        .trim()
        .is_empty();
    Ok(if dirty { format!("{sha}-dirty") } else { sha })
}

/// Sum every `test result: ok. P passed; F failed` line in stdout/stderr.
/// (Shared with `seven-run`, which trusts the same parse.)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Counts {
    pub passed: u64,
    pub failed: u64,
}

pub fn parse_counts(stdout: &str, stderr: &str) -> Counts {
    let mut passed = 0u64;
    let mut failed = 0u64;
    for stream in [stdout, stderr] {
        for line in stream.lines() {
            if let Some(rest) = line.trim().strip_prefix("test result:") {
                let mut tokens = rest.split_whitespace();
                while let Some(t) = tokens.next() {
                    if let Ok(n) = t.parse::<u64>() {
                        if let Some(label) = tokens.next() {
                            match label.trim_end_matches(';') {
                                "passed" => passed += n,
                                "failed" => failed += n,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    Counts { passed, failed }
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn vals(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn stamp_replaces_stale_value_and_preserves_surroundings() {
        let text = "before <!-- truth:primitive_count -->24<!-- /truth --> prims after";
        let (out, keys) = stamp_text(text, &vals(&[("primitive_count", "80")])).unwrap();
        assert_eq!(
            out,
            "before <!-- truth:primitive_count -->80<!-- /truth --> prims after"
        );
        assert_eq!(keys, vec!["primitive_count"]);
    }

    #[test]
    fn stamp_is_idempotent() {
        let v = vals(&[("primitive_count", "80")]);
        let text = "x <!-- truth:primitive_count -->80<!-- /truth --> y";
        let (once, _) = stamp_text(text, &v).unwrap();
        let (twice, _) = stamp_text(&once, &v).unwrap();
        assert_eq!(once, text);
        assert_eq!(once, twice);
    }

    #[test]
    fn stamp_handles_multiple_markers_in_order() {
        let v = vals(&[("version", "0.8.1"), ("latest_tag", "v0.8.1")]);
        let text = "v=<!-- truth:version -->old<!-- /truth --> tag=<!-- truth:latest_tag -->old<!-- /truth -->";
        let (out, keys) = stamp_text(text, &v).unwrap();
        assert_eq!(
            out,
            "v=<!-- truth:version -->0.8.1<!-- /truth --> tag=<!-- truth:latest_tag -->v0.8.1<!-- /truth -->"
        );
        assert_eq!(keys, vec!["version", "latest_tag"]);
    }

    #[test]
    fn stamp_rejects_unknown_key() {
        let err =
            stamp_text("<!-- truth:bogus -->1<!-- /truth -->", &vals(&[("a", "1")])).unwrap_err();
        assert!(err.contains("unknown truth key `bogus`"), "{err}");
    }

    #[test]
    fn stamp_rejects_unterminated_marker() {
        let err = stamp_text(
            "<!-- truth:version -->0.8.1",
            &vals(&[("version", "0.8.1")]),
        )
        .unwrap_err();
        assert!(err.contains("missing"), "{err}");
    }

    #[test]
    fn check_detects_planted_mismatch() {
        let v = vals(&[("primitive_count", "80")]);
        let text = "n=<!-- truth:primitive_count -->81<!-- /truth -->";
        let ms = check_text("README.md", text, &v).unwrap();
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0].key, "primitive_count");
        assert_eq!(ms[0].expected, "80");
        assert_eq!(ms[0].found, "81");
        assert_eq!(ms[0].location, "README.md");
    }

    #[test]
    fn check_passes_on_current_text() {
        let v = vals(&[("primitive_count", "80")]);
        let text = "n=<!-- truth:primitive_count -->80<!-- /truth -->";
        assert!(check_text("README.md", text, &v).unwrap().is_empty());
    }

    #[test]
    fn workspace_version_parses_from_workspace_package_section() {
        let toml = "[workspace]\nmembers=[]\n\n[workspace.package]\nversion = \"0.8.1\"\nedition = \"2021\"\n";
        assert_eq!(parse_workspace_version(toml).unwrap(), "0.8.1");
    }

    #[test]
    fn workspace_version_ignores_other_sections() {
        let toml = "[package]\nversion = \"9.9.9\"\n";
        assert!(parse_workspace_version(toml).is_err());
    }

    #[test]
    fn parse_counts_sums_multiple_result_lines() {
        let out = "test result: ok. 23 passed; 0 failed; 0 ignored\nnoise\ntest result: ok. 7 passed; 1 failed; 0 ignored\n";
        let c = parse_counts(out, "");
        assert_eq!(
            c,
            Counts {
                passed: 30,
                failed: 1
            }
        );
    }
}
