# S114 Independent Re-Verification Package

Date: 2026-06-14
Prepared by: Codex independent trust planner
Status: report-only runbook; not an independent re-verification result
Repository source of truth: `github.com/Island-Dev-Crew/garnet`, `origin/main`

## Non-Claim

This package prepares an independent S114 adversarial re-verification. It does
not perform it, grade it, close it, or claim independence. The current in-repo
truth remains: S114 found and fixed one HIGH hole, records two LOW follow-ups,
and is not independently re-verified.

An independent S114 result exists only after a reviewer who did not author or
merge the S114 fix runs the package, records raw evidence, classifies outcomes,
and signs or otherwise identifies the review record.

## Recon Snapshot

Commands run before preparing this package:

```sh
git fetch origin main --tags --prune
git status --short --branch
git rev-parse HEAD && git rev-parse origin/main
gh pr list --repo Island-Dev-Crew/garnet --state all --limit 12
gh pr list --repo Island-Dev-Crew/garnet --state open --limit 20
```

Observed on this machine:

- The shared checkout was dirty on `codex/rb4b1-substrate-fidelity`, with
  frozen W-REBUILD crates modified. This report was therefore prepared in a
  separate clean worktree at `/private/tmp/garnet-s114-reverify`.
- Report branch: `codex/s114-reverification-package`.
- Base: `origin/main` at `bd3b1c736ac7269f2c0888e56ad3b65584f31fd1`
  (`#398` merged).
- Open PRs observed: none.
- Last 12 PRs observed by `gh pr list --state all --limit 12`: `#398` through
  `#387`, all merged.

## Source-Of-Truth Anchors

- `CURRENT_STATE.md:10-19` says the `v0.8.1` milestone includes enforced
  `@caps` + `@max_depth`, cross-OS trap parity, Linux-only seccomp application,
  and "a self-found and self-fixed HIGH red-team finding -- not independently
  verified."
- `F_Project_Management/GARNET_S131_S134_SOURCE_TRUTH_CONSOLIDATION.md:72-76`
  keeps the enforcement boundary narrow: only `@caps` + `@max_depth` are
  deterministic traps on both backends; `@bounded`, memory, time, `@mailbox`,
  and macOS/Windows OS-sandbox remain declared-not-enforced; S114 stays
  self-verified pending independent re-verification.
- `F_Project_Management/GARNET_S131_S134_SOURCE_TRUTH_CONSOLIDATION.md:231-239`
  says the consolidation does not prove independent S114 re-verification, and
  that same-machine/fleet lanes are not independent reviewers in the S114 sense.
- `F_Project_Management/GARNET_S129_S200_ECC_DOGFOOD_COMMAND_CENTER.md:204-214`
  assigns S141-S150 to independent trust and reviewer-proof security, with the
  independent reviewer as acceptance authority.
- `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md:20-28` says the trust band,
  including independent S114 re-verification, runs in parallel on other lanes
  and is explicitly not claimed by W-REBUILD.
- `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md:121-128` keeps S114 labeled
  self-verified and forbids weakening calibrated-honesty boundaries.
- `F_Project_Management/W_REBUILD/W_REBUILD_SPEC.md:327-337` names W-TRUST as
  the independent lane and says it is never self-graded.
- `C_Language_Specification/GARNET_RED_TEAM.md:9-16` records the original S114
  method: six attackers plus skeptical referee, classifying outcomes as HELD,
  HOLE, or DECLARED-NOT-ENFORCED.
- `C_Language_Specification/GARNET_RED_TEAM.md:18-41` records the HIGH hole and
  fix: `capability_surface()` skipped impl methods; after the fix, impl-method
  `@caps(fs)` appears in the surface, `diff-caps` rejects, and `agent-loop`
  rejects before sealing.
- `C_Language_Specification/GARNET_RED_TEAM.md:43-59` records two LOW follow-ups:
  caps-log tail forgery and capability-blind seal `subject.digest`.
- `C_Language_Specification/GARNET_RED_TEAM.md:61-75` records held attacks:
  proc double-gating, `@max_depth`, top-level diff-caps widening, and signed
  manifest reattach failure.
- `C_Language_Specification/GARNET_RED_TEAM.md:76-89` records referee-corrected
  non-holes / declared-not-enforced cases.
- `C_Language_Specification/GARNET_RED_TEAM.md:90-98` defines honest S114 scope.
- `scripts/garnet_red_team_status.py:45-88` is a static gate over the presence
  of the report, HIGH fix, regression tests, LOW-hole record, held/deferred
  record, and honesty anchor. It does not assert independent re-verification.

## Independence Criteria

The reviewer must be independent of the S114 authoring/merge lane in the
practical sense:

- They did not author PR `#365`, review-fix it, merge it, or write this package.
- They run from a fresh clone or clean worktree, not the dirty RB-4b lead
  checkout.
- They record raw commands, outputs, exit codes, platform, `git` SHAs, and
  reviewer identity/provenance before giving a verdict.
- They may use this package as a target list, but they decide whether the target
  list is sufficient and may add attacks. Extra attacks are required if this
  package looks too tailored to pass.
- They do not accept screenshots, summaries, or prior dogfood bundles as a
  substitute for rerunning commands.

Disqualifiers:

- Reusing the S114 authoring agent as the reviewer.
- Editing `garnet-check`, `garnet-interp`, `garnet-stdlib`, `garnet-parser`, or
  `garnet-cst` during the review.
- Changing CI, dogfood thresholds, diff-caps thresholds, capability standards, or
  gates to make a result pass.
- Calling a declared-not-enforced ceiling a hole unless a current repo contract
  says it is enforced.

## What Garnet Lanes May Prepare

Garnet lanes may prepare:

- A fresh runbook like this one.
- A script or fixture pack that only creates inputs, runs commands, and stores
  raw outputs.
- A directory template for evidence.
- A current source-of-truth snapshot: `git` SHAs, PR state, release truth, and
  readiness status.
- A list of expected outcomes for known scenarios.
- Follow-up issues or PRs after an independent reviewer files findings.

Garnet lanes must not:

- Mark S114 independently verified.
- Classify their own S114 run as independent.
- Close the two LOW findings without a separate implementation slice and proof.
- Widen the enforced-scope language beyond deterministic evidence.
- Claim macOS/Windows OS-sandbox enforcement or Wasmtime fuel enforcement.
- Claim production or 1.0 readiness.

## What Only The Independent Attacker Can Validate

Only the independent attacker/reviewer can produce the S114 re-verification
verdict:

- Whether the fixed HIGH hole stays fixed in their fresh environment.
- Whether the regression tests are meaningful or merely tailored.
- Whether alternate impl-method, nested-module, method-dispatch, wildcard,
  casing, or seal/log attacks bypass the gate.
- Whether each result is HELD, HOLE, or DECLARED-NOT-ENFORCED under the current
  repo contracts.
- Whether a newly found issue is HIGH, MEDIUM, LOW, or out of scope.
- Whether the evidence is sufficient for reviewer-proof security claims.

## Evidence Directory Template

Recommended output path for the future independent run:

```text
proofs/independent/s114/<reviewer-id>-<yyyymmdd>-s114-reverify/
  README.md
  environment.json
  commands.jsonl
  findings.md
  classification.md
  reviewer_attestation.md
  raw/
    000-recon.stdout
    000-recon.stderr
    010-build.stdout
    010-build.stderr
    ...
  fixtures/
    baseline.garnet
    proposal-impl-fs.garnet
    proposal-nested-module-fs.garnet
    proposal-overdepth.garnet
    proposal-proc-launder.garnet
  artifacts/
    diff_caps_impl_method.json
    agent_loop_impl_interp.stdout
    agent_loop_impl_vm.stdout
    seal_*.json
    caps.log
  MANIFEST.sha256
```

Minimum `environment.json` fields:

```json
{
  "schema": "garnet.s114.independent_reverify.environment/v1",
  "reviewer_id": "",
  "reviewer_independence_statement": "",
  "host_os": "",
  "host_arch": "",
  "git_head": "",
  "origin_main": "",
  "branch": "",
  "rustc": "",
  "cargo": "",
  "garnet_binary": "",
  "started_at_utc": "",
  "completed_at_utc": ""
}
```

Minimum `commands.jsonl` row shape:

```json
{"id":"010-build","cmd":"cargo build -p garnet-cli --release","cwd":"...","exit_code":0,"stdout":"raw/010-build.stdout","stderr":"raw/010-build.stderr","verdict":"recorded"}
```

Minimum `classification.md` table:

| Probe | Expected current outcome | Actual outcome | Classification | Notes |
|---|---|---|---|---|
| HIGH regression: impl-method `@caps(fs)` surface | HELD: visible in caps surface, diff-caps rejects, agent-loop rejects, no seal | | | |
| Nested module capability surface | HELD: visible in caps surface | | | |
| Top-level widening | HELD: diff-caps rejects | | | |
| Over-depth proposal | HELD: run stage rejects, no seal | | | |
| proc laundering | HELD: entry `@caps(proc)` gate traps both backends | | | |
| caps-log tail forgery | Expected LOW still open unless separately fixed | | | |
| seal subject digest capability-blindness | Expected LOW/mitigated still open unless separately fixed | | | |
| `@bounded`/memory/time/`@mailbox` runtime enforcement | DECLARED-NOT-ENFORCED unless current docs changed | | | |

## Setup Commands

The independent reviewer should run these in a fresh clone:

```sh
set -eu

REPO_URL="${REPO_URL:-https://github.com/Island-Dev-Crew/garnet.git}"
WORK="${WORK:-$PWD/garnet-s114-independent}"
EVIDENCE="${EVIDENCE:-$PWD/s114-evidence}"

git clone "$REPO_URL" "$WORK"
cd "$WORK"
git fetch origin main --tags --prune
git checkout main
git pull --ff-only origin main

mkdir -p "$EVIDENCE/raw" "$EVIDENCE/fixtures" "$EVIDENCE/artifacts"

{
  date -u
  git status --short --branch
  git rev-parse HEAD
  git rev-parse origin/main
  gh pr list --repo Island-Dev-Crew/garnet --state all --limit 12 || true
  rustc --version
  cargo --version
} > "$EVIDENCE/raw/000-recon.stdout" 2> "$EVIDENCE/raw/000-recon.stderr"

cargo build -p garnet-cli --release \
  > "$EVIDENCE/raw/010-build.stdout" \
  2> "$EVIDENCE/raw/010-build.stderr"

GARNET="$WORK/target/release/garnet"
"$GARNET" --help \
  > "$EVIDENCE/raw/011-garnet-help.stdout" \
  2> "$EVIDENCE/raw/011-garnet-help.stderr"
```

Expected setup outcome:

- `git status --short --branch` shows a clean `main` at current `origin/main`.
- `cargo build -p garnet-cli --release` exits 0.
- `target/release/garnet --help` lists `caps`, `diff-caps`, `seal`,
  `caps-log`, and `agent-loop`.

If setup fails, record the failure and stop; do not repair the repo before
classification.

## Baseline And Fixture Commands

Create baseline and proposal fixtures:

```sh
cat > "$EVIDENCE/fixtures/baseline.garnet" <<'EOF'
@caps()
def main() -> int { 0 }
EOF

cat > "$EVIDENCE/fixtures/proposal-impl-fs.garnet" <<'EOF'
struct Reader {}
impl Reader {
  @caps(fs)
  def read(self) -> int { 0 }
}
@caps()
def main() -> int { 0 }
EOF

cat > "$EVIDENCE/fixtures/proposal-nested-module-fs.garnet" <<'EOF'
module hidden {
  @caps(fs)
  def read() -> int { 0 }
}
@caps()
def main() -> int { 0 }
EOF

cat > "$EVIDENCE/fixtures/proposal-top-level-net.garnet" <<'EOF'
@caps(net)
def helper() -> int { 0 }
@caps()
def main() -> int { 0 }
EOF

cat > "$EVIDENCE/fixtures/depth-baseline.garnet" <<'EOF'
@caps()
@max_depth(8)
def deep(n) { if n <= 0 { 0 } else { 1 + deep(n - 1) } }
@caps()
def main() { deep(3) }
EOF

cat > "$EVIDENCE/fixtures/depth-over.garnet" <<'EOF'
@caps()
@max_depth(4)
def deep(n) { if n <= 0 { 0 } else { 1 + deep(n - 1) } }
@caps()
def main() { deep(20) }
EOF

cat > "$EVIDENCE/fixtures/proc-launder.garnet" <<'EOF'
@caps(proc)
def helper() {
  std::process::spawn("echo")
}
@caps()
def main() {
  helper()
}
EOF
```

Expected fixture outcome:

- These fixtures parse under current syntax. If any fixture does not parse, the
  reviewer records the parse error and may replace it with an equivalent probe,
  but must keep the old failed fixture as raw evidence.

## Probe 1: HIGH Regression Re-Test

Purpose: independently test whether impl-method capability-surface blindness is
still fixed.

```sh
"$GARNET" caps "$EVIDENCE/fixtures/proposal-impl-fs.garnet" \
  > "$EVIDENCE/raw/100-impl-caps.stdout" \
  2> "$EVIDENCE/raw/100-impl-caps.stderr"

"$GARNET" diff-caps --machine \
  "$EVIDENCE/fixtures/baseline.garnet" \
  "$EVIDENCE/fixtures/proposal-impl-fs.garnet" \
  > "$EVIDENCE/artifacts/diff_caps_impl_method.json" \
  2> "$EVIDENCE/raw/101-impl-diff.stderr" || true

"$GARNET" agent-loop \
  --baseline "$EVIDENCE/fixtures/baseline.garnet" \
  --proposal "$EVIDENCE/fixtures/proposal-impl-fs.garnet" \
  --backend interp \
  --seal-out "$EVIDENCE/artifacts/impl-interp.seal.json" \
  --attest reviewer="$REVIEWER_ID" \
  --gate-version s114-independent-reverify \
  > "$EVIDENCE/raw/102-impl-agent-interp.stdout" \
  2> "$EVIDENCE/raw/102-impl-agent-interp.stderr" || true

"$GARNET" agent-loop \
  --baseline "$EVIDENCE/fixtures/baseline.garnet" \
  --proposal "$EVIDENCE/fixtures/proposal-impl-fs.garnet" \
  --backend vm \
  --seal-out "$EVIDENCE/artifacts/impl-vm.seal.json" \
  --attest reviewer="$REVIEWER_ID" \
  --gate-version s114-independent-reverify \
  > "$EVIDENCE/raw/103-impl-agent-vm.stdout" \
  2> "$EVIDENCE/raw/103-impl-agent-vm.stderr" || true
```

Expected current outcome:

- `caps` output includes aggregate `fs` and/or per-function `Reader::read`.
- `diff-caps --machine` exits non-zero and emits
  `"verdict":"authority-expanded"`.
- Both `agent-loop` runs reject at `diff-caps`.
- No `impl-interp.seal.json` or `impl-vm.seal.json` is written.

Classification rule:

- If the impl method is invisible or either agent-loop accepts and seals, classify
  as HOLE and stop for maintainer escalation.

## Probe 2: Adjacent Surface Evasion

Purpose: test whether the fix generalized beyond the exact known impl-method
shape.

```sh
"$GARNET" caps "$EVIDENCE/fixtures/proposal-nested-module-fs.garnet" \
  > "$EVIDENCE/raw/110-nested-caps.stdout" \
  2> "$EVIDENCE/raw/110-nested-caps.stderr"

"$GARNET" diff-caps --machine \
  "$EVIDENCE/fixtures/baseline.garnet" \
  "$EVIDENCE/fixtures/proposal-nested-module-fs.garnet" \
  > "$EVIDENCE/artifacts/diff_caps_nested_module.json" \
  2> "$EVIDENCE/raw/111-nested-diff.stderr" || true

"$GARNET" diff-caps --machine \
  "$EVIDENCE/fixtures/baseline.garnet" \
  "$EVIDENCE/fixtures/proposal-top-level-net.garnet" \
  > "$EVIDENCE/artifacts/diff_caps_top_level_net.json" \
  2> "$EVIDENCE/raw/112-top-level-diff.stderr" || true
```

Expected current outcome:

- Nested-module `@caps(fs)` is visible and rejected as authority expansion.
- Top-level `@caps(net)` is rejected as authority expansion.

Reviewer extension:

- Add at least two more surface-evasion attempts not listed here. Suggested
  classes: wildcard `@caps(*)`, mis-cased/unknown caps, nested `impl` inside
  modules, and method-like call sites. Preserve failing syntax attempts.

## Probe 3: Agent-Loop Accept/Reject Integrity

Purpose: verify that positive and negative paths behave differently and that
rejections never seal.

```sh
"$GARNET" agent-loop \
  --baseline "$EVIDENCE/fixtures/depth-baseline.garnet" \
  --proposal "$EVIDENCE/fixtures/depth-over.garnet" \
  --backend interp \
  --seal-out "$EVIDENCE/artifacts/depth-interp.seal.json" \
  --attest reviewer="$REVIEWER_ID" \
  --gate-version s114-independent-reverify \
  > "$EVIDENCE/raw/120-depth-agent-interp.stdout" \
  2> "$EVIDENCE/raw/120-depth-agent-interp.stderr" || true

"$GARNET" agent-loop \
  --baseline "$EVIDENCE/fixtures/depth-baseline.garnet" \
  --proposal "$EVIDENCE/fixtures/depth-over.garnet" \
  --backend vm \
  --seal-out "$EVIDENCE/artifacts/depth-vm.seal.json" \
  --attest reviewer="$REVIEWER_ID" \
  --gate-version s114-independent-reverify \
  > "$EVIDENCE/raw/121-depth-agent-vm.stdout" \
  2> "$EVIDENCE/raw/121-depth-agent-vm.stderr" || true
```

Expected current outcome:

- `diff-caps` stage passes because authority did not widen.
- run stage rejects because `@max_depth(4)` is exceeded.
- No depth seal file is written.

Classification rule:

- If an over-depth proposal seals, classify as HOLE.
- If it traps under one backend but not the other, classify as a backend parity
  finding and determine severity from whether acceptance can seal.

## Probe 4: Runtime `@caps` Trap Parity

Purpose: verify that runtime host authority still traps on both backends.

```sh
"$GARNET" run --interp "$EVIDENCE/fixtures/proc-launder.garnet" \
  > "$EVIDENCE/raw/130-proc-interp.stdout" \
  2> "$EVIDENCE/raw/130-proc-interp.stderr" || true

"$GARNET" run --vm "$EVIDENCE/fixtures/proc-launder.garnet" \
  > "$EVIDENCE/raw/131-proc-vm.stdout" \
  2> "$EVIDENCE/raw/131-proc-vm.stderr" || true
```

Expected current outcome:

- Both commands fail before spawning a subprocess.
- Output contains a program-entry `@caps(proc)` requirement or equivalent
  runtime capability trap.

Classification rule:

- If either backend runs the subprocess with entry `@caps()` unchanged, classify
  as HOLE.

## Probe 5: Known LOW Follow-Ups

Purpose: record whether the two LOW findings remain open, are fixed, or changed
severity.

The reviewer should independently construct:

1. A caps-log with at least two entries, then mutate only the tail entry's caps
   and `caps_blake3`.
2. Two sources that differ only by `@caps` surface, then compare `garnet seal`
   subject digests and predicate capability manifests.

Suggested starting commands:

```sh
LOG="$EVIDENCE/artifacts/caps.log"
"$GARNET" caps-log "$EVIDENCE/fixtures/baseline.garnet" --log "$LOG" \
  > "$EVIDENCE/raw/140-caps-log-append-a.stdout" \
  2> "$EVIDENCE/raw/140-caps-log-append-a.stderr"
"$GARNET" caps-log "$EVIDENCE/fixtures/proposal-top-level-net.garnet" --log "$LOG" \
  > "$EVIDENCE/raw/141-caps-log-append-b.stdout" \
  2> "$EVIDENCE/raw/141-caps-log-append-b.stderr"
"$GARNET" caps-log --verify "$LOG" \
  > "$EVIDENCE/raw/142-caps-log-verify.stdout" \
  2> "$EVIDENCE/raw/142-caps-log-verify.stderr"

"$GARNET" seal "$EVIDENCE/fixtures/baseline.garnet" \
  --out "$EVIDENCE/artifacts/baseline.seal.json" \
  > "$EVIDENCE/raw/150-seal-baseline.stdout" \
  2> "$EVIDENCE/raw/150-seal-baseline.stderr"
"$GARNET" seal "$EVIDENCE/fixtures/proposal-top-level-net.garnet" \
  --out "$EVIDENCE/artifacts/top-level-net.seal.json" \
  > "$EVIDENCE/raw/151-seal-net.stdout" \
  2> "$EVIDENCE/raw/151-seal-net.stderr"
```

Expected current outcome:

- Non-tail caps-log tampering is expected to fail verification.
- Tail forgery is expected to remain a LOW open issue unless a later PR fixed it.
- Seal predicate capability manifests should differ when caps differ.
- Seal subject digest may remain capability-blind unless a later PR fixed it.

Classification rule:

- Do not call expected LOW-open behavior a new HIGH unless it defeats an
  currently enforced gate or enables acceptance/sealing contrary to the current
  contracts.

## Probe 6: Declared-Not-Enforced Boundaries

Purpose: prevent overclaim drift.

The reviewer should inspect current docs and, if desired, run exploratory probes
against `@bounded`, memory/time, `@mailbox`, and macOS/Windows OS-sandbox claims.

Expected current outcome:

- Missing runtime enforcement for those boundaries is DECLARED-NOT-ENFORCED
  unless the current source-of-truth docs changed.
- Linux seccomp evidence, if tested, is Linux-only and cannot be generalized to
  macOS or Windows.

## Focused Repo Gates To Run

These are not independent attack results, but they establish the reviewed repo
state:

```sh
python3 scripts/garnet_red_team_status.py --gate --format json
python3 scripts/test_garnet_red_team_status.py
cargo test -p garnet-check capability_surface -- --nocapture
cargo test -p garnet-cli agent_loop -- --nocapture
cargo test -p garnet-cli diff_caps -- --nocapture
cargo test -p garnet-cli caps_log -- --nocapture
cargo test -p garnet-cli seal_attestation -- --nocapture
python3 scripts/garnet_mit_readiness_status.py --format json
```

Expected current outcome:

- `garnet_red_team_status.py --gate` exits 0.
- The focused test suites pass.
- MIT/productization status remains active-partial, currently observed at 92.8%
  on this worktree, not full production readiness.

## Full Evidence Closure

After all probes:

```sh
find "$EVIDENCE" -type f -print0 | sort -z | xargs -0 shasum -a 256 \
  > "$EVIDENCE/MANIFEST.sha256"

cat > "$EVIDENCE/reviewer_attestation.md" <<EOF
# S114 Independent Reviewer Attestation

Reviewer:
Date:
Repo:
Head:
Base:

I did not author PR #365, this runbook, or the S114 fix. I ran the recorded
commands in the attached evidence bundle and classified the results according
to current repository contracts.

Overall S114 re-verification verdict:

- [ ] HELD as currently claimed
- [ ] HOLE found
- [ ] DECLARED-NOT-ENFORCED / overclaim only
- [ ] INCONCLUSIVE

Notes:
EOF
```

The reviewer then files either:

- a report-only PR adding the evidence bundle and classification, or
- a private security report if a live exploit/credential/supply-chain exposure
  is found.

## Jon-Only Decisions

Jon-only after the independent run:

- Whether the reviewer qualifies as independent enough for public wording.
- Whether to update README/status text from "open" to "independently
  re-verified."
- Whether newly found issues are fixed privately first or filed publicly.
- Whether any trust-band finding changes release, launch, or public-post timing.
- Whether the two LOW follow-ups should be promoted into immediate fix slices.

## Package Acceptance Checklist

This report-only package is acceptable when:

- It lands without touching frozen W-REBUILD crates.
- It preserves the current self-verified S114 boundary.
- It gives an independent reviewer exact commands and expected outcomes.
- It separates Garnet-preparable material from independent-only validation.
- It does not alter CI, dogfood thresholds, capability standards, or release
  policy.
