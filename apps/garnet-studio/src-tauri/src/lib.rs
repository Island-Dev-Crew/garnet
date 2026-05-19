pub mod commands;
pub mod evidence;
pub mod paths;

use commands::*;
use std::fs;
use std::path::PathBuf;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            cli_health,
            cli_parse,
            cli_check,
            cli_run,
            cli_convert,
            advisory_assist_plan,
            advisory_bundle,
            advisory_review,
            advisory_handoff,
            objective_pulse,
            agentic_dogfood_matrix,
            create_evidence_bundle,
            get_evidence_dir,
            get_language_taxonomy,
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
        "health": commands::cli_health(),
        "taxonomy": commands::get_language_taxonomy(),
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
