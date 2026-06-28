use crate::paths;
use chrono::Local;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct EvidenceBundle {
    pub path: String,
    pub timestamp: String,
    pub manifest_path: String,
}

pub fn timestamp() -> String {
    Local::now().format("%Y%m%d-%H%M%S").to_string()
}

pub fn create_bundle() -> Result<EvidenceBundle, String> {
    create_named_bundle("manual")
}

pub fn create_named_bundle(action: &str) -> Result<EvidenceBundle, String> {
    let ts = timestamp();
    let bundle_dir =
        paths::evidence_base_dir().join(format!("garnet-studio-windows-linux-{}-{}", action, ts));

    fs::create_dir_all(&bundle_dir)
        .map_err(|err| format!("failed to create evidence directory: {err}"))?;

    for subdir in [
        "cli-health",
        "parse",
        "check",
        "run",
        "convert",
        "bootstrap",
        "advisory",
        "dogfood",
    ] {
        fs::create_dir_all(bundle_dir.join(subdir))
            .map_err(|err| format!("failed to create evidence subdirectory {subdir}: {err}"))?;
    }

    let contract = serde_json::json!({
        "bundle_version": "1.0.0",
        "created_at": ts,
        "action": action,
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "include_source": false,
        "source_included": false,
        "no_provider_api_calls": true,
        "advisory_output_marked_safe": false
    });
    write_json(bundle_dir.join("evidence-contract.json"), &contract)?;
    let manifest_path = write_manifest(&bundle_dir)?;

    Ok(EvidenceBundle {
        path: bundle_dir.to_string_lossy().to_string(),
        timestamp: ts,
        manifest_path: manifest_path.to_string_lossy().to_string(),
    })
}

pub fn write_command_evidence(
    bundle_dir: &Path,
    category: &str,
    command: &[String],
    stdout: &str,
    stderr: &str,
    exit_code: i32,
) -> Result<(), String> {
    let dir = bundle_dir.join(category);
    fs::create_dir_all(&dir).map_err(|err| format!("failed to create {category}: {err}"))?;
    fs::write(dir.join("stdout.txt"), stdout)
        .map_err(|err| format!("failed to write stdout evidence: {err}"))?;
    fs::write(dir.join("stderr.txt"), stderr)
        .map_err(|err| format!("failed to write stderr evidence: {err}"))?;
    let command_data = serde_json::json!({
        "command": command,
        "exit_code": exit_code,
        "source_included": false,
        "provider_api_called": false
    });
    write_json(dir.join("command.json"), &command_data)?;
    write_manifest(bundle_dir)?;
    Ok(())
}

fn write_json(path: PathBuf, value: &serde_json::Value) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_string_pretty(value)
            .map_err(|err| format!("failed to serialize evidence JSON: {err}"))?
            + "\n",
    )
    .map_err(|err| format!("failed to write evidence JSON: {err}"))
}

fn write_manifest(directory: &Path) -> Result<PathBuf, String> {
    let manifest_path = directory.join("MANIFEST.sha256");
    let mut lines = Vec::new();
    collect_manifest_lines(directory, directory, &mut lines)?;
    lines.sort();
    fs::write(&manifest_path, lines.join("\n") + "\n")
        .map_err(|err| format!("failed to write manifest: {err}"))?;
    Ok(manifest_path)
}

fn collect_manifest_lines(
    root: &Path,
    directory: &Path,
    lines: &mut Vec<String>,
) -> Result<(), String> {
    for entry in
        fs::read_dir(directory).map_err(|err| format!("failed to read directory: {err}"))?
    {
        let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_manifest_lines(root, &path, lines)?;
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("MANIFEST.sha256") {
            continue;
        }
        let bytes =
            fs::read(&path).map_err(|err| format!("failed to read evidence file: {err}"))?;
        let digest = Sha256::digest(bytes);
        let relative = path
            .strip_prefix(root)
            .map_err(|err| format!("failed to relativize evidence path: {err}"))?
            .to_string_lossy()
            .replace('\\', "/");
        lines.push(format!("{digest:x}  {relative}"));
    }
    Ok(())
}
