# Directives ledger — June 2026 reassessment

**Authority recovered:** the Lane 0 directive calls for “Master Plan v3.2,
chapter 5,” but no tracked artifact by that title exists at base
`231aefa91985e5a0520c493c7f0fc3e54d74efc8`. The recoverable repository
authority is
[`research/2026-06/GARNET_REASSESSMENT_2026-06-11.md`](2026-06/GARNET_REASSESSMENT_2026-06-11.md),
especially §1 and §5. The missing Master Plan v3.2 chapter 5 artifact is recorded
as absent; Lane 0 does not fabricate it or import an external file.

States are limited to `implemented`, `partial`, `planned`, and `research`.
“Implemented” means current code or governance plus a current test/contract
anchor. A design note alone stays partial or planned.

| ID | State | Directive | Current repository evidence and boundary |
|---|---|---|---|
| D1 | partial | Frame caps as types-for-acceptance tooling. | `README.md` and `garnet-check-v0.3/src/capability_surface.rs` expose the capability surface; the comparative TypeScript market framing remains prose, not measured adoption evidence. |
| D2 | implemented | Make `@caps` first-class rather than an external allow-list. | `README.md`, `garnet-check-v0.3/src/caps_graph.rs`, and `garnet-cli/tests/diff_caps.rs` tie declarations, propagation, and review output to executable behavior. |
| D3 | research | Design diff-caps to become an irreversible workflow entitlement. | `garnet-cli/tests/diff_caps.rs` proves the gate mechanics; “teams can never go back” remains an adoption hypothesis without user evidence. |
| D4 | partial | Make an attested MCP/tool-server library Garnet’s category-defining library. | `garnet-cli/src/mcp.rs`, `garnet-cli/src/mcp_schema.rs`, and their protocol tests provide schema/lifecycle foundations; no category-complete library or sealed tool server exists. |
| D5 | partial | Use the senior-multiplier framing and enforce a reusable envelope. | The capability, bounds, seal, and diff gates exist, but no tracked study measures senior-review capacity or junior/agent throughput. |
| D6 | implemented | Adopt “No authority without evidence.” | The exact principle is current at `README.md:10` and in `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md`; truth gates preserve its bounded interpretation. |
| D7 | partial | Carry capabilities in a typed core IR and re-check after every lowering pass. | RB-4b.3 (`012021a`) landed one static AST→bytecode capability-containment re-check plus a deterministic planted authority-laundering trap (`garnet-vm/src/caps_recheck.rs`, `F_Project_Management/W_REBUILD/W_REBUILD_FINAL_REPORT.md`, and the resolved-partial note in `W_REBUILD_SPEC.md`). A typed core representation, re-checking every lowering/optimization pass, and seal-predicate integration remain open; the one-pass mechanism is not the full production claim. |
| D8 | planned | Use the “copilots need pilots / agent is the telephone switch” threat-wall framing. | The source argument remains in the reassessment. No current public-surface acceptance gate requires this copy. |
| D9 | implemented | Use editions for compatible surface evolution. | `garnet-cli/src/edition_manifest.rs` and `garnet-cli/tests/edition_compatibility.rs` prove opt-in editions and capability-manifest invariance. |
| D10 | partial | Compile and trap-test every capability/bounds documentation claim. | Targeted claim fixtures and conformance tests exist, but `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md` still describes the corpus-wide semantic-doctest target; every claim is not yet covered. |
| D11 | implemented | Pre-register RFC governance plus final comment period. | `GOVERNANCE.md` contains the 10-day FCP contract, while `rfcs/README.md`, `rfcs/0000-template.md`, and `scripts/garnet_governance_status.py --gate` keep the scope honest. No duplicate stub is added. |
| D12 | implemented | Put the known-linker-mistakes doctrine in the R5 decision memo. | `F_Project_Management/W_REBUILD/RB6_BACKEND_IR_DECISION_MEMO.md` records the quoted doctrine and the integration lean. |
| D13 | planned | Credit C++ Profiles in the prior-art ledger. | The reassessment names the source, but no current normative prior-art ledger entry was found. The credit remains planned. |
| D14 | partial | Require the Core Ring before public launch. | `F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md` registers the W-SHIP gate; Tier 1 plus the MCP library are not shipped, so no shelf claim is promoted. |
| D15 | implemented | Provide deterministic `diff-caps --machine` output. | `garnet-cli/src/cmd/diff_caps.rs` and `garnet-cli/tests/diff_caps.rs` pin the JSON schema and authority-expansion exit behavior. |
| D16 | research | Preserve “joyful always” as a tool-design constraint. | The phrase survives in `F_Project_Management/GARNET_PROJECT_HANDOFF.md`; aesthetic preference is not a runtime or launch claim and has no acceptance metric yet. |

This ledger supersedes silence, not the source. Changes to a status must name
new code and evidence; historical prose is not enough to promote a directive.
