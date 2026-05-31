//! `garnet caps-log <file.garnet> [--log <path>]` and `garnet caps-log --verify
//! <path>` — a capability **transparency log stub** (S68).
//!
//! Inspired by Certificate Transparency / Sigstore Rekor: an append-only,
//! hash-chained log of capability-manifest entries, so a program's authority
//! history is tamper-evident. Each entry chains to the previous by embedding the
//! BLAKE3 of the prior log line; `--verify` recomputes the chain.
//!
//! This also seeds a **cross-language capability-manifest standard**: the entry
//! schema (`program`, `caps`, `caps_blake3`, `prev_blake3`, `index`) is language-
//! agnostic — any toolchain that emits a capability surface can append to the
//! same log shape.
//!
//! ## Honest scope (do not soften)
//! This is a **local, hash-chained STUB**, not a distributed/witnessed
//! transparency log: there is no public log server, no signed tree head, no
//! gossip/witness, and no inclusion proof against an external root. It gives
//! tamper-evidence for a *local* append-only file; a real transparency log
//! (Rekor-style) is out of scope.

use crate::cap_manifest::CapabilityManifest;
use crate::diagnostics::json_escape;
use crate::{cap_manifest, read_file};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const GENESIS: &str = "genesis";

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn entry_line(
    index: usize,
    program: &str,
    caps: &[String],
    caps_blake3: &str,
    prev: &str,
) -> String {
    let caps_json: Vec<String> = caps
        .iter()
        .map(|c| format!("\"{}\"", json_escape(c)))
        .collect();
    format!(
        "{{\"index\":{index},\"program\":\"{}\",\"caps\":[{}],\"caps_blake3\":\"{}\",\"prev_blake3\":\"{}\"}}",
        json_escape(program),
        caps_json.join(","),
        json_escape(caps_blake3),
        json_escape(prev),
    )
}

pub fn run(args: &[String]) -> ExitCode {
    let mut file: Option<String> = None;
    let mut log: Option<PathBuf> = None;
    let mut verify = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--log" => {
                let Some(v) = args.get(i + 1) else {
                    eprintln!("garnet caps-log: --log requires a <path>");
                    return ExitCode::from(2);
                };
                log = Some(PathBuf::from(v));
                i += 2;
            }
            "--verify" => {
                verify = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other if !other.starts_with("--") => {
                file = Some(args[i].clone());
                i += 1;
            }
            other => {
                eprintln!("garnet caps-log: unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
    }

    if verify {
        let Some(log) = log.or_else(|| file.clone().map(PathBuf::from)) else {
            eprintln!("garnet caps-log: --verify requires a <log path>");
            return ExitCode::from(2);
        };
        return verify_log(&log);
    }

    let Some(file) = file else {
        print_help();
        return ExitCode::from(2);
    };
    append_entry(&file, log.as_deref())
}

fn caps_for(file: &str) -> Result<(String, Vec<String>, String), ExitCode> {
    let surface = match cap_manifest::surface_for_path(&PathBuf::from(file)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("garnet caps-log: {e}");
            return Err(ExitCode::from(1));
        }
    };
    let caps = surface.aggregate.clone();
    let manifest_json = CapabilityManifest::from_surface(surface).to_json();
    let caps_blake3 = blake3_hex(manifest_json.as_bytes());
    let program = Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("program")
        .to_string();
    Ok((program, caps, caps_blake3))
}

fn append_entry(file: &str, log: Option<&Path>) -> ExitCode {
    let (program, caps, caps_blake3) = match caps_for(file) {
        Ok(v) => v,
        Err(code) => return code,
    };

    let existing = log
        .filter(|p| p.is_file())
        .and_then(|p| read_file(p).ok())
        .unwrap_or_default();
    let lines: Vec<&str> = existing.lines().filter(|l| !l.trim().is_empty()).collect();
    let index = lines.len();
    let prev = match lines.last() {
        Some(last) => blake3_hex(last.as_bytes()),
        None => GENESIS.to_string(),
    };
    let entry = entry_line(index, &program, &caps, &caps_blake3, &prev);

    match log {
        Some(path) => {
            let mut content = existing;
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&entry);
            content.push('\n');
            if let Err(e) = std::fs::write(path, content) {
                eprintln!("garnet caps-log: failed to write {}: {e}", path.display());
                return ExitCode::from(1);
            }
            println!(
                "garnet caps-log: appended entry {index} ({program}) -> {}",
                path.display()
            );
        }
        None => println!("{entry}"),
    }
    ExitCode::SUCCESS
}

fn verify_log(path: &Path) -> ExitCode {
    let content = match read_file(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("garnet caps-log: {e}");
            return ExitCode::from(1);
        }
    };
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    let mut prev = GENESIS.to_string();
    for (i, line) in lines.iter().enumerate() {
        let needle = "\"prev_blake3\":\"";
        let Some(start) = line.find(needle).map(|p| p + needle.len()) else {
            eprintln!("garnet caps-log: entry {i} missing prev_blake3");
            return ExitCode::from(1);
        };
        let end = match line[start..].find('"') {
            Some(e) => start + e,
            None => {
                eprintln!("garnet caps-log: entry {i} malformed prev_blake3");
                return ExitCode::from(1);
            }
        };
        let recorded_prev = &line[start..end];
        if recorded_prev != prev {
            eprintln!(
                "garnet caps-log: CHAIN BROKEN at entry {i}: prev_blake3 `{recorded_prev}` != expected `{prev}`"
            );
            return ExitCode::from(1);
        }
        prev = blake3_hex(line.as_bytes());
    }
    println!(
        "garnet caps-log: chain intact — {} entr{} verified (append-only)",
        lines.len(),
        if lines.len() == 1 { "y" } else { "ies" }
    );
    ExitCode::SUCCESS
}

fn print_help() {
    println!("usage:");
    println!("  garnet caps-log <file.garnet> [--log <path>]   append a capability entry");
    println!("  garnet caps-log --verify <log path>            verify the hash chain");
    println!();
    println!("  An append-only, BLAKE3-chained capability transparency log STUB");
    println!("  (local + tamper-evident; not a distributed/witnessed log).");
}
