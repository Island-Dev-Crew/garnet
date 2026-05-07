# Garnet Security Dogfood Rubric

Date: 2026-05-07
Status: active dogfood-readiness scoring input

This rubric makes security a first-class dogfood dimension. A readiness score
is not complete if it only measures parser/runtime progress while leaving trust
boundaries, supply chain, persistence, or release integrity unreviewed.

## Required Security Surfaces

| Surface | Required question | Garnet applicability |
|---|---|---|
| Frontend/XSS | Can untrusted text execute or alter UI state? | not applicable until a web UI ships; docs site remains static artifact review |
| Backend/API | Are inputs authenticated, authorized, validated, and rate-bounded? | not applicable until a server process ships |
| Database | Are queries parameterized and migrations/data paths bounded? | applicable to `garnet-cli/src/knowledge.rs` and `garnet-cli/src/strategies.rs` |
| Secrets | Are keys/tokens absent from source and generated with safe file handling? | applicable to machine keys, signing keys, CI/release scripts |
| Command execution | Can user input reach shell/process execution? | applicable to `xtask` and any future build/compiler subprocess path |
| Filesystem authority | Can source programs read/write/delete arbitrary host paths? | applicable to `garnet-stdlib/src/fs.rs` and CapCaps checks |
| Network authority | Can source programs reach loopback, metadata, private nets, or amplify traffic? | applicable to `garnet-stdlib/src/net.rs` and NetDefaults |
| Sandbox/capabilities | Are converter/runtime outputs quarantined until reviewed? | applicable to `garnet-convert`, `@sandbox`, `@caps`, and safe-mode checks |
| Supply chain | Are dependency CVEs, licenses, duplicate crates, and SBOMs checked? | applicable to CI `cargo audit`, `cargo deny`, CodeQL, CycloneDX |
| Release integrity | Are artifacts signed, checksummed, reproducible, and smoke-installed? | applicable to release jobs, installers, manifests, org release path |
| Privacy/logging | Do caches, logs, and reports avoid leaking secrets or personal data? | applicable to cache episodes, knowledge DB, generated reports |

## Current Evidence Snapshot

Fresh local checks on PR #2 branch `codex/garnet-readiness-remediation`:

| Gate | Evidence | Result |
|---|---|---|
| Dependency CVE scan | `cargo audit` | pass, exit 0 |
| Dependency policy scan | `cargo deny --all-features check` | pass, exit 0; duplicate-crate warnings only |
| Source trust-boundary scan | `rg` over Rust sources for command, db, fs, net, unsafe, eval/exec, and secret patterns | reviewed; no hardcoded secret hit in source scan |
| Database query construction | `rg` for `params!`, `prepare`, `execute`, `query_map`, and SQL-format patterns in `garnet-cli/src` | parameterized query usage observed; no string-built SQL hit in focused scan |
| Network policy | direct inspection of `garnet-stdlib/src/net.rs` | strict default denies loopback/private/link-local/cloud metadata and rechecks peer address |
| Filesystem policy | direct inspection of `garnet-stdlib/src/fs.rs` | functions rely on source-layer CapCaps; needs active bridge/cap tests before production claims |

## Security Findings For Readiness Scoring

| ID | Severity | Domain | Finding | Required before 85+ |
|---|---|---|---|---|
| SEC-001 | medium | filesystem/sandbox | `garnet-stdlib/src/fs.rs` trusts source-layer CapCaps and has no intrinsic path sandbox. That is acceptable only if checker/interpreter bridge tests prove unprivileged source cannot reach these functions. | active negative tests for missing `@caps(fs)` on read/write/remove and a path traversal fixture |
| SEC-002 | medium | network/SSRF | NetDefaults is strong in Rust, but public readiness depends on CLI/interpreter tests proving source-level `net` and `net_internal` authority are enforced consistently. | active loopback/private/metadata-denial tests through Garnet source, not only Rust unit tests |
| SEC-003 | medium | converter/sandbox | Converter frontends quarantine `unsafe`, `eval`, and `exec`, but dogfood scoring needs fixture evidence for every frontend before claiming migration safety. | canonical unsafe/eval/exec converter corpus gate in CI |
| SEC-004 | low | supply-chain | `cargo deny` reports duplicate `cpufeatures` and `unicode-width` lock entries. This is not a vulnerability, but it should remain visible in release readiness. | document duplicate acceptance or reduce duplicates when dependency graph allows |
| SEC-005 | medium | release-integrity | PR context skips release publication and org release still requires browser/desktop authority. | org release publication plus curl installer smoke before public complete-release claim |

## Required Security Gates By Phase

| Phase | Security gate |
|---|---|
| Phase 1 parser parity | no new authority without conformance docs and capability status |
| Phase 2 managed runtime | source-level FS/net/caps negative tests run through `garnet check` and `garnet run` |
| Phase 3 actors/sendable | nonsendable message rejection and bounded mailbox stress gate |
| Phase 4 safe mode | ownership/capability bypass probes become active tests |
| Phase 5 traits/generics | coherence and dynamic dispatch must not bypass capability propagation |
| Phase 6 Memory Core | machine-key isolation, cache tamper detection, privacy/log retention tests |
| Phase 7 release/proof/empirics | `cargo audit`, `cargo deny`, CodeQL, SBOM, signed artifacts, installer smoke |

## Dogfood Skill Scoring Inputs

Security gaps must be counted separately from general findings:

- `security_coverage_gaps`: one per important security surface that was not inventoried or tested.
- `unreviewed_high_risk_trust_boundaries`: one per unreviewed command execution, file/network authority, database, auth/session, sandbox, or release-signing boundary.

These inputs are additive to critical/high/medium/low findings. A target cannot
score 85+ if high-risk trust boundaries are merely assumed safe.
