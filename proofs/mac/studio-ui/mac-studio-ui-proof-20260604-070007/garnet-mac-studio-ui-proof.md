# Garnet Mac Studio UI Proof

- Status: `passed`
- App bundle: `target/release/bundle/macos/Garnet Studio.app`
- UI path: `Release / Readiness -> Mac Domain Proofs`
- Domains: `6/6`
- Commands recorded: `15`
- Screenshot: `screenshots/mac-domain-proofs-ui-window.png`
- Target evidence: `target-evidence/garnet-mac-domain-proofs-20260604-015936/garnet-mac-domain-proofs.json`

## Domain Verdicts
- `data_pipeline_net_egress`: `passed`, sealed=`false`, artifacts=decision.md, diff_caps.txt
- `supply_chain_proc_escalation`: `passed`, sealed=`false`, artifacts=capability_manifest.json, decision.md, diff_caps.txt
- `config_processor_depth_trap`: `passed`, sealed=`false`, artifacts=decision.md, diff_caps.txt, run_trap.txt
- `accept_provenance_dossier`: `passed`, sealed=`true`, artifacts=capability_manifest.json, decision.md, diff_caps.txt, run_output.txt, seal.json, transparency_log.jsonl
- `pr_review_collapse`: `passed`, sealed=`false`, artifacts=capability_manifest.json, decision.md, diff_caps.txt
- `mcp_tool_authority_creep`: `passed`, sealed=`false`, artifacts=decision.md, mcp_caps.json, mcp_caps.txt

## Honest Scope
- This is a macOS Tauri UI proof: Computer Use clicked the Release / Readiness panel and the Mac Domain Proofs button in the packaged app bundle.
- The button runs the six-domain S105 Mac proof recorder through the Tauri backend; it does not individually open each source file through a native file picker.
- Only accept_provenance_dossier is sealed; refusal/report domains are intentionally unsealed and preserve negative evidence instead of fake seals.
- mcp_tool_authority_creep is a static report surface with enforced=false, not a runtime hard trap.
- No Windows/Linux ownership, OS sandbox enforcement, production, or v1.0 claim is made.
