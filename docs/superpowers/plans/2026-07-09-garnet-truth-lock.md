# Garnet Truth Lock Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Garnet's launch truth reproducible and consistent across committed native Linux evidence, machine-readable reporters, procedural contracts, generated public metrics, and a tracked pre-launch ledger.

**Architecture:** Land Truth Lock as four focused PRs. First, teach the platform reporters to consume committed native Linux L1-L4 evidence without broadening OS-sandbox claims. Second, add a launch-readiness aggregator whose Markdown rendering is the canonical ledger. Third, repair procedural-document drift. Fourth, re-measure and stamp machine truth from the final Truth Lock commit. Site layout and marketing copy stay out of this workstream.

**Tech Stack:** Python 3 reporters and `unittest`, Rust `xtask truth`, Markdown contracts, Git/GitHub, Garnet dogfood readiness.

## Global Constraints

- Base every slice on freshly fetched `origin/main`; use a separate worktree.
- Preserve research-grade v0.x wording; never claim production or 1.0.
- Linux seccomp application is Linux-only; macOS/Windows OS sandboxes remain unproven.
- Native ARM64 Debian evidence is recorded proof, not universal Linux distribution proof.
- S114 is independently re-verified with fixes, pending Jon's acceptance/relabel.
- Do not edit CI, dogfood thresholds, release policy, tags, signing, or public-posting controls.
- One coherent slice per PR; full local ladder, dogfood 5/5, sealed bundle, full remote CI, then established merge flow.
- Do not modify the gate a PR merges under.

---

### Task 1: Land the approved design and execution plans

**Files:**
- Create: `F_Project_Management/GARNET_LAUNCH_CONVERGENCE_DESIGN_2026_07_09.md`
- Create: `F_Project_Management/GARNET_LAUNCH_CONVERGENCE_EXECUTION_INDEX_2026_07_09.md`
- Create: `docs/superpowers/plans/2026-07-09-garnet-truth-lock.md`

**Interfaces:**
- Consumes: approved decomposition and live baseline `9c9ca9e`.
- Produces: durable constraints and the ordered Truth Lock plan used by workers.

- [ ] **Step 1: Verify the docs-only diff**

```sh
git diff --check
python3 scripts/check-agent-contracts.py
python3 scripts/test_check_agent_contracts.py
cargo fmt --all -- --check
```

Expected: every command exits `0`; agent contracts report `23 contracts`.

- [ ] **Step 2: Scan for unresolved placeholders**

```sh
rg -n 'TB[D]|TO[D]O|FIXM[E]|implement[ ]later|fill[ ]in details' \
  F_Project_Management/GARNET_LAUNCH_CONVERGENCE_DESIGN_2026_07_09.md \
  F_Project_Management/GARNET_LAUNCH_CONVERGENCE_EXECUTION_INDEX_2026_07_09.md \
  docs/superpowers/plans/2026-07-09-garnet-truth-lock.md
```

Expected: no matches.

- [ ] **Step 3: Commit and publish the docs branch**

```sh
git add F_Project_Management/GARNET_LAUNCH_CONVERGENCE_DESIGN_2026_07_09.md \
  F_Project_Management/GARNET_LAUNCH_CONVERGENCE_EXECUTION_INDEX_2026_07_09.md \
  docs/superpowers/plans/2026-07-09-garnet-truth-lock.md
git commit -m "docs: define launch convergence execution"
git push -u fork codex/launch-convergence-design-20260709
```

Expected: remote branch SHA equals local `HEAD`.

- [ ] **Step 4: Open, gate, and merge the docs PR**

Validate the exact dogfood body before opening:

```sh
python3 scripts/check_dogfood_pr_body.py \
  --base origin/main --head HEAD \
  --body-file /tmp/garnet-launch-convergence-design-pr.md
```

Expected: `dogfood-pr-body: ok`. Open from `Navigata1`, wait for every remote
check, then merge through `IslandDevCrew`. Do not tag.

---

### Task 2: Consume native Linux proof in readiness reporters

**Files:**
- Modify: `scripts/garnet_windows_linux_studio_status.py`
- Modify: `scripts/test_garnet_windows_linux_studio_status.py`
- Modify: `scripts/garnet_mit_readiness_status.py`
- Modify: `scripts/test_garnet_mit_readiness_status.py`
- Modify: `scripts/garnet_mac_side_continuation_status.py`
- Modify: `scripts/test_garnet_mac_side_continuation_status.py`
- Modify: `F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json`
- Modify: `CURRENT_STATE.md` (native Linux truth only)

**Interfaces:**
- Consumes: `garnet_seccomp_apply_status.read_status()`, `garnet_native_debian_cli_install_status.evaluate()`, and `garnet_native_linux_studio_status.evaluate()`.
- Produces: committed lanes `linux_seccomp_apply`, `native_debian_cli_install`, and `native_linux_studio`; removes the already-closed non-WSL Linux blocker.

- [ ] **Step 1: Write the failing Windows/Linux reporter test**

```python
def test_native_linux_evidence_closes_non_wsl_blocker(self) -> None:
    status = status_mod.read_status()
    truth = " ".join(status.current_truth)
    blocked = " ".join(status.user_assistance_needed)
    deferred = " ".join(status.next_slices)

    self.assertIn("native ARM64 Debian", truth)
    self.assertIn("non-WSL", truth)
    self.assertNotIn("Linux VM/container", blocked)
    self.assertNotIn("non-WSL Linux desktop", deferred)
    self.assertIn("signed Linux distribution", deferred)
    self.assertEqual(
        "native-arm64-build-install-launch-verified",
        status.packaging_gates["linux_package_choice"].status,
    )
```

Run `python3 scripts/test_garnet_windows_linux_studio_status.py`.

Expected: FAIL because the existing reporter still calls non-WSL Linux proof open.

- [ ] **Step 2: Import and evaluate native reporters**

Add:

```python
import garnet_native_debian_cli_install_status  # noqa: E402
import garnet_native_linux_studio_status  # noqa: E402
import garnet_seccomp_apply_status  # noqa: E402
```

At the start of `read_status`, evaluate:

```python
native_cli = garnet_native_debian_cli_install_status.evaluate()
native_studio = garnet_native_linux_studio_status.evaluate()
seccomp = garnet_seccomp_apply_status.read_status()
native_linux_verified = native_cli.ok and native_studio.ok and seccomp.ok
```

When true, append native ARM64 Debian CLI, Tauri Studio, and Linux-only seccomp
truth; set the Linux packaging gate to
`native-arm64-build-install-launch-verified`; remove non-WSL proof from
assistance/next slices; preserve unsigned packages, x86_64 breadth, other
distros, and production as deferred.

- [ ] **Step 3: Make the Windows/Linux reporter test pass**

```sh
python3 scripts/test_garnet_windows_linux_studio_status.py
python3 scripts/garnet_windows_linux_studio_status.py --format json
```

Expected: tests pass; JSON names native ARM64 Debian proof and contains no
non-WSL Linux assistance request.

- [ ] **Step 4: Write failing MIT reporter tests**

```python
def test_native_linux_lanes_are_committed_and_honest(self) -> None:
    status = status_mod.read_status()
    lanes = {lane.id: lane for lane in status.lanes}

    for lane_id in (
        "linux_seccomp_apply",
        "native_debian_cli_install",
        "native_linux_studio",
    ):
        self.assertEqual("verified", lanes[lane_id].status)
        self.assertEqual("committed", lanes[lane_id].evidence_class)
        self.assertEqual(100.0, lanes[lane_id].completion_percent)

    distribution = lanes["windows_linux_distribution"]
    self.assertNotIn("Linux VM/container", " ".join(distribution.blocked_by))
    self.assertIn("unsigned", " ".join(distribution.deferred).lower())
```

Run `python3 scripts/test_garnet_mit_readiness_status.py`.

Expected: FAIL because the lanes do not exist and the stale blocker remains.

- [ ] **Step 5: Add the three MIT lanes**

Append `ObjectiveLane` values with the following first lane shape and analogous
CLI/Studio boundaries:

```python
ObjectiveLane(
    id="linux_seccomp_apply",
    label="Native Linux seccomp application",
    status="verified" if seccomp.ok else "planned",
    completion_percent=100.0 if seccomp.ok else 0.0,
    evidence="Recorded UTM Debian ARM64 proof applies the generated policy and traps a denied socket syscall.",
    blocked_by=[] if seccomp.ok else ["recorded deterministic seccomp application proof"],
    deferred=["macOS/Windows OS-sandbox application remains unverified"],
    evidence_class="committed",
)
```

The CLI lane defers signed/universal Linux packaging. The Studio lane defers
signing, non-ARM64 breadth, and production distribution.

- [ ] **Step 6: Refresh the committed readiness baseline**

Regenerate the baseline after adding the three committed lane IDs, then run the
regression gate:

```sh
python3 scripts/garnet_mit_readiness_status.py --format json \
  > F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
```

Expected: the baseline contains all three native Linux lane IDs and the
regression check exits `0`. Do not hand-edit computed percentages.

- [ ] **Step 7: Repair the Mac continuation reporter**

Replace the stale target-system sentence with:

```python
"Native ARM64 Linux CLI, seccomp, and Studio proof is committed; remaining target-system work is Windows signing/ARM64 and broader Linux distribution"
```

Keep the lane non-Mac-actionable but remove `Linux runtime execution` from its
blockers. Add a test for the sentence and blocker absence.

- [ ] **Step 8: Reconcile current-state Linux truth**

Update only the native Linux status paragraphs in `CURRENT_STATE.md`: cite the
committed ARM64 Debian CLI, seccomp-application, and Tauri Studio proof; remove
the closed clean/non-WSL Linux runtime blocker; retain unsigned, non-ARM64,
broader-distro, production, and macOS/Windows sandbox boundaries.

- [ ] **Step 9: Run focused reporter verification**

```sh
python3 scripts/test_garnet_windows_linux_studio_status.py
python3 scripts/test_garnet_mit_readiness_status.py
python3 scripts/test_garnet_mac_side_continuation_status.py
python3 scripts/garnet_seccomp_apply_status.py --gate
python3 scripts/garnet_native_debian_cli_install_status.py --gate
python3 scripts/garnet_native_linux_studio_status.py --gate
python3 scripts/garnet_mit_readiness_status.py --check-no-regression
python3 scripts/garnet_mit_readiness_status.py --format json > /tmp/mit-after-native-linux.json
```

Expected: all commands exit `0`; JSON contains all three committed native lanes.

- [ ] **Step 10: Run the full slice ladder and commit**

```sh
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/check-agent-contracts.py
python3 scripts/test_check_agent_contracts.py
git diff --check
git add scripts/garnet_windows_linux_studio_status.py \
  scripts/test_garnet_windows_linux_studio_status.py \
  scripts/garnet_mit_readiness_status.py \
  scripts/test_garnet_mit_readiness_status.py \
  scripts/garnet_mac_side_continuation_status.py \
  scripts/test_garnet_mac_side_continuation_status.py \
  F_Project_Management/GARNET_v0_5_READINESS_BASELINE.json \
  CURRENT_STATE.md
git commit -m "fix(readiness): consume native Linux completion proof"
```

Expected: zero failures and one readiness-truth commit. Build dogfood evidence,
open a PR, wait for full CI, and merge before Task 3.

---

### Task 3: Add the machine-readable launch ledger

**Files:**
- Create: `scripts/garnet_launch_readiness_status.py`
- Create: `scripts/test_garnet_launch_readiness_status.py`
- Create: `F_Project_Management/LAUNCH/LAUNCH_READINESS.md`
- Modify: `F_Project_Management/AGENTS.md`

**Interfaces:**
- Consumes: release-readiness, red-team, evidence-integrity, native Linux,
  playground, WASM, stdlib, promo, and MIT reporter APIs.
- Produces: `LaunchReadinessStatus`, JSON/human/Markdown renderers, and the
  canonical pre-launch ledger. `--gate` exits `1` until launch-critical gates
  pass and is not added to CI in this slice.

- [ ] **Step 1: Write the failing reporter tests**

```python
def test_current_launch_state_is_hold_with_unmeasured_foundation(self) -> None:
    status = status_mod.read_status()
    gates = {gate.id: gate for gate in status.gates}
    self.assertEqual("blocked", gates["foundation_integrity"].state)
    self.assertEqual("external-pending", gates["s114_acceptance"].state)
    self.assertEqual("remaining", gates["live_wasm_playground"].state)
    self.assertEqual("manual-deferred", gates["minimum_sealed_shelf"].state)
    self.assertEqual("jon-only", gates["launch_fire"].state)
    self.assertEqual("unmeasured", status.evidence_base_status)
    self.assertFalse(status.launch_ready)
    self.assertEqual("HOLD", status.recommendation)

def test_gate_fails_while_playground_and_shelf_are_remaining(self) -> None:
    proc = subprocess.run(
        [sys.executable, str(SCRIPT), "--gate", "--format", "json"],
        capture_output=True, text=True,
    )
    self.assertEqual(1, proc.returncode)
    self.assertFalse(json.loads(proc.stdout)["launch_ready"])
```

Run `python3 scripts/test_garnet_launch_readiness_status.py`.

Expected: FAIL because the reporter does not exist.

- [ ] **Step 2: Implement the reporter data model**

```python
@dataclass(frozen=True)
class LaunchGate:
    id: str
    label: str
    state: str
    evidence: list[str]
    blockers: list[str]

@dataclass(frozen=True)
class LaunchReadinessStatus:
    schema: str
    source: str
    evidence_base: str
    evidence_base_status: str
    release_grade: str
    recommendation: str
    launch_ready: bool
    gates: list[LaunchGate]
    deferred: list[str]
    jon_only: list[str]
```

Use schema `garnet.launch_readiness/v1`. Resolve `evidence_base` from
`docs/truth.json.workspace_tests.measured_at_commit`. The existing truth
generator intentionally records `git rev-parse --short HEAD`, so accept 7-40
hex characters only when `git rev-parse <value>^{commit}` resolves and that
commit is reachable from `HEAD`; reject any `-dirty` suffix. Otherwise emit
`evidence_base_status="unmeasured"`, retain the literal value for diagnosis,
and block `foundation_integrity` on a fresh `xtask truth --with-tests`
measurement. A dirty, malformed, missing, or unreachable value must never be
presented as the canonical launch evidence base.

- [ ] **Step 3: Derive the current gates from structured APIs**

```python
release = garnet_v0_8_1_release_readiness.read_readiness(binary_strict=True)
red_team = garnet_red_team_status.read_status()
integrity = garnet_evidence_integrity_status.read_status()
seccomp = garnet_seccomp_apply_status.read_status()
native_cli = garnet_native_debian_cli_install_status.evaluate()
native_studio = garnet_native_linux_studio_status.evaluate()
playground = garnet_playground_readiness.read_readiness()
wasm = garnet_wasm_readiness.read_readiness()
stdlib = garnet_stdlib_layer_gate.read_status()
promo = garnet_promo_video_status.read_status()
mit = garnet_mit_readiness_status.read_status()
```

Foundation is `pass` only when release readiness, red team, evidence integrity,
and the measured-evidence-base validator pass. Native Linux requires all three
native gates. The red-team API proves its static contract only; S114 acceptance
is an explicit external gate with state `external-pending` until Jon records a
decision outside this reporter. Static playground is `partial`. Live WASM stays
`remaining`. The shelf has no reporter in Truth Lock, so represent it as the
explicit manual/deferred fence `manual-deferred`, never as reporter-derived
machine truth. Promo is `pending-human`. Launch fire is always the external
`jon-only` gate.

Add mocked dependency tests that exercise every consumed reporter and at least
one failure path per launch gate. Tests must prove that release, red-team,
evidence-integrity, native CLI, native Studio, seccomp, playground, WASM,
stdlib, promo, and MIT inputs are invoked and that a failed dependency changes
the corresponding gate or blocker. Add explicit tests for dirty, malformed,
missing, and unreachable evidence-base values, plus a clean reachable short SHA
control matching the truth generator's current contract.

- [ ] **Step 4: Render JSON, human, and Markdown**

Support:

```sh
python3 scripts/garnet_launch_readiness_status.py --format json
python3 scripts/garnet_launch_readiness_status.py --format human
python3 scripts/garnet_launch_readiness_status.py --format markdown
python3 scripts/garnet_launch_readiness_status.py --gate --format json
```

Markdown must preserve JSON gate order and include commit, release grade,
recommendation, evidence, blockers, deferred fences, and Jon-only actions.

- [ ] **Step 5: Generate and pin the canonical ledger**

```sh
mkdir -p F_Project_Management/LAUNCH
python3 scripts/garnet_launch_readiness_status.py --format markdown \
  > F_Project_Management/LAUNCH/LAUNCH_READINESS.md
```

Add a test comparing renderer output byte-for-byte with the tracked file after
normalizing only the repository root. The measured evidence base stays literal.
Update `F_Project_Management/AGENTS.md`
to name the reporter as machine authority only for reporter-derived inputs and
the evidence-base validator. S114 acceptance, the not-yet-reported shelf, and
launch fire remain external/manual gates. Markdown is rendered state.

- [ ] **Step 6: Verify the expected red launch gate**

```sh
python3 scripts/test_garnet_launch_readiness_status.py
python3 scripts/garnet_launch_readiness_status.py --format json
if python3 scripts/garnet_launch_readiness_status.py --gate --format json; then
  echo "expected pre-launch gate to remain HOLD" >&2
  exit 1
fi
```

Expected: tests/status pass; explicit gate exits `1` because W-PLAY and shelf
are unfinished.

- [ ] **Step 7: Run the full ladder and commit**

```sh
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/check-agent-contracts.py
python3 scripts/test_check_agent_contracts.py
git diff --check
git add scripts/garnet_launch_readiness_status.py \
  scripts/test_garnet_launch_readiness_status.py \
  F_Project_Management/LAUNCH/LAUNCH_READINESS.md \
  F_Project_Management/AGENTS.md
git commit -m "feat(readiness): add canonical launch ledger"
```

Expected: all verification passes while the launch gate stays explicitly red.
Build dogfood evidence, open a PR, wait for full CI, and merge.

---

### Task 4: Repair procedural contract drift

**Files:**
- Modify: `garnet-interp-v0.3/AGENTS.md`
- Modify: `garnet-cli/AGENTS.md`
- Modify: `garnet-prim-macros/AGENTS.md`
- Modify: `CURRENT_STATE.md`
- Modify: `scripts/test_check_agent_contracts.py`

**Interfaces:**
- Consumes: RFC-0002, `garnet-cli/src/panic_firewall.rs`, overflow parity tests,
  and primitive-macro helper tests.
- Produces: current procedural truth without runtime-semantic changes.

- [ ] **Step 1: Add a failing contract test for stale statements**

```python
interp = (ROOT / "garnet-interp-v0.3" / "AGENTS.md").read_text(encoding="utf-8")
cli = (ROOT / "garnet-cli" / "AGENTS.md").read_text(encoding="utf-8")
macros = (ROOT / "garnet-prim-macros" / "AGENTS.md").read_text(encoding="utf-8")

self.assertNotIn("Add/sub/mul overflow policy (wraps in release", interp)
self.assertIn("checked by default", interp)
self.assertIn("panic firewall", cli)
self.assertIn("trybuild", macros)
self.assertIn("not yet trybuild-exercised", macros)
```

Run `python3 scripts/test_check_agent_contracts.py`.

Expected: FAIL on the stale overflow sentence and missing CLI firewall contract.

- [ ] **Step 2: Update the three owning contracts**

`garnet-interp-v0.3/AGENTS.md` must state:

```text
Integer arithmetic is checked by default for division, remainder, addition,
subtraction, multiplication, and unary negation on interpreter and VM paths.
Overflow produces the byte-identical controlled diagnostic proven by
overflow_guards.rs and overflow_parity.rs. Explicit wrapping operations remain
deferred and must not be implied.
```

`garnet-cli/AGENTS.md` must record that `eval`, `repl`, `test`, and `doctest` use
the unwinding panic firewall, while stack overflow and other aborting faults are
outside `catch_unwind` and require structural guards.

`garnet-prim-macros/AGENTS.md` must keep `trybuild` named as not implemented and
required before materially widening the Core Ring macro surface.

- [ ] **Step 3: Reconcile `CURRENT_STATE.md`**

Add a dated current-truth note pointing to:

- RFC-0002 and the checked-overflow tests;
- the panic firewall and cyclic-value structural guard;
- native Linux L1-L4 evidence; and
- the canonical launch-readiness reporter and ledger.

Do not rewrite unrelated historical sections.

- [ ] **Step 4: Verify and commit the docs-only correction**

```sh
python3 scripts/check-agent-contracts.py
python3 scripts/test_check_agent_contracts.py
cargo fmt --all -- --check
git diff --check
git add garnet-interp-v0.3/AGENTS.md garnet-cli/AGENTS.md \
  garnet-prim-macros/AGENTS.md CURRENT_STATE.md \
  scripts/test_check_agent_contracts.py
git commit -m "docs: reconcile launch-critical runtime contracts"
```

Expected: all checks pass. Open a focused docs/contract PR, wait for full CI,
and merge.

---

### Task 5: Re-measure and stamp machine truth

**Files:**
- Modify: `docs/truth.json`
- Modify: generated markers in `README.md`, `FAQ.md`, `docs/index.html`, and `docs/status.html`
- Modify: `F_Project_Management/LAUNCH/LAUNCH_READINESS.md`
- Modify: any additional path explicitly reported by `xtask truth`

**Interfaces:**
- Consumes: final merged Truth Lock reporter and contract state.
- Produces: a fresh workspace test measurement and byte-consistent public markers.

- [ ] **Step 1: Start from final Truth Lock `origin/main`**

```sh
git fetch origin main --tags --prune
git rev-parse HEAD
git rev-parse origin/main
```

Expected: both SHAs equal the latest merged Truth Lock commit.

- [ ] **Step 2: Regenerate truth with a real test measurement**

```sh
cargo run -p xtask -- truth --with-tests
```

Expected: workspace tests report zero failures; `docs/truth.json` records the
current commit rather than `c4b9e28-dirty`; generated markers update.

Regenerate the launch ledger from that measured truth:

```sh
python3 scripts/garnet_launch_readiness_status.py --format markdown \
  > F_Project_Management/LAUNCH/LAUNCH_READINESS.md
```

Expected: `evidence_base_status` renders as measured and the tracked ledger
contains the same literal measured commit as `docs/truth.json`.

- [ ] **Step 3: Prove generated state is stable**

```sh
cargo run -p xtask -- truth --check --with-tests
git diff --check
```

Expected: truth check exits `0`. Count divergence stops the slice.

- [ ] **Step 4: Run the final Truth Lock ladder**

```sh
python3 scripts/garnet_readiness_status.py --format json
python3 scripts/garnet_mit_readiness_status.py --format json
python3 scripts/garnet_launch_readiness_status.py --format json
python3 scripts/check-agent-contracts.py
python3 scripts/test_check_agent_contracts.py
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Expected: every command exits `0`; only the explicit launch `--gate` remains red.

- [ ] **Step 5: Commit, gate, and merge generated truth**

```sh
git add docs/truth.json README.md FAQ.md docs/index.html docs/status.html \
  F_Project_Management/LAUNCH/LAUNCH_READINESS.md
git add -u
git commit -m "docs(truth): refresh launch baseline measurement"
```

Build dogfood evidence, validate the PR body, wait for full remote CI, and
merge. Do not combine W-PLAY or site redesign with this PR.

---

### Task 6: Truth Lock completion audit

**Files:**
- Read: every file modified by Tasks 2-5
- Read: `F_Project_Management/LAUNCH/LAUNCH_READINESS.md`

**Interfaces:**
- Consumes: merged PRs from Tasks 2-5.
- Produces: evidence that Truth Lock is complete and W-PLAY can begin.

- [ ] **Step 1: Verify live merged state**

```sh
git fetch origin main --tags --prune
git log --oneline --decorate -12 origin/main
gh pr list --repo Island-Dev-Crew/garnet --state open --limit 20
```

Expected: every Truth Lock PR is merged; no overlapping reporter PR is open.

- [ ] **Step 2: Re-run authoritative status commands**

```sh
python3 scripts/garnet_seccomp_apply_status.py --gate
python3 scripts/garnet_native_debian_cli_install_status.py --gate
python3 scripts/garnet_native_linux_studio_status.py --gate
python3 scripts/garnet_mit_readiness_status.py --format json
python3 scripts/garnet_launch_readiness_status.py --format json
cargo run -p xtask -- truth --check
```

Expected: native gates pass; MIT status includes native Linux; launch status
recommends HOLD only for genuine remaining gates; committed machine truth is
internally consistent. The remeasurement evidence remains the Task 5 PR log.

- [ ] **Step 3: Confirm documented boundaries**

```sh
rg -n "production|1\.0|OS-sandbox|seccomp|independent|unsigned|notar" \
  F_Project_Management/LAUNCH/LAUNCH_READINESS.md \
  CURRENT_STATE.md garnet-cli/AGENTS.md garnet-interp-v0.3/AGENTS.md
```

Expected: every strong term is bounded by platform or deferred status. Any
unbounded use reopens the owning slice.

- [ ] **Step 4: Begin W-PLAY planning**

Write the detailed W-PLAY plan against the final `origin/main` and begin its
first red test. Do not mark the full launch goal complete.
