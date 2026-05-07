# Garnet v4.2 Readiness Remediation Plan

Date: 2026-05-06
Mode: dogfood-readiness Part 2
Branch: codex/garnet-readiness-remediation

## Purpose

This plan turns the full-project dogfood audit findings into a bounded
implementation pass. The goal is not to make broad language-completeness claims
by prose. The goal is to move visible project evidence closer to what Garnet
claims: a credible research-grade language/toolchain with runnable canonical
examples, clear current-vs-historical provenance, and an explicit route toward a
complete language implementation.

## Remediation Order

1. **Executable evidence first.**
   Replace or reduce the 10 canonical MVP examples to supported Garnet syntax
   that must parse, check, and run on current `main`. Preserve the older
   ambition as historical context, but do not let stale drafts masquerade as
   current runtime proof.

2. **CI-enforced dogfood.**
   Add canonical example smokes to the CLI integration tests and CI so future
   changes cannot silently regress the app-level evidence.

3. **Template adoption path.**
   Fix the `agent-orchestrator` template so `garnet new`, `garnet test`, and
   `garnet run` all work for the first-user path.

4. **Current truth surface.**
   Add a root reviewer/current-state guide that separates current proof,
   historical proof, archive material, generated artifacts, and local scratch.
   Update root documentation to point fresh MIT reviewers, contributors, and
   agents to the right starting files.

5. **Historical ledger and IA cleanup.**
   Mark older handoff claims as historical where they conflict with current
   executable evidence. Document the intended `research/` information
   architecture without doing a high-risk bulk move in this pass unless tests
   and links can prove it safely.

6. **Language-completeness pathway.**
   Add a conformance suite skeleton derived from the Mini-Spec matrix. The
   skeleton should make partial/deferred language features visible as testable
   work items instead of prose-only aspirations.

7. **Release path.**
   Verify what can be built and published from this environment. If the active
   GitHub identity cannot publish official `Island-Dev-Crew/garnet` release
   assets, preserve the exact blocker and prepare the release workflow so a
   maintainer with write permission can publish the assets cleanly.

8. **Re-score.**
   Re-run the focused Part 1 probes after remediation and generate an addendum
   plus a refreshed slide deck.

## Non-Goals

- Do not merge PR #1 in this pass.
- Do not claim Garnet is a complete production language merely because examples
  now run.
- Do not do unrelated runtime rewrites when a smaller example/template repair
  proves the user path.
- Do not publish incomplete official release assets under the organization if
  the authenticated account lacks permission.

## Verification Targets

- `cargo test -p garnet-cli --test examples`
- `cargo test -p garnet-cli new_cmd`
- `cargo test -p garnet-interp --test conformance_skeleton`
- `cargo fmt --all -- --check`
- `cargo test --workspace --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`
- focused `garnet parse/check/run` probes across `examples/mvp_*.garnet`
- `garnet new/test/run` probes across all templates

