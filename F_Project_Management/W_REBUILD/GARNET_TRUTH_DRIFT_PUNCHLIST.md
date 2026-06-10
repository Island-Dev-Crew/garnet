# Garnet Truth-Drift Punch-List
**Prepared 2026-06-10 · against `main` @ `5161e64` (post-S130) · full-source audit**

Every item below was verified against the actual files, not page renders. Part A is public-truth
drift (fix in Phase 1 / S-FD1). Part B is the code-health census that motivates the hardening and
rebuild slices. Part C is the permanent guard so this class of bug cannot recur.

---

## Part A — Public-truth drift (exact locations)

### A1. The stdlib primitive count exists at THREE different values
The registry's `build_prims()` contains **80** `p(...)` constructor calls — that is the machine truth.

| Location | Says | Should say |
|---|---|---|
| `README.md:145` (Architecture snapshot table, `garnet-stdlib` row) | "24 registry primitives with capability metadata" | generated count (80 at audit time) |
| `README.md:212` (Verification status) | "✅ 24 stdlib registry primitives bridged through the interpreter" | generated count + honest bridged-vs-registered split if they differ |
| `FAQ.md:57` ("Is Garnet production-ready?" → Ready list) | "the 24 bridged stdlib registry primitives" | generated count |
| `docs/index.html:1452` (By the Numbers) | `<div class="big">24</div>` Stdlib registry primitives | generated count |
| `CURRENT_STATE.md` (S17 row) | "expands to **77 registry primitives** (40 Layer-0, 37 Layer-1)" | reconcile to 80; the S2x slices added `crypto::`/`memory::` rows after S17 |

**Fix:** do not hand-edit a fourth value into history. Implement Part C first, then let the
generator stamp all five locations in one PR.

### A2. Version narrative drift — `main` identifies as two different versions
- `README.md:193` — "Current `main` is post-v0.5.0 source; the latest tagged release is v0.8.1."
  But the workspace `Cargo.toml` (`[workspace.package] version = "0.8.1"`) and the S123 bump mean
  `main` **is** 0.8.1 source. Rewrite: "Current `main` carries v0.8.1; the latest tagged release is v0.8.1."
- `FAQ.md:55` — answer header reads "**v0.5.0 is research-grade and not production-complete.**"
  while `FAQ.md:3` says "latest tag v0.8.1." Update the whole answer body to v0.8.1.
- `FAQ.md:57` — "v0.5.0 Linux/macOS CLI release assets" → v0.8.1 assets (deb/rpm/darwin tarballs, signed).
- `FAQ.md:57` — "S1 LSP source surfaces" → stale; `docs/status.html` records S16 (symbols, rename,
  quick fixes, semantic tokens) as the verified surface.
- `FAQ.md:49` (performance answer) — "v0.5.0 adds the S2 bytecode VM scaffold" → re-anchor to the
  current version story; the claim boundary itself ("production VM performance is not claimed") is
  correct and should be preserved verbatim.

### A3. Stale-versioned release assets on the v0.8.1 Release
The published v0.8.1 Release carries `garnet-0.7.0-lsp-mvp-darwin-arm64.vsix` and
`garnet-0.7.0-lsp-mvp-linux-x64.vsix` alongside the 0.8.1 binaries. `CURRENT_STATE.md:120`
says local packaging "now emits `garnet-0.7.0-lsp-precision.vsix`" — three names, two versions,
one release.

**Fix:** rebuild the VSIX at 0.8.1 from the release toolchain, regenerate `SHA256SUMS(.asc)`,
and replace the assets (asset replacement on a published release is **Jon-gated** — prepare the
artifacts + checksums, escalate the swap). Update the CURRENT_STATE row in the same PR.

### A4. Blog cadence promise already broken
`docs/blog/index.html:65` — "Planned · 2026-06-01 · What @caps is…" is unshipped as of 2026-06-10,
on a page that states "monthly is the floor, not the ceiling." Either ship the post (recommended —
it is the best launch-prep content in the queue) or re-date it honestly. A public promise with a
date is a claim; treat it with the same discipline as a test count.

### A5. Hardcoded stat figures on the site (drift-prone, same class as A1)
`docs/index.html:1450–1467` — `1193` workspace tests, `136` security tests, `0.93×` expressiveness,
`92.3%`, `87/87` are all hand-typed. They are correct *today* and will drift exactly like the
primitive count did. Route them through the Part C generator (a JSON the page reads, or a
build-time stamp).

### A6. README shape (not a line-fix — the S-FD1 rewrite)
`README.md` is 223 lines but renders ~34 KB because lines 90–140 are six consecutive
paragraph-length `python3 scripts/garnet_*` walkthroughs (converter advisory bundle/review/handoff,
promo-video lanes, readiness reporters). These are auditor content on the front door. Relocate to
`docs/internals/` (or `research/`), and purge internal vocabulary from public surfaces:
"Objective Pulse," "Continuation Pulse," "manifested … dogfood bundle," "MIT readiness pulse,"
bare S-numbers. See `README_PROPOSED.md` for the replacement.

---

## Part B — Code-health census (motivates the hardening + rebuild slices)

Counts are from non-test `src/` only, `main` @ `5161e64`. ~53,400 LOC of Rust across 14 crates.

| Crate | `.unwrap()` | `.expect(` | `panic!(` | `.clone()` | Reading |
|---|---|---|---|---|---|
| `garnet-interp-v0.3` | 71 | 20 | 40 | 78 | user-reachable crash surface in the runtime — top hardening target |
| `garnet-cli` | 116 | 8 | 10 | 59 | CLI paths that should be miette diagnostics, not aborts |
| `garnet-stdlib` | 70 | 10 | 21 | 8 | native prims must never panic across the bridge |
| `garnet-check-v0.3` | 1 | 12 | 0 | **201** | CapCaps propagation clones cap-sets along the call graph — bitset rebuild target |
| `garnet-parser-v0.3` | 4 | 91 | 0 | 21 | expects are mostly invariant-style; acceptable, audit the 4 unwraps |
| `garnet-vm` | 0 | 3 | 2 | 19 | clean scaffold |
| `garnet-memory-v0.3` | 0 | 43 | 0 | 10 | expect-style invariants; acceptable |

Structural findings (the rebuild evidence):
1. **Two CSTs coexist.** `garnet-parser-v0.3/src/cst.rs` (510 lines) is self-described as a
   "temporary legacy oracle" pending the S16 rowan migration; `garnet-cst/` (3,369 LOC) is the
   canonical trivia-preserving substrate. Finish the migration, delete the legacy.
2. **The stdlib bridge is a hand-maintained monolith.** `garnet-interp-v0.3/src/stdlib_bridge.rs`
   is 2,511 lines (~166 match/dispatch arms) maintained *in parallel with* `garnet-stdlib/src/registry.rs`
   (80 entries). Two sources of truth for one surface — the A1 drift is the symptom.
3. **String-keyed environment chain.** `garnet-interp-v0.3/src/env.rs` — every variable lookup
   walks `Rc<Env>` parents hashing `String` keys in `RefCell<HashMap<String, Value>>`. Correct,
   slow, and fixable without semantic change (interning + slot resolution).
4. **The VM is a third execution path with per-function interpreter fallback**
   (`garnet-vm/src/bytecode.rs:92 fallback_reason`). Parity cost already proven by the S101
   enforcement-parity campaign; a third path multiplies it.
5. Debt hygiene is otherwise excellent: 2 TODO/FIXME markers in 53K LOC, `deny.toml` present,
   miette wired in parser/interp/cli.

---

## Part C — The permanent guard: `cargo xtask truth`

One slice, ends the drift class forever:

1. Add an `xtask truth` command that emits `docs/truth.json`:
   `{ version, primitive_count, primitives_by_layer, workspace_test_count, security_test_count,
   tracked_slices, readiness_pct, latest_tag }` — each value extracted from its machine source
   (`Cargo.toml`, `all_prims()`, `cargo test -- --list` counts or the CI summary, the ledger reporters).
2. README + FAQ stat lines become stamped between `<!-- truth:primitive_count -->` markers by the
   same xtask; `docs/index.html` "By the Numbers" reads `truth.json` (or is stamped at build).
3. CI gate: `xtask truth --check` fails the build when any public surface disagrees with the
   machine values — the same shape as the S124 release-tag-must-match-Cargo-version guard, extended
   from the version to *every* public number.

Suggested PR order: **C (the guard) → A1+A5 (let it stamp) → A2 (version narrative) → A6 (README
rewrite) → A3 (VSIX re-cut, Jon-gated) → A4 (ship the blog post).**

---

*Prepared from a full-source snapshot; nothing in this list modifies claim boundaries — every fix
narrows public copy toward machine truth, never past it.*
