use crate::evidence;
use crate::paths;
use crate::settings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
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
    // The four step scripts come from the same `BootstrapStep`-derived source
    // the run path uses (see `bootstrap_step_files`), so Write and Run cannot
    // drift; the two non-step artifacts (plan JSON + README) are added here.
    let mut files: Vec<(&'static str, String)> = vec![
        ("bootstrap-plan.json", plan_json),
        ("README.md", bootstrap_readme(&health, &plan)),
    ];
    files.extend(bootstrap_step_files(&health));
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

/// The only bootstrap steps Studio will execute. Anything outside this typed
/// allowlist is refused before any process is spawned — Studio never runs an
/// arbitrary shell string, only these four repo-generated PowerShell scripts,
/// each through the same `run_process_with_timeout` path the rest of the app
/// uses. There is deliberately no generic "run this command" surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BootstrapStep {
    Preflight,
    InstallPython,
    BuildCli,
    ConfigureEnv,
}

impl BootstrapStep {
    const ALL: [BootstrapStep; 4] = [
        BootstrapStep::Preflight,
        BootstrapStep::InstallPython,
        BootstrapStep::BuildCli,
        BootstrapStep::ConfigureEnv,
    ];

    fn id(self) -> &'static str {
        match self {
            BootstrapStep::Preflight => "preflight",
            BootstrapStep::InstallPython => "install-python",
            BootstrapStep::BuildCli => "build-cli",
            BootstrapStep::ConfigureEnv => "configure-env",
        }
    }

    fn parse(raw: &str) -> Result<BootstrapStep, String> {
        let id = raw.trim();
        BootstrapStep::ALL
            .into_iter()
            .find(|step| step.id() == id)
            .ok_or_else(|| {
                let allowed = BootstrapStep::ALL
                    .iter()
                    .map(|step| step.id())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("unknown bootstrap step {id:?}; allowed steps: {allowed}")
            })
    }

    /// `build-cli` and `configure-env` operate on the Garnet checkout, so they
    /// require a validated repo root; `preflight` and `install-python` do not.
    fn needs_repo(self) -> bool {
        matches!(self, BootstrapStep::BuildCli | BootstrapStep::ConfigureEnv)
    }

    /// `cargo build --release` is the only step that legitimately runs for
    /// minutes; it gets the matrix budget, the rest the normal command budget.
    fn is_long_running(self) -> bool {
        matches!(self, BootstrapStep::BuildCli)
    }

    fn script_name(self) -> &'static str {
        match self {
            BootstrapStep::Preflight => "run-bootstrap-preflight.ps1",
            BootstrapStep::InstallPython => "install-python-winget.ps1",
            BootstrapStep::BuildCli => "build-garnet-cli-from-repo.ps1",
            BootstrapStep::ConfigureEnv => "configure-garnet-env.ps1",
        }
    }

    /// The exact script body that runs — identical to what
    /// `studio_bootstrap_write_scripts` emits, so "Run" and "Write scripts"
    /// can never diverge.
    fn script_contents(self, health: &HealthStatus) -> String {
        match self {
            BootstrapStep::Preflight => bootstrap_preflight_script(),
            BootstrapStep::InstallPython => install_python_winget_script(),
            BootstrapStep::BuildCli => build_garnet_cli_script(health),
            BootstrapStep::ConfigureEnv => configure_garnet_env_script(health),
        }
    }
}

/// The `(filename, body)` pairs for the four typed steps, derived from
/// `BootstrapStep::ALL`. Both the run path (`studio_bootstrap_run_step`) and the
/// write path (`studio_bootstrap_write_scripts`) build their script set from
/// this one source, so a "Run" step and its written-out script can never drift.
fn bootstrap_step_files(health: &HealthStatus) -> Vec<(&'static str, String)> {
    BootstrapStep::ALL
        .iter()
        .map(|step| (step.script_name(), step.script_contents(health)))
        .collect()
}

/// Repo gate, kept pure for testability: steps that touch the checkout require a
/// validated repo root; the others pass the (optional) root through unchanged.
fn bootstrap_repo_gate(
    step: BootstrapStep,
    repo_root: Option<PathBuf>,
) -> Result<Option<PathBuf>, String> {
    if step.needs_repo() && repo_root.is_none() {
        return Err(format!(
            "bootstrap step '{}' needs a Garnet repo checkout, but none was found. \
             Set GARNET_REPO to a valid Garnet repository (or run the generated script manually).",
            step.id()
        ));
    }
    Ok(repo_root)
}

/// Locate a PowerShell host. Windows ships `powershell.exe` (Windows
/// PowerShell 5.1); `pwsh` (PowerShell 7+) is the fallback. The generated
/// scripts use Windows idioms (winget, User-scope env, LOCALAPPDATA), so this
/// is a Windows-shaped feature — on a host with no PowerShell it refuses
/// honestly rather than claiming a run happened.
fn bootstrap_powershell() -> Option<PathBuf> {
    paths::find_executable("powershell").or_else(|| paths::find_executable("pwsh"))
}

pub(crate) fn studio_bootstrap_run_step_impl(step: String) -> CommandResult {
    // Allowlist first: refuse anything that is not one of the four typed steps,
    // before any repo lookup, filesystem write, or process spawn.
    let step = match BootstrapStep::parse(&step) {
        Ok(step) => step,
        Err(err) => return contract_error(err),
    };
    studio_bootstrap_run_step_resolved(step, paths::find_repo_root())
}

/// Inner runner with the repo root injected, so the refusal branches (repo
/// gate, OS gate) are unit-testable without depending on the test host's
/// checkout (`find_repo_root` otherwise walks up to the real repo in tests).
fn studio_bootstrap_run_step_resolved(
    step: BootstrapStep,
    repo_root: Option<PathBuf>,
) -> CommandResult {
    // Repo gate: build-cli / configure-env require a validated checkout. It runs
    // before the OS gate so the "needs a repo" refusal is reproducible on any
    // platform (the test host always has the checkout).
    let repo_root = match bootstrap_repo_gate(step, repo_root) {
        Ok(repo_root) => repo_root,
        Err(err) => return contract_error(err),
    };

    // Windows-only: the generated scripts use winget, User-scope environment
    // variables, and LOCALAPPDATA, and `[Environment]::SetEnvironmentVariable(
    // ..., 'User')` is a silent no-op off Windows. Refuse honestly rather than
    // run a Windows-shaped script under pwsh and report a hollow "Passed".
    if !cfg!(windows) {
        return contract_error(
            "Garnet Studio bootstrap steps configure a Windows machine (winget, User-scope \
             environment variables, LOCALAPPDATA) and only run on Windows. Use \"Generate Setup \
             Scripts\" to inspect them, or run your platform's setup manually.",
        );
    }

    // Resolve the PowerShell host (no Tauri shell plugin is used; this goes
    // through the same typed Command path as every other Studio command).
    let powershell =
        match bootstrap_powershell() {
            Some(program) => program,
            None => return contract_error(
                "Garnet Studio bootstrap steps run Windows PowerShell scripts, but no PowerShell \
                 host (powershell or pwsh) was found on this machine.",
            ),
        };

    // Write the exact script into a bootstrap-run evidence bundle, then run it.
    // The script that executed is preserved alongside its output.
    let health = cli_health_impl();
    let bundle = match evidence::create_named_bundle("bootstrap-run") {
        Ok(bundle) => bundle,
        Err(err) => return contract_error(err),
    };
    let bundle_path = PathBuf::from(&bundle.path);
    let run_dir = bundle_path.join("bootstrap-run");
    if let Err(err) = fs::create_dir_all(&run_dir) {
        return contract_error(format!("failed to create bootstrap-run directory: {err}"));
    }
    let script_path = run_dir.join(step.script_name());
    if let Err(err) = fs::write(&script_path, step.script_contents(&health)) {
        return contract_error(format!("failed to write {}: {err}", step.script_name()));
    }

    let args = vec![
        "-NoProfile".to_string(),
        "-NonInteractive".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-File".to_string(),
        display_path(&script_path),
    ];
    let command = vec![
        "garnet-studio".to_string(),
        "bootstrap-run".to_string(),
        step.id().to_string(),
    ];
    let settings = settings::load();
    let timeout = Duration::from_secs(if step.is_long_running() {
        settings.matrix_timeout_secs
    } else {
        settings.command_timeout_secs
    });

    run_process_with_timeout(
        "bootstrap-run",
        powershell,
        &args,
        repo_root,
        command,
        Some(bundle_path),
        timeout,
    )
}

#[tauri::command]
pub async fn studio_bootstrap_run_step(step: String) -> Result<CommandResult, String> {
    run_blocking(move || studio_bootstrap_run_step_impl(step)).await
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

/// The `garnet.diff-caps.machine/1` verdict, deserialized verbatim from the CLI.
/// The Studio NEVER recomputes the band or verdict — they are authoritative from
/// `garnet diff-caps --machine`; this only carries what the CLI decided.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiffCapsVerdict {
    pub schema: String,
    pub verdict: String,
    pub authority_expanded: bool,
    pub capability_band: String,
    pub exit_code: i32,
    #[serde(default)]
    pub aggregate_gained: Vec<String>,
    #[serde(default)]
    pub aggregate_removed: Vec<String>,
    #[serde(default)]
    pub wildcard_introduced: bool,
    #[serde(default)]
    pub functions_added: Vec<String>,
    #[serde(default)]
    pub functions_removed: Vec<String>,
    #[serde(default)]
    pub functions_caps_expanded: Vec<DiffCapsFnExpansion>,
    #[serde(default)]
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiffCapsFnExpansion {
    pub name: String,
    #[serde(default)]
    pub gained: Vec<String>,
}

/// What the Diff-Caps panel receives: the parsed verdict (when the gate ran and
/// emitted machine JSON on exit 0/1) plus the evidence trail. On a usage error
/// (exit 2, no JSON) `verdict` is None and `stderr` carries the reason.
#[derive(Debug, Serialize)]
pub struct DiffCapsReport {
    pub ran: bool,
    pub verdict: Option<DiffCapsVerdict>,
    pub exit_code: i32,
    pub stderr: String,
    pub evidence_path: Option<String>,
    pub command: Vec<String>,
}

/// Build the panel report from a raw diff-caps run. Exit 0 (no expansion) and
/// exit 1 (authority expanded) are BOTH valid verdicts — the gate working, not a
/// failure — so we parse the machine JSON for either; only a usage error (exit 2
/// / no JSON) or unparseable output yields `ran = false`.
fn diff_caps_report_from(result: CommandResult) -> DiffCapsReport {
    let verdict = if matches!(result.exit_code, 0 | 1) {
        serde_json::from_str::<DiffCapsVerdict>(result.stdout.trim()).ok()
    } else {
        None
    };
    let stderr = if verdict.is_some() {
        result.stderr
    } else if matches!(result.exit_code, 0 | 1) && result.truncated {
        // The gate ran and emitted a verdict, but its machine JSON was truncated
        // at the display cap — NEVER claim the paths were invalid; point at the
        // sealed full output instead (fail-safe: do not downgrade a real
        // authority-expansion to "could not run").
        match result.evidence_path.as_deref() {
            Some(path) => format!(
                "diff-caps emitted a verdict (exit {}) but its machine JSON was truncated at the display cap; the full verdict is in the evidence bundle at {path}.",
                result.exit_code
            ),
            None => format!(
                "diff-caps emitted a verdict (exit {}) but its machine JSON was truncated at the display cap.",
                result.exit_code
            ),
        }
    } else if result.stderr.trim().is_empty() {
        format!(
            "diff-caps did not emit a verdict (exit {}); check that both paths are valid .garnet files or directories.",
            result.exit_code
        )
    } else {
        result.stderr
    };
    DiffCapsReport {
        ran: verdict.is_some(),
        verdict,
        exit_code: result.exit_code,
        stderr,
        evidence_path: result.evidence_path,
        command: result.command,
    }
}

pub(crate) fn studio_diff_caps_impl(old_path: String, new_path: String) -> DiffCapsReport {
    let result = run_garnet(
        "diff-caps",
        &[
            "diff-caps".to_string(),
            "--machine".to_string(),
            old_path,
            new_path,
        ],
        false,
    );
    diff_caps_report_from(result)
}

#[tauri::command]
pub async fn studio_diff_caps(
    old_path: String,
    new_path: String,
) -> Result<DiffCapsReport, String> {
    run_blocking(move || studio_diff_caps_impl(old_path, new_path)).await
}

/// A `(start, len)` byte span, matching the object form `garnet check --format
/// json` emits (`{"start":N,"len":M}`). Parse diagnostics carry one; check
/// diagnostics are message-only (`span: null`) today.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagSpan {
    pub start: usize,
    pub len: usize,
}

/// One diagnostic from the check JSON. severity/code/message/span are the CLI's
/// own (S44 single source of truth) — the Studio never reclassifies them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VelocityDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub span: Option<DiagSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VelocitySummary {
    #[serde(default)]
    pub errors: usize,
    #[serde(default)]
    pub warnings: usize,
    #[serde(default)]
    pub infos: usize,
    #[serde(default)]
    pub ok: bool,
}

/// The exact wire form of `garnet check --format json`. Both keys are REQUIRED:
/// a bare `{}` / `[]` / `{"x":1}` (a stale or wrong binary printing some other
/// JSON) must NOT deserialize as a clean check report — that would read as a
/// false green.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct CheckJson {
    diagnostics: Vec<VelocityDiagnostic>,
    summary: VelocitySummary,
}

/// What the velocity editor receives. `ran` is true when the CLI emitted
/// parseable diagnostics JSON (exit 0 OR 1 — diagnostics present is the check
/// working, not a failure). `ran = false` means the gate could not run (no CLI /
/// timeout / no JSON).
#[derive(Debug, Serialize)]
pub struct VelocityCheckReport {
    pub ran: bool,
    pub diagnostics: Vec<VelocityDiagnostic>,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub ok: bool,
    pub exit_code: i32,
    pub stderr: String,
}

fn velocity_error(message: impl Into<String>, exit_code: i32) -> VelocityCheckReport {
    VelocityCheckReport {
        ran: false,
        diagnostics: Vec::new(),
        errors: 0,
        warnings: 0,
        infos: 0,
        ok: false,
        exit_code,
        stderr: message.into(),
    }
}

/// Build the report from a raw `check --format json` run. Parses the JSON for any
/// exit (diagnostics-present is exit 1, still a valid result); non-JSON output
/// (CLI missing, timeout, usage error) degrades to `ran = false` with the stderr.
fn velocity_report_from(result: CommandResult) -> VelocityCheckReport {
    match serde_json::from_str::<CheckJson>(result.stdout.trim()) {
        Ok(parsed) => VelocityCheckReport {
            ran: true,
            errors: parsed.summary.errors,
            warnings: parsed.summary.warnings,
            infos: parsed.summary.infos,
            ok: parsed.summary.ok,
            diagnostics: parsed.diagnostics,
            exit_code: result.exit_code,
            stderr: result.stderr,
        },
        Err(_) => {
            let stderr = if result.stderr.trim().is_empty() {
                format!(
                    "garnet check produced no diagnostics JSON (exit {}); the CLI may be missing or the check timed out.",
                    result.exit_code
                )
            } else {
                result.stderr
            };
            velocity_error(stderr, result.exit_code)
        }
    }
}

/// Sequence counter so concurrent debounced checks never collide on a temp path.
static VELOCITY_SEQ: AtomicU64 = AtomicU64::new(0);

pub(crate) fn studio_velocity_check_impl(source: String) -> VelocityCheckReport {
    studio_velocity_check_with_cli(source, paths::find_garnet_cli())
}

/// Inner runner with the CLI path injected, so the no-CLI refusal (and the
/// "no temp file is written before that refusal" invariant) is unit-testable.
fn studio_velocity_check_with_cli(source: String, cli: Option<PathBuf>) -> VelocityCheckReport {
    let cli = match cli {
        Some(path) => path,
        None => {
            return velocity_error(
                "Garnet CLI not found. Set GARNET_CLI or add garnet to PATH.",
                -1,
            )
        }
    };

    // Ephemeral scratch file — NOT an evidence bundle. Live checks must never
    // seal a bundle per keystroke; only the explicit Check/Run buttons do that.
    let scratch = paths::evidence_base_dir().join("garnet-studio-velocity-scratch");
    if let Err(err) = fs::create_dir_all(&scratch) {
        return velocity_error(format!("failed to create scratch directory: {err}"), -1);
    }
    let seq = VELOCITY_SEQ.fetch_add(1, Ordering::Relaxed);
    let temp = scratch.join(format!("buffer-{}-{seq}.garnet", evidence::timestamp()));
    if let Err(err) = fs::write(&temp, &source) {
        return velocity_error(format!("failed to write buffer: {err}"), -1);
    }

    let args = vec![
        "check".to_string(),
        "--format".to_string(),
        "json".to_string(),
        display_path(&temp),
    ];
    let command = command_vector(&cli, &args);
    // No bundle (None) → ephemeral; short timeout → never stall the editor.
    let result = run_process_with_timeout(
        "velocity-check",
        cli,
        &args,
        paths::find_repo_root(),
        command,
        None,
        Duration::from_secs(10),
    );

    let _ = fs::remove_file(&temp); // best-effort cleanup; never a sealed artifact

    velocity_report_from(result)
}

#[tauri::command]
pub async fn studio_velocity_check(source: String) -> Result<VelocityCheckReport, String> {
    run_blocking(move || studio_velocity_check_impl(source)).await
}

// ── Phase 4: Enforced / Declared Legend ─────────────────────────────────────
//
// Calibrated honesty made visible: which fences are runtime-ENFORCED, which are
// merely DECLARED, and which are platform-DEFERRED. The catalog below is the
// single source of truth (it mirrors the parser's `Annotation` set + the
// named-deferred fence list in CLAUDE.md / GARNET_RED_TEAM.md); the renderer
// GENERATES the panel from it — no status is hand-written into HTML.
//
// For the two enforced fences we go further than asserting: a live `garnet
// check` PROBE re-confirms, this run, that the STATIC gate still fires (caps
// under-declaration → `check.caps_coverage`; an out-of-range `@max_depth` →
// `check.annotation_error`). The probe confirms the static gate ONLY; the
// RUNTIME trap (`require_capability`; the recursion trap at N+1) is attested by
// S99/S100/red-team and is reported as attested, never as "confirmed here".

/// Whether a fence is runtime-enforced, merely declared, or platform-deferred.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FenceStatus {
    Enforced,
    Declared,
    Deferred,
}

/// One row of the enforcement legend. A pure data record — the frontend renders
/// it; the status is never spelled into markup.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnforcementFence {
    /// Display name, e.g. `@caps`, `@max_depth`, `@bounded`, `memory`.
    pub name: String,
    pub status: FenceStatus,
    /// Where it bites, e.g. "VM + interpreter", "Wasmtime fuel only", "—".
    pub backends: String,
    /// One-line basis: the trap that enforces it, or why it is deferred.
    pub basis: String,
    /// For ENFORCED fences: how the runtime trap is attested (NOT re-run by the
    /// live probe). Empty for declared / deferred rows.
    pub runtime_attested_by: String,
    /// For ENFORCED fences: the `garnet check` diagnostic code whose presence the
    /// live probe re-confirms. Empty when there is no static-gate probe.
    pub probe_code: String,
}

/// The result of one live static-gate probe for an enforced fence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnforcementProbe {
    /// The fence this probe backs (`@caps` / `@max_depth`).
    pub fence: String,
    /// The check diagnostic code the fixture is expected to provoke.
    pub expected_code: String,
    /// True iff the live `garnet check` emitted `expected_code`. False is an
    /// honest "not confirmed here" — never silently treated as a pass.
    pub confirmed: bool,
    /// True iff the check actually ran (CLI present, JSON parsed). When false the
    /// probe is INCONCLUSIVE, not a confirmation.
    pub ran: bool,
    pub exit_code: i32,
    /// Every diagnostic code the fixture actually produced, verbatim.
    pub observed_codes: Vec<String>,
}

/// The whole legend payload the frontend renders.
#[derive(Debug, Serialize)]
pub struct EnforcementLegend {
    pub fences: Vec<EnforcementFence>,
    pub probes: Vec<EnforcementProbe>,
    /// True iff a Garnet CLI was found to run the probes. When false the enforced
    /// rows still render, but as claimed-not-confirmed-here (honest).
    pub cli_available: bool,
}

/// Probe fixture: a `@caps()` function that transitively calls the `read_file`
/// primitive (which requires `fs`). `garnet check` must flag `check.caps_coverage`.
const CAPS_PROBE_SRC: &str =
    "@caps()\ndef caller() -> String {\n  read_file(\"/tmp/garnet-studio-legend-probe\")\n}\n";

/// Probe fixture: `@max_depth(100)` is outside the enforced `1..=64` range, so
/// `garnet check` must flag `check.annotation_error`.
const DEPTH_PROBE_SRC: &str = "@max_depth(100)\n@caps()\ndef f() -> Bool { true }\n";

/// The fence catalog — the single source of truth, drawn from the parser's
/// `Annotation` set and the named-deferred fence list (CLAUDE.md / red-team).
fn enforcement_catalog() -> Vec<EnforcementFence> {
    vec![
        EnforcementFence {
            name: "@caps".to_string(),
            status: FenceStatus::Enforced,
            backends: "VM + interpreter".to_string(),
            basis: "Deny-by-default host authority: an undeclared fs/net/env/proc \
                    primitive traps at the boundary. The static caps-coverage gate \
                    flags a function that transitively requires a capability it does \
                    not declare."
                .to_string(),
            runtime_attested_by:
                "S100 require_capability trap (VM + interp); S114-FIX-2 deny-by-default at \
                 active_frames == 0; red-team"
                    .to_string(),
            probe_code: "check.caps_coverage".to_string(),
        },
        EnforcementFence {
            name: "@max_depth".to_string(),
            status: FenceStatus::Enforced,
            backends: "VM + interpreter".to_string(),
            basis: "Per-function recursion ceiling, trapped at depth N+1. The static \
                    gate enforces the 1..=64 range at check time."
                .to_string(),
            runtime_attested_by: "S99 recursion-depth trap (VM + interp); red-team".to_string(),
            probe_code: "check.annotation_error".to_string(),
        },
        EnforcementFence {
            name: "@bounded".to_string(),
            status: FenceStatus::Declared,
            backends: "Wasmtime fuel only".to_string(),
            basis: "Lowers to Wasmtime fuel metering on the VM path (S39); not enforced \
                    on the interpreter. Declared, not a cross-backend trap."
                .to_string(),
            runtime_attested_by: String::new(),
            probe_code: String::new(),
        },
        EnforcementFence {
            name: "@mailbox".to_string(),
            status: FenceStatus::Declared,
            backends: "actor runtime".to_string(),
            basis: "Overrides the default 1024-message inbox cap for an actor; not \
                    enforced at the host-authority boundary."
                .to_string(),
            runtime_attested_by: String::new(),
            probe_code: String::new(),
        },
        EnforcementFence {
            name: "memory".to_string(),
            status: FenceStatus::Declared,
            backends: "—".to_string(),
            basis: "Named-deferred resource ceiling: declared in source, no runtime trap."
                .to_string(),
            runtime_attested_by: String::new(),
            probe_code: String::new(),
        },
        EnforcementFence {
            name: "time".to_string(),
            status: FenceStatus::Declared,
            backends: "—".to_string(),
            basis: "Named-deferred resource ceiling: `check` flags top-level \
                    under-declaration, but there is no runtime trap."
                .to_string(),
            runtime_attested_by: String::new(),
            probe_code: String::new(),
        },
        EnforcementFence {
            name: "OS sandbox (macOS / Windows)".to_string(),
            status: FenceStatus::Deferred,
            backends: "Linux seccomp only".to_string(),
            basis: "Platform OS-sandbox application is deferred off Linux. Linux applies a \
                    seccomp policy; macOS and Windows do not apply an OS sandbox."
                .to_string(),
            runtime_attested_by: String::new(),
            probe_code: String::new(),
        },
    ]
}

/// Interpret a check report into a probe verdict. Pure (no CLI) so the
/// confirm/inconclusive logic is unit-testable. A probe confirms ONLY when the
/// check ran AND emitted the expected code; a stale/missing CLI (`ran = false`)
/// is inconclusive, never a confirmation.
fn probe_from_report(
    fence: &str,
    expected_code: &str,
    report: &VelocityCheckReport,
) -> EnforcementProbe {
    let observed_codes: Vec<String> = report.diagnostics.iter().map(|d| d.code.clone()).collect();
    let confirmed = report.ran && observed_codes.iter().any(|c| c == expected_code);
    EnforcementProbe {
        fence: fence.to_string(),
        expected_code: expected_code.to_string(),
        confirmed,
        ran: report.ran,
        exit_code: report.exit_code,
        observed_codes,
    }
}

/// Run one live static-gate probe through the velocity check plumbing (ephemeral
/// temp file, no seal, check-only — never executes the fixture).
fn run_enforcement_probe(
    fence: &str,
    expected_code: &str,
    src: &str,
    cli: Option<PathBuf>,
) -> EnforcementProbe {
    let report = studio_velocity_check_with_cli(src.to_string(), cli);
    probe_from_report(fence, expected_code, &report)
}

pub(crate) fn studio_enforcement_legend_impl() -> EnforcementLegend {
    studio_enforcement_legend_with_cli(paths::find_garnet_cli())
}

/// Pairs each enforced fence with the fixture whose `garnet check` run must
/// reproduce that fence's catalog `probe_code`. The EXPECTED code is read from
/// the catalog, never duplicated here — one source of truth for the code.
const PROBE_FIXTURES: [(&str, &str); 2] =
    [("@caps", CAPS_PROBE_SRC), ("@max_depth", DEPTH_PROBE_SRC)];

/// Inner builder with the CLI path injected, so the no-CLI (probes inconclusive)
/// path is unit-testable.
fn studio_enforcement_legend_with_cli(cli: Option<PathBuf>) -> EnforcementLegend {
    let fences = enforcement_catalog();
    let cli_available = cli.is_some();
    // Each probe's expected code comes from the matching catalog row's
    // `probe_code`, so the code the UI displays and the code the probe
    // re-confirms can never silently diverge.
    let probes = PROBE_FIXTURES
        .iter()
        .filter_map(|(name, src)| {
            let code = fences.iter().find(|f| &f.name == name)?.probe_code.clone();
            Some(run_enforcement_probe(name, &code, src, cli.clone()))
        })
        .collect();
    EnforcementLegend {
        fences,
        probes,
        cli_available,
    }
}

#[tauri::command]
pub async fn studio_enforcement_legend() -> Result<EnforcementLegend, String> {
    run_blocking(studio_enforcement_legend_impl).await
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
        r#"param(
  [string]$Repo = {repo_default},
  [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Garnet\bin')
)

$ErrorActionPreference = 'Stop'

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
        r#"param(
  [string]$Repo = {repo_default},
  [string]$Cli = {cli_default}
)

$ErrorActionPreference = 'Stop'

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

    fn diff_caps_command_result(stdout: &str, stderr: &str, exit_code: i32) -> CommandResult {
        CommandResult {
            success: exit_code == 0,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            exit_code,
            command: vec![
                "garnet".to_string(),
                "diff-caps".to_string(),
                "--machine".to_string(),
            ],
            evidence_path: Some("C:/dogfood/diff-caps-20260628".to_string()),
            timed_out: false,
            duration_ms: 7,
            truncated: false,
        }
    }

    // The EXACT three-clause scope string the CLI emits
    // (garnet-cli/src/cmd/diff_caps.rs). Pinned verbatim so a future CLI wording
    // drift fails this suite instead of silently shipping.
    const DIFF_CAPS_SCOPE: &str = "declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface";

    const DIFF_CAPS_EXPANDED: &str = r#"{"schema":"garnet.diff-caps.machine/1","verdict":"authority-expanded","authority_expanded":true,"capability_band":"2/5","exit_code":1,"aggregate_gained":["net","proc"],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[{"name":"handle_request","gained":["net"]}],"scope":"declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface"}"#;

    #[test]
    fn diff_caps_treats_exit_1_expansion_as_a_valid_verdict_not_a_failure() {
        // exit 1 means "authority expanded" — the gate WORKING, not an error.
        let report = diff_caps_report_from(diff_caps_command_result(DIFF_CAPS_EXPANDED, "", 1));
        assert!(report.ran, "exit 1 must parse a verdict");
        let v = report.verdict.expect("verdict present");
        assert_eq!(v.capability_band, "2/5");
        assert_eq!(v.verdict, "authority-expanded");
        assert!(v.authority_expanded);
        assert_eq!(
            v.aggregate_gained,
            vec!["net".to_string(), "proc".to_string()]
        );
        assert_eq!(v.functions_caps_expanded[0].name, "handle_request");
        assert_eq!(v.functions_caps_expanded[0].gained, vec!["net".to_string()]);
        // the evidence trail is preserved
        assert_eq!(
            report.evidence_path.as_deref(),
            Some("C:/dogfood/diff-caps-20260628")
        );
    }

    #[test]
    fn diff_caps_parses_clean_verdict_from_exit_0() {
        let json = r#"{"schema":"garnet.diff-caps.machine/1","verdict":"no-authority-expansion","authority_expanded":false,"capability_band":"5/5","exit_code":0,"aggregate_gained":[],"aggregate_removed":[],"wildcard_introduced":false,"functions_added":[],"functions_removed":[],"functions_caps_expanded":[],"scope":"declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface"}"#;
        let report = diff_caps_report_from(diff_caps_command_result(json, "", 0));
        assert!(report.ran);
        let v = report.verdict.unwrap();
        assert_eq!(v.capability_band, "5/5");
        assert_eq!(v.verdict, "no-authority-expansion");
        assert!(!v.authority_expanded);
        // The scope caveat must be present on the CLEAN (5/5) path too — green is
        // "no declared widening", NOT "safe".
        assert_eq!(v.scope, DIFF_CAPS_SCOPE);
    }

    #[test]
    fn diff_caps_reports_usage_error_without_a_verdict() {
        // exit 2 = usage error, no machine JSON; the panel must NOT invent a verdict.
        let report = diff_caps_report_from(diff_caps_command_result("", "path not found", 2));
        assert!(!report.ran);
        assert!(report.verdict.is_none());
        assert_eq!(report.exit_code, 2);
        assert!(report.stderr.contains("path not found"));
    }

    #[test]
    fn diff_caps_surfaces_the_cli_band_verbatim_never_recomputed() {
        // The band/verdict are carried straight from the CLI JSON — the Studio
        // deserializes, it never derives the band from the gained/removed sets.
        let report = diff_caps_report_from(diff_caps_command_result(DIFF_CAPS_EXPANDED, "", 1));
        let v = report.verdict.unwrap();
        // The full three-clause caveat is carried verbatim, matching the CLI.
        assert_eq!(v.scope, DIFF_CAPS_SCOPE);
        assert_eq!(v.schema, "garnet.diff-caps.machine/1");
    }

    #[test]
    fn diff_caps_truncated_machine_json_does_not_blame_the_paths() {
        // A real exit-1 expansion whose JSON was truncated at the display cap must
        // NOT be downgraded to "check that both paths are valid" — that inverts the
        // gate. It must name the truncation and point at the evidence bundle.
        let mut result = diff_caps_command_result("{\"schema\":\"garnet.dif…[truncated]", "", 1);
        result.truncated = true;
        let report = diff_caps_report_from(result);
        assert!(!report.ran);
        assert!(report.stderr.contains("truncated"));
        assert!(report.stderr.contains("evidence bundle"));
        assert!(!report.stderr.contains("check that both paths"));
    }

    #[test]
    fn diff_caps_unparseable_success_output_degrades_to_no_verdict() {
        // exit 0/1 but stdout is not valid machine JSON (stray banner, schema/2,
        // partial pipe): a no-verdict error, never a half-rendered card.
        for exit in [0, 1] {
            let report =
                diff_caps_report_from(diff_caps_command_result("not json at all", "", exit));
            assert!(!report.ran, "exit {exit} garbage stdout must not parse");
            assert!(report.verdict.is_none());
            assert!(report.stderr.contains("did not emit a verdict"));
        }
        // A present-but-incompatible payload (missing the required `verdict` field).
        let partial = r#"{"schema":"garnet.diff-caps.machine/1","capability_band":"2/5"}"#;
        let report = diff_caps_report_from(diff_caps_command_result(partial, "", 1));
        assert!(!report.ran);
    }

    #[test]
    fn diff_caps_synthetic_message_fires_on_empty_streams() {
        // exit 2 with EMPTY stderr must still give actionable guidance with the
        // real exit interpolated — the message is otherwise dead in coverage.
        let report = diff_caps_report_from(diff_caps_command_result("", "", 2));
        assert!(report.stderr.contains("did not emit a verdict"));
        assert!(report.stderr.contains("exit 2"));
    }

    fn velocity_cmd_result(stdout: &str, stderr: &str, exit_code: i32) -> CommandResult {
        CommandResult {
            success: exit_code == 0,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
            exit_code,
            command: vec![
                "garnet".to_string(),
                "check".to_string(),
                "--format".to_string(),
                "json".to_string(),
            ],
            evidence_path: None,
            timed_out: false,
            duration_ms: 5,
            truncated: false,
        }
    }

    #[test]
    fn velocity_parses_a_check_diagnostic_with_no_span() {
        let json = r#"{"diagnostics":[{"severity":"error","code":"check.caps_coverage","message":"fs::read_file requires @caps(fs)","span":null}],"summary":{"errors":1,"warnings":0,"infos":0,"ok":false}}"#;
        let report = velocity_report_from(velocity_cmd_result(json, "", 1));
        assert!(report.ran);
        assert_eq!(report.errors, 1);
        assert!(!report.ok);
        let d = &report.diagnostics[0];
        assert_eq!(d.severity, "error");
        assert_eq!(d.code, "check.caps_coverage");
        assert!(d.span.is_none(), "check diagnostics are message-only today");
    }

    #[test]
    fn velocity_parses_a_parse_diagnostic_with_a_byte_span() {
        // span is the object {start,len} the CLI emits — proves the DiagSpan
        // struct matches the wire form (a (usize,usize) tuple would mis-parse).
        let json = r#"{"diagnostics":[{"severity":"error","code":"parse.reserved_word","message":"reserved word","span":{"start":12,"len":3}}],"summary":{"errors":1,"warnings":0,"infos":0,"ok":false}}"#;
        let report = velocity_report_from(velocity_cmd_result(json, "", 1));
        assert!(report.ran);
        let span = report.diagnostics[0]
            .span
            .as_ref()
            .expect("parse span present");
        assert_eq!(span.start, 12);
        assert_eq!(span.len, 3);
    }

    #[test]
    fn velocity_clean_buffer_is_ok_with_no_diagnostics() {
        let json = r#"{"diagnostics":[],"summary":{"errors":0,"warnings":0,"infos":0,"ok":true}}"#;
        let report = velocity_report_from(velocity_cmd_result(json, "", 0));
        assert!(report.ran);
        assert!(report.ok);
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn velocity_exit_1_diagnostics_is_a_result_not_a_failure() {
        let json = r#"{"diagnostics":[{"severity":"warning","code":"check.boundary_note","message":"note","span":null}],"summary":{"errors":0,"warnings":1,"infos":0,"ok":true}}"#;
        let report = velocity_report_from(velocity_cmd_result(json, "", 1));
        assert!(
            report.ran,
            "diagnostics present (exit 1) is the check working"
        );
        assert_eq!(report.warnings, 1);
    }

    #[test]
    fn velocity_non_json_output_degrades_to_ran_false() {
        let report =
            velocity_report_from(velocity_cmd_result("garnet: command not found", "boom", -1));
        assert!(!report.ran);
        assert!(report.diagnostics.is_empty());
        assert_eq!(report.stderr, "boom");
    }

    #[test]
    fn velocity_empty_streams_get_actionable_guidance() {
        let report = velocity_report_from(velocity_cmd_result("", "", 2));
        assert!(!report.ran);
        assert!(report.stderr.contains("no diagnostics JSON"));
        assert!(report.stderr.contains("exit 2"));
    }

    #[test]
    fn velocity_rejects_a_bare_json_object_as_not_a_check_report() {
        // A stale/wrong CLI printing some other JSON must NOT read as a clean run —
        // both `diagnostics` and `summary` keys are required to count as "ran".
        for junk in [
            "{}",
            "[]",
            r#"{"hello":"world"}"#,
            r#"{"summary":{"ok":true}}"#,
        ] {
            let report = velocity_report_from(velocity_cmd_result(junk, "", 0));
            assert!(!report.ran, "{junk:?} must not parse as a check report");
        }
    }

    #[test]
    fn velocity_check_refuses_without_a_cli_and_writes_no_temp_file() {
        // The CLI-not-found refusal happens before any scratch temp file is written.
        let report = studio_velocity_check_with_cli("let x = 1".to_string(), None);
        assert!(!report.ran);
        assert_eq!(report.exit_code, -1);
        assert!(report.stderr.contains("Garnet CLI not found"));
    }

    // ── Phase 4: Enforced / Declared Legend ──────────────────────────────

    fn legend_report(ran: bool, codes: &[&str]) -> VelocityCheckReport {
        VelocityCheckReport {
            ran,
            diagnostics: codes
                .iter()
                .map(|c| VelocityDiagnostic {
                    severity: "error".to_string(),
                    code: (*c).to_string(),
                    message: "x".to_string(),
                    span: None,
                })
                .collect(),
            errors: codes.len(),
            warnings: 0,
            infos: 0,
            ok: false,
            exit_code: if codes.is_empty() { 0 } else { 1 },
            stderr: String::new(),
        }
    }

    #[test]
    fn enforcement_catalog_covers_every_fence_with_an_honest_status() {
        let fences = enforcement_catalog();
        let names: Vec<&str> = fences.iter().map(|f| f.name.as_str()).collect();
        for expected in [
            "@caps",
            "@max_depth",
            "@bounded",
            "@mailbox",
            "memory",
            "time",
            "OS sandbox (macOS / Windows)",
        ] {
            assert!(names.contains(&expected), "missing fence: {expected}");
        }

        for f in &fences {
            match f.status {
                FenceStatus::Enforced => {
                    // Enforced rows MUST carry a static-gate probe code and a
                    // runtime-trap attestation — never a bare "enforced" claim.
                    assert!(!f.probe_code.is_empty(), "{} needs a probe code", f.name);
                    assert!(
                        !f.runtime_attested_by.is_empty(),
                        "{} needs a runtime attestation",
                        f.name
                    );
                }
                FenceStatus::Declared | FenceStatus::Deferred => {
                    // A declared/deferred fence must NOT advertise a probe or a
                    // runtime trap — that would overclaim enforcement.
                    assert!(
                        f.probe_code.is_empty(),
                        "{} is not enforced; it must not carry a probe code",
                        f.name
                    );
                    assert!(
                        f.runtime_attested_by.is_empty(),
                        "{} is not enforced; it must not claim a runtime trap",
                        f.name
                    );
                }
            }
        }

        let enforced: Vec<&str> = fences
            .iter()
            .filter(|f| f.status == FenceStatus::Enforced)
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(
            enforced,
            vec!["@caps", "@max_depth"],
            "only @caps and @max_depth are enforced"
        );
    }

    #[test]
    fn enforcement_probe_confirms_only_on_the_expected_code_from_a_run_that_ran() {
        // Confirmed: the check ran and emitted the expected code.
        let hit = probe_from_report(
            "@caps",
            "check.caps_coverage",
            &legend_report(true, &["check.caps_coverage"]),
        );
        assert!(hit.confirmed);
        assert!(hit.ran);
        assert_eq!(hit.observed_codes, vec!["check.caps_coverage".to_string()]);

        // Ran, but a different code → not confirmed (no false enforcement claim).
        let miss = probe_from_report(
            "@caps",
            "check.caps_coverage",
            &legend_report(true, &["parse.reserved_word"]),
        );
        assert!(!miss.confirmed);

        // Did not run (no CLI / no JSON) → inconclusive, even if a code is present.
        let stale = probe_from_report(
            "@caps",
            "check.caps_coverage",
            &legend_report(false, &["check.caps_coverage"]),
        );
        assert!(!stale.confirmed, "a check that did not run cannot confirm");
        assert!(!stale.ran);
    }

    #[test]
    fn enforcement_legend_without_a_cli_renders_but_marks_probes_inconclusive() {
        // No CLI: the legend still lists every fence (honesty must render even
        // offline), but the enforced rows are NOT confirmed here.
        let legend = studio_enforcement_legend_with_cli(None);
        assert!(!legend.cli_available);
        assert_eq!(legend.fences.len(), 7);
        assert_eq!(legend.probes.len(), 2);
        for p in &legend.probes {
            assert!(!p.ran, "{} probe cannot run without a CLI", p.fence);
            assert!(!p.confirmed, "{} probe must not be confirmed", p.fence);
        }
    }

    #[test]
    fn enforcement_probe_expected_code_is_sourced_from_the_catalog() {
        // The code each probe re-confirms must be the SAME code the catalog row
        // displays for that fence — one source of truth, so the UI can never show
        // a code the probe did not actually check.
        let legend = studio_enforcement_legend_with_cli(None);
        for probe in &legend.probes {
            let fence = legend
                .fences
                .iter()
                .find(|f| f.name == probe.fence)
                .expect("every probe's fence is in the catalog");
            assert_eq!(
                probe.expected_code, fence.probe_code,
                "{}: probe expected_code must equal the catalog probe_code",
                probe.fence
            );
            assert!(
                !probe.expected_code.is_empty(),
                "{}: empty code",
                probe.fence
            );
        }
    }

    #[test]
    fn enforcement_probe_confirms_amid_noise_but_not_on_an_empty_run() {
        // A confirm is valid when the expected code is present ALONGSIDE others.
        let noisy = probe_from_report(
            "@caps",
            "check.caps_coverage",
            &legend_report(true, &["check.caps_coverage", "check.boundary_note"]),
        );
        assert!(
            noisy.confirmed,
            "expected code present among others must confirm"
        );

        // A check that ran but produced NO diagnostics is not a confirmation.
        let empty = probe_from_report("@caps", "check.caps_coverage", &legend_report(true, &[]));
        assert!(!empty.confirmed, "a run with no diagnostics cannot confirm");
        assert!(empty.ran);
    }

    #[test]
    fn enforcement_legend_with_a_real_cli_confirms_both_static_gates() {
        // Live coverage of the confirm path: wherever a Garnet CLI is found
        // (locally, and on CI where the release binary is built), the two
        // static-gate probes must actually reproduce their expected diagnostic —
        // this is what pins "confirmed live this run" against fixture or
        // diagnostic-code drift. With no CLI we SKIP loudly, never pass vacuously.
        let Some(cli) = paths::find_garnet_cli() else {
            eprintln!(
                "SKIP enforcement_legend_with_a_real_cli_confirms_both_static_gates: \
                 no Garnet CLI found (set GARNET_CLI or add garnet to PATH)."
            );
            return;
        };
        let legend = studio_enforcement_legend_with_cli(Some(cli));
        assert!(legend.cli_available);
        assert_eq!(legend.probes.len(), 2);
        for probe in &legend.probes {
            assert!(probe.ran, "{} probe must run with a real CLI", probe.fence);
            assert!(
                probe.confirmed,
                "{} probe must confirm: expected `{}` in {:?} (exit {})",
                probe.fence, probe.expected_code, probe.observed_codes, probe.exit_code
            );
            assert!(
                probe.observed_codes.contains(&probe.expected_code),
                "{} observed_codes must include the expected code",
                probe.fence
            );
        }
    }

    #[test]
    fn bootstrap_step_parse_rejects_anything_off_the_allowlist() {
        // Studio must never run an arbitrary string as a bootstrap step.
        for bad in [
            "rm -rf /",
            "install-everything",
            "build",
            "",
            "preflight; calc",
        ] {
            let err = BootstrapStep::parse(bad).unwrap_err();
            assert!(err.contains("unknown bootstrap step"), "got: {err}");
            assert!(
                err.contains("preflight"),
                "error must list allowed steps: {err}"
            );
        }
    }

    #[test]
    fn bootstrap_step_parse_accepts_exactly_the_four_typed_ids() {
        assert_eq!(
            BootstrapStep::parse("preflight"),
            Ok(BootstrapStep::Preflight)
        );
        assert_eq!(
            BootstrapStep::parse("install-python"),
            Ok(BootstrapStep::InstallPython)
        );
        // surrounding whitespace is trimmed, not an injection vector
        assert_eq!(
            BootstrapStep::parse("  build-cli  "),
            Ok(BootstrapStep::BuildCli)
        );
        assert_eq!(
            BootstrapStep::parse("configure-env"),
            Ok(BootstrapStep::ConfigureEnv)
        );
    }

    #[test]
    fn bootstrap_repo_gate_blocks_repo_steps_when_no_checkout_is_found() {
        for step in [BootstrapStep::BuildCli, BootstrapStep::ConfigureEnv] {
            let err = bootstrap_repo_gate(step, None).unwrap_err();
            assert!(err.contains(step.id()), "error must name the step: {err}");
            assert!(
                err.contains("Garnet repo"),
                "error must explain the cause: {err}"
            );
        }
    }

    #[test]
    fn bootstrap_repo_gate_lets_repoless_steps_run_and_passes_a_repo_through() {
        assert_eq!(
            bootstrap_repo_gate(BootstrapStep::Preflight, None),
            Ok(None)
        );
        assert_eq!(
            bootstrap_repo_gate(BootstrapStep::InstallPython, None),
            Ok(None)
        );
        let repo = Some(PathBuf::from("/some/garnet"));
        assert_eq!(
            bootstrap_repo_gate(BootstrapStep::BuildCli, repo.clone()),
            Ok(repo)
        );
    }

    #[test]
    fn bootstrap_step_intent_mapping_is_stable() {
        assert!(BootstrapStep::BuildCli.needs_repo());
        assert!(BootstrapStep::ConfigureEnv.needs_repo());
        assert!(!BootstrapStep::Preflight.needs_repo());
        assert!(!BootstrapStep::InstallPython.needs_repo());
        // Only the cargo build legitimately runs for minutes.
        assert!(BootstrapStep::BuildCli.is_long_running());
        assert!(!BootstrapStep::Preflight.is_long_running());
        assert!(!BootstrapStep::ConfigureEnv.is_long_running());
    }

    #[test]
    fn bootstrap_run_step_impl_refuses_unknown_step_before_spawning_anything() {
        let result = studio_bootstrap_run_step_impl("install-everything".to_string());
        assert!(!result.success);
        assert!(result.stderr.contains("unknown bootstrap step"));
        // Refused at the allowlist — no process spawned, no evidence bundle.
        assert!(result.evidence_path.is_none());
        assert_eq!(result.exit_code, -1);
    }

    fn sample_health() -> HealthStatus {
        HealthStatus {
            cli_found: true,
            cli_path: "C:/Garnet/bin/garnet.exe".to_string(),
            cli_version: "garnet 0.8.1".to_string(),
            repo_found: true,
            repo_path: "C:/Garnet".to_string(),
            python_found: true,
            python_version: "Python 3.12.0".to_string(),
            evidence_dir: "C:/dogfood/garnet-studio-windows-linux".to_string(),
            platform: "windows".to_string(),
            arch: "x86_64".to_string(),
        }
    }

    #[test]
    fn bootstrap_run_step_resolved_refuses_a_repo_step_without_a_checkout() {
        // The repo gate must refuse through the WIRED impl path (before any
        // bundle is created or process spawned), not only as a pure function.
        for step in [BootstrapStep::BuildCli, BootstrapStep::ConfigureEnv] {
            let result = studio_bootstrap_run_step_resolved(step, None);
            assert!(!result.success, "{} should be refused", step.id());
            assert!(result.stderr.contains("needs a Garnet repo"));
            assert!(
                result.evidence_path.is_none(),
                "no bundle may be created for a refused step"
            );
            assert_eq!(result.exit_code, -1);
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn bootstrap_run_step_resolved_refuses_off_windows_instead_of_faking_success() {
        // preflight needs no repo, so it clears the repo gate and hits the OS
        // gate; off Windows it must refuse rather than run a Windows-shaped
        // script under pwsh and report a hollow "Passed".
        let result = studio_bootstrap_run_step_resolved(BootstrapStep::Preflight, None);
        assert!(!result.success);
        assert!(result.stderr.contains("only run on Windows"));
        assert!(result.evidence_path.is_none());
    }

    #[test]
    fn repo_scripts_declare_the_param_block_first() {
        // PowerShell requires param() to be the first statement; a regression
        // that puts $ErrorActionPreference (or anything) before it makes the
        // build-cli / configure-env scripts a parse error at runtime.
        let health = sample_health();
        for step in [BootstrapStep::BuildCli, BootstrapStep::ConfigureEnv] {
            let body = step.script_contents(&health);
            let first = body
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty() && !line.starts_with('#'))
                .unwrap_or("");
            assert!(
                first.starts_with("param("),
                "{} must open with param(): got {first:?}",
                step.id()
            );
            let param_at = body.find("param(").unwrap();
            let eap_at = body.find("$ErrorActionPreference").unwrap();
            assert!(
                param_at < eap_at,
                "{}: param() must precede $ErrorActionPreference",
                step.id()
            );
        }
    }

    #[test]
    fn bootstrap_step_files_are_the_single_source_for_write_and_run() {
        let health = sample_health();
        let files = bootstrap_step_files(&health);
        let names: Vec<_> = files.iter().map(|(name, _)| *name).collect();
        let expected: Vec<_> = BootstrapStep::ALL.iter().map(|s| s.script_name()).collect();
        assert_eq!(
            names, expected,
            "the write path must emit exactly the enum's scripts"
        );
        // Each written body is byte-identical to what the run path executes.
        for (step, (_, body)) in BootstrapStep::ALL.iter().zip(files.iter()) {
            assert_eq!(*body, step.script_contents(&health));
        }
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
