//! S67 — MCP tool-capability declarations (CI-gated cross-OS proof).
//!
//! `garnet mcp-caps <file.mcpcaps>` reports a tool-set's per-tool + aggregate
//! capability surface and flags high-authority tools. Self-declared, not
//! MCP-host enforced. Runs on every OS in the `cargo test --workspace` matrix.

use std::path::{Path, PathBuf};
use std::process::Command;

fn garnet() -> Command {
    Command::new(env!("CARGO_BIN_EXE_garnet"))
}

fn toolset() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples/mcp/agent_toolset.mcpcaps")
}

#[test]
fn reports_aggregate_authority_and_flags_high_authority() {
    let out = garnet().arg("mcp-caps").arg(toolset()).output().unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "{s}");
    assert!(s.contains("aggregate authority:"), "{s}");
    // shell declares both ffi and proc -> both flagged high-authority.
    assert!(s.contains("high-authority: `shell` declares `ffi`"), "{s}");
    assert!(s.contains("high-authority: `shell` declares `proc`"), "{s}");
    // honest scope surfaced
    assert!(s.contains("NOT MCP-host enforced"), "{s}");
}

#[test]
fn json_lists_aggregate_and_is_unenforced() {
    let out = garnet()
        .args(["mcp-caps", "--format", "json"])
        .arg(toolset())
        .output()
        .unwrap();
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains(r#""schema":"garnet.mcp_caps/v1""#), "{s}");
    assert!(s.contains(r#""enforced":false"#), "{s}");
    // aggregate is the sorted union across tools.
    assert!(
        s.contains(r#""aggregate":["ffi","fs","net","net_internal","proc","time"]"#),
        "{s}"
    );
}
