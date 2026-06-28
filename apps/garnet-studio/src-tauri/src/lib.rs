pub mod commands;
pub mod evidence;
pub mod paths;
pub mod settings;

use commands::*;
use std::fs;
use std::path::PathBuf;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            cli_health,
            studio_bootstrap_plan,
            studio_bootstrap_write_scripts,
            studio_bootstrap_run_step,
            cli_parse,
            cli_check,
            cli_run,
            studio_diff_caps,
            studio_velocity_check,
            studio_enforcement_legend,
            cli_convert,
            advisory_assist_plan,
            advisory_bundle,
            advisory_review,
            advisory_handoff,
            objective_pulse,
            agentic_dogfood_matrix,
            domain_proof_matrix,
            mac_domain_proofs,
            windows_linux_studio_status,
            converter_status,
            provider_options,
            mit_demo_route,
            mit_deck_outline,
            mit_deck_preview,
            mac_continuation_pulse,
            proof_benchmark_status,
            benchmark_no_run,
            notarization_status,
            windows_vm_installer_status,
            create_evidence_bundle,
            get_evidence_dir,
            get_language_taxonomy,
            get_app_info,
            get_truth_summary,
            studio_get_settings,
            studio_set_settings,
            list_evidence_files,
            read_evidence_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Garnet Studio");
}

pub fn run_smoke() -> Result<String, String> {
    let bundle = evidence::create_named_bundle("smoke")?;
    let bundle_path = PathBuf::from(&bundle.path);
    let payload = serde_json::json!({
        "status": "passed",
        "mode": "studio-smoke",
        "health": commands::cli_health_impl(),
        "taxonomy": commands::get_language_taxonomy(),
        "app_version": env!("CARGO_PKG_VERSION"),
        "source_included": false,
        "provider_api_called": false
    });
    fs::write(
        bundle_path.join("studio-smoke.json"),
        serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("failed to serialize smoke payload: {err}"))?
            + "\n",
    )
    .map_err(|err| format!("failed to write smoke payload: {err}"))?;
    evidence::write_command_evidence(
        &bundle_path,
        "smoke",
        &["garnet-studio".to_string(), "--studio-smoke".to_string()],
        "Garnet Studio smoke passed\n",
        "",
        0,
    )?;
    Ok(bundle.path)
}

pub fn run_domain_proof_smoke() -> Result<String, String> {
    let bundle = evidence::create_named_bundle("domain-proof-shell-smoke")?;
    let bundle_path = PathBuf::from(&bundle.path);
    let result = commands::domain_proof_matrix_impl();
    let stdout_has_matrix = result.stdout.contains("Garnet Studio Domain Proof Matrix");
    let status = if result.success && stdout_has_matrix {
        "passed"
    } else {
        "failed"
    };
    let payload = serde_json::json!({
        "status": status,
        "mode": "studio-domain-proof-smoke",
        "domain_matrix_command_success": result.success,
        "domain_matrix_exit_code": result.exit_code,
        "domain_matrix_command": result.command,
        "domain_matrix_evidence_path": result.evidence_path,
        "stdout_has_domain_matrix": stdout_has_matrix,
        "source_included": false,
        "provider_api_called": false,
        "linux_enforcement_claimed": false,
        "linux_desktop_gui_claimed": false,
        "non_wsl_linux_desktop_claimed": false,
        "signed_msi_claimed": false,
        "winget_claimed": false,
        "windows_arm64_claimed": false,
        "honest_scope": [
            "Studio domain proof smoke exercises the Tauri command wrapper around the repo domain matrix.",
            "WSL rows are execution/portability evidence only, not Linux seccomp or OS-sandbox enforcement.",
            "This is not clean/non-WSL Linux desktop GUI install/launch proof.",
            "No signed MSI, winget, Windows ARM64, production, or v1.0 claim is made."
        ]
    });
    fs::write(
        bundle_path.join("domain-proof-shell-smoke.json"),
        serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("failed to serialize domain proof payload: {err}"))?
            + "\n",
    )
    .map_err(|err| format!("failed to write domain proof payload: {err}"))?;
    evidence::write_command_evidence(
        &bundle_path,
        "domain-proof-shell-smoke",
        &[
            "garnet-studio".to_string(),
            "--studio-domain-proof-smoke".to_string(),
        ],
        &result.stdout,
        &result.stderr,
        result.exit_code,
    )?;

    if status == "passed" {
        Ok(bundle.path)
    } else {
        Err(format!(
            "domain proof matrix command failed or did not emit matrix markdown; evidence={}",
            bundle.path
        ))
    }
}

pub fn run_release_readiness_smoke() -> Result<String, String> {
    let bundle = evidence::create_named_bundle("release-readiness-shell-smoke")?;
    let bundle_path = PathBuf::from(&bundle.path);
    let checks = [
        (
            "windows-linux-studio-status",
            commands::windows_linux_studio_status_impl(),
            "Garnet Windows/Linux Studio Status",
        ),
        (
            "objective-pulse",
            commands::objective_pulse_impl(),
            "Garnet MIT Readiness Objective Status",
        ),
        (
            "converter-status",
            commands::converter_status_impl(),
            "Garnet Converter Adoption Status",
        ),
        (
            "windows-vm-installer-status",
            commands::windows_vm_installer_status_impl(),
            "Garnet Windows Studio Clean-VM Installer Status",
        ),
    ];
    let command_summaries: Vec<_> = checks
        .iter()
        .map(|(id, result, heading)| {
            serde_json::json!({
                "id": id,
                "success": result.success,
                "exit_code": result.exit_code,
                "command": result.command,
                "evidence_path": result.evidence_path,
                "stdout_has_expected_heading": result.stdout.contains(heading),
            })
        })
        .collect();
    let all_passed = checks
        .iter()
        .all(|(_, result, heading)| result.success && result.stdout.contains(heading));
    let status = if all_passed { "passed" } else { "failed" };
    let payload = serde_json::json!({
        "status": status,
        "mode": "studio-release-readiness-smoke",
        "release_readiness_commands": command_summaries,
        "source_included": false,
        "provider_api_called": false,
        "linux_enforcement_claimed": false,
        "linux_desktop_gui_claimed": false,
        "non_wsl_linux_desktop_claimed": false,
        "signed_msi_claimed": false,
        "winget_claimed": false,
        "windows_arm64_claimed": false,
        "honest_scope": [
            "Studio release/readiness smoke exercises the Tauri command wrappers behind the Release / Readiness panel.",
            "WSL rows are execution/portability evidence only, not Linux seccomp or OS-sandbox enforcement.",
            "This is not clean/non-WSL Linux desktop GUI install/launch proof.",
            "No signed MSI, winget, Windows ARM64, production, or v1.0 claim is made."
        ]
    });
    fs::write(
        bundle_path.join("release-readiness-shell-smoke.json"),
        serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("failed to serialize release/readiness payload: {err}"))?
            + "\n",
    )
    .map_err(|err| format!("failed to write release/readiness payload: {err}"))?;
    let stdout = checks
        .iter()
        .map(|(id, result, heading)| {
            format!(
                "{id}: success={} exit_code={} stdout_has_expected_heading={}\n",
                result.success,
                result.exit_code,
                result.stdout.contains(heading)
            )
        })
        .collect::<String>();
    let stderr = checks
        .iter()
        .filter(|(_, result, _)| !result.stderr.trim().is_empty())
        .map(|(id, result, _)| format!("{id}: {}\n", result.stderr.trim()))
        .collect::<String>();
    evidence::write_command_evidence(
        &bundle_path,
        "release-readiness-shell-smoke",
        &[
            "garnet-studio".to_string(),
            "--studio-release-readiness-smoke".to_string(),
        ],
        &stdout,
        &stderr,
        if all_passed { 0 } else { 1 },
    )?;

    if all_passed {
        Ok(bundle.path)
    } else {
        Err(format!(
            "release/readiness reporter command failed or missed an expected heading; evidence={}",
            bundle.path
        ))
    }
}
