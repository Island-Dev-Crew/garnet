//! S32 Layer 1 (manifest side) — resolve the project [`Edition`] from
//! `Garnet.toml`.
//!
//! Canonical form (Mini-Spec §16.3): a `[project]` table with
//! `edition = "v1.0"`. Two legacy forms are accepted as **deprecated aliases**
//! (warn, never break):
//!   * the old `[package]` table name, and
//!   * the old `edition = "garnet-0.3"` value the pre-S32 template shipped.
//!
//! An *unknown* edition value is a hard error — pinning to an edition the
//! compiler does not implement must fail loudly rather than silently degrade.
//!
//! Garnet.toml is hand-parsed (the CLI carries no `toml` dependency), matching
//! `cmd::add::read_dependency_table`.

use garnet_parser::Edition;
use std::path::{Path, PathBuf};

/// The edition resolved for a project, plus an optional one-line advisory the
/// caller should print to stderr (a deprecation note for a legacy form).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEdition {
    pub edition: Edition,
    pub warning: Option<String>,
}

impl ResolvedEdition {
    fn current_no_warning() -> Self {
        ResolvedEdition {
            edition: Edition::current(),
            warning: None,
        }
    }
}

/// Resolve the edition for a source file by walking up to its `Garnet.toml`.
/// A bare-file run (no enclosing project) or a manifest without an `edition`
/// key uses the current default edition with no warning.
pub fn resolve_edition_for(source: &Path) -> Result<ResolvedEdition, String> {
    let Some(manifest) = find_manifest_for(source) else {
        return Ok(ResolvedEdition::current_no_warning());
    };
    match std::fs::read_to_string(&manifest) {
        Ok(text) => resolve_edition_from_manifest(&text),
        // An unreadable manifest is not an edition error; fall back to default.
        Err(_) => Ok(ResolvedEdition::current_no_warning()),
    }
}

/// Pure mapping from `Garnet.toml` text to an [`Edition`] (factored out so it is
/// directly testable without touching the filesystem).
pub fn resolve_edition_from_manifest(text: &str) -> Result<ResolvedEdition, String> {
    let Some((table, value)) = read_edition_field(text) else {
        return Ok(ResolvedEdition::current_no_warning());
    };
    let legacy_table = table.as_deref() == Some("package");

    // Legacy alias VALUE shipped by the pre-S32 template.
    if value == "garnet-0.3" {
        return Ok(ResolvedEdition {
            edition: Edition::current(),
            warning: Some(format!(
                "Garnet.toml: edition \"garnet-0.3\" is a deprecated alias for \"{0}\"; \
                 update to a `[project]` table with `edition = \"{0}\"` (Mini-Spec §16.3).",
                Edition::current().name()
            )),
        });
    }

    match Edition::parse(&value) {
        Ok(edition) => Ok(ResolvedEdition {
            edition,
            warning: legacy_table.then(|| {
                "Garnet.toml: the `[package]` table is a deprecated alias for `[project]`; \
                 rename it (Mini-Spec §16.3)."
                    .to_string()
            }),
        }),
        Err(e) => Err(format!(
            "Garnet.toml: {e}. Pin a known edition under `[project]` \
             (e.g. `edition = \"{}\"`).",
            Edition::current().name()
        )),
    }
}

/// Walk upward from `source`'s parent looking for `Garnet.toml`.
fn find_manifest_for(source: &Path) -> Option<PathBuf> {
    let mut cur = source
        .canonicalize()
        .ok()
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .or_else(|| source.parent().map(Path::to_path_buf))?;
    loop {
        let candidate = cur.join("Garnet.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// Find the first `edition = "..."` assignment and the most recent table header
/// preceding it. Returns `(table_name, value)`; `table_name` is `None` if the
/// key appears before any table header.
fn read_edition_field(text: &str) -> Option<(Option<String>, String)> {
    let mut current_table: Option<String> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_table = Some(
                line.trim_matches(|c| c == '[' || c == ']')
                    .trim()
                    .to_string(),
            );
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            if key.trim() == "edition" {
                // Strip surrounding quotes and any trailing inline comment.
                let val = val.split('#').next().unwrap_or(val).trim();
                let val = val.trim_matches('"').trim().to_string();
                return Some((current_table.clone(), val));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_project_v1_0_has_no_warning() {
        let text = "[project]\nname = \"x\"\nedition = \"v1.0\"\n";
        let r = resolve_edition_from_manifest(text).unwrap();
        assert_eq!(r.edition, Edition::V1_0);
        assert!(r.warning.is_none());
    }

    #[test]
    fn project_v2_0_selects_next() {
        let text = "[project]\nedition = \"v2.0\"\n";
        let r = resolve_edition_from_manifest(text).unwrap();
        assert_eq!(r.edition, Edition::Next);
        assert!(r.warning.is_none());
    }

    #[test]
    fn legacy_alias_value_maps_to_v1_0_with_warning() {
        let text = "[package]\nname = \"x\"\nedition = \"garnet-0.3\"\n";
        let r = resolve_edition_from_manifest(text).unwrap();
        assert_eq!(r.edition, Edition::V1_0);
        let w = r.warning.expect("expected a deprecation warning");
        assert!(w.contains("garnet-0.3") && w.contains("v1.0"));
    }

    #[test]
    fn legacy_package_table_with_canonical_value_warns_about_table() {
        let text = "[package]\nedition = \"v1.0\"\n";
        let r = resolve_edition_from_manifest(text).unwrap();
        assert_eq!(r.edition, Edition::V1_0);
        assert!(r.warning.unwrap().contains("[package]"));
    }

    #[test]
    fn unknown_edition_is_a_hard_error() {
        let text = "[project]\nedition = \"v9.9\"\n";
        let err = resolve_edition_from_manifest(text).unwrap_err();
        assert!(
            err.contains("unknown edition") && err.contains("v9.9"),
            "got: {err}"
        );
    }

    #[test]
    fn missing_edition_key_defaults_quietly() {
        let text = "[project]\nname = \"x\"\n";
        let r = resolve_edition_from_manifest(text).unwrap();
        assert_eq!(r.edition, Edition::current());
        assert!(r.warning.is_none());
    }

    #[test]
    fn inline_comment_after_value_is_stripped() {
        let text = "[project]\nedition = \"v1.0\"   # canonical\n";
        let r = resolve_edition_from_manifest(text).unwrap();
        assert_eq!(r.edition, Edition::V1_0);
    }
}
