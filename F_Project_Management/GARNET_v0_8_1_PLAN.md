# Garnet v0.8.1 runway — S81–S110 plan (Windows-audit-driven)

> **Status of this document.** This is the committed, cross-machine source of
> truth for the v0.8.1 runway plan (so Windows Codex and any reviewer can read it
> by path). S81–S90 are **merged** on `origin/main` (tip `db06f36`); S91–S110
> below are **re-sliced but not yet executed** — held for Jon's execution choice.
> No S91+ implementation has begun. The tag is escalated to Jon, never autonomous.

## Execution status (2026-06-01) — S81–S90 burn-down COMPLETE, Windows lane reported

The full burn-down is merged on `origin/main` (**tip `db06f36`, S88**), and the
Windows lane has reported back (the `[WINDOWS-PROVE]` slices recorded real Windows
proofs). **All 14 `WIN-*` findings are closed; zero open PRs.**

| Slice | impl PR | Windows-proof PR | Closes |
|---|---|---|---|
| P0 commit Windows audit | #298 | — | (tracked-truth import) |
| S81 case-insensitive `.GARNET` | #299 | #303 | WIN-S33/36/37/46-001 |
| S82 seal LF/CRLF determinism | #301 | #305 | WIN-S38-001 |
| S83 post-tag release truth | #311 | — | WIN-S80-002 |
| S84 Exp 3 WSL/bash path | #300 | (in #300) | WIN-S71-001 |
| S85 interp deep-recursion | #302 | #306 | WIN-S73-001 |
| S86 binary-strict cut readiness | #313 | — | WIN-S80-001 |
| S87 Windows reporter hardening | #309 | — | WIN-S6-001 / S31-001 / S31-002 |
| S88 release-tooling status | #312 | — | WIN-S38-002 / S39-001 (honest-partial) |
| S89 `@max_depth` enforcement seed | #304 | #308 | (new surface) |
| S90 `@caps` enforcement seed | #307 | #310 | (new surface) |

**Honest-partial note:** WIN-S38-002 (cosign/syft/cyclonedx) and WIN-S39-001
(Wasmtime fuel) closed as *infra-honest* — S88 reports the tools **absent** rather
than faking signed/SBOM/fuel proof. They remain `pending-infra` for the *real*
enforcement, which several S91+ slices inherit (see the infra-gate tags below).

My local uncommitted S84 edit on `codex/s84-wsl-bash` is **superseded** by merged
#300 — to be discarded, not committed.

**Decision (2026-06-01, Jon):** the burn-down has landed → run the **strategic
re-plan lane**: re-slice S91–S110 at full PR-sized resolution, grounded in what
S89/S90 *actually* enforce (not what was planned), present, and **STOP for Jon's
option choice** (1 execute+bypass / 2 execute+approve / 3 refine / 4 tell Claude
what to change; default 4). No S91+ implementation until Jon picks execution.

## Context

The S30–S80 run is complete (50/50) and **`v0.8.0` is tagged** (`cc165e8`, tagger
Jon Isaac, 2026-05-31). This is **not** relitigated. **S81–S110 is strictly the
v0.8.1 runway.**

A read-only Codex **Windows audit of S1–S80** (HEAD `cc165e8`) is the new
source-of-truth evidence. `cargo test`/`clippy` are green on Windows and most
gates pass, but the audit found **14 open `WIN-*` findings** that cluster tightly:
a single case-sensitive `.GARNET` discovery bug spans four trust slices; seal
hashing is CRLF-unstable; the Paper VI Exp 3 reporter feeds Windows absolute paths
to WSL bash; the interpreter stack-overflows on a deep-recursion fixture the VM
runs fine; and the S80 cut aggregate uses `--no-run`, so READY can hide direct
Windows binary failures. Plus a post-tag split-truth (v0.8.0 is cut, but
`GARNET_v0_8_0_CUT.md`/ledger still read pre-tag/pending).

**This plan fixes the audit-proven trust + Windows/runtime gaps FIRST (S81–S88),
ordered by blast radius — not win-number — then opens two new-surface enforcement
seeds (S89–S90), then turns to the strategic v0.8.1 arc (S91–S110, resolution
decreasing).** Calibrated honesty is preserved throughout: v0.8.0 (and v0.8.1)
are research-grade-prototype milestones, never production/1.0.

**Handoff discipline (critical):** planning happens on Mac; **Windows-specific
fixes must be executed and verified on Windows.** Every slice that touches
scripts, paths, binaries, sealing, process execution, or examples is tagged
`[MAC-PLANNABLE + WINDOWS-PROVE]` (writable/unit-testable on Mac; must be proven
on Windows) or `[WINDOWS-EXECUTION-REQUIRED]` (the fix's *failure* only reproduces
on Windows, so authoring may happen on Mac but verification is Windows-gated), and
names its **Windows proof command**.

Operating rules unchanged: one slice per PR; minimum real behavior; focused tests
→ workspace verify → dogfood gate; honest CHANGELOG/contract/ledger; PR from
Navigata1 → CI green → merge as IslandDevCrew → switch back; each slice earns its
fused 5/5.

---

## Pre-S81 (P0): commit the Windows audit as tracked truth `[MAC-PLANNABLE]`

The audit findings are currently **untracked** (only in the handoff tarball). The
burn-down needs a committed source of truth.

- **Deliverable:** `F_Project_Management/WINDOWS_AUDIT_S1_S80.md` (the summary + the
  14-finding ledger + the resolved `WIN-S70-001`), plus the two machine ledgers
  `.dogfood/windows-core-audit.json` and `.dogfood/windows-audit-goal.json`
  committed verbatim. A small `scripts/garnet_windows_audit_status.py --gate`
  asserts every open `WIN-*` id appears in the tracked doc with an owning slice.
- **Dogfood:** `python3 scripts/test_garnet_windows_audit_status.py`; gate rc 0.
- **Windows proof:** none (pure docs/data import). Lands as the S81 PR's first
  commit or a standalone P0 PR — recommend **standalone P0 PR** so S81 stays the
  narrow `.GARNET` fix.

---

## S81–S88 — Audit burn-down (ordered by blast radius)

### S81 — case-insensitive `.GARNET` target discovery `[MAC-PLANNABLE + WINDOWS-PROVE]`
**Goal:** ONE fix clears FOUR findings. The shared target collector compares the
file extension case-sensitively, so on Windows' case-insensitive filesystem an
uppercase `.GARNET` file is silently skipped by `garnet verify`, capability
manifests, `diff-caps`, and sandbox-policy directory walks — a *trust* hole
(a planted `BAD.GARNET` passed a 5/5 gate). Make the shared collector lowercase
the extension before the `== "garnet"` compare.
**Root/files:** `garnet-cli/src/cmd/verify_gate.rs:218` (the case-sensitive check;
collector documented at `:189`), reused by `garnet-cli/src/cap_manifest.rs:91`.
**Change:** one fix in the shared collector; add regression coverage for the
verify / caps / diff-caps / sandbox-policy directory walks.
**Dogfood:** `cargo test -p garnet-cli verify_gate` + a new unit test asserting an
uppercase `.GARNET` fixture is discovered by the collector (Mac-writable, since it
tests the collector logic, not the filesystem). Reporter `garnet_garnet_ext_discovery_status.py --gate`.
**Windows proof cmd:** `garnet verify <dir with a clean main.garnet + a parse-broken BAD.GARNET>` → exits **1** (BAD.GARNET discovered, not skipped); on Windows.
**Out of scope:** non-`.garnet` extensions; symlink/junction discovery.
**Closes:** WIN-S33-001, WIN-S36-001, WIN-S37-001, WIN-S46-001.

### S82 — seal source-hash determinism (LF/CRLF) `[MAC-PLANNABLE + WINDOWS-PROVE]`
**Goal:** The seal full predicate's `source_blake3` hashes raw bytes; the repo
does not pin `*.garnet` line endings while Windows checkouts run
`core.autocrlf=true`, so the same logical source yields a different predicate on
Windows vs Mac/Linux — the reproducible-bundle pillar is broken cross-platform.
**Change:** add `.gitattributes` with `*.garnet text eol=lf` (and seal/dogfood
text artifacts as needed); **document the seal canonicalization contract**
(raw-byte source hash + `eol=lf` pin) in `C_Language_Specification/GARNET_ATTESTATION.md`.
**Dogfood:** a Rust regression in `garnet-cli/tests/` asserting a `\n` vs `\r\n`
variant of the same source produces the **same** predicate once normalized;
reporter `garnet_seal_determinism_status.py --gate` asserts the `.gitattributes`
pin + the documented contract.
**Windows proof cmd:** fresh Windows checkout (with `.gitattributes`) → `garnet seal <file>` predicate `source_blake3` matches the Mac value for the same commit.
**Out of scope:** signing/SBOM (S88); changing the hash algorithm.
**Closes:** WIN-S38-001.

### S83 — post-tag release-truth reconciliation `[MAC-PLANNABLE]`
**Goal:** Both truths in one place: **v0.8.0 was cut by Jon (2026-05-31)** AND
**the S80 PR itself only produced cut-readiness evidence.** Today
`GARNET_v0_8_0_CUT.md` still says "READY TO CUT (pending Jon)", the cut gate says
it does not authorize a tag, and `.dogfood/goal.json` keeps `s80` pending with
null confidence — a split-truth surface to remove before academic review.
**Change:** add a dated "Post-cut release truth" note to `GARNET_v0_8_0_CUT.md`
(tag cut by Jon @ `cc165e8`; the gate remains advisory-only); advance
`.dogfood/goal.json` `s80 → merged(5)` with a `cut_record`; reconcile the
`GARNET_v0_8_0_cut_readiness` doc/anchor wording. **No code behavior change.**
**Dogfood:** `garnet_release_truth_status.py --gate` asserts the cut note + the
ledger `s80=merged` + the "S80 PR was readiness-only" sentence coexist.
**Windows proof cmd:** none (pure docs/ledger). 
**Closes:** WIN-S80-002.

### S84 — Paper VI Exp 3 reporter: WSL/bash path handling `[WINDOWS-EXECUTION-REQUIRED]`
**Goal:** `garnet_paper_vi_exp3_status.py` runs the lane scripts with `cwd=HARNESS`
but passes the **absolute** Windows path to bash; under WSL that becomes
`/bin/bash: C:\...\run_stateless.sh: No such file or directory` (exit 127), so the
provider-free gate fails on Windows.
**Change:** when `os.name == "nt"`, invoke the lane scripts by **relative POSIX
name** from the HARNESS cwd (the cwd is already HARNESS), or translate via
`wslpath`. Mac behavior is unchanged (relative path from cwd still resolves).
**Files:** `scripts/garnet_paper_vi_exp3_status.py` (`_provider_free_run_ok`/`_run`).
**Dogfood:** `python3 scripts/test_garnet_paper_vi_exp3_status.py` (still 6/6 on Mac).
**Windows proof cmd:** on Windows, `python scripts/test_garnet_paper_vi_exp3_status.py` → **6/6** (`provider_free_run_ok=true`); `python scripts/garnet_paper_vi_exp3_status.py --gate` rc 0 (no `--no-run`).
**Out of scope:** the provider-backed h₃a re-run (pending-infra; S95).
**Closes:** WIN-S71-001.
**Tag rationale:** the failure only reproduces under Windows+WSL; the one-line fix
is Mac-authorable but **must be proven on Windows**.

### S85 — interpreter deep-recursion robustness (parity fix) `[WINDOWS-EXECUTION-REQUIRED]`
**Goal:** `garnet run --interp examples/mvp_function_call_demo.garnet` stack-
overflows on Windows (exit `0xC00000FD`) where `--vm` succeeds (`=> 7105`), so the
binary-backed VM/interp parity campaign is 32/33 on Windows. Preferred fix is
**robustness, not a Windows patch**: run the interpreter on a spawned thread with
an explicit larger stack (`std::thread::Builder::stack_size`), so the default
~1 MB Windows thread stack no longer caps recursion depth below the VM's.
**Files:** the interp entry in `garnet-cli/src/cmd/run.rs` (`run_interp`) /
`garnet-interp-v0.3` eval entry.
**Change:** spawn the interpreter eval on a large-stack thread; join the result.
Apply on all platforms (deterministic, cross-platform).
**Dogfood:** `cargo test --workspace`; the parity campaign reports 33/33.
**Windows proof cmd:** `.\target\debug\garnet.exe run --interp .\examples\mvp_function_call_demo.garnet` → exit 0, `=> 7105`; `python scripts\garnet_vm_interp_parity.py --gate` → **33/33** on Windows.
**Out of scope:** a true tail-call/bounded-recursion guarantee (that is S89/S93);
this is a host-stack robustness fix + an honest scope note if any fixture still
exceeds the larger stack.
**Closes:** WIN-S73-001.

### S86 — binary-strict mode for S80 cut readiness `[MAC-PLANNABLE + WINDOWS-PROVE]` — **depends S84, S85**
**Goal:** The S80 aggregate runs S71/S72/S73 gates with `--no-run`, so READY can
hide direct binary failures (exactly what happened on Windows). Add a
`--binary-strict` / `--windows-audit` mode that runs the binary-dependent runway
gates **without** `--no-run` and surfaces direct failures, so aggregate READY
cannot be misread as full Windows runtime proof.
**Files:** `scripts/garnet_v0_8_0_cut_readiness.py` (the `RUNWAY_GATES` `--no-run`
entries at the S71/S72/S73 lines).
**Change:** a `--binary-strict` flag that drops `--no-run` for the binary gates and
reports any direct failure as a blocking finding; default (lenient) mode unchanged
for the python-only CI job.
**Dogfood:** `python3 scripts/test_garnet_v0_8_0_cut_readiness.py`; strict mode rc 0
**only after S84+S85 land** (the binary gates must actually pass).
**Windows proof cmd:** on Windows, `python scripts\garnet_v0_8_0_cut_readiness.py --gate --binary-strict` → rc 0 (post-S84/S85); pre-fix it would have failed — proving the strict mode is honest.
**Closes:** WIN-S80-001.
**Depends:** S84 (S71 gate green) + S85 (S73 gate green) — strict mode must have
something true to be strict about.

### S87 — Windows hardening sweep (stdout/temp/committed-only) `[MAC-PLANNABLE + WINDOWS-PROVE]`
**Goal:** Three reporter-robustness gaps: (a) Markdown mode can fail on Windows
cp1252 stdout (WIN-S6-001); (b) the MIT readiness reporter aborts when temp
fixtures hit a denied Windows temp dir (WIN-S31-001); (c) full readiness JSON/MD is
machine-specific, so cross-machine byte comparison needs a committed-only surface
(WIN-S31-002).
**Change:** (a) force UTF-8 stdout in the reporters (`sys.stdout.reconfigure(encoding="utf-8")`
guarded, or encode explicitly); (b) make temp-fixture creation degrade gracefully
(skip-with-note) when temp is denied; (c) emit a committed-only readiness subset
for deterministic cross-machine comparison.
**Files:** `scripts/garnet_memory_eviction_status.py`, `scripts/garnet_mit_readiness_status.py`,
+ a shared stdout helper.
**Dogfood:** unit tests for each; a `garnet_windows_hardening_status.py --gate`.
**Windows proof cmd:** on Windows (cp1252 console), `python scripts\garnet_memory_eviction_status.py --format md` and `...\garnet_mit_readiness_status.py` run clean; the committed-only surface is byte-identical Mac↔Windows.
**Closes:** WIN-S6-001, WIN-S31-001, WIN-S31-002.

### S88 — release-tooling provisioning + proof lane `[WINDOWS-EXECUTION-REQUIRED + pending-infra]`
**Goal:** Two environment-gated lanes: external signing/SBOM (`cosign`/`syft`/
`cyclonedx` absent on the Windows machine, WIN-S38-002) and bounded-execution
runtime proof (no local Wasmtime fuel/epoch, WIN-S39-001). Provide a provisioning
path + a proof lane that is **honest when tools are absent** (never stamps
signed/SBOM-verified or fuel-proven without the tool present).
**Change:** `scripts/garnet_release_tooling_status.py` detects tool presence and
reports per-tool honestly; a documented provisioning step (install cosign/syft/
cyclonedx; install wasmtime) + a fuel/epoch proof harness that runs only when
`wasmtime` is present.
**Dogfood:** the gate passes in "tools-absent" mode by reporting absent (not
faking); a CI/local lane stamps signed/SBOM/fuel only when present.
**Windows proof cmd:** on a tooled Windows box, `cosign verify-blob` / `syft` /
`cyclonedx` succeed and `wasmtime` fuel proof runs; otherwise the gate prints the
honest "tools absent — not verified here" status.
**Closes:** WIN-S38-002, WIN-S39-001.

---

## S89–S90 — first NEW-SURFACE slices (only after the burn-down)

### S89 — `@bounded` runtime enforcement seed `[MAC-PLANNABLE + WINDOWS-PROVE]`
**Goal:** Begin making the kernel **real**: enforce ONE ceiling (recursion/step
depth, governed by the already-parsed `@bounded`/`@max_depth`) in the interpreter,
with a deterministic trap on exceed (building on the S85 large-stack thread + the
S40 `explosive.rs` identification). Honest scope: one ceiling enforced; memory/
time/mailbox remain declared-not-enforced (named).
**Dogfood:** a fixture exceeding the ceiling traps deterministically; one within
runs; interp/VM parity of the trap (extends S73 campaign). Reporter gate.
**Windows proof cmd:** `garnet.exe run` the over-ceiling fixture → deterministic trap (same on Windows).

### S90 — `@caps` host-authority runtime enforcement seed `[MAC-PLANNABLE + WINDOWS-PROVE]`
**Goal:** A managed fn declaring `@caps(net)` cannot exercise an **undeclared**
host-authority capability at runtime — the interpreter checks declared `@caps` at
the `std::env`/`std::process`/`std::fs` boundary and traps on undeclared use.
Honest scope: host-authority stdlib surfaces only; pure computation unaffected.
**Dogfood:** an undeclared-capability fixture traps; a declared one runs; reporter
gate. **Windows proof cmd:** the trap fires identically on Windows.

## S91–S110 — strategic v0.8.1 arc (re-sliced at full resolution, 2026-06-01)

**Grounding (as-merged, from S89/S90 source + S46/S68/S38):**
- S89 enforces **ONE** ceiling — `@max_depth(N)` recursion, deterministic trap
  `bounded: @max_depth(N) exceeded for `fn` (recursion depth D)`. `@bounded`
  (Wasmtime fuel), memory, time, mailbox = **declared-not-enforced**.
- S90 enforces `@caps` at the interpreter's **env/proc/fs/log-to-file** bridges,
  trap `capability: `fn` requires @caps(C), not declared in the calling chain`.
  **Named gaps:** (a) `net` is **not** gated at `bridge_net_tcp_connect` (only
  `NetPolicy` address-filtering); (b) `managed_frames == 0 → allow` lets a direct
  host call, a `Safe` fn with no managed ancestor, or an FFI/`proc` child bypass;
  (c) the **VM backend enforces neither**.
- S46 emits seccomp/WASI/egress policy with `"enforced": false` — **generation
  only**; nothing applies it at an OS boundary. **Wasmtime/seccomp absent** on the
  dev machines (WIN-S39-001 `pending-infra`); **cosign/syft/cyclonedx absent**
  (WIN-S38-002 `pending-infra`).

So S91+ first **closes the enforcement gaps the interpreter can actually see on
Mac** (real traps, Mac-provable), and **quarantines the OS-boundary + provider +
signing work behind honest infra-gate tags** — never claiming "enforced/signed/
fuel-proven" without the boundary or tool present.

**Tag legend:** `[MAC-PROVE]` author+verify on Mac · `[WIN-PROVE]` author on Mac,
must pass its named command on Windows · `[LINUX-INFRA]` needs a Linux kernel +
seccomp/Wasmtime (pending-infra; honest-declared until run) · `[ACCT-GATED]` needs
a provider key / billing → **escalate to Jon, never autonomous**.

### P0-v0.8.1 — re-initialize the goal ledger `[MAC-PROVE]`
Re-init `.dogfood/goal.json` for a `v0_8_1` goal with the S81–S110 slices (S81–S90
pre-marked merged; S91–S110 pending). `scripts/garnet_v0_8_1_goal_status.py --gate`
asserts the ledger shape + that every S91+ slice carries a proof tag. No code.
*First execution step once Jon picks execution.*

**Stage A — make `@caps` enforcement sound (close the S90 gaps)**

### S91 — `@caps` interpreter soundness: gate `net` + program-entry frame `[MAC-PROVE]` — dep S90
**Goal:** make the *declared* `@caps` actually enforced across **every** host-
authority surface the interpreter owns. (1) Add `require_capability("net", …)` to
`bridge_net_tcp_connect` (today only `NetPolicy` filters addresses). (2) Ensure the
top-level program runs **inside** a managed frame so `managed_frames == 0` is true
only for genuine host/test calls — closing the "entry-point launders authority"
read of gap (b). **Files:** `garnet-interp-v0.3/src/{stdlib_bridge.rs,eval.rs}`.
**Dogfood:** `garnet-cli/tests/caps_enforcement.rs` gains `undeclared_net_traps` +
`program_entry_is_a_managed_frame`; `scripts/garnet_caps_enforcement_status.py`
asserts the `net` gate. **Win-proof:** `garnet.exe run` an undeclared-`net`
fixture → same trap on Windows. **Out:** OS-level net egress (→ S92/`[LINUX-INFRA]`);
VM backend (→ S96/later). **Honest scope:** in-process interpreter authority only.

### S92 — authority-laundering closure: spawn/FFI gate + OS-policy application `[MAC-PROVE]` + `[LINUX-INFRA]` — dep S91
**Goal:** two honest layers. **(a) Mac-provable:** gate the `proc`-spawn and FFI
entry behind `@caps(proc)`/`@caps(ffi)` *even when reached through a `Safe` fn*
(close the laundering path the interpreter **can** see). **(b) `[LINUX-INFRA]`:**
apply the S46-generated seccomp/WASI profile to the spawned child so it cannot
exceed the declared surface; **honestly document the residual** — Garnet enforces
the *Garnet* frame chain; OS containment of the child is the seccomp layer, which
is *declared-only* on any host without seccomp/Wasmtime. **Files:** interp
spawn/FFI bridges; `garnet-cli/src/sandbox.rs` (apply path, infra-gated).
**Dogfood:** Mac tests for the spawn/FFI gate trap; a `[LINUX-INFRA]` lane that
runs only when seccomp present, else reports "declared, not applied — no seccomp."
**Linux-proof:** on Linux, spawn-with-policy denies an out-of-surface syscall
(exit non-zero). **Out:** macOS/Windows OS sandbox (documented reality, not faked).

**Stage B — first formal + measurement increments**

### S93 — static bounded-loop verifier (first formal increment) `[MAC-PROVE]` — dep S89, S74, S75
**Goal:** the S75 feasibility verdict's first real step, **without** Wasmtime:
statically derive a loop's iteration bound where decidable for the safe subset
(counted `for` over constant/param-bounded ranges), and **reject the uncheckable**
with a clear diagnostic. Complements S89's runtime ceiling (this is compile-time).
**Files:** `garnet-check-v0.3/src/bounds.rs` (exists — extend), checker wiring.
**Dogfood:** fixtures — a statically-bounded loop passes; an unbounded `while`
is rejected with "bound not statically derivable"; reporter gate.
**Win-proof:** same accept/reject on Windows. **Honest scope:** decidable subset
only; general termination stays out (named). **Out:** Wasmtime fuel runtime (`@bounded`, `[LINUX-INFRA]`).

### S94 — Paper VI Exp 1 (LLM pass@1): wire provider behind a flag `[ACCT-GATED]`
**Goal:** wire the Exp 1 harness to a real provider behind `--provider`; run only
if a key/credits land (**escalate to Jon — billing**). Absent a key, the gate is
**honest-pending** with the harness fully wired + a recorded "not measured here."
**Dogfood:** harness unit tests (mock provider) pass on Mac; the real run is
account-gated. **Honest scope:** no invented pass@1 number; stays pending until a
real provider runs. *Escalation slice — not autonomous.*

### S95 — Paper VI Exp 3 re-run at ~5K-LOC `[MAC-PROVE]` — dep S84
**Goal:** resolve the recorded h₃a 6.5%→10% question honestly on a larger codebase
using the **provider-free** compiler-as-agent harness (no `[ACCT-GATED]` needed).
**Dogfood:** a ~5K-LOC fixture lane; `garnet_paper_vi_exp3_status.py` reports the
re-measured delta; **no number is changed without the harness producing it.**
**Win-proof:** the lane runs green on Windows (builds on the S84 WSL fix).

### S96 — linear/effect-typed safe-mode seed `[MAC-PROVE]` — dep S92
**Goal:** S74's proposal → a first real increment toward *provable* `@caps`
soundness: thread a single capability/effect dimension through `Safe`-fn signatures
in the checker, so an undeclared effect is a **type error** (not just a runtime
trap). **Files:** `garnet-check-v0.3` type/effect pass. **Dogfood:** a Safe fn that
uses `fs` without declaring it fails type-check; reporter gate. **Honest scope:**
one effect dimension, seed only — not a full effect system; runtime trap (S90/S91)
remains the backstop. **Win-proof:** same type error on Windows.

**Stage C — provenance & the cross-language standard**

### S97 — provenance/attestation hardening: bind + verify the chain `[MAC-PROVE]` — dep S82
**Goal:** today S65/S66 authorship + `model`/`prompt_sha256`/`tool` are
**self-declared, unbound**. Harden by **binding** the attestation block to the
`build_manifest` hash inside the sealed predicate and adding a `garnet seal
--verify` path that fails if the attestation was reattached to a different artifact.
**Files:** `garnet-cli/src/{seal.rs,manifest.rs}`, `GARNET_ATTESTATION.md`.
**Dogfood:** a tamper test (swap source → verify fails); reporter gate.
**Honest scope:** binds + verifies the **chain**, does **not** attest the *truth*
of the human/AI claim (a process/identity question — named); cosign signing stays
**wrapped** (absent → unsigned, honestly). **Win-proof:** verify passes/fails
identically on Windows.

### S98 — transparency log → cross-language manifest standard seed `[MAC-PROVE]` — dep S97
**Goal:** advance S68's local BLAKE3 chain + RFC-0001 to a **versioned, language-
agnostic JSON schema** (`capability-manifest/v1`) with a conformance test + the
Garnet tools as the reference impl. **Files:** `rfcs/0001-…md`,
`C_Language_Specification/GARNET_CAPABILITY_TRANSPARENCY.md`, a schema + conformance
fixture. **Honest scope:** **intent + reference impl only — no OWASP/LF body has
adopted anything**; the log stays local/tamper-evident, **not Rekor** (no witness
/inclusion proof). **Win-proof:** conformance fixture validates on Windows.

**Stage D — real-world proofs (agents running real Garnet, enforced)**

### S99 — agent-driven build/test loop harness `[MAC-PROVE]` — dep S89–S92, S98
**Goal:** the reusable gated primitive: a proposer emits a Garnet change →
`diff-caps` gates the authority delta (block on `aggregate_added`/wildcard) → the
**enforced** kernel (S89–S92) runs it → `seal` (S97) attests it → the S68 log
records it. **Files:** new `garnet-cli` loop harness + tests. **Honest scope:** the
proposer is a **scripted/mock** agent first; the **real-LLM** proposer is
`[ACCT-GATED]` (rides S94). **Win-proof:** the mock loop runs green on Windows.

### S100 — proof #1: capability-bounded data pipeline `[MAC-PROVE]` — dep S99
Real Garnet pipeline built + run + smoke-tested **by the S99 loop**, authority
enforced + sealed. Reporter gate; **Win-proof** on Windows.

### S101 — proof #2: MCP-tool-governed orchestrator `[MAC-PROVE]` — dep S99, S67
Per-tool capability **budgets** enforced (extends S67 `mcp-caps`); over-budget tool
use traps. **Honest scope:** Garnet is not an MCP host — it governs the *declared*
tool surface + the Garnet-side budget, not the live MCP transport (named).

### S102 — proof #3: regulated/auditable transformation `[MAC-PROVE]` — dep S99, S98
A transformation with **full provenance** (sealed predicate + transparency-log
entry) end-to-end through the S99 loop. Reporter gate; **Win-proof** on Windows.

### S103 — agent-run bounded simulation / digital-twin `[MAC-PROVE]` — dep S99, S89, S93
Agents run **bounded** experiments — each capped by `@max_depth` (S89) + static
loop bounds (S93), capability-budgeted + sealed. Deterministic trap on over-budget.
**Win-proof** on Windows.

### S104 — AI-PR-review-collapse measured on a REAL repo `[MAC-PROVE]` — dep S99
Measure review-time collapse with `diff-caps` as the **acceptance gate** on real
PRs (not the S49 demo). **Honest scope:** a measured study on a chosen real repo,
methodology disclosed; **no invented collapse number.** **Win-proof** on Windows.

**Stage E — positioning, validation, the v0.8.1 cut**

### S105 — "ultrapunch" dossier + ranked trophies `[MAC-PROVE]` — dep S104
Prove the #1 capability — **capability-bounded acceptance of agent-authored code,
enforced** — + ranked runners-up, **evidenced on the S100–S104 systems**. Honest
concession carried verbatim: every pillar is precedented (Austral/E/Koka/Wasmtime/
in-toto); **the integration + diff-gating discipline is the novelty.** No invented
measurement.

### S106 — 5–10 use-case domains as proof artifacts `[MAC-PROVE]` — dep S105
Domains rendered as **proof artifacts** (built/run/sealed), not marketing copy.

### S107 — academic-review-grade evidence package `[MAC-PROVE]` — dep S105, S106
The CMU/MIT/Rice/UC-Berkeley package — **every claim sourced** to a slice, test, or
sealed artifact; the honest dossier (including "what we refuse to claim").

### S108 — cross-platform + reproducibility hardening `[WIN-PROVE]` + `[LINUX-INFRA]` — dep S82, S86
Sustain Windows parity (S81–S88 stay green) + **byte-reproducible seals across
machines** (Mac↔Windows↔Linux). **Proof:** the same commit seals to the same
`source_hash`/`ast_hash` on each platform.

### S109 — v0.8.1 release-readiness gate `[MAC-PROVE]` — dep all above
Whole-runway aggregator (like S60/S80) that is **binary-strict by default** (the
S86 lesson — READY must carry real binary results, never `--no-run`). Reports each
S91–S108 slice's status + every honest-pending/`pending-infra` item explicitly.

### S110 — v0.8.1 cut decision + 1.0 feasibility note `[MAC-PROVE]` — dep S109
Ship the cut-readiness **verdict**; **escalate the tag to Jon — never autonomous**;
record the honest 1.0 horizon (~a year, validation-gated, never slice-count-gated).
**1.0 is held past S110 until the validation stages land.**

---

## Dependency order — and why

1. **S81 first (blast radius):** one `.GARNET` fix clears four trust findings and
   is Mac-unit-testable now.
2. **S82 next (cross-platform pillar):** the reproducible seal is broken on Windows
   until eol is pinned — everything provenance-related rides on it.
3. **S83 (cheap, removes split-truth):** pure-docs reconciliation before review.
4. **S84 → S85 → S86:** the two Windows binary fixes must land before the
   binary-strict S80 mode (S86) can be honest; S86 *depends* on both.
5. **S87 → S88:** reporter/tooling hardening after the trust + binary fixes.
6. **S89–S90:** first new surface (runtime-enforcement seeds) only after the
   burn-down — no broad new surface on top of unproven Windows trust.
7. **S91+ (re-sliced, full resolution):** **Stage A** close the S90 `@caps` gaps
   the interpreter can see on Mac (S91 net+entry-frame → S92 spawn/FFI laundering,
   with the OS-boundary application quarantined behind `[LINUX-INFRA]`) → **Stage B**
   first formal + measurement increments (S93 static bounded-loop, S94/S95 Paper VI,
   S96 effect-typed seed) → **Stage C** provenance/standard (S97 bind+verify seal,
   S98 manifest-standard seed) → **Stage D** the agent loop (S99) and the real-world
   proofs it builds+runs+seals (S100–S104) → **Stage E** positioning/validation/cut
   (S105 dossier → S106 domains → S107 academic → S108 repro → S109 binary-strict
   readiness gate → S110 escalate-the-tag). Each strategic slice depends on the
   *actual* enforcement it builds on, never the planned one.

---

## Audit Findings Burn-Down — ALL 14 CLOSED (2026-06-01)

| Finding | Sev | Closed by | Status |
|---|---|---|---|
| WIN-S33-001 (.GARNET verify) | high | **S81** #299/#303 | ✅ closed, Windows-proven |
| WIN-S36-001 (.GARNET caps manifest) | high | **S81** #299/#303 | ✅ closed, Windows-proven |
| WIN-S37-001 (.GARNET diff-caps) | high | **S81** #299/#303 | ✅ closed, Windows-proven |
| WIN-S46-001 (.GARNET sandbox policy) | high | **S81** #299/#303 | ✅ closed, Windows-proven |
| WIN-S38-001 (seal LF/CRLF source hash) | high | **S82** #301/#305 | ✅ closed, Windows-proven |
| WIN-S80-002 (post-tag split truth) | medium | **S83** #311 | ✅ closed (docs) |
| WIN-S71-001 (Exp 3 WSL abs paths) | high | **S84** #300 | ✅ closed, Windows-proven |
| WIN-S73-001 (interp stack-overflow parity) | high | **S85** #302/#306 | ✅ closed, Windows-proven |
| WIN-S80-001 (S80 `--no-run` hides failures) | high | **S86** #313 | ✅ closed |
| WIN-S6-001 (cp1252 stdout) | medium | **S87** #309 | ✅ closed |
| WIN-S31-001 (denied temp fixtures) | high | **S87** #309 | ✅ closed |
| WIN-S31-002 (committed-only surface) | advisory | **S87** #309 | ✅ closed |
| WIN-S38-002 (cosign/SBOM absent) | medium | **S88** #312 | ✅ closed *honest-partial* — tools reported absent, never faked; **real signing/SBOM stays `pending-infra`** |
| WIN-S39-001 (no Wasmtime fuel/epoch proof) | pending-infra | **S88** #312 / **S89** #304 | ✅ closed *honest-partial* — S89 enforces `@max_depth` in-engine; **Wasmtime `@bounded` fuel stays `pending-infra`** (inherited by S92/S93) |
| WIN-S70-001 | resolved | (pre-existing) | ✅ recorded in P0 doc |

**Forward ledger (v0.8.1 runway, S91–S110):** 0/20 implemented — re-sliced at full
resolution (above), held for Jon's execution choice. Mac-provable: S91, S93, S95,
S96, S97, S98, S99–S107, S109, S110. Infra/account-gated (honest-pending until the
boundary/tool/provider lands): S92 (`[LINUX-INFRA]` seccomp), S94 (`[ACCT-GATED]`
provider), S108 (`[LINUX-INFRA]` repro). The Wasmtime-fuel `@bounded` runtime stays
deferred across the whole runway (named, never faked).

---

## What We Refuse To Claim

- **No production or 1.0 claim.** v0.8.0 is a research-grade-prototype milestone;
  v0.8.1 is the next research-grade milestone. 1.0 stays held (~a year,
  validation-gated).
- **No "enforced" without a trap.** Every runtime-enforcement slice (S89+) must
  *deterministically trap*, proven by test, or be labelled generated/declared —
  never "enforced" falsely. The kernel is honestly *declared/checked* today; S89+
  makes increments real, one ceiling/capability at a time.
- **No faked cross-platform proof.** Windows-tagged slices are not "done" until
  their named Windows proof command passes **on Windows**; Mac authoring is not
  Windows verification.
- **No signed/SBOM/fuel stamp without the tool.** S88 reports tools-absent
  honestly; absence is never "verified."
- **No standard "adopted."** RFC-0001 / S98 remain *intent + reference impl*; no
  OWASP/LF body has adopted anything.
- **No measurement invented.** Paper VI h₃a stays the recorded 6.5% partial until a
  real re-run (S95, provider-free); Exp 1 stays `[ACCT-GATED]` until a real provider
  runs (S94, escalated to Jon).
- **READY ≠ proven.** After S86, aggregate READY must carry binary-strict results;
  a green aggregate that hid direct binary failures is exactly the bug we closed.
- **`@caps` enforcement is interpreter-scoped (S90/S91).** It traps undeclared
  env/proc/fs/net use *inside the tree-walk interpreter*. It is **not** an OS
  sandbox: the VM backend does not enforce it, and containing a spawned child needs
  the S46 seccomp/WASI policy *applied* — which is `[LINUX-INFRA]` (S92), declared-
  only on any host without seccomp/Wasmtime. We never call generated policy
  "enforced."
- **No real-world proof on a mock agent.** S99–S104 run a scripted proposer until a
  real provider lands (S94); a mock-driven loop is labelled mock, never "an agent
  built this," and S104's review-collapse carries its disclosed methodology — no
  invented collapse number.
- **The tag is Jon's.** S110 escalates the v0.8.1 cut decision; no tag is ever
  pushed autonomously.

---

## Verification (per slice + handoff)

- **Per slice (Mac-authorable):** `cargo test --workspace --no-fail-fast` 0 failed;
  `cargo fmt --all -- --check` / `clippy -D warnings` / `git diff --check` clean;
  the new `--gate` rc 0; focused tests proportional to code substance; sealed
  dogfood bundle; ledger advance; PR-body gate; CI green; merge as IslandDevCrew.
- **Windows handoff:** each `WINDOWS-*`-tagged slice carries its named Windows proof
  command; the slice is only marked Windows-complete when that command passes on
  the Windows machine and the result is recorded back into
  `WINDOWS_AUDIT_S1_S80.md` (the committed burn-down ledger).
- **Re-init the goal ledger** for the v0.8.1 goal (`.dogfood/goal.json`) — this is
  **P0-v0.8.1**, the first execution step once Jon picks execution.

## This plan's own honest scope

S81–S90 are **merged and audit-anchored** (all 14 findings closed, Windows lane
reported). **S91–S110 are now re-sliced at full PR-sized resolution** (2026-06-01),
grounded in the *as-merged* S89/S90 enforcement — not the original plan. The
load-bearing commitments: **(1) close the enforcement gaps the interpreter can see
on Mac first (S91/S92a), with real trapping proof; (2) quarantine OS-boundary,
provider, and signing work behind honest infra/account-gate tags — never claim
enforced/signed/fuel-proven without the boundary or tool present; (3) the agent
loop (S99) and its real-world proofs (S100–S104) run a labelled mock proposer until
a real provider is authorized; (4) every Windows-tagged slice proven on Windows;
(5) the v0.8.1 tag is escalated to Jon, never autonomous.** S91–S96 are the
highest-confidence increments; S99–S107 depend on the loop landing and stay
re-sliceable as the early stages report.
