use crate::evidence;
use crate::paths;
use crate::settings;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const ACTIVE_CONVERSION: &[&str] = &["rust", "ruby", "python", "go"];
const ADVISORY_LANGUAGES: &[&str] = &[
    "javascript",
    "typescript",
    "swift",
    "java",
    "c",
    "c++",
    "c#",
    "perl",
    "kotlin",
    "shell",
    "sql",
    "other",
];

/// Hard cap on stdout/stderr returned to the webview. When an evidence bundle
/// exists, the full untruncated streams are written there before the cap is
/// applied; the cap only protects the UI payload and DOM from multi-megabyte
/// reporter output. The truncation marker is honest about whether a bundle
/// holds the full streams.
const PAYLOAD_STREAM_CAP: usize = 256 * 1024;
const PAYLOAD_TRUNCATION_MARKER_WITH_BUNDLE: &str =
    "\n…[output truncated for display — the full output is in the evidence bundle]";
const PAYLOAD_TRUNCATION_MARKER_NO_BUNDLE: &str =
    "\n…[output truncated for display — no evidence bundle was created for this run]";
/// Cap for in-app evidence file reads.
const EVIDENCE_READ_CAP: usize = 512 * 1024;
/// Cap for evidence directory listings.
const EVIDENCE_LIST_CAP: usize = 500;
/// Categories that may legitimately run for a long time (full matrices).
const LONG_RUNNING_CATEGORIES: &[&str] = &[
    "agentic-dogfood",
    "domain-proof-matrix",
    "mac-domain-proofs",
];

#[derive(Debug, Serialize)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub command: Vec<String>,
    pub evidence_path: Option<String>,
    pub timed_out: bool,
    pub duration_ms: u64,
    pub truncated: bool,
}

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub cli_found: bool,
    pub cli_path: String,
    pub cli_version: String,
    pub repo_found: bool,
    pub repo_path: String,
    pub python_found: bool,
    pub python_version: String,
    pub evidence_dir: String,
    pub platform: String,
    pub arch: String,
}

#[derive(Debug, Serialize)]
pub struct BootstrapRequirement {
    pub id: String,
    pub label: String,
    pub found: bool,
    pub detected: String,
    pub action: String,
    pub command: String,
    pub evidence_note: String,
}

#[derive(Debug, Serialize)]
pub struct BootstrapPlan {
    pub ready: bool,
    pub ready_count: usize,
    pub total_count: usize,
    pub evidence_dir: String,
    pub summary: String,
    pub requirements: Vec<BootstrapRequirement>,
    pub safety_notes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct LanguageGroup {
    pub name: String,
    pub languages: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub app_version: String,
    pub tauri_version: String,
    pub platform: String,
    pub arch: String,
    pub settings_path: String,
}

/// Live values from the repo's RB-0a truth surface (`docs/truth.json`).
/// Every field is optional: a missing repo or missing file degrades to
/// `found = false` and the UI states that truth is unavailable rather than
/// showing stale hardcoded numbers.
#[derive(Debug, Default, Serialize)]
pub struct TruthSummary {
    pub found: bool,
    pub path: String,
    pub version: Option<String>,
    pub latest_tag: Option<String>,
    pub generated_at_commit: Option<String>,
    pub readiness_pct: Option<f64>,
    pub tracked_slices: Option<String>,
    pub primitive_count: Option<u64>,
    pub workspace_tests_passed: Option<u64>,
    pub workspace_tests_failed: Option<u64>,
    /// `workspace_tests.measured_at_commit` — distinct from
    /// `generated_at_commit`, because `xtask truth --skip-tests` carries old
    /// test counts forward while the overall stamp advances. The tests tile
    /// must attribute counts to THIS commit, never the newer stamp.
    pub workspace_tests_measured_at_commit: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EvidenceFileInfo {
    pub relative_path: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct EvidenceListing {
    pub root: String,
    pub files: Vec<EvidenceFileInfo>,
    pub truncated: bool,
}

#[derive(Debug, Serialize)]
pub struct EvidenceText {
    pub path: String,
    pub content: String,
    pub size: u64,
    pub truncated: bool,
}

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| format!("studio task failed to complete: {err}"))
}

/// Short, hard budget for the health version probes. They run on the boot
/// path (the splash waits on them), so they must never hang the shell.
const HEALTH_PROBE_TIMEOUT_SECS: u64 = 10;

fn run_version_probe(program: &Path, arg: &str) -> CommandResult {
    run_process_with_timeout(
        "health-probe",
        program.to_path_buf(),
        &[arg.to_string()],
        None,
        vec![display_path(program), arg.to_string()],
        None,
        Duration::from_secs(HEALTH_PROBE_TIMEOUT_SECS),
    )
}

pub(crate) fn cli_health_impl() -> HealthStatus {
    let (cli_found, cli_path, cli_version) = match paths::find_garnet_cli() {
        Some(path) => {
            let probe = run_version_probe(&path, "version");
            let version = if probe.timed_out {
                format!("unknown (version probe timed out after {HEALTH_PROBE_TIMEOUT_SECS}s)")
            } else {
                let trimmed = probe.stdout.trim();
                if trimmed.is_empty() {
                    "unknown".to_string()
                } else {
                    trimmed.to_string()
                }
            };
            (true, display_path(&path), version)
        }
        None => (false, String::new(), String::new()),
    };

    let (repo_found, repo_path) = match paths::find_repo_root() {
        Some(path) => (true, display_path(&path)),
        None => (false, String::new()),
    };

    let (python_found, python_version) = {
        let probe = run_version_probe(Path::new(paths::python_cmd()), "--version");
        let stdout = probe.stdout.trim().to_string();
        let stderr = probe.stderr.trim().to_string();
        let version = if stdout.is_empty() { stderr } else { stdout };
        (probe.success, version)
    };

    HealthStatus {
        cli_found,
        cli_path,
        cli_version,
        repo_found,
        repo_path,
        python_found,
        python_version,
        evidence_dir: display_path(&paths::evidence_base_dir()),
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

#[tauri::command]
pub async fn cli_health() -> Result<HealthStatus, String> {
    run_blocking(cli_health_impl).await
}

pub(crate) fn studio_bootstrap_plan_from_health(health: &HealthStatus) -> BootstrapPlan {
    let requirements = vec![
        BootstrapRequirement {
            id: "garnet-cli".to_string(),
            label: "Garnet CLI".to_string(),
            found: health.cli_found,
            detected: if health.cli_found {
                health.cli_path.clone()
            } else {
                "not found".to_string()
            },
            action: if health.cli_found {
                "Keep this CLI on PATH or in GARNET_CLI.".to_string()
            } else {
                "Install Garnet CLI from the repo build, then set GARNET_CLI.".to_string()
            },
            command: if health.cli_found {
                format!("{} version", health.cli_path)
            } else {
                "cargo build --release -p garnet-cli".to_string()
            },
            evidence_note:
                "Required before Parse / Check / Run and domain proof actions can execute."
                    .to_string(),
        },
        BootstrapRequirement {
            id: "repo".to_string(),
            label: "Repository checkout".to_string(),
            found: health.repo_found,
            detected: if health.repo_found {
                health.repo_path.clone()
            } else {
                "not found".to_string()
            },
            action: if health.repo_found {
                "Keep this checkout in GARNET_REPO.".to_string()
            } else {
                "Set GARNET_REPO to the local Garnet checkout before running repo reporters."
                    .to_string()
            },
            command: if health.repo_found {
                format!("$env:GARNET_REPO = {}", ps_quote(&health.repo_path))
            } else {
                "[Environment]::SetEnvironmentVariable('GARNET_REPO', '<path-to-garnet>', 'User')"
                    .to_string()
            },
            evidence_note:
                "Required for readiness reporters, Studio evidence readers, and build-from-repo CLI setup."
                    .to_string(),
        },
        BootstrapRequirement {
            id: "python".to_string(),
            label: "Python interpreter".to_string(),
            found: health.python_found,
            detected: if health.python_found {
                health.python_version.clone()
            } else {
                "not found".to_string()
            },
            action: if health.python_found {
                "Keep Python available on PATH.".to_string()
            } else {
                "Install Python with winget or another operator-approved installer.".to_string()
            },
            command: if health.python_found {
                "python --version".to_string()
            } else {
                "winget install --id Python.Python.3.12 -e --source winget".to_string()
            },
            evidence_note:
                "Required for the repo-owned readiness, converter, and handoff scripts.".to_string(),
        },
    ];
    let ready_count = requirements
        .iter()
        .filter(|requirement| requirement.found)
        .count();
    let total_count = requirements.len();
    let ready = ready_count == total_count;
    BootstrapPlan {
        ready,
        ready_count,
        total_count,
        evidence_dir: health.evidence_dir.clone(),
        summary: if ready {
            "All local prerequisites are detected. Studio can run CLI-backed actions on this machine."
                .to_string()
        } else {
            format!(
                "{ready_count} of {total_count} prerequisites are detected. Generate Setup Scripts for local, operator-run bootstrap commands."
            )
        },
        requirements,
        safety_notes: vec![
            "No provider APIs are called.".to_string(),
            "No source files are bundled by default.".to_string(),
            "Generated scripts are written to dogfood evidence and must be run explicitly by the operator."
                .to_string(),
            "The Tauri shell/plugin permission surface is unchanged.".to_string(),
        ],
    }
}

pub(crate) fn studio_bootstrap_plan_impl() -> BootstrapPlan {
    studio_bootstrap_plan_from_health(&cli_health_impl())
}

#[tauri::command]
pub async fn studio_bootstrap_plan() -> Result<BootstrapPlan, String> {
    run_blocking(studio_bootstrap_plan_impl).await
}

pub(crate) fn studio_bootstrap_write_scripts_impl() -> CommandResult {
    let started = Instant::now();
    let health = cli_health_impl();
    let plan = studio_bootstrap_plan_from_health(&health);
    let bundle = match evidence::create_named_bundle("bootstrap-setup") {
        Ok(bundle) => bundle,
        Err(err) => return contract_error(err),
    };
    let bundle_path = PathBuf::from(&bundle.path);
    let bootstrap_dir = bundle_path.join("bootstrap");
    if let Err(err) = fs::create_dir_all(&bootstrap_dir) {
        return contract_error(format!("failed to create bootstrap directory: {err}"));
    }

    let plan_json = match serde_json::to_string_pretty(&plan) {
        Ok(plan_json) => plan_json + "\n",
        Err(err) => return contract_error(format!("failed to serialize bootstrap plan: {err}")),
    };
    let files = [
        ("bootstrap-plan.json", plan_json),
        ("README.md", bootstrap_readme(&health, &plan)),
        ("install-python-winget.ps1", install_python_winget_script()),
        (
            "build-garnet-cli-from-repo.ps1",
            build_garnet_cli_script(&health),
        ),
        (
            "configure-garnet-env.ps1",
            configure_garnet_env_script(&health),
        ),
        ("run-bootstrap-preflight.ps1", bootstrap_preflight_script()),
    ];
    let mut written = Vec::new();
    for (name, contents) in files {
        let path = bootstrap_dir.join(name);
        if let Err(err) = fs::write(&path, contents) {
            return contract_error(format!("failed to write {name}: {err}"));
        }
        written.push(display_path(&path));
    }

    let stdout = format!(
        "Generated Garnet Studio bootstrap scripts.\n\n{}\n\nNext: open PowerShell, inspect README.md, run only the scripts you approve, then restart Studio and Run Health Check.\n",
        written.join("\n")
    );
    let command = vec![
        "garnet-studio".to_string(),
        "bootstrap-setup".to_string(),
        "write-scripts".to_string(),
    ];
    if let Err(err) =
        evidence::write_command_evidence(&bundle_path, "bootstrap", &command, &stdout, "", 0)
    {
        return contract_error(err);
    }

    CommandResult {
        success: true,
        stdout,
        stderr: String::new(),
        exit_code: 0,
        command,
        evidence_path: Some(display_path(&bundle_path)),
        timed_out: false,
        duration_ms: started.elapsed().as_millis() as u64,
        truncated: false,
    }
}

#[tauri::command]
pub async fn studio_bootstrap_write_scripts() -> Result<CommandResult, String> {
    run_blocking(studio_bootstrap_write_scripts_impl).await
}

#[tauri::command]
pub async fn cli_parse(file_path: String) -> Result<CommandResult, String> {
    run_blocking(move || run_garnet("parse", &["parse".to_string(), file_path], false)).await
}

#[tauri::command]
pub async fn cli_check(file_path: String) -> Result<CommandResult, String> {
    run_blocking(move || run_garnet("check", &["check".to_string(), file_path], false)).await
}

#[tauri::command]
pub async fn cli_run(file_path: String) -> Result<CommandResult, String> {
    run_blocking(move || run_garnet("run", &["run".to_string(), file_path], true)).await
}

pub(crate) fn cli_convert_impl(source_file: String, source_lang: String) -> CommandResult {
    let language = match normalize_language(&source_lang, ACTIVE_CONVERSION) {
        Ok(language) => language,
        Err(err) => return contract_error(err),
    };
    let bundle = match evidence::create_named_bundle("convert") {
        Ok(bundle) => bundle,
        Err(err) => return contract_error(err),
    };
    run_garnet_with_bundle(
        "convert",
        &[
            "convert".to_string(),
            language,
            source_file,
            "--out".to_string(),
            bundle.path.clone(),
        ],
        false,
        Some(PathBuf::from(bundle.path)),
    )
}

#[tauri::command]
pub async fn cli_convert(
    source_file: String,
    source_lang: String,
) -> Result<CommandResult, String> {
    run_blocking(move || cli_convert_impl(source_file, source_lang)).await
}

pub(crate) fn advisory_assist_plan_impl(source_file: String, language: String) -> CommandResult {
    let language = match normalize_language(&language, ADVISORY_LANGUAGES) {
        Ok(language) => language,
        Err(err) => return contract_error(err),
    };
    run_advisory_script(
        "assist-plan",
        "garnet_converter_assist_plan.py",
        vec![
            "--language".to_string(),
            language,
            "--source".to_string(),
            source_file,
        ],
    )
}

#[tauri::command]
pub async fn advisory_assist_plan(
    source_file: String,
    language: String,
) -> Result<CommandResult, String> {
    run_blocking(move || advisory_assist_plan_impl(source_file, language)).await
}

#[tauri::command]
pub async fn advisory_bundle(
    source_file: String,
    language: String,
) -> Result<CommandResult, String> {
    run_blocking(move || {
        let language = match normalize_language(&language, ADVISORY_LANGUAGES) {
            Ok(language) => language,
            Err(err) => return contract_error(err),
        };
        run_advisory_script(
            "advisory-bundle",
            "garnet_converter_advisory_bundle.py",
            vec![
                "--language".to_string(),
                language,
                "--source".to_string(),
                source_file,
            ],
        )
    })
    .await
}

#[tauri::command]
pub async fn advisory_review(bundle_dir: String) -> Result<CommandResult, String> {
    run_blocking(move || {
        run_advisory_script(
            "advisory-review",
            "garnet_converter_advisory_review.py",
            vec!["--bundle-dir".to_string(), bundle_dir],
        )
    })
    .await
}

#[tauri::command]
pub async fn advisory_handoff(
    bundle_dir: String,
    review_dir: String,
) -> Result<CommandResult, String> {
    run_blocking(move || {
        run_advisory_script(
            "advisory-handoff",
            "garnet_converter_advisory_handoff.py",
            vec![
                "--bundle-dir".to_string(),
                bundle_dir,
                "--review-dir".to_string(),
                review_dir,
            ],
        )
    })
    .await
}

pub(crate) fn objective_pulse_impl() -> CommandResult {
    run_python_script(
        "objective-pulse",
        "garnet_mit_readiness_status.py",
        vec!["--format".to_string(), "markdown".to_string()],
    )
}

#[tauri::command]
pub async fn objective_pulse() -> Result<CommandResult, String> {
    run_blocking(objective_pulse_impl).await
}

#[tauri::command]
pub async fn agentic_dogfood_matrix() -> Result<CommandResult, String> {
    run_blocking(|| {
        run_python_script(
            "agentic-dogfood",
            "run_agentic_dogfood_matrix.py",
            vec!["--copy-to-desktop".to_string(), "--strict".to_string()],
        )
    })
    .await
}

pub(crate) fn domain_proof_matrix_impl() -> CommandResult {
    let cli = match paths::find_garnet_cli() {
        Some(path) => path,
        None => {
            return contract_error("garnet CLI not found. Set GARNET_CLI or add garnet to PATH.")
        }
    };
    run_domain_matrix_report_script(
        "domain-proof-matrix",
        "smoke_garnet_studio_domain_matrix.py",
        "markdown",
        vec!["--garnet".to_string(), display_path(&cli)],
    )
}

#[tauri::command]
pub async fn domain_proof_matrix() -> Result<CommandResult, String> {
    run_blocking(domain_proof_matrix_impl).await
}

pub(crate) fn mac_domain_proofs_impl() -> CommandResult {
    let cli = match paths::find_garnet_cli() {
        Some(path) => path,
        None => {
            return contract_error("garnet CLI not found. Set GARNET_CLI or add garnet to PATH.")
        }
    };
    let repo = match paths::find_repo_root() {
        Some(path) => path,
        None => return contract_error("Garnet repository root not found. Set GARNET_REPO."),
    };
    let bundle = repo
        .join("target")
        .join("mac-studio-domain-proofs")
        .join(format!(
            "garnet-mac-domain-proofs-{}",
            evidence::timestamp()
        ));
    if let Err(err) = fs::create_dir_all(&bundle) {
        return contract_error(format!(
            "failed to create Mac domain proof evidence directory: {err}"
        ));
    }
    run_python_script_with_bundle(
        "mac-domain-proofs",
        "smoke_garnet_mac_domain_proofs.py",
        vec![
            "--garnet".to_string(),
            display_path(&cli),
            "--output-dir".to_string(),
            display_path(&bundle),
            "--format".to_string(),
            "md".to_string(),
        ],
        bundle,
    )
}

#[tauri::command]
pub async fn mac_domain_proofs() -> Result<CommandResult, String> {
    run_blocking(mac_domain_proofs_impl).await
}

pub(crate) fn windows_linux_studio_status_impl() -> CommandResult {
    run_python_script(
        "windows-linux-studio-status",
        "garnet_windows_linux_studio_status.py",
        vec!["--format".to_string(), "markdown".to_string()],
    )
}

#[tauri::command]
pub async fn windows_linux_studio_status() -> Result<CommandResult, String> {
    run_blocking(windows_linux_studio_status_impl).await
}

pub(crate) fn converter_status_impl() -> CommandResult {
    run_python_script(
        "converter-status",
        "garnet_converter_status.py",
        vec!["--format".to_string(), "markdown".to_string()],
    )
}

#[tauri::command]
pub async fn converter_status() -> Result<CommandResult, String> {
    run_blocking(converter_status_impl).await
}

#[tauri::command]
pub async fn provider_options() -> Result<CommandResult, String> {
    run_blocking(|| {
        run_report_script_with_output_dir(
            "provider-options",
            "garnet_converter_llm_feasibility.py",
            "markdown",
        )
    })
    .await
}

#[tauri::command]
pub async fn mit_demo_route() -> Result<CommandResult, String> {
    run_blocking(|| {
        run_report_script_with_output_dir("mit-demo-route", "garnet_mit_demo_route.py", "markdown")
    })
    .await
}

#[tauri::command]
pub async fn mit_deck_outline() -> Result<CommandResult, String> {
    run_blocking(|| {
        run_report_script_with_output_dir(
            "mit-deck-outline",
            "garnet_mit_deck_outline.py",
            "markdown",
        )
    })
    .await
}

#[tauri::command]
pub async fn mit_deck_preview() -> Result<CommandResult, String> {
    run_blocking(|| {
        run_report_script_with_output_dir("mit-deck-preview", "garnet_mit_deck_preview.py", "html")
    })
    .await
}

#[tauri::command]
pub async fn mac_continuation_pulse() -> Result<CommandResult, String> {
    run_blocking(|| {
        run_python_script(
            "mac-continuation-pulse",
            "garnet_mac_side_continuation_status.py",
            vec!["--format".to_string(), "markdown".to_string()],
        )
    })
    .await
}

#[tauri::command]
pub async fn proof_benchmark_status() -> Result<CommandResult, String> {
    run_blocking(|| {
        run_report_script_with_output_dir(
            "proof-benchmark-status",
            "garnet_proof_benchmark_status.py",
            "markdown",
        )
    })
    .await
}

#[tauri::command]
pub async fn benchmark_no_run() -> Result<CommandResult, String> {
    run_blocking(|| {
        run_report_script_with_output_dir(
            "benchmark-no-run",
            "garnet_benchmark_no_run.py",
            "markdown",
        )
    })
    .await
}

#[tauri::command]
pub async fn notarization_status() -> Result<CommandResult, String> {
    run_blocking(|| {
        run_python_script(
            "notarization-status",
            "garnet_studio_notarization_status.py",
            vec!["--format".to_string(), "markdown".to_string()],
        )
    })
    .await
}

pub(crate) fn windows_vm_installer_status_impl() -> CommandResult {
    run_python_script(
        "windows-vm-installer-status",
        "garnet_windows_clean_vm_installer_status.py",
        vec!["--format".to_string(), "markdown".to_string()],
    )
}

#[tauri::command]
pub async fn windows_vm_installer_status() -> Result<CommandResult, String> {
    run_blocking(windows_vm_installer_status_impl).await
}

#[tauri::command]
pub async fn create_evidence_bundle() -> Result<evidence::EvidenceBundle, String> {
    match run_blocking(evidence::create_bundle).await {
        Ok(inner) => inner,
        Err(err) => Err(err),
    }
}

#[tauri::command]
pub fn get_evidence_dir() -> String {
    display_path(&paths::evidence_base_dir())
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        tauri_version: tauri::VERSION.to_string(),
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        settings_path: display_path(&settings::settings_path()),
    }
}

#[tauri::command]
pub fn studio_get_settings() -> settings::StudioSettings {
    settings::load()
}

#[tauri::command]
pub fn studio_set_settings(
    settings: settings::StudioSettings,
) -> Result<settings::StudioSettings, String> {
    settings::save(settings)
}

pub(crate) fn get_truth_summary_impl() -> TruthSummary {
    let Some(repo) = paths::find_repo_root() else {
        return TruthSummary {
            error: Some("Garnet repository root not found. Set GARNET_REPO.".to_string()),
            ..TruthSummary::default()
        };
    };
    let path = repo.join("docs").join("truth.json");
    let display = display_path(&path);
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(err) => {
            return TruthSummary {
                path: display,
                error: Some(format!("failed to read docs/truth.json: {err}")),
                ..TruthSummary::default()
            }
        }
    };
    let value: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(err) => {
            return TruthSummary {
                path: display,
                error: Some(format!("failed to parse docs/truth.json: {err}")),
                ..TruthSummary::default()
            }
        }
    };
    TruthSummary {
        found: true,
        path: display,
        version: value
            .get("version")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        latest_tag: value
            .get("latest_tag")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        generated_at_commit: value
            .get("generated_at_commit")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        readiness_pct: value.get("readiness_pct").and_then(|v| v.as_f64()),
        tracked_slices: value
            .get("tracked_slices")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        primitive_count: value.get("primitive_count").and_then(|v| v.as_u64()),
        workspace_tests_passed: value
            .get("workspace_tests")
            .and_then(|tests| tests.get("passed"))
            .and_then(|v| v.as_u64()),
        workspace_tests_failed: value
            .get("workspace_tests")
            .and_then(|tests| tests.get("failed"))
            .and_then(|v| v.as_u64()),
        workspace_tests_measured_at_commit: value
            .get("workspace_tests")
            .and_then(|tests| tests.get("measured_at_commit"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        error: None,
    }
}

#[tauri::command]
pub async fn get_truth_summary() -> Result<TruthSummary, String> {
    run_blocking(get_truth_summary_impl).await
}

pub(crate) fn list_evidence_files_impl(dir: String) -> Result<EvidenceListing, String> {
    let root = resolve_within_evidence_roots(&dir)?;
    if !root.is_dir() {
        return Err("evidence path is not a directory".to_string());
    }
    let mut files = Vec::new();
    let mut truncated = false;
    collect_evidence_files(&root, &root, &mut files, &mut truncated)?;
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(EvidenceListing {
        root: display_path(&root),
        files,
        truncated,
    })
}

#[tauri::command]
pub async fn list_evidence_files(dir: String) -> Result<EvidenceListing, String> {
    match run_blocking(move || list_evidence_files_impl(dir)).await {
        Ok(inner) => inner,
        Err(err) => Err(err),
    }
}

pub(crate) fn read_evidence_text_impl(path: String) -> Result<EvidenceText, String> {
    let resolved = resolve_within_evidence_roots(&path)?;
    if !resolved.is_file() {
        return Err("evidence path is not a file".to_string());
    }
    let metadata =
        fs::metadata(&resolved).map_err(|err| format!("failed to stat evidence file: {err}"))?;
    let size = metadata.len();
    // Bounded read: never allocate more than the cap + 1 sentinel byte, even
    // for the multi-hundred-MB stdout captures a matrix run can leave behind.
    let file =
        fs::File::open(&resolved).map_err(|err| format!("failed to open evidence file: {err}"))?;
    let mut bytes = Vec::with_capacity(EVIDENCE_READ_CAP.min(size as usize) + 1);
    file.take(EVIDENCE_READ_CAP as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|err| format!("failed to read evidence file: {err}"))?;
    let truncated = bytes.len() > EVIDENCE_READ_CAP;
    let slice = if truncated {
        &bytes[..EVIDENCE_READ_CAP]
    } else {
        &bytes[..]
    };
    Ok(EvidenceText {
        path: display_path(&resolved),
        content: String::from_utf8_lossy(slice).to_string(),
        size,
        truncated,
    })
}

#[tauri::command]
pub async fn read_evidence_text(path: String) -> Result<EvidenceText, String> {
    match run_blocking(move || read_evidence_text_impl(path)).await {
        Ok(inner) => inner,
        Err(err) => Err(err),
    }
}

#[tauri::command]
pub fn get_language_taxonomy() -> Vec<LanguageGroup> {
    vec![
        LanguageGroup {
            name: "Active conversion".into(),
            languages: list(&["Rust", "Ruby", "Python", "Go"]),
        },
        LanguageGroup {
            name: "Advisory planning".into(),
            languages: list(&[
                "JavaScript",
                "TypeScript",
                "Swift",
                "Java",
                "C",
                "C++",
                "C#",
                "Perl",
                "Kotlin",
                "Shell",
                "SQL",
                "Other",
            ]),
        },
        LanguageGroup {
            name: "Native boundary recommended".into(),
            languages: list(&[
                "C",
                "C++",
                "Objective-C",
                "Assembly",
                "CUDA",
                "platform-specific code",
            ]),
        },
        LanguageGroup {
            name: "Future backend lowering".into(),
            languages: list(&[
                "Wasm",
                "LLVM-style native targets",
                "native package toolchains",
            ]),
        },
    ]
}

fn allowed_evidence_roots() -> Vec<PathBuf> {
    vec![
        paths::evidence_base_dir(),
        paths::domain_matrix_evidence_base_dir(),
    ]
}

/// Resolve a caller-supplied path and require it to live under one of the
/// Studio evidence roots. Canonicalization (which also resolves symlinks)
/// happens on both sides, so `..` traversal and link escapes cannot reach
/// outside the evidence trees. This keeps the in-app evidence reader from
/// becoming a general filesystem read primitive.
fn resolve_within_evidence_roots(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("evidence path is empty".to_string());
    }
    let canonical = PathBuf::from(trimmed)
        .canonicalize()
        .map_err(|err| format!("evidence path is not readable: {err}"))?;
    for root in allowed_evidence_roots() {
        if let Ok(root_canonical) = root.canonicalize() {
            if canonical.starts_with(&root_canonical) {
                return Ok(canonical);
            }
        }
    }
    Err("path is outside the Studio evidence roots; refusing to read it".to_string())
}

fn collect_evidence_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<EvidenceFileInfo>,
    truncated: &mut bool,
) -> Result<(), String> {
    let entries =
        fs::read_dir(directory).map_err(|err| format!("failed to read evidence dir: {err}"))?;
    for entry in entries {
        if files.len() >= EVIDENCE_LIST_CAP {
            *truncated = true;
            return Ok(());
        }
        let entry = entry.map_err(|err| format!("failed to read evidence entry: {err}"))?;
        let path = entry.path();
        // file_type() does NOT follow links: skip symlinks/junctions entirely
        // so a link planted inside a bundle can never widen the listing
        // outside the evidence roots (reads are independently canonicalized,
        // but enumeration must hold the same boundary).
        let file_type = entry
            .file_type()
            .map_err(|err| format!("failed to read evidence entry type: {err}"))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_evidence_files(root, &path, files, truncated)?;
            continue;
        }
        let size = entry.metadata().map(|meta| meta.len()).unwrap_or(0);
        let relative = path
            .strip_prefix(root)
            .map(|relative| relative.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| display_path(&path));
        files.push(EvidenceFileInfo {
            relative_path: relative,
            size,
        });
    }
    Ok(())
}

fn run_garnet(category: &str, args: &[String], _executes_source: bool) -> CommandResult {
    let bundle = evidence::create_named_bundle(category)
        .ok()
        .map(|bundle| PathBuf::from(bundle.path));
    run_garnet_with_bundle(category, args, _executes_source, bundle)
}

fn run_garnet_with_bundle(
    category: &str,
    args: &[String],
    _executes_source: bool,
    bundle: Option<PathBuf>,
) -> CommandResult {
    let cli = match paths::find_garnet_cli() {
        Some(path) => path,
        None => {
            return contract_error("garnet CLI not found. Set GARNET_CLI or add garnet to PATH.")
        }
    };
    let command = command_vector(&cli, args);
    run_process(
        category,
        cli,
        args,
        paths::find_repo_root(),
        command,
        bundle,
    )
}

fn run_advisory_script(category: &str, script_name: &str, mut args: Vec<String>) -> CommandResult {
    let bundle = match evidence::create_named_bundle(category) {
        Ok(bundle) => bundle,
        Err(err) => return contract_error(err),
    };
    args.push("--output-dir".to_string());
    args.push(bundle.path.clone());
    run_python_script_with_bundle(category, script_name, args, PathBuf::from(bundle.path))
}

fn run_python_script(category: &str, script_name: &str, args: Vec<String>) -> CommandResult {
    let bundle = evidence::create_named_bundle(category)
        .ok()
        .map(|bundle| PathBuf::from(bundle.path));
    match bundle {
        Some(bundle) => run_python_script_with_bundle(category, script_name, args, bundle),
        None => run_python_script_without_bundle(category, script_name, args),
    }
}

fn run_report_script_with_output_dir(
    category: &str,
    script_name: &str,
    format: &str,
) -> CommandResult {
    run_report_script_with_output_dir_and_args(category, script_name, format, Vec::new())
}

fn run_report_script_with_output_dir_and_args(
    category: &str,
    script_name: &str,
    format: &str,
    extra_args: Vec<String>,
) -> CommandResult {
    let bundle = match evidence::create_named_bundle(category) {
        Ok(bundle) => bundle,
        Err(err) => return contract_error(err),
    };
    let args = report_script_args(bundle.path.clone(), format, extra_args);
    run_python_script_with_bundle(category, script_name, args, PathBuf::from(bundle.path))
}

fn run_domain_matrix_report_script(
    category: &str,
    script_name: &str,
    format: &str,
    extra_args: Vec<String>,
) -> CommandResult {
    let bundle = paths::domain_matrix_evidence_base_dir().join(format!(
        "garnet-studio-domain-matrix-{}",
        evidence::timestamp()
    ));
    if let Err(err) = fs::create_dir_all(&bundle) {
        return contract_error(format!(
            "failed to create domain matrix evidence directory: {err}"
        ));
    }
    let args = report_script_args(display_path(&bundle), format, extra_args);
    run_python_script_with_bundle(category, script_name, args, bundle)
}

fn report_script_args(
    bundle_path: String,
    format: &str,
    mut extra_args: Vec<String>,
) -> Vec<String> {
    let mut args = vec![
        "--output-dir".to_string(),
        bundle_path,
        "--format".to_string(),
        format.to_string(),
    ];
    args.append(&mut extra_args);
    args
}

fn run_python_script_with_bundle(
    category: &str,
    script_name: &str,
    args: Vec<String>,
    bundle: PathBuf,
) -> CommandResult {
    run_python_script_inner(category, script_name, args, Some(bundle))
}

fn run_python_script_without_bundle(
    category: &str,
    script_name: &str,
    args: Vec<String>,
) -> CommandResult {
    run_python_script_inner(category, script_name, args, None)
}

fn run_python_script_inner(
    category: &str,
    script_name: &str,
    args: Vec<String>,
    bundle: Option<PathBuf>,
) -> CommandResult {
    let repo = match paths::find_repo_root() {
        Some(path) => path,
        None => return contract_error("Garnet repository root not found. Set GARNET_REPO."),
    };
    let script = repo.join("scripts").join(script_name);
    if !script.exists() {
        return contract_error(format!("script not found: {}", display_path(&script)));
    }

    let mut command = vec![paths::python_cmd().to_string(), display_path(&script)];
    command.extend(args.iter().cloned());
    run_process(
        category,
        PathBuf::from(paths::python_cmd()),
        &args_for_script(&script, &args),
        Some(repo),
        command,
        bundle,
    )
}

fn args_for_script(script: &Path, args: &[String]) -> Vec<String> {
    let mut all = vec![display_path(script)];
    all.extend(args.iter().cloned());
    all
}

fn timeout_for_category(category: &str) -> Duration {
    let settings = settings::load();
    let secs = if LONG_RUNNING_CATEGORIES.contains(&category) {
        settings.matrix_timeout_secs
    } else {
        settings.command_timeout_secs
    };
    Duration::from_secs(secs)
}

fn spawn_reader<R: Read + Send + 'static>(mut reader: R) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer);
        buffer
    })
}

fn join_reader(handle: Option<thread::JoinHandle<Vec<u8>>>) -> String {
    let bytes = handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    String::from_utf8_lossy(&bytes).to_string()
}

fn truncate_for_payload(text: String, has_bundle: bool) -> (String, bool) {
    if text.len() <= PAYLOAD_STREAM_CAP {
        return (text, false);
    }
    let mut cut = PAYLOAD_STREAM_CAP;
    while cut > 0 && !text.is_char_boundary(cut) {
        cut -= 1;
    }
    let mut truncated = text[..cut].to_string();
    truncated.push_str(if has_bundle {
        PAYLOAD_TRUNCATION_MARKER_WITH_BUNDLE
    } else {
        PAYLOAD_TRUNCATION_MARKER_NO_BUNDLE
    });
    (truncated, true)
}

fn run_process(
    category: &str,
    program: PathBuf,
    args: &[String],
    working_dir: Option<PathBuf>,
    command: Vec<String>,
    bundle: Option<PathBuf>,
) -> CommandResult {
    run_process_with_timeout(
        category,
        program,
        args,
        working_dir,
        command,
        bundle,
        timeout_for_category(category),
    )
}

/// Best-effort kill of the child's whole process tree, then the child itself.
/// The Studio's long actions are Python wrappers that spawn grandchildren
/// (the garnet CLI, matrix probes); killing only the direct child would leave
/// those running after a timeout. Windows uses `taskkill /T`; Unix children
/// are spawned in their own process group (see the spawn site) so the group
/// can be signalled. The direct `child.kill()` remains as the fallback if the
/// platform tool is unavailable.
fn kill_process_tree(child: &mut std::process::Child) {
    let pid = child.id();
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-KILL".to_string(), format!("-{pid}")])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

/// Spawn the child with piped output, drain both pipes on reader threads (so a
/// full pipe can never deadlock the child), and poll for exit until the
/// deadline. On timeout the child's process tree is killed (best-effort — see
/// `kill_process_tree`), the partial output is preserved, and the result is
/// marked `timed_out`. When an evidence bundle exists, full output is written
/// to it before the UI payload is capped.
#[allow(clippy::too_many_arguments)]
fn run_process_with_timeout(
    category: &str,
    program: PathBuf,
    args: &[String],
    working_dir: Option<PathBuf>,
    command: Vec<String>,
    bundle: Option<PathBuf>,
    timeout: Duration,
) -> CommandResult {
    let started = Instant::now();
    let mut process = Command::new(&program);
    process
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Own process group so kill_process_tree can signal the whole tree.
        process.process_group(0);
    }
    if let Some(dir) = &working_dir {
        process.current_dir(dir);
    }

    let mut child = match process.spawn() {
        Ok(child) => child,
        Err(err) => {
            return CommandResult {
                success: false,
                stdout: String::new(),
                stderr: format!("failed to execute command: {err}"),
                exit_code: -1,
                command,
                evidence_path: bundle.map(|path| display_path(&path)),
                timed_out: false,
                duration_ms: started.elapsed().as_millis() as u64,
                truncated: false,
            }
        }
    };

    let stdout_reader = child.stdout.take().map(spawn_reader);
    let stderr_reader = child.stderr.take().map(spawn_reader);

    let deadline = started + timeout;
    let mut timed_out = false;
    let mut monitor_failed = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    timed_out = true;
                    kill_process_tree(&mut child);
                    break child.wait().ok();
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(_) => {
                // Monitoring failed; never leave the child running with the
                // deadline silently abandoned.
                monitor_failed = true;
                kill_process_tree(&mut child);
                break child.wait().ok();
            }
        }
    };

    let stdout_full = join_reader(stdout_reader);
    let mut stderr_full = join_reader(stderr_reader);
    if timed_out || monitor_failed {
        if !stderr_full.is_empty() && !stderr_full.ends_with('\n') {
            stderr_full.push('\n');
        }
        if timed_out {
            stderr_full.push_str(&format!(
                "studio: command timed out after {}s and its process tree was terminated",
                timeout.as_secs()
            ));
        } else {
            stderr_full
                .push_str("studio: failed to monitor the command; its process tree was terminated");
        }
    }

    let exit_code = status.and_then(|status| status.code()).unwrap_or(-1);
    let success = !timed_out && status.map(|status| status.success()).unwrap_or(false);

    if let Some(path) = &bundle {
        let _ = evidence::write_command_evidence(
            path,
            category,
            &command,
            &stdout_full,
            &stderr_full,
            exit_code,
        );
    }

    let has_bundle = bundle.is_some();
    let (stdout, stdout_truncated) = truncate_for_payload(stdout_full, has_bundle);
    let (stderr, stderr_truncated) = truncate_for_payload(stderr_full, has_bundle);

    CommandResult {
        success,
        stdout,
        stderr,
        exit_code,
        command,
        evidence_path: bundle.map(|path| display_path(&path)),
        timed_out,
        duration_ms: started.elapsed().as_millis() as u64,
        truncated: stdout_truncated || stderr_truncated,
    }
}

fn contract_error(message: impl Into<String>) -> CommandResult {
    CommandResult {
        success: false,
        stdout: String::new(),
        stderr: message.into(),
        exit_code: -1,
        command: Vec::new(),
        evidence_path: None,
        timed_out: false,
        duration_ms: 0,
        truncated: false,
    }
}

fn normalize_language(language: &str, allowed: &[&str]) -> Result<String, String> {
    let normalized = language.trim().to_lowercase();
    if allowed.iter().any(|item| *item == normalized) {
        Ok(normalized)
    } else {
        Err(format!(
            "{language} is not available for this action. Active conversion is Rust, Ruby, Python, and Go; advisory planning is separated."
        ))
    }
}

fn command_vector(program: &Path, args: &[String]) -> Vec<String> {
    let mut command = vec![display_path(program)];
    command.extend(args.iter().cloned());
    command
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn bootstrap_readme(health: &HealthStatus, plan: &BootstrapPlan) -> String {
    let mut requirements = String::new();
    for requirement in &plan.requirements {
        requirements.push_str(&format!(
            "- {}: {}. Action: {}. Command: `{}`.\n",
            requirement.label, requirement.detected, requirement.action, requirement.command
        ));
    }
    format!(
        r#"# Garnet Studio Windows Bootstrap Setup

Generated by Garnet Studio from the CLI Health panel.

Status: {ready_count}/{total_count} prerequisites detected.
Evidence root: {evidence_dir}
Host: {platform} / {arch}

## Requirements
{requirements}
## Files
- `install-python-winget.ps1` installs Python through winget when Python is missing.
- `build-garnet-cli-from-repo.ps1` builds `garnet-cli` from a local checkout and stages `garnet.exe`.
- `configure-garnet-env.ps1` writes user-scoped `GARNET_REPO`, `GARNET_CLI`, and PATH entries.
- `run-bootstrap-preflight.ps1` records what the machine can see before and after setup.
- `bootstrap-plan.json` is the machine-readable plan that Studio displayed.

## Operator Sequence
1. Inspect every script in this directory.
2. If Python is missing, run `.\install-python-winget.ps1`.
3. If the repo is missing, clone or copy the Garnet checkout locally, then run `.\configure-garnet-env.ps1 -Repo <path-to-garnet>`.
4. Run `.\build-garnet-cli-from-repo.ps1 -Repo <path-to-garnet>`.
5. Restart Garnet Studio and run CLI Health again.

## Safety Contract
- No provider APIs are called.
- No source files are bundled by default.
- No shell/plugin permission is added to the Tauri app.
- These scripts are generated evidence and do not run until an operator executes them.
"#,
        ready_count = plan.ready_count,
        total_count = plan.total_count,
        evidence_dir = health.evidence_dir,
        platform = health.platform,
        arch = health.arch,
        requirements = requirements,
    )
}

fn install_python_winget_script() -> String {
    r#"$ErrorActionPreference = 'Stop'

Write-Host 'Garnet Studio Python bootstrap'
if (Get-Command python -ErrorAction SilentlyContinue) {
  python --version
  Write-Host 'Python is already available on PATH.'
  exit 0
}

if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
  throw 'winget was not found. Install Python manually from python.org or enable App Installer, then rerun CLI Health.'
}

winget install --id Python.Python.3.12 -e --source winget
Write-Host 'Restart the terminal or Studio, then run: python --version'
"#
    .to_string()
}

fn build_garnet_cli_script(health: &HealthStatus) -> String {
    let repo_default = if health.repo_found {
        ps_quote(&health.repo_path)
    } else {
        "$env:GARNET_REPO".to_string()
    };
    format!(
        r#"$ErrorActionPreference = 'Stop'

param(
  [string]$Repo = {repo_default},
  [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Garnet\bin')
)

if ([string]::IsNullOrWhiteSpace($Repo) -or -not (Test-Path $Repo)) {{
  throw 'Garnet repo not found. Pass -Repo <path-to-garnet> or set GARNET_REPO first.'
}}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {{
  throw 'Rust cargo was not found. Install Rust with rustup, then rerun this script.'
}}

Push-Location $Repo
try {{
  cargo build --release -p garnet-cli
}} finally {{
  Pop-Location
}}

$BuiltCli = Join-Path $Repo 'target\release\garnet.exe'
if (-not (Test-Path $BuiltCli)) {{
  throw "Expected CLI was not produced at $BuiltCli"
}}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$InstalledCli = Join-Path $InstallDir 'garnet.exe'
Copy-Item -LiteralPath $BuiltCli -Destination $InstalledCli -Force
[Environment]::SetEnvironmentVariable('GARNET_CLI', $InstalledCli, 'User')

$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not (($UserPath -split ';') -contains $InstallDir)) {{
  [Environment]::SetEnvironmentVariable('Path', "$UserPath;$InstallDir", 'User')
}}

& $InstalledCli version
Write-Host "GARNET_CLI set to $InstalledCli"
"#
    )
}

fn configure_garnet_env_script(health: &HealthStatus) -> String {
    let repo_default = if health.repo_found {
        ps_quote(&health.repo_path)
    } else {
        "$env:GARNET_REPO".to_string()
    };
    let cli_default = if health.cli_found {
        ps_quote(&health.cli_path)
    } else {
        "(Join-Path $env:LOCALAPPDATA 'Garnet\\bin\\garnet.exe')".to_string()
    };
    format!(
        r#"$ErrorActionPreference = 'Stop'

param(
  [string]$Repo = {repo_default},
  [string]$Cli = {cli_default}
)

if ([string]::IsNullOrWhiteSpace($Repo) -or -not (Test-Path $Repo)) {{
  throw 'GARNET_REPO target does not exist. Pass -Repo <path-to-garnet>.'
}}
if ([string]::IsNullOrWhiteSpace($Cli) -or -not (Test-Path $Cli)) {{
  throw 'GARNET_CLI target does not exist. Build the CLI first or pass -Cli <path-to-garnet.exe>.'
}}

[Environment]::SetEnvironmentVariable('GARNET_REPO', $Repo, 'User')
[Environment]::SetEnvironmentVariable('GARNET_CLI', $Cli, 'User')

$CliDir = Split-Path -Parent $Cli
$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not (($UserPath -split ';') -contains $CliDir)) {{
  [Environment]::SetEnvironmentVariable('Path', "$UserPath;$CliDir", 'User')
}}

Write-Host "GARNET_REPO set to $Repo"
Write-Host "GARNET_CLI set to $Cli"
Write-Host 'Restart Garnet Studio and run CLI Health again.'
"#
    )
}

fn bootstrap_preflight_script() -> String {
    r#"$ErrorActionPreference = 'Continue'

Write-Host 'Garnet Studio bootstrap preflight'
Write-Host "Host: $([System.Environment]::OSVersion.VersionString)"
Write-Host "Arch: $env:PROCESSOR_ARCHITECTURE"
Write-Host "GARNET_REPO=$env:GARNET_REPO"
Write-Host "GARNET_CLI=$env:GARNET_CLI"

foreach ($Name in @('python', 'cargo', 'garnet', 'winget')) {
  $Command = Get-Command $Name -ErrorAction SilentlyContinue
  if ($Command) {
    Write-Host "$Name=$($Command.Source)"
    if ($Name -eq 'python') { python --version }
    if ($Name -eq 'cargo') { cargo --version }
    if ($Name -eq 'garnet') { garnet version }
  } else {
    Write-Host "$Name=(not found)"
  }
}
"#
    .to_string()
}

fn list(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_rejects_advisory_languages() {
        let result = cli_convert_impl("sample.ts".to_string(), "TypeScript".to_string());
        assert!(!result.success);
        assert!(result.stderr.contains("not available"));
    }

    #[test]
    fn advisory_plan_rejects_active_conversion_languages() {
        let result = advisory_assist_plan_impl("sample.rs".to_string(), "Rust".to_string());
        assert!(!result.success);
        assert!(result.stderr.contains("not available"));
    }

    #[test]
    fn taxonomy_preserves_copy_truth() {
        let taxonomy = get_language_taxonomy();
        assert_eq!(
            taxonomy[0].languages,
            list(&["Rust", "Ruby", "Python", "Go"])
        );
        assert!(taxonomy[1].languages.contains(&"TypeScript".to_string()));
        assert!(taxonomy[2].languages.contains(&"CUDA".to_string()));
        assert!(taxonomy[3].languages.contains(&"Wasm".to_string()));
    }

    #[test]
    fn report_script_args_preserve_output_format_and_extra_garnet_cli() {
        let args = report_script_args(
            "bundle-dir".to_string(),
            "markdown",
            vec!["--garnet".to_string(), "target/release/garnet".to_string()],
        );

        assert_eq!(
            args,
            vec![
                "--output-dir",
                "bundle-dir",
                "--format",
                "markdown",
                "--garnet",
                "target/release/garnet"
            ]
        );
    }

    #[test]
    fn domain_matrix_root_matches_objective_pulse_scanner() {
        let root = paths::domain_matrix_evidence_base_dir()
            .to_string_lossy()
            .replace('\\', "/");

        assert!(root.ends_with("Desktop/dogfood/garnet-studio-domain-matrix"));
    }

    #[test]
    fn bootstrap_plan_turns_missing_health_into_actionable_setup_steps() {
        let health = HealthStatus {
            cli_found: false,
            cli_path: String::new(),
            cli_version: String::new(),
            repo_found: false,
            repo_path: String::new(),
            python_found: false,
            python_version: String::new(),
            evidence_dir: "C:/dogfood/garnet-studio-windows-linux".to_string(),
            platform: "windows".to_string(),
            arch: "x86_64".to_string(),
        };

        let plan = studio_bootstrap_plan_from_health(&health);
        assert!(!plan.ready);
        assert_eq!(0, plan.ready_count);
        assert_eq!(3, plan.total_count);
        assert!(plan
            .requirements
            .iter()
            .any(|step| step.id == "garnet-cli" && step.action.contains("Install Garnet CLI")));
        assert!(plan
            .requirements
            .iter()
            .any(|step| step.id == "repo" && step.action.contains("Set GARNET_REPO")));
        assert!(plan
            .requirements
            .iter()
            .any(|step| step.id == "python" && step.action.contains("Install Python")));
        assert!(plan
            .safety_notes
            .iter()
            .any(|note| note.contains("No provider APIs are called")));
    }

    #[test]
    fn payload_truncation_caps_oversized_output_and_marks_it_honestly() {
        let oversized = "g".repeat(PAYLOAD_STREAM_CAP + 64);
        let (text, truncated) = truncate_for_payload(oversized.clone(), true);
        assert!(truncated);
        assert!(text.ends_with(PAYLOAD_TRUNCATION_MARKER_WITH_BUNDLE));
        assert!(text.len() <= PAYLOAD_STREAM_CAP + PAYLOAD_TRUNCATION_MARKER_WITH_BUNDLE.len());

        // Without a bundle the marker must not claim the evidence bundle
        // holds the full streams — there is no bundle.
        let (text, truncated) = truncate_for_payload(oversized, false);
        assert!(truncated);
        assert!(text.ends_with(PAYLOAD_TRUNCATION_MARKER_NO_BUNDLE));

        let small = "ok".to_string();
        let (text, truncated) = truncate_for_payload(small.clone(), true);
        assert!(!truncated);
        assert_eq!(text, small);
    }

    #[test]
    fn run_process_kills_a_hung_command_on_timeout() {
        let python = paths::python_cmd();
        let args = vec!["-c".to_string(), "import time; time.sleep(30)".to_string()];
        let result = run_process_with_timeout(
            "timeout-test",
            PathBuf::from(python),
            &args,
            None,
            vec![python.to_string()],
            None,
            Duration::from_secs(1),
        );
        if result.stderr.contains("failed to execute command") {
            // Python not installed on this machine; the timeout path cannot be
            // exercised here. The contract is still covered on CI.
            return;
        }
        assert!(result.timed_out);
        assert!(!result.success);
        assert!(result.stderr.contains("timed out after 1s"));
        assert!(result.stderr.contains("process tree was terminated"));
        assert!(result.duration_ms >= 1000);
    }

    #[test]
    fn health_version_probe_is_time_bounded() {
        // The probes run on the boot path (the splash waits on them); this
        // pins that they go through the timeout machinery rather than a raw
        // blocking Command::output().
        let probe = run_version_probe(Path::new(paths::python_cmd()), "--version");
        if probe.stderr.contains("failed to execute command") {
            return; // python missing on this machine; covered on CI.
        }
        assert!(!probe.timed_out);
        assert!(probe.duration_ms < HEALTH_PROBE_TIMEOUT_SECS * 1000);
    }

    #[test]
    fn evidence_reader_rejects_paths_outside_evidence_roots() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let err = resolve_within_evidence_roots(&manifest.to_string_lossy())
            .expect_err("a path outside the evidence roots must be rejected");
        assert!(err.contains("outside the Studio evidence roots"));

        let err = resolve_within_evidence_roots("  ").expect_err("an empty path must be rejected");
        assert!(err.contains("empty"));
    }

    #[test]
    fn truth_summary_defaults_to_not_found_shape() {
        let summary = TruthSummary::default();
        assert!(!summary.found);
        assert!(summary.version.is_none());
        assert!(summary.error.is_none());
    }

    #[test]
    fn crate_version_matches_workspace_release_version() {
        let root_manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../Cargo.toml");
        let contents = fs::read_to_string(&root_manifest)
            .expect("workspace root Cargo.toml must be readable from the studio crate");
        let workspace_version = contents
            .split("[workspace.package]")
            .nth(1)
            .and_then(|section| {
                section.lines().find_map(|line| {
                    let line = line.trim();
                    line.strip_prefix("version = \"")
                        .and_then(|rest| rest.strip_suffix('"'))
                })
            })
            .expect("workspace.package version must be present in the root manifest");
        assert_eq!(
            env!("CARGO_PKG_VERSION"),
            workspace_version,
            "garnet-studio version must track the workspace release version \
             (it is excluded from the workspace, so this guard is the sync gate)"
        );
    }

    #[test]
    fn tauri_conf_does_not_hardcode_a_second_version_stamp() {
        let conf_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
        let raw = fs::read_to_string(conf_path).expect("tauri.conf.json must be readable");
        let value: serde_json::Value =
            serde_json::from_str(&raw).expect("tauri.conf.json must parse");
        assert!(
            value.get("version").is_none(),
            "tauri.conf.json must not duplicate the version; Cargo.toml is the single stamp"
        );
    }
}
