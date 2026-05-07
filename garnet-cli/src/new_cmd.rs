//! `garnet new` — project scaffolding (v4.2 Phase 6B).
//!
//! `garnet new --template <name> <dir>` creates `<dir>/` populated from
//! one of three embedded templates:
//!
//!   cli                — a minimal CLI application
//!   web-api            — an HTTP/1.1 service (MVP 5 shape)
//!   agent-orchestrator — Researcher / Synthesizer / Reviewer (MVP 6)
//!
//! Templates are embedded in the `garnet` binary via `include_str!` so
//! `garnet new` is a single-binary operation — no network, no sidecar,
//! nothing to "first install." This matches the Phase 6D install-to-
//! first-run-in-under-2-minutes target.
//!
//! The project name (`<dir>`'s basename) is substituted into every
//! template file at write time via a literal `{{name}}` placeholder.

use std::fs;
use std::path::{Path, PathBuf};

/// The three canonical templates shipped with v4.2. Each tuple is
/// `(relative_path_within_project, template_content_with_{{name}}_placeholder)`.
struct TemplateSpec {
    /// Human-readable template key, as passed to `--template <name>`.
    key: &'static str,
    /// Short description printed after creation.
    description: &'static str,
    /// Files to emit. Paths use forward slashes; converted per-OS at write time.
    files: &'static [(&'static str, &'static str)],
}

const CLI_TEMPLATE: TemplateSpec = TemplateSpec {
    key: "cli",
    description: "a minimal CLI application",
    files: &[
        ("Garnet.toml", include_str!("../templates/cli/Garnet.toml")),
        (
            "src/main.garnet",
            include_str!("../templates/cli/src/main.garnet"),
        ),
        (
            "tests/test_main.garnet",
            include_str!("../templates/cli/tests/test_main.garnet"),
        ),
        (".gitignore", include_str!("../templates/cli/.gitignore")),
        ("README.md", include_str!("../templates/cli/README.md")),
    ],
};

const WEB_API_TEMPLATE: TemplateSpec = TemplateSpec {
    key: "web-api",
    description: "an HTTP/1.1 API service",
    files: &[
        (
            "Garnet.toml",
            include_str!("../templates/web-api/Garnet.toml"),
        ),
        (
            "src/main.garnet",
            include_str!("../templates/web-api/src/main.garnet"),
        ),
        (
            "tests/test_main.garnet",
            include_str!("../templates/web-api/tests/test_main.garnet"),
        ),
        (
            ".gitignore",
            include_str!("../templates/web-api/.gitignore"),
        ),
        ("README.md", include_str!("../templates/web-api/README.md")),
    ],
};

const AGENT_TEMPLATE: TemplateSpec = TemplateSpec {
    key: "agent-orchestrator",
    description: "a Researcher / Synthesizer / Reviewer orchestrator",
    files: &[
        (
            "Garnet.toml",
            include_str!("../templates/agent-orchestrator/Garnet.toml"),
        ),
        (
            "src/main.garnet",
            include_str!("../templates/agent-orchestrator/src/main.garnet"),
        ),
        (
            "tests/test_main.garnet",
            include_str!("../templates/agent-orchestrator/tests/test_main.garnet"),
        ),
        (
            ".gitignore",
            include_str!("../templates/agent-orchestrator/.gitignore"),
        ),
        (
            "README.md",
            include_str!("../templates/agent-orchestrator/README.md"),
        ),
    ],
};

/// All bundled templates in deterministic order.
const TEMPLATES: &[&TemplateSpec] = &[&CLI_TEMPLATE, &WEB_API_TEMPLATE, &AGENT_TEMPLATE];

/// Result of a successful `garnet new` invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewProjectReport {
    /// Resolved absolute path of the created project root.
    pub root: PathBuf,
    /// Template key used.
    pub template: String,
    /// Relative paths (in canonical forward-slash form) of every file written.
    pub files_written: Vec<String>,
}

/// Errors surfaced from `create_project`. Intentionally `String`-valued so
/// the CLI can format them without dragging `thiserror` into the API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewProjectError {
    UnknownTemplate {
        got: String,
        available: Vec<String>,
    },
    TargetExists {
        path: PathBuf,
    },
    InvalidProjectName {
        name: String,
        reason: String,
    },
    Io {
        step: String,
        path: PathBuf,
        msg: String,
    },
}

impl std::fmt::Display for NewProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewProjectError::UnknownTemplate { got, available } => {
                write!(
                    f,
                    "unknown template `{got}`; available: {}",
                    available.join(", ")
                )
            }
            NewProjectError::TargetExists { path } => {
                write!(f, "target directory `{}` already exists", path.display())
            }
            NewProjectError::InvalidProjectName { name, reason } => {
                write!(f, "invalid project name `{name}`: {reason}")
            }
            NewProjectError::Io { step, path, msg } => {
                write!(f, "{step} `{}` failed: {msg}", path.display())
            }
        }
    }
}

impl std::error::Error for NewProjectError {}

/// Return the list of template keys, useful for help text.
pub fn available_templates() -> Vec<&'static str> {
    TEMPLATES.iter().map(|t| t.key).collect()
}

/// Return the list of (key, description) pairs — renders nicely in help
/// output where the reader wants a one-line summary per template.
pub fn template_descriptions() -> Vec<(&'static str, &'static str)> {
    TEMPLATES.iter().map(|t| (t.key, t.description)).collect()
}

/// Create a new Garnet project at `target_dir` using the named template.
/// `target_dir` must not already exist; the project name derived from the
/// last path component is substituted for `{{name}}` in every emitted file.
pub fn create_project(
    template_key: &str,
    target_dir: &Path,
) -> Result<NewProjectReport, NewProjectError> {
    let template = TEMPLATES
        .iter()
        .find(|t| t.key == template_key)
        .ok_or_else(|| NewProjectError::UnknownTemplate {
            got: template_key.to_string(),
            available: available_templates()
                .into_iter()
                .map(String::from)
                .collect(),
        })?;

    if target_dir.exists() {
        return Err(NewProjectError::TargetExists {
            path: target_dir.to_path_buf(),
        });
    }

    // Derive and validate the project name from the last path component.
    let name = target_dir
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| NewProjectError::InvalidProjectName {
            name: target_dir.display().to_string(),
            reason: "target path has no final component".into(),
        })?;
    validate_project_name(name)?;

    // Create the project root + all required sub-directories.
    fs::create_dir_all(target_dir).map_err(|e| NewProjectError::Io {
        step: "create_dir_all".into(),
        path: target_dir.to_path_buf(),
        msg: e.to_string(),
    })?;

    let mut files_written = Vec::with_capacity(template.files.len());
    for (rel_path, raw_contents) in template.files {
        let rendered = raw_contents.replace("{{name}}", name);
        let full = target_dir.join(rel_path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).map_err(|e| NewProjectError::Io {
                step: "create_dir_all".into(),
                path: parent.to_path_buf(),
                msg: e.to_string(),
            })?;
        }
        fs::write(&full, rendered).map_err(|e| NewProjectError::Io {
            step: "write".into(),
            path: full.clone(),
            msg: e.to_string(),
        })?;
        files_written.push((*rel_path).to_string());
    }

    Ok(NewProjectReport {
        root: target_dir.to_path_buf(),
        template: template.key.to_string(),
        files_written,
    })
}

/// Enforce a Cargo-like project-name policy so a fresh project integrates
/// cleanly with downstream tooling:
/// - 1..=64 chars
/// - ASCII alphanumerics, `_` and `-`
/// - must start with a letter
/// - cannot collide with the handful of reserved Garnet keywords that would
///   be awkward as a package name (`safe`, `def`, `fn`, `actor`, `struct`,
///   `enum`, `trait`, `impl`, `module`).
fn validate_project_name(name: &str) -> Result<(), NewProjectError> {
    const RESERVED: &[&str] = &[
        "safe", "def", "fn", "actor", "struct", "enum", "trait", "impl", "module",
    ];
    let err = |reason: &str| NewProjectError::InvalidProjectName {
        name: name.to_string(),
        reason: reason.to_string(),
    };
    if name.is_empty() {
        return Err(err("name is empty"));
    }
    if name.len() > 64 {
        return Err(err("name exceeds 64 characters"));
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() {
        return Err(err("name must start with an ASCII letter"));
    }
    for c in chars {
        if !(c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(err(&format!(
                "name contains invalid character `{c}` — allowed: ASCII alphanumerics, _, -"
            )));
        }
    }
    if RESERVED.iter().any(|r| r.eq_ignore_ascii_case(name)) {
        return Err(err("name collides with a reserved Garnet keyword"));
    }
    Ok(())
}

/// Short "what-to-do-next" block printed after a successful `garnet new`.
/// Returns the rendered string so it can be tested deterministically.
pub fn next_steps_hint(report: &NewProjectReport) -> String {
    let dir = report.root.display();
    format!("\nNext steps:\n  cd {dir}\n  garnet run src/main.garnet\n  garnet test\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn available_templates_are_three_canonical() {
        let mut names = available_templates();
        names.sort();
        assert_eq!(names, vec!["agent-orchestrator", "cli", "web-api"]);
    }

    #[test]
    fn template_descriptions_populated_and_non_empty() {
        let descriptions = template_descriptions();
        assert_eq!(descriptions.len(), 3);
        for (key, desc) in descriptions {
            assert!(!key.is_empty(), "template key must be non-empty");
            assert!(
                !desc.is_empty(),
                "template `{key}` description must be non-empty"
            );
        }
    }

    #[test]
    fn cli_template_creates_all_files() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("my_app");
        let report = create_project("cli", &target).unwrap();
        assert_eq!(report.template, "cli");
        // Every file listed in the template spec must exist on disk.
        for rel in &report.files_written {
            assert!(
                target.join(rel).exists(),
                "expected file {rel} to exist under {}",
                target.display()
            );
        }
        // Garnet.toml must carry the substituted project name.
        let toml = std::fs::read_to_string(target.join("Garnet.toml")).unwrap();
        assert!(
            toml.contains("name = \"my_app\""),
            "expected `name = \"my_app\"` in Garnet.toml; got:\n{toml}"
        );
        // {{name}} placeholder must not leak through.
        assert!(!toml.contains("{{name}}"));
    }

    #[test]
    fn web_api_template_substitutes_name_in_main() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("my_service");
        let _ = create_project("web-api", &target).unwrap();
        let main = std::fs::read_to_string(target.join("src/main.garnet")).unwrap();
        assert!(main.contains("my_service"));
        assert!(!main.contains("{{name}}"));
    }

    #[test]
    fn agent_orchestrator_template_emits_runnable_three_role_pipeline() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("my_agents");
        let _ = create_project("agent-orchestrator", &target).unwrap();
        let main = std::fs::read_to_string(target.join("src/main.garnet")).unwrap();
        assert!(main.contains("def researcher"));
        assert!(main.contains("def synthesizer"));
        assert!(main.contains("def reviewer"));
        assert!(main.contains("def orchestrate"));
        assert!(main.contains("@caps()"));
        assert!(!main.contains("spawn Researcher::new"));
    }

    #[test]
    fn unknown_template_returns_listing() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("anything");
        match create_project("nonexistent", &target) {
            Err(NewProjectError::UnknownTemplate { got, available }) => {
                assert_eq!(got, "nonexistent");
                assert!(available.contains(&"cli".to_string()));
            }
            other => panic!("expected UnknownTemplate, got {other:?}"),
        }
    }

    #[test]
    fn refuses_existing_target_directory() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("occupied");
        std::fs::create_dir(&target).unwrap();
        match create_project("cli", &target) {
            Err(NewProjectError::TargetExists { path }) => assert_eq!(path, target),
            other => panic!("expected TargetExists, got {other:?}"),
        }
    }

    #[test]
    fn invalid_project_name_rejected_starts_with_digit() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("1bad");
        match create_project("cli", &target) {
            Err(NewProjectError::InvalidProjectName { reason, .. }) => {
                assert!(reason.contains("ASCII letter"));
            }
            other => panic!("expected InvalidProjectName, got {other:?}"),
        }
    }

    #[test]
    fn invalid_project_name_rejected_reserved_keyword() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("actor");
        match create_project("cli", &target) {
            Err(NewProjectError::InvalidProjectName { reason, .. }) => {
                assert!(reason.contains("reserved"));
            }
            other => panic!("expected InvalidProjectName, got {other:?}"),
        }
    }

    #[test]
    fn invalid_project_name_rejected_special_chars() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("bad name");
        match create_project("cli", &target) {
            Err(NewProjectError::InvalidProjectName { reason, .. }) => {
                assert!(reason.contains("invalid character"));
            }
            other => panic!("expected InvalidProjectName, got {other:?}"),
        }
    }

    #[test]
    fn next_steps_hint_mentions_both_run_and_test() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("hint_demo");
        let report = create_project("cli", &target).unwrap();
        let hint = next_steps_hint(&report);
        assert!(hint.contains("cd "));
        assert!(hint.contains("garnet run"));
        assert!(hint.contains("garnet test"));
    }

    #[test]
    fn project_gitignore_excludes_garnet_cache_and_keys() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("ignored_paths");
        let _ = create_project("cli", &target).unwrap();
        let gitignore = std::fs::read_to_string(target.join(".gitignore")).unwrap();
        assert!(gitignore.contains(".garnet-cache/"));
        assert!(gitignore.contains("*.key"));
    }

    #[test]
    fn each_template_files_are_well_formed_text() {
        // Guard: every bundled template file must be non-empty valid UTF-8
        // (which include_str! already guarantees) and must NOT contain any
        // stray NUL bytes — a regression that could arise if a file was
        // accidentally saved as UTF-16 or binary.
        for t in TEMPLATES {
            for (rel, content) in t.files {
                assert!(!content.is_empty(), "{}::{rel} is empty", t.key);
                assert!(!content.contains('\0'), "{}::{rel} contains NUL", t.key);
            }
        }
    }
}
