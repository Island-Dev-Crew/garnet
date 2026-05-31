//! `garnet mcp-caps <file.mcpcaps>` — capability declarations for an MCP/agent
//! tool-set (S67).
//!
//! The documented MCP gap is the **absence of capability attestation**: an
//! agent's tools do not declare what authority they need, so a tool-set's
//! aggregate authority is invisible. This brings Garnet's `@caps` model to MCP
//! tools: a `.mcpcaps` manifest names each tool's required capabilities, and this
//! command reports the per-tool + aggregate capability surface and flags the
//! high-authority tools (the same lens `garnet check`/`diff-caps`/`sandbox` apply
//! to programs).
//!
//! Format (trivial, no serde — the repo's hand-rolled stance): one
//! `tool: cap1, cap2` per line; `#` comments and blank lines ignored.
//!
//! ## Honest scope (do not soften)
//! These are **self-declared** tool capabilities, **not** runtime-enforced —
//! Garnet is not an MCP host and does not intercept tool calls. The value is a
//! reviewable, diffable declaration of a tool-set's authority surface (the
//! `@caps` posture: declared, not inferred); enforcing it at the MCP boundary is
//! out of scope.

use crate::diagnostics::json_escape;
use crate::read_file;
use garnet_parser::ast::Capability;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;

enum Format {
    Human,
    Json,
}

struct Tool {
    name: String,
    caps: Vec<String>,
}

/// Capabilities that warrant explicit review when a tool declares them.
fn is_high_authority(cap: &str) -> bool {
    matches!(cap, "ffi" | "proc" | "*")
}

fn is_known(cap: &str) -> bool {
    !matches!(Capability::from_ident(cap), Capability::Other(_))
}

pub fn run(args: &[String]) -> ExitCode {
    let mut format = Format::Human;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                match args.get(i + 1).map(String::as_str) {
                    Some("human") => format = Format::Human,
                    Some("json") => format = Format::Json,
                    other => {
                        eprintln!("--format expects human|json, got {other:?}");
                        return ExitCode::from(2);
                    }
                }
                i += 2;
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
                eprintln!("unknown mcp-caps flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(file) = file else {
        print_help();
        return ExitCode::from(2);
    };
    let src = match read_file(&PathBuf::from(&file)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("garnet mcp-caps: {e}");
            return ExitCode::from(1);
        }
    };

    let mut tools: Vec<Tool> = Vec::new();
    for (lineno, raw) in src.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, caps_str)) = line.split_once(':') else {
            eprintln!(
                "garnet mcp-caps: line {}: expected `tool: caps`",
                lineno + 1
            );
            return ExitCode::from(1);
        };
        let mut caps: Vec<String> = caps_str
            .split(',')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();
        caps.sort();
        caps.dedup();
        tools.push(Tool {
            name: name.trim().to_string(),
            caps,
        });
    }
    tools.sort_by(|a, b| a.name.cmp(&b.name));

    let aggregate: Vec<String> = tools
        .iter()
        .flat_map(|t| t.caps.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let high: Vec<(&str, &str)> = tools
        .iter()
        .flat_map(|t| {
            t.caps
                .iter()
                .filter(|c| is_high_authority(c))
                .map(move |c| (t.name.as_str(), c.as_str()))
        })
        .collect();
    let unknown: Vec<(&str, &str)> = tools
        .iter()
        .flat_map(|t| {
            t.caps
                .iter()
                .filter(|c| !is_known(c))
                .map(move |c| (t.name.as_str(), c.as_str()))
        })
        .collect();

    match format {
        Format::Human => print_human(&tools, &aggregate, &high, &unknown),
        Format::Json => print_json(&tools, &aggregate, &high, &unknown),
    }
    ExitCode::SUCCESS
}

fn print_human(
    tools: &[Tool],
    aggregate: &[String],
    high: &[(&str, &str)],
    unknown: &[(&str, &str)],
) {
    println!("MCP tool-capability surface ({} tools):", tools.len());
    for t in tools {
        let caps = if t.caps.is_empty() {
            "(none)".to_string()
        } else {
            t.caps.join(", ")
        };
        println!("  {} -> {caps}", t.name);
    }
    println!(
        "aggregate authority: {}",
        if aggregate.is_empty() {
            "(none)".into()
        } else {
            aggregate.join(", ")
        }
    );
    for (tool, cap) in high {
        println!("  ! high-authority: `{tool}` declares `{cap}` — review");
    }
    for (tool, cap) in unknown {
        println!("  ? unknown capability: `{tool}` declares `{cap}` (not a known @caps name)");
    }
    println!("note: self-declared tool capabilities — reviewable/diffable, NOT MCP-host enforced.");
}

fn print_json(
    tools: &[Tool],
    aggregate: &[String],
    high: &[(&str, &str)],
    unknown: &[(&str, &str)],
) {
    let tools_json: Vec<String> = tools
        .iter()
        .map(|t| {
            format!(
                "{{\"name\":\"{}\",\"caps\":[{}]}}",
                json_escape(&t.name),
                t.caps
                    .iter()
                    .map(|c| format!("\"{}\"", json_escape(c)))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect();
    let agg: Vec<String> = aggregate
        .iter()
        .map(|c| format!("\"{}\"", json_escape(c)))
        .collect();
    let high_json: Vec<String> = high
        .iter()
        .map(|(t, c)| {
            format!(
                "{{\"tool\":\"{}\",\"cap\":\"{}\"}}",
                json_escape(t),
                json_escape(c)
            )
        })
        .collect();
    let unknown_json: Vec<String> = unknown
        .iter()
        .map(|(t, c)| {
            format!(
                "{{\"tool\":\"{}\",\"cap\":\"{}\"}}",
                json_escape(t),
                json_escape(c)
            )
        })
        .collect();
    println!(
        "{{\"schema\":\"garnet.mcp_caps/v1\",\"tools\":[{}],\"aggregate\":[{}],\"high_authority\":[{}],\"unknown\":[{}],\"enforced\":false}}",
        tools_json.join(","),
        agg.join(","),
        high_json.join(","),
        unknown_json.join(",")
    );
}

fn print_help() {
    println!("usage: garnet mcp-caps [--format human|json] <file.mcpcaps>");
    println!();
    println!("  Report the capability surface of an MCP/agent tool-set declared as");
    println!("  `tool: cap1, cap2` lines. Self-declared (not MCP-host enforced):");
    println!("  reviewable + diffable, bringing @caps to agent tools.");
}
