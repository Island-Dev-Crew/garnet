# Garnet Studio Suite Handoff — macOS parity, agent-legibility, and the Co-typist option (2026-06-12)

**From:** Windows NUC Claude lane (Fable 5), immediately after PR #391 squash-merged
to main (`35a17fa`, "Studio suite UX overhaul"). **To:** (a) the W-REBUILD lead lane /
synthesis session — this file is an **addendum input** alongside the three synthesis
documents, registering two queued workstream proposals; (b) the macOS Studio owner;
(c) optionally Codex (extra-high reasoning) for the Co-typist buildout — §5 is written
to be executed cold.

Everything here is **proposal/design, not claim**: nothing below is built unless its
own PR lands with evidence. Jon-gated boundaries are marked.

---

## 1 · The standard PR #391 set (what "meet and exceed" means)

The Windows/Linux Tauri shell now has, all contract-tested
(`scripts/test_garnet_windows_linux_studio_shell.py`, crate tests, smoke flags):

| # | Standard | Mechanism on Windows/Linux |
|---|---|---|
| 1 | **Version truth** — one stamp, tracking the workspace release, drift-gated | Cargo.toml single stamp; CI gate in shell contract test |
| 2 | **Launch experience** — splash holds during boot with live status; no white flash; hard dismissal ceiling | in-app overlay + `backgroundColor`; 700 ms–25 s bounds |
| 3 | **Simple/Power modes** — clean default surface, full cockpit one toggle away, persisted | CSS-hidden panels (DOM intact for contract copy) + settings file |
| 4 | **Validated settings** — mode/theme/timeouts, clamped backend-side, corrupt file never blocks boot | `settings.rs` normalized() |
| 5 | **Process discipline** — per-category timeout, best-effort process-tree kill, `timed_out`/duration surfaced, payload caps with honest markers, full output in evidence when a bundle exists | `run_process_with_timeout` + `kill_process_tree` |
| 6 | **Truth surface** — live stats from `docs/truth.json` with stamping/measured commits shown; explicit "unavailable" state; zero hand-written numbers | `get_truth_summary` |
| 7 | **Hover help everywhere** — every control explains itself with claim-boundary-honest copy | `data-tip` tooltips (30+) |
| 8 | **Converter UX** — output previewed in-app via read-only, evidence-root-constrained readers | `list_evidence_files`/`read_evidence_text` (canonicalized, symlink-skipping, size-capped) |
| 9 | **Keyboard-first + status bar + themes + a11y** | Ctrl+1…8/Ctrl+Enter; versions/mode/evidence root; dark/light/system; focus rings, reduced motion, tabpanel semantics |
| 10 | **Boundaries intact** — no provider APIs, `core:default` permissions only, no new deps, no claim upgrades | unchanged contracts, extended tests |

**macOS task:** behavioral parity with rows 1–9 in the SwiftUI app
(`apps/garnet-studio-macos`), exceeding where native affordances allow (real
`NSWindow` launch sequencing, system Settings scene, native tooltips/help tags,
`@AppStorage`-backed modes). Do **not** port Tauri code; port the *standard*. Row 1
matters most: the Mac app must take its version from one stamp wired to the workspace
release, gated by a test, before any new DMG evidence is cut. Row 5 applies to every
`Process` the Mac app spawns (same tree-kill + timeout semantics via `SIGKILL` to a
process group). The Windows shell contract test is the template for an equivalent
`test_garnet_macos_studio_shell.py`.

## 2 · Computer-use / browser-use applicability (analysis requested by Jon)

**Embedding browser-use INTO Studio: recommend against, for now.** The shell's
safety contract (no provider APIs, no network handoff, `core:default` only) is the
product's spine; an embedded browsing surface contradicts it for marginal value. The
one legitimate need — viewing HTML evidence (deck previews) — is served by a future
read-only local-file viewer pane behind the same evidence-root constraint as the
converter preview. Queued as a small slice; no general web access ever.

**Computer-use: invert it — make Studio the most *agent-legible* desktop app in its
class.** Agents shouldn't need pixel-driving to operate Garnet Studio; PR #391
already moved this way (stable element ids, roles, deterministic typed
`CommandResult`s, `--studio-smoke` entry points). The serious proposal:

> **Studio MCP server (queued workstream proposal #1):** expose the existing typed
> Tauri commands (health, parse/check/run, convert, reporters, evidence) as an MCP
> tool server — same allowlist, same evidence bundles, same claim boundaries, no new
> authority. An agent then drives Studio through the identical contract a human
> clicks, and every agent action leaves the same manifest-sealed evidence. This
> dovetails with the W-SHIP "Ring Tier 1 + MCP/tool-server" note in the command
> center and is the *Garnet-native* answer to computer-use: agent-first APIs, not
> screen scraping. Sequencing: post-RB-3 (PrimMeta makes capability surfacing
> first-class); CLI/MCP surface, not frozen-crate work.

Pixel-level computer-use remains what it is today: an *evidence tool* (clean-VM
install proofs, screenshot evidence), not a product feature.

## 3 · The Co-typist option (queued workstream proposal #2) — serious design

**What Jon asked:** a Cotypist-style LLM completion element, considered seriously
for Garnet Studio; Windows has no Cotypist equivalent — a buildable, rebrandable
**Garnet suite product** opportunity.

**What already exists (committed truth):** `garnet-suggest-llm` (S19/S69) — a
feature-gated (`llm`) compiler-as-agent advisory tier. Deterministic
`garnet-check` suggest rules run FIRST and are authoritative; LLM output is
additive, labeled `@stability(non-deterministic)`; transports are
request/response-compatible with **Ollama (local)**, Anthropic Messages, and OpenAI
Chat; `LlmClient::complete`/`LlmTransport::send` are documented `@caps(net)`
authority boundaries; reproducibility logs record prompt hashes, model identity,
temperature, and raw responses without credentials. Default builds contact no
provider (`scripts/check_determinism_no_llm.py` gates this). **The hard part — the
honest-AI substrate — is already designed.** Nobody has wired it to an interactive
surface.

**Product shape — "Garnet Co-typist":** deterministic-first, local-first, inline
advisory completion for Garnet code, in three layers:

1. **LSP layer (the real one):** inline completion in `garnet-lsp`
   (`textDocument/inlineCompletion`), composing: deterministic checker
   suggestions (instant, authoritative) → local model via Ollama transport
   (advisory, greyed/labeled) → remote providers ONLY with user-supplied
   credentials through CLI-owned config. Ships in the existing VSIX; this is the
   editor-grade Cotypist and the piece worth branding.
2. **CLI layer:** a `garnet suggest` interactive surface (post-RB-7, riding the
   reedline REPL: completion + `?doc` + `:caps` already planned there — Co-typist
   becomes the REPL's advisory tier, not a competitor to it).
3. **Studio layer (last):** the shell stays a thin wrapper — it invokes the CLI
   like it invokes parse/check/run, and renders labeled suggestions. **The Studio
   shell contract ("no provider API call path… from this shell") is not silently
   weakened:** local-model traffic still crosses a network boundary, so enabling
   the Co-typist panel requires (a) default OFF, (b) an explicit opt-in setting
   with the authority spelled out in the UI (mirroring `@caps` honesty), and
   (c) a Jon-approved amendment to `apps/garnet-studio/src-tauri/AGENTS.md` in the
   same PR — per that file's own rule that permission/contract widenings demand
   contract+tests+security review together.

**Non-negotiables carried from the existing contract:** suggestions are never
auto-applied; every accepted suggestion is evidence-logged (reproducibility log per
crate contract); deterministic tier remains authoritative; no credentials ever
stored or proxied by the Studio shell; no enforcement/production claims ride along.
Branding/licensing ("rebrand and license as a Garnet suite product") is real and
viable — local-first Windows inline completion has no incumbent — and is
**Jon-owned** (naming, license, any marketplace presence).

**Freeze sequencing (why not built today):** the deterministic tier lives in
`garnet-check` (W-REBUILD-frozen, RB-3 dispatch rebuild in flight) and the REPL is
RB-7 (lead lane). What is parallel-safe NOW: `garnet-lsp` + `garnet-suggest-llm` +
`editors/vscode` (none frozen — the win-codex S16 surfaces) using the *existing*
check API read-only. That is the Codex lane below.

## 4 · Sequencing summary

| When | What | Who |
|---|---|---|
| Now (parallel-safe) | Co-typist LSP spike per §5 (reads frozen crates' APIs, edits none) | Codex lane (or any free lane) |
| Now (parallel-safe) | macOS Studio parity per §1 | Mac Studio owner |
| Post-RB-3 | Studio MCP server proposal; PrimMeta-aware prompts for Co-typist | synthesis session assigns |
| Post-RB-7 | Co-typist in the REPL; Studio editor surface | synthesis session assigns |
| Always Jon | Shell-contract amendment for any Studio LLM panel; branding/licensing; release acts | Jon |

## 5 · Codex buildout brief (self-contained — paste to the Codex lane)

> ROLE: Codex on a free lane — Garnet Co-typist LSP spike (S-number from the
> command center runway when scheduled; coordinate via PR, no F_Project_Management
> edits beyond your slice ledger entry).
> SURFACES (writable): `garnet-lsp`, `garnet-suggest-llm`, `editors/vscode`.
> FROZEN (read-only, W-REBUILD): `garnet-check*`, `garnet-interp*`,
> `garnet-stdlib`, `garnet-parser*`, `garnet-cst`. Do not edit them; consume
> `garnet-check`'s existing suggest API as-is.
> SLICES (one PR each):
> 1. `garnet-lsp`: completion provider that surfaces the deterministic suggest
>    rules as inline completions; zero LLM involvement; tests against fixture
>    files; VSIX still packs green.
> 2. `garnet-suggest-llm`: wire the Ollama transport end-to-end behind the `llm`
>    feature with a recorded-fixture test (no live model in CI);
>    `check_determinism_no_llm.py` must stay green on default builds.
> 3. `garnet-lsp` + feature flag: advisory tier behind an explicit LSP
>    initialization option (default off), suggestions labeled
>    `non-deterministic` in the item detail, reproducibility log written per the
>    crate contract. Editor smoke: `code --install-extension` + one recorded
>    inline completion on a `.garnet` fixture.
> HARD STOPS: no frozen-crate edits; no credentials in code, config defaults, or
> logs; no auto-apply; no Studio shell changes (that contract amendment is
> Jon-gated and out of scope); no gate/CI changes; no naming/branding decisions.
> DOGFOOD: the standard ladder + the crate's Required Checks; PR body per
> `check_dogfood_pr_body.py`.

## 6 · Claim boundaries

This document proves nothing and claims nothing about runtime behavior. It records:
the standard PR #391 actually landed (verifiable at `35a17fa`), the committed state
of `garnet-suggest-llm`, and designs whose acceptance criteria live in their own
future PRs. The Co-typist does not exist yet on any platform; the macOS app does
not yet meet rows 1–9; no enforcement, production, or v1.0 claims are made or
implied anywhere above.
