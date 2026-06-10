# Fleet Report — MacBook Air · Claude (Fable 5) · 2026-06-10

**Lane:** Goal Mode 6 — independent read-only audit / public-story lane (S131–S200 runway).
**Machine:** MacBook Air (`idc2.0`), canonical clone `~/clawd/repos/garnet-agent-contracts`.
**Agent/model:** Claude Code, `claude-fable-5[1m]`, ultracode (6 parallel read-only audit lenses + first-party verification).
**Repo truth at audit:** `HEAD = origin/main = 366e69f` (#380 W-REBUILD pack); latest release `v0.8.1` (signed, published 2026-06-07, gate flags all true: `release_ready / binary_strict / sub_gates_pass / cross_os_complete / integrity_ok`); open PRs: none; 1 star / 1 fork.
**Template note:** `FLEET_REPORTS/TEMPLATE.md` did not exist at authoring time (see §4); this report is self-describing and should be reconciled to the template when it lands.
**Authority note:** read-only audit. No tags, no merges, no gate/CI changes, no implementation. Nothing in this report is a release decision.

---

## §1 Machine-local state inventory (input to S131–S134 consolidation)

Verified on this machine, this session:

| Item | State | Consolidation verdict suggested |
|---|---|---|
| Canonical clone `~/clawd/repos/garnet-agent-contracts` | clean tree at `366e69f` | keep (this machine's working clone) |
| Untracked file `F_Project_Management/GARNET_CODEX_TO_CLAUDE_HANDOFF_2026-05-09.md` | untracked since May | commit-as-episodic or archive — decide in consolidation |
| Local tag `v0.4.2` | **drifted**: local `1ccde0d` vs origin `6e945d6` (fetch clobber-rejected) | re-point local tag to origin's |
| 27 local branches | 22 fully merged; 3 squash-merged with `git cherry` = 0 (`codex/phase4bi-…`, `codex/reconcile-…`, `codex/website-refactor`) | safe to delete those 25 |
| `agent-mac-codex/s18-llm-package` (3 commits), `agent-mac-codex/s19-suggest-llm` (4 commits) | `git cherry` finds patch-content **not** on main (commits are "Record S18/S19 … boundary" process records + one substantive each) | **needs human/lead review** — substance may have landed via different commits, but that is unverified; do not delete blind |
| Stashes | none | — |
| `~/Desktop/dogfood/` | 679 sealed evidence bundles | durable evidence root; keep |
| Non-git research bundles (`~/Desktop/GARNET*`, `~/Downloads/GARNET`, `~/Downloads/Garnet_Final`) | no `.git`; historical handoff corpora | archive-or-ignore; not source truth |

---

## §2 The six audit questions

Method: six independent read-only lenses (parallel subagents, each citing file:line), plus first-party spot-verification of load-bearing claims by this lane. Items marked **[1P]** were verified directly by me; lens findings I did not independently re-verify are attributed to their lens.

### Q1 — Can a stranger understand Garnet in 30 seconds?

**README ≈ 5/10, site ≈ 6/10. Both fail the same way: the differentiated story is present but mis-placed.**

- **[HIGH]** The self-declared headline differentiator — `diff-caps`, "what new authority am I granting?" — is **absent from README.md entirely** (zero grep hits), while `docs/index.html:845` itself calls it "the headline … no cross-language equivalent." A stranger leaves believing Garnet is "Ruby-mode plus Rust-mode," which the site's own thesis calls well-precedented. Biggest underclaim on the front door.
- **[HIGH]** The site hero (index.html:777–791) carries no agent/capability positioning — it first appears in the 4th content block, past any 30-second budget. The agent thesis lives only in the invisible `<meta name="description">`.
- **[MEDIUM]** README:39–44 is a jargon storm ("pending-infra," four memory kinds, Ed25519 hot-reload) before WHO/WHY is ever stated; Install opens with re-cut archaeology and S-numbers **[1P — confirmed on direct read]**.
- **[MEDIUM]** The gateway heading `docs/index.html:836` parses as broken English: "Built for code agents write and humans accept" (missing "that"). One-word fix, high credibility leverage.
- **[LOW]** The site hero shows release badges + Get Started with no research-grade marker; the README discloses it only inside an install parenthetical. Mild overclaim-by-omission against the calibrated-honesty brand.

**Missing sentence (both surfaces, first five lines):** *"Garnet is for teams accepting AI-agent-written code: every function declares its authority, and `garnet diff-caps` shows in one screen exactly what new authority a change grants."*

### Q2 — Is the README still an internal ledger?

**Yes — ~40–45% internal process by word count** (1,848 words; the six consecutive `python3 scripts/garnet_*` walkthrough paragraphs at lines ~147–158 are ~599 words alone) **[1P — confirmed; line 150 is a single ~2,400-char paragraph]**. macOS notarization preflight sits inside the public Quickstart (126–134). `README_PROPOSED.md` fixes the shape decisively (876 words, zero S-numbers/gate names/dogfood paths, fixes the 24→80 primitive underclaim) — **but is not landable as-is**:

- **[HIGH]** Its logo path `docs/assets/garnet-logo.png` is a dead link (real logo: `garnet-cli/assets/garnet-logo.png`).
- **[HIGH]** Its flagship "Sixty seconds of Garnet" sample appears not to run: `TOML.parse` is not a registered primitive (stdlib has `json.rs`, no toml) and `FileNotFound` appears nowhere in stdlib/interp source. RB-0b acceptance should add "sample passes `garnet check` / `garnet run`."
- **[MEDIUM]** Truth markers (`<!-- truth:… -->`) have no generator yet — RB-0a must land first (the spec already orders it this way).
- **[LOW]** "Rust 1.75+" MSRV is pinned nowhere in the workspace; "No ambient authority, ever." sits uneasily with the documented managed-mode `*` wildcard; compiler-error text in the sample is a paraphrase of the real `caps coverage:` string.

### Q3 — Which public claims are stale or overcomplicated?

**All six punchlist Part A items are live at `366e69f`; zero are fixed** **[1P — A1 (`README:145` "24 primitives"), A2 (`README:193` "post-v0.5.0"), A4 (blog post 9 days past its 2026-06-01 date) confirmed on direct read; A3 (0.7.0 VSIXs on the v0.8.1 release) confirmed live via `gh release view`]**. New drift the punchlist missed (lens-verified):

- **[HIGH]** `docs/getting-started.html:40` tells first-time users the installer targets **v0.5.0** — the worst class: misleading-current on the first page a new user reads.
- **[MEDIUM]** `FAQ.md:133` — the entire "What's coming after v0.5.0?" roadmap section is ~3 versions / ~40 slices behind.
- **[MEDIUM]** `docs/status.html` — the page that promises "the exact current truth" — never mentions v0.8.x, and it (plus getting-started.html) is **omitted from Part C's stamping surfaces**, so the planned guard would leave both free to drift again.
- **[LOW]** `README:71` calls v0.4.2 "the previous release" (it's two tags back); the v0.4.2 conformance matrix is presented as current implementation-vs-spec status.
- **Punchlist self-errata:** A6's line range is wrong (walkthroughs are at ~147–158, not 90–140), and two of its four "purge" vocabulary terms ("Continuation Pulse," "MIT readiness pulse") don't appear in README.md at all (they're on the site). Execute S-FD1 from a fresh grep, not A6's citations.
- **Exempt-historical (do NOT "fix"):** dated blog summaries (71.3%/55.8%), "Discussions live as of v0.5.0" — scrubbing these would rewrite history; the Part C checker needs an exemption list to avoid false positives.

### Q4 — What needs to be true before launch?

Ordered blocker list (everything else is explicitly *not* a blocker):

1. **Truth-drift cleanup + permanent guard (RB-0a → stamp A1/A5 → A2 → A6/RB-0b).** Three public values for one primitive count and a main that self-identifies as v0.5-era is five-minute HN ammunition against a calibrated-honesty brand.
2. **RB-2 crash-surface sweep before any wave that invites `curl | sh` + `garnet run`.** 71 unwraps / 40 `panic!` in `garnet-interp-v0.3/src` **[lens-verified count matches the spec]**; HN's first move with a new language is feeding it garbage. The spec's "first impression you don't get back" logic applies to the CLI wave, not just the playground.
3. **VSIX 0.8.1 re-pack + Jon-gated asset swap (A3/RB-0d).** Mixed-version assets on a release whose story is "deterministic, signed, verifiable" invites the exact "sloppy" comment the signing work pre-empts.
4. **Ship or honestly re-date the @caps blog post (A4).** A dated public promise is a claim; it is currently broken on the front door.
5. **status.html v0.8.1 anchor + de-jargonization** (it is where the launch story sends skeptics).
6. **The W-REBUILD deploy gate itself** (see §4) — process blocker that sequences all of the above.

**Explicit non-blockers:** the playground (excluded by W-REBUILD), RB-1/RB-3/RB-4/RB-5 internals (invisible to visitors; the truth guard holds the drift class meanwhile), RB-6/native backend, Marketplace publication. Launch copy stays research-grade v0.x — no production/1.0, no "independent" on S114, no "enforced" beyond `@caps`+`@max_depth`.

### Q5 — Which screenshots/diagrams would clarify the presentation?

Repo has **zero SVGs/GIFs/diagrams** beyond the logo; the only architecture image (`garnet_architecture_v2_1.png`) is a stale *aspirational* "Proposed v2.1" drawing that would overclaim if reused. Prioritized, all honesty-guarded:

1. **The diff-caps one-screen capture** — the flagship claim is made in prose 3× and never shown; real 5-line verbatim output already exists in `proofs/` ("`+ caps GAINED: net … AUTHORITY EXPANDED — review required`"). EASY; pure underclaim repair.
2. **Two-mode bridge diagram** (managed⇄safe, asymmetric error transformation, "every crossing logged"). EASY; semantics pinned in Mini-Spec §7.4.
3. **Caps-propagation call-graph diagram** (declared/inherited passes vs trap), reusing the site's exact error string. EASY.
4. **Cross-OS trap-parity matrix** rendered on status.html — *must keep the Honest Scope rows* (WSL exclusion, seccomp Linux-only, byte-equal=false rows); cropping to green rows would silently widen the claim. EASY.
5. **Signed-release verification flow** (sha256 → fingerprint → gpg verify), preserving the "unsigned ≠ tampered" branch. EASY; real terminal capture MEDIUM.
6. **README terminal recording** of the real quickstart (asciinema/VHS from an actual v0.8.1 run — never animate a typed-up fake). MEDIUM.
- **Blocked:** LSP screenshot — `editors/vscode/README.md` still describes the S1 MVP ("rename is deferred"), contradicting root README's S16 feature list; reconcile before captioning any screenshot.
- **Avoid as a class:** screenshots of metric dashboards (1193 tests / 92.3% / 87/87) — they bake drift-prone numbers into PNGs (punchlist A5 class). Date-stamp any number that must appear in an image.

### Q6 — Which W-REBUILD recommendations are high-impact but unsafe to rush?

The spec's factual premises all verified exact at HEAD (201 clones in garnet-check, 71/40 interp crash counts, 80 primitives, 2,511-line bridge, 125-line repl) **[lens-verified]**. The plan is sound; the risk is concentrated:

- **[HIGH] RB-5 (env rebuild) — the most dangerous slice in the plan.** Deep semantic surface (shadowing/closure-capture/recursion via (depth,slot) resolution) yet it is the only deep-semantics slice **without a differential-oracle acceptance criterion** (RB-1 has one; RB-5 says only "tests stay green" — the weakest oracle exactly where bugs are least visible). Its possibly-wrong numbers then feed Jon's RB-6 backend decision. **Recommend amending before execution:** keep the HashMap-chain Env behind a test-only cfg and differential-run both Envs on a shadowing/closure/recursion corpus.
- **[HIGH] RB-1 (caps bitset) — security semantics with a vanishing oracle.** The old set-based impl is deleted "in the same PR once green," and there is no generator-coverage requirement (star-caps, reserve bits, empty sets, deep graphs). Recommend retaining the old impl behind `#[cfg(test)]` through RB-3 and adding explicit generator-coverage criteria.
- **[HIGH] RB-3 (keystone) — a proc-macro bug is systemic across all 80 primitives at once.** The required fixture corpus never demands error-path coverage (arity violations, caps denials, check-ordering). Recommend per-primitive success + arity-violation + caps-denied cases with diagnostic-output comparison before the old bridge path is deleted.
- **[HIGH] RB-0a — self-certifying wrong truth.** The planted-mismatch proof tests guard *sensitivity*, not source-*correctness*; a generator wired to a stale CI summary converts visible manual drift into automated drift with a green checkmark. Recommend one-time cross-derivation of each truth.json field from an independent source in the PR body.
- **[MEDIUM]** RB-4b's "spans/diagnostics preserved or improved" is not mechanically verifiable — require a diagnostics snapshot suite **before** RB-4a deletes the legacy CST oracle. The §2 ladder omits `cargo audit`/`cargo deny` despite RB-5/RB-7 adding new deps (lasso, reedline). The spec's freeze-list uses unsuffixed crate names that don't match the `-v0.3` directories — one-line errata needed.
- **Sequencing is right as written:** RB-0 band ∥ trust band are parallel-safe; RB-1..RB-5 strictly sequential on the frozen lead lane. One worthwhile re-order: the RB-6 wasm32 feasibility *spike* (read-only) could run early to de-risk the playground premise.
- **Calibration note:** the spec itself contains **no overclaim** — every checkable number was exact, and its scoped-claim phrasing (RB-2 "never never-panics," RB-5 "measured Nx on this machine") is the template the rest of the runway should copy.

---

## §3 Blocked gate (restated for the record)

The W-REBUILD §1 deploy gate is **unmet (2 of 3)** as of `366e69f`:

1. ❌ `F_Project_Management/FLEET_REPORTS/` did not exist before this report (and `TEMPLATE.md` still does not).
2. ❌ No S131–S134 consolidation PR on main (`git log` confirms).
3. ✅ This machine's tree is clean on current `origin/main`.

**Additional dead reference:** `GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md` — named as source-of-truth by the spec's P0, the §2 prompt, and the kickoff invocation — is **not tracked anywhere on main** (`git ls-files` + `find` both empty). A cold agent obeying the spec hits a dead reference immediately. Either commit the file or amend the spec to its real location *before* any lane pastes the §2 goal mode. Flagged, not fabricated.

---

## §4 What this lane recommends next (in order)

1. **Land the missing scaffolding (docs-only):** command-center file (or spec errata pointing at its real home) + `FLEET_REPORTS/TEMPLATE.md`. This is the actual blocking item ahead of any RB slice.
2. **Collect the remaining fleet reports** (macbook-pro × claude+codex, windows-nuc × claude+codex) — §1 above is this machine's contribution.
3. **S131–S134 consolidation PR** — include this machine's §1 verdicts (tag re-point, 25 branch deletions, s18/s19 branch review, untracked handoff file disposition).
4. **Then RB-0 band**, with the spec amendments from Q6 (RB-5 differential oracle, RB-1 oracle retention, RB-3 error-path corpus, RB-0a cross-derivation) and the Q2/Q3 additions (sample-must-run acceptance for RB-0b; status.html + getting-started.html added to Part C surfaces; punchlist A6 errata).
5. **Launch sequencing** per Q4 — RB-0 + RB-2 + VSIX swap + blog post before any public wave.

---

## §5 Provenance

- Recon: `git fetch origin main --tags --prune`; `git status`; `gh repo view`; `gh release view v0.8.1`; readiness gate run (all flags true).
- Method: 6 parallel read-only audit subagents (stranger-30s, readme-ledger, stale-claims, launch-readiness, visuals, wrebuild-risk), each required to cite file:line; load-bearing claims spot-verified first-party by this lane (marked [1P]).
- Surfaces read: README.md, CURRENT_STATE.md, docs/index.html, docs/status.html, docs/getting-started.html, FAQ.md (via lenses), W_REBUILD_SPEC.md, README_PROPOSED.md, GARNET_TRUTH_DRIFT_PUNCHLIST.md, post-0.8.1 handoff, coordination ledger, goal ledger, Mini-Spec §7.4 (via lens), proofs/cross-os matrix (via lens).
- Boundaries respected: no tag pushed, no merge, no ECC install, no gate/CI/threshold edits, no production/1.0 claims, S114 stays labeled self-verified.
