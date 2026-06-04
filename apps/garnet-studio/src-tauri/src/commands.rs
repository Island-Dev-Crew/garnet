use crate::evidence;
use crate::paths;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

#[derive(Debug, Serialize)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub command: Vec<String>,
    pub evidence_path: Option<String>,
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
pub struct LanguageGroup {
    pub name: String,
    pub languages: Vec<String>,
}

#[tauri::command]
pub fn cli_health() -> HealthStatus {
    let (cli_found, cli_path, cli_version) = match paths::find_garnet_cli() {
        Some(path) => {
            let version = Command::new(&path)
                .arg("version")
                .output()
                .ok()
                .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
                .filter(|version| !version.is_empty())
                .unwrap_or_else(|| "unknown".to_string());
            (true, display_path(&path), version)
        }
        None => (false, String::new(), String::new()),
    };

    let (repo_found, repo_path) = match paths::find_repo_root() {
        Some(path) => (true, display_path(&path)),
        None => (false, String::new()),
    };

    let (python_found, python_version) =
        match Command::new(paths::python_cmd()).arg("--version").output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let version = if stdout.is_empty() { stderr } else { stdout };
                (output.status.success(), version)
            }
            Err(_) => (false, String::new()),
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
pub fn cli_parse(file_path: String) -> CommandResult {
    run_garnet("parse", &["parse".to_string(), file_path], false)
}

#[tauri::command]
pub fn cli_check(file_path: String) -> CommandResult {
    run_garnet("check", &["check".to_string(), file_path], false)
}

#[tauri::command]
pub fn cli_run(file_path: String) -> CommandResult {
    run_garnet("run", &["run".to_string(), file_path], true)
}

#[tauri::command]
pub fn cli_convert(source_file: String, source_lang: String) -> CommandResult {
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
pub fn advisory_assist_plan(source_file: String, language: String) -> CommandResult {
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
pub fn advisory_bundle(source_file: String, language: String) -> CommandResult {
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
}

#[tauri::command]
pub fn advisory_review(bundle_dir: String) -> CommandResult {
    run_advisory_script(
        "advisory-review",
        "garnet_converter_advisory_review.py",
        vec!["--bundle-dir".to_string(), bundle_dir],
    )
}

#[tauri::command]
pub fn advisory_handoff(bundle_dir: String, review_dir: String) -> CommandResult {
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
}

#[tauri::command]
pub fn objective_pulse() -> CommandResult {
    run_python_script(
        "objective-pulse",
        "garnet_mit_readiness_status.py",
        vec!["--format".to_string(), "markdown".to_string()],
    )
}

#[tauri::command]
pub fn agentic_dogfood_matrix() -> CommandResult {
    run_python_script(
        "agentic-dogfood",
        "run_agentic_dogfood_matrix.py",
        vec!["--copy-to-desktop".to_string(), "--strict".to_string()],
    )
}

#[tauri::command]
pub fn domain_proof_matrix() -> CommandResult {
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
pub fn mac_domain_proofs() -> CommandResult {
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
pub fn windows_linux_studio_status() -> CommandResult {
    run_python_script(
        "windows-linux-studio-status",
        "garnet_windows_linux_studio_status.py",
        vec!["--format".to_string(), "markdown".to_string()],
    )
}

#[tauri::command]
pub fn converter_status() -> CommandResult {
    run_python_script(
        "converter-status",
        "garnet_converter_status.py",
        vec!["--format".to_string(), "markdown".to_string()],
    )
}

#[tauri::command]
pub fn provider_options() -> CommandResult {
    run_report_script_with_output_dir(
        "provider-options",
        "garnet_converter_llm_feasibility.py",
        "markdown",
    )
}

#[tauri::command]
pub fn mit_demo_route() -> CommandResult {
    run_report_script_with_output_dir("mit-demo-route", "garnet_mit_demo_route.py", "markdown")
}

#[tauri::command]
pub fn mit_deck_outline() -> CommandResult {
    run_report_script_with_output_dir("mit-deck-outline", "garnet_mit_deck_outline.py", "markdown")
}

#[tauri::command]
pub fn mit_deck_preview() -> CommandResult {
    run_report_script_with_output_dir("mit-deck-preview", "garnet_mit_deck_preview.py", "html")
}

#[tauri::command]
pub fn mac_continuation_pulse() -> CommandResult {
    run_python_script(
        "mac-continuation-pulse",
        "garnet_mac_side_continuation_status.py",
        vec!["--format".to_string(), "markdown".to_string()],
    )
}

#[tauri::command]
pub fn proof_benchmark_status() -> CommandResult {
    run_report_script_with_output_dir(
        "proof-benchmark-status",
        "garnet_proof_benchmark_status.py",
        "markdown",
    )
}

#[tauri::command]
pub fn benchmark_no_run() -> CommandResult {
    run_report_script_with_output_dir("benchmark-no-run", "garnet_benchmark_no_run.py", "markdown")
}

#[tauri::command]
pub fn notarization_status() -> CommandResult {
    run_python_script(
        "notarization-status",
        "garnet_studio_notarization_status.py",
        vec!["--format".to_string(), "markdown".to_string()],
    )
}

#[tauri::command]
pub fn windows_vm_installer_status() -> CommandResult {
    run_python_script(
        "windows-vm-installer-status",
        "garnet_windows_clean_vm_installer_status.py",
        vec!["--format".to_string(), "markdown".to_string()],
    )
}

#[tauri::command]
pub fn create_evidence_bundle() -> Result<evidence::EvidenceBundle, String> {
    evidence::create_bundle()
}

#[tauri::command]
pub fn get_evidence_dir() -> String {
    display_path(&paths::evidence_base_dir())
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

fn run_process(
    category: &str,
    program: PathBuf,
    args: &[String],
    working_dir: Option<PathBuf>,
    command: Vec<String>,
    bundle: Option<PathBuf>,
) -> CommandResult {
    let mut process = Command::new(&program);
    process.args(args);
    if let Some(dir) = &working_dir {
        process.current_dir(dir);
    }

    match process.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);
            if let Some(path) = &bundle {
                let _ = evidence::write_command_evidence(
                    path, category, &command, &stdout, &stderr, exit_code,
                );
            }
            CommandResult {
                success: output.status.success(),
                stdout,
                stderr,
                exit_code,
                command,
                evidence_path: bundle.map(|path| display_path(&path)),
            }
        }
        Err(err) => CommandResult {
            success: false,
            stdout: String::new(),
            stderr: format!("failed to execute command: {err}"),
            exit_code: -1,
            command,
            evidence_path: bundle.map(|path| display_path(&path)),
        },
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

fn list(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_rejects_advisory_languages() {
        let result = cli_convert("sample.ts".to_string(), "TypeScript".to_string());
        assert!(!result.success);
        assert!(result.stderr.contains("not available"));
    }

    #[test]
    fn advisory_plan_rejects_active_conversion_languages() {
        let result = advisory_assist_plan("sample.rs".to_string(), "Rust".to_string());
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
}
