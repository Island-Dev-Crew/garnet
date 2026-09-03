# Landing Arc 4 register sweep — 2026-09-03 (hardening and crown arc)

Records lane (L2), swept at `origin/main` = `beeb5e7b23e892521da439da67d44a37f23b5584`
(`beeb5e7`, "policy(deny): remove the inert arrayref@0.3.9 yank exception (DP7 precondition) (#540)").
Every entry below was raised inside the 2026-09-02/03 hardening probes (H1, H2, H3) and the crown
ceremony (scopes B, C, D), and carries the command that reproduces it.

**Review family is stated per entry and it is load-bearing.** The crown's cross-family Codex pass
died on scopes B and C twice, so the runtime-trap, checker-laundering and seal/shelf findings below
(U-91 … U-95, U-106 … U-114) are **same-family (Claude)** standing reviews, not independent ones.
Nothing in that set may be cited as independently reviewed, and every cure to it still needs a Codex
record before the trust-kernel gates treat it as closed. The Python-gate findings (U-96 … U-105)
are **cross-family (Codex)**. U-116 was raised on both sides independently.

- Sweep seat: Claude Opus 5, records lane, macOS, worktree detached from `origin/main`.
- Sweep date: 2026-09-03.
- Authority: the directing seat's standing delegation for this arc. This file is written and
  committed but **not pushed**; the directing seat pins the SHA and lands it after the merge wave.

## Collision sweep

- swept-at: 2026-09-03, after `git fetch --prune` on both remotes.
- source: 476 advertised refs — every `origin` branch head (11, including `origin/main` at
  `beeb5e7b2`) and every `fork` branch head except `fork/main` per the boot fence (465) —
  deduplicated to **475 unique trees**. Zero `refs/pull/*`. No hand-listing.
- pattern: `git grep -I -hoE 'U-[0-9]+([^0-9]|$)' <tree>`, then `grep -oE 'U-[0-9]+' | sort -u`.
  `\b` is not a git-grep token; the `([^0-9]|$)` anchor is what keeps `U-91` from matching `U-910`.
- result **before**: the distinct token set runs `U-1`, `U-04` … `U-90` — **census 73**, the `U-1`
  prose token included (it is a quoted, corrected sentence in the #545 front-door record, not an
  allocation). **No occurrence at or above U-91 exists in any swept tree.**
- commit-message sweep over the same 476 refs (`git log --format='%H %s%n%b'`) tops out at U-90 as
  well; the directing seat's U-91 and U-92 commit-message uses are not yet on an advertised head,
  which is why those two ids are **fixed by that prior use** rather than allocated freshly here.
- result **after** this file: `U-1`, `U-04` … `U-116` — **census 99** (73 + 26 allocations).
- Recompute at this record's tip:
  `git grep -I -hoE 'U-[0-9]+([^0-9]|$)' <tip> -- '*.md' '*.json' '*.txt' '*.py' | grep -oE 'U-[0-9]+' | sort -u | wc -l`

## U-91 — Capability laundering through function values: the checker builds no edge, and the runtime is satisfied by any active frame

- Substance: `garnet check` builds callee edges only from `Expr::Call` whose callee is an
  `Ident`/`Path` naming a declared user fn or a registry primitive. A `def` reached through a
  *value* produces no edge at all, so the caller's transitive capability set stays empty and
  coverage passes. **Nine independent shapes** reproduce it (H1 §3): alias-to-`def`, closure,
  higher-order parameter, string interpolation, actor handler, top-level `let` initializer,
  `method_missing`, `@dynamic` `def_method`, map-of-functions — the `Expr::Closure` and `Expr::Str`
  arms of `walk_expr_for_callees` are no-ops, and `Item::Actor` / `Item::Let` / `Item::Const` fall
  into the `_ => {}` arm of the collectors. At run time `call_fn` pushes a caps frame built from the
  **callee's own** annotations and `require_capability` is satisfied if **any** active frame declared
  the capability — a multiset union, not the entry budget. Each layer assumes the other covers it.
  The L2 case verbatim, recomputed by this seat at the swept head:

  ```garnet
  @caps(fs)
  def leak() {
    write_file("leak.txt", "L2 closure")
  }

  @caps()
  def main() {
    let g = || leak()
    g()
  }
  ```

  `check` → `2 functions checked, 2 boundary call sites, 0 diagnostics`, exit 0;
  `run --interp` → `=> nil`, exit 0; `run --vm` → `=> nil`, exit 0; `leak.txt` written both times.
  An entry point declaring `@caps()` therefore exercises `fs` authority it never declared, and
  `L11` does the same for `env` (`std::env::get("HOME")` returns the host value).
  **Authority-widening: yes.** The enforcement scope's universal sentence — "If any function reaches
  a capability-bearing primitive without the calling chain declaring that capability, `garnet check`
  rejects the program" — is falsified as written.
- What still holds, and it is not small: (1) every one of the **ten** shapes where *nobody* declares
  the capability is refused — nine trapped at run on both backends, one (`c2h`, inline module)
  rejected at check — so the S90/S114 runtime backstop is not bypassable by indirection alone;
  (2) the **three** `Guard::GateEntry` primitives (`std::process::spawn`, `spawn_args`, `output`)
  resist every laundering shape tested, because `require_entry_capability` checks the program entry's
  budget rather than the frame union — `L10` and `DOC3` trap with "requires program entry
  `@caps(proc)`, not declared by the entry point"; (3) the **manifest layer is not fooled** — on the
  laundered program `garnet caps` aggregates `["fs"]`, `garnet sandbox` emits `caps: fs`, and
  `garnet diff-caps baseline laundered` exits 1 with `AUTHORITY EXPANDED — review required
  (capability band 2/5)`. The H1 seat probed for a shape that exercises authority while the manifest
  misses it and found none. The cure that already exists in-tree is extending the S92 entry-frame
  check from 3 primitives to the full 12-row `Guard::Gate` surface.
- Raised by: Claude Opus 5, H1 hardening seat (`/tmp/garnet-hardening/H1-checker-claude.md`,
  ~50 executed cases by that report's own count) — **same-family**. Confirmed by: Claude Opus 5,
  records seat, by first-hand recomputation of the L2 case at `beeb5e7b2` — **also same-family.**
  **No cross-family record exists for this finding. A cure requires a Codex record.**
- Command: `printf '@caps(fs)\ndef leak() {\n  write_file("leak.txt", "L2 closure")\n}\n\n@caps()\ndef main() {\n  let g = || leak()\n  g()\n}\n' > p.garnet && garnet check p.garnet; garnet run --interp p.garnet; garnet run --vm p.garnet; ls leak.txt`
  — mechanism at `sed -n '473,481p' garnet-check-v0.3/src/caps_graph.rs` (the no-op `Expr::Closure` /
  `Expr::Str` arms) and `sed -n '777,795p' garnet-interp-v0.3/src/eval.rs` (`has(needed)` over the
  frame union).
- Route: L8 product (extend the entry-frame check and/or teach `caps_graph` to see function values),
  plus L5 public truth for the scope document's universal sentence. Trust-kernel paths on both
  crates; human-merge-only. Status: **open**. The **public-truth half is open as PR #553** (branch
  `mission/capability-claim-truth`), which bounds the claim on the enforcement scope document,
  `docs/why.html`, the live front door, `README.md`, `CURRENT_STATE.md` and `CLAUDE.md`, and re-pins
  the scope reporter's enforced-claim hash in the same commit so the copy and its gate move together.
  The **enforcement cure is separate and still in progress**; this id stays open until that lands,
  because bounding a claim is not closing a gap. Disposition: open — and the universal sentence must
  not be republished in its old form in the meantime.

## U-92 — VM/interpreter `@caps` trap parity is falsified as published

- Substance: `garnet-vm` installs only the **program-entry** caps frame
  (`garnet-vm/src/vm.rs:270`, `enter_entry_caps_frame`) and pushes no per-callee `CapsGuard` on its
  native frames, so a natively-lowered function's own `@caps(...)` is invisible to
  `require_capability`. The VM therefore **traps** helper-declared cases the interpreter **allows**.
  Recomputed at the swept head: a `@caps()` entry calling a `@caps(fs)` helper that reads a file
  gives `run --interp` → `=> hello-garnet`, exit 0, while `run --vm` → `vm error: runtime error:
  capability: 'fs::read_file' requires @caps(fs), not declared in the calling chain`, exit 1.
- **Fail-closed, and it matters which way:** the VM's active frame set is a strict subset of the
  interpreter's, so the VM can only ever be *more* restrictive. There is no input for which `--vm`
  grants authority `--interp` denies — **no authority widening**.
- **AMENDED 2026-09-03, same day, before this entry was cited anywhere:** this entry originally
  added that the divergent program "is one `garnet check` already rejects … reachable only through
  `garnet run`, which does not run the checker." **That clause is falsified** and is withdrawn. The
  cross-family review of PR #553 produced a counterexample the checker accepts, and the implementing
  seat reproduced it independently: `@caps(fs) def alpha(n)` reading via bare `read_file` and
  conditionally calling an unannotated `beta`, which calls `alpha`, with `@caps() def main()`
  calling `beta(0)` — `garnet check` reports `3 functions checked, 4 boundary call sites, 0
  diagnostics`, exit 0; `run --interp` returns the file, exit 0; `run --vm` traps, exit 1. The
  divergence is bounded by **native lowering**, not by the call shape and not by the checker's
  verdict: the same program written with the qualified `fs::read_file` lowers as `1 native / 1
  fallback` and keeps parity on both backends, while the bare form lowers `2 native / 0 fallback`
  and diverges. The correction to the public surfaces is PR #553; this record is amended so the
  canonical register does not carry a claim its own arc disproved.
- Why it is still a finding: the enforcement scope's "May say (true)" list asserts that "`@caps` and
  `@max_depth` trap identically on both backends for the gated surface"
  (`C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md:88`), `docs/why.html:551` carries
  VM/interpreter scope parity as one of only two published `enforced:` claims, and the CLI's own
  `--version` banner repeats "`@max_depth` + `@caps` trap-parity with interp, S99–S101". All three
  are broader than the code. The in-tree parity tests do not cover the breaking case: every VM parity
  case in `garnet-cli/tests/caps_enforcement.rs` declares capabilities on the **entry point**, which
  is exactly the surface where parity does hold. The gap between claim and coverage is the finding.
  (`@max_depth` parity is genuinely two implementations and byte-identical in diagnostics — crown
  scope B §4.4 — so this is a `@caps`-only break.)
- Raised by: Claude Opus 5, crown scope B (`/tmp/garnet-crown/B-runtime-claude.md`, finding B-1) —
  **same-family**; recorded as the standing review because the cross-family Codex pass died on this
  scope twice. Confirmed by: Claude Opus 5, records seat, first-hand recomputation — **same-family.**
  **A cure needs a Codex record.**
- Command: `printf 'hello-garnet' > data.txt && printf '@caps(fs)\ndef helper() {\n  read_file("data.txt")\n}\n\n@caps()\ndef main() {\n  helper()\n}\n' > p.garnet && garnet check p.garnet; garnet run --interp p.garnet; garnet run --vm p.garnet`
  — claim surfaces at `sed -n '86,90p' C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md`
  and `sed -n '551p' docs/why.html`.
- Route: L8 product (push a `CapsGuard` per native VM frame) **or** L5 public truth (narrow the scope
  document and `why.html` to the entry-declared surface and pin the divergence with a test). Either
  closes it; the second is cheaper and the first is the real parity. Both touch trust-kernel paths.
  Status: **open** — the claim-narrowing half rides **PR #553** with U-91 (scope document,
  `docs/why.html`, the live front door, `README.md`, `CURRENT_STATE.md`, `CLAUDE.md`, plus the scope
  reporter's enforced-claim hash re-pinned in the same commit). The parity cure in the VM, and the
  test that would pin the helper-declared case, are not in it. Disposition: open until one of the two
  directions actually lands.

## U-93 — `garnet diff-caps` silently skips any `vendor/` directory, so a declared capability expansion inside one passes as band 5/5

- Substance: the shared directory walker skips any directory named
  `target | .git | node_modules | vendor | .garnet-cache` at **any depth**
  (`garnet-cli/src/cmd/verify_gate.rs:210-215`, reached via `cap_manifest.rs:133` →
  `cmd/diff_caps.rs:71`). A `.garnet` file under a bare `vendor/` declaring `@caps(net, fs)` is
  therefore invisible to the authority gate, and the machine verdict reports
  `"verdict":"no-authority-expansion"`, `"capability_band":"5/5"`, exit 0, **with no disclosure that
  any path was omitted**. The identical file placed outside `vendor/` correctly reports
  `"verdict":"authority-expanded"`, `"aggregate_gained":["fs","net"]`, band 2/5, exit 1. Both
  recomputed by this seat at the swept head. The predicate's own `scope` string does not cover the
  gap: it disclaims *undeclared* authority, whereas here the authority is **declared** and simply
  never read. The documented vendored-dependency path is `.garnet/vendor/<name>`
  (`garnet-cli/AGENTS.md:44`), so the skip is strictly broader than the documented one.
  **Authority-widening: yes** — this is exactly the class CLAUDE.md integrity rule 2 names.
- Raised by: Claude Opus 5, crown scope C (`/tmp/garnet-crown/C-seal-claude.md`, blocking B-1) —
  **same-family**; the Codex pass died on this scope. Confirmed by: Claude Opus 5, records seat,
  first-hand recomputation — **same-family. A cure needs a Codex record.**
- Command: `mkdir -p old new/vendor new2 && printf '@caps()\ndef main(v) { v * 2 }\n' > old/tool.garnet && cp old/tool.garnet new/tool.garnet && printf '@caps(net, fs)\ndef exfil() { 1 }\n' > new/vendor/evil.garnet && garnet diff-caps --machine old new; echo "exit=$?"; cp old/tool.garnet new2/tool.garnet && cp new/vendor/evil.garnet new2/evil.garnet && garnet diff-caps --machine old new2; echo "exit=$?"`
- Blast radius, wider than `diff-caps` alone: the same walker feeds `garnet caps`, and `caps` is the
  manifest that `garnet seal` embeds. Recomputed pre-cure at the swept head, `garnet caps` on a tree
  carrying a bare `vendor/evil.garnet` returns
  `{"schema":"garnet-capability-manifest-v1","aggregate":[],"functions":[{"name":"main","caps":[]}],"wildcard":false}`
  — `exfil` and its declared `fs`/`net` are simply absent. The #555 cure measures the same tree at
  `"aggregate":["fs","net"]` after the fix. **So a sealed package could have carried a manifest
  understating its own declared authority**, which is a larger claim than a gate mis-verdict.
  `agent-loop` inherits the disclosure but never had the defect: its stage-1 `check` rejects a
  directory proposal before the walk matters.
- Route: L8 product (`garnet-cli`). Status: **cure open as PR #555**, branch
  `mission/diff-caps-vendor-blindspot` — it narrows the `vendor` skip to exactly `.garnet/vendor`
  **relative to the scan root** (so a nested `vendor/` no longer hides anything), removes the
  `node_modules` skip entirely, and adds `skipped_path_count` / `skipped_paths` to the machine
  verdict, carrying **rule names and counts, never paths**, so the omission becomes visible to an
  agent reviewer without turning the verdict into a directory listing. No pre-existing test covered
  the defect. Disposition: open until #555 merges — and, with U-91, one of the two
  authority-widening findings of this arc.

## U-94 — `garnet verify` compares 3 of 8 manifest fields while its module doc says "any mismatch"

- Substance: `garnet-cli/src/cmd/verify.rs:42,49,56` compares only `source_hash`, `ast_hash` and
  `schema`. `prelude_hash`, `target_triple`, `parser_version`, `interp_version` and
  `deterministic_flags` are parsed (`manifest.rs:215`) and then never compared, while
  `garnet-cli/src/manifest.rs:1-7` states that `garnet verify` "re-derives the manifest from the
  source and exits non-zero **on any mismatch**." On an **unsigned** manifest — the default, and the
  shape shipped in the flagship fixture — a tampered `prelude_hash`, `target_triple`,
  `parser_version` or `deterministic_flags` each print `OK tool.garnet matches manifest (unsigned)`
  and exit 0; only a tampered `schema` fails. All five verdicts recomputed by this seat.
  **Authority-widening: no** (the capability surface is unchanged); it weakens the provenance gate.
- Signing closes it: `canonical_signing_payload` (`manifest.rs:94`) covers all eight fields, and the
  crown scope C report records signed + tampered `prelude_hash` and signed + tampered `target_triple`
  both failing with a signature-verification error, exit 2. The minimum shelf is also unaffected —
  `minimum_shelf::verify_seal` (`minimum_shelf.rs:186`) compares the whole canonical build manifest
  as a JSON value, which is the stricter behavior `verify` should have. The exposure is precisely
  the unsigned path the CLI advertises as passing.
- Raised by: Claude Opus 5, crown scope C (blocking B-2) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, first-hand recomputation of the five-manifest tamper set —
  **same-family. A cure needs a Codex record.**
- Command: `printf '@caps()\ndef main(value) { value * 2 }\n' > tool.garnet && garnet build --deterministic tool.garnet && M=tool.garnet.manifest.json && sed 's/unknown-target/x86_64-evil/' $M > m_target.json && sed 's/"parser_version": "0.8.1"/"parser_version": "9.9.9"/' $M > m_ver.json && for f in $M m_target.json m_ver.json; do garnet verify tool.garnet $f; echo "exit=$?"; done`
- Route: L8 product (`garnet-cli`). Status: **cure open as PR #554**, branch
  `mission/manifest-verify-strictness` — every field is compared with a **field-specific** message
  rather than a generic mismatch. No pre-existing test covered the defect. Disposition: open until
  #554 merges.

## U-95 — A signed manifest's signature covers a re-serialization, not the file bytes, so an injected unknown key survives "signature valid"

- Substance: the line-oriented manifest parser silently drops unrecognized keys
  (`garnet-cli/src/manifest.rs:255`, the `_ => {}` arm), and `canonical_signing_payload`
  (`manifest.rs:94`) re-serializes from the **parsed struct**, never from the on-disk bytes. The
  signed artifact is therefore the canonical re-serialization of eight recognized fields; any other
  content in the file sits outside the signature. Recomputed at the swept head: injecting
  `"note": "TRUSTED BY VENDOR"` as the first key of a signed manifest still yields
  `OK tool.garnet matches manifest + signature valid`, exit 0, with the same `signed_by` digest as
  the untouched control. A consumer that reads the manifest with a real JSON parser sees
  attacker-controlled fields inside a document the CLI just blessed.
  **Authority-widening: no** — it is a signature-scope confusion.
- Raised by: Claude Opus 5, crown scope C (blocking B-3) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, first-hand recomputation — **same-family. A cure needs a Codex
  record.**
- Command: `garnet keygen k.key && garnet build --deterministic --sign k.key tool.garnet && python3 -c "s=open('tool.garnet.manifest.json').read(); s=s.replace('{\n','{\n  \"note\": \"TRUSTED BY VENDOR\",\n',1); open('m_signed_extra.json','w').write(s)" && garnet verify tool.garnet m_signed_extra.json --signature; echo "exit=$?"`
- Route: L8 product (`garnet-cli`). Status: **cure open as PR #554** with U-94 — unrecognized **and
  duplicate** keys are rejected, and a parse → re-serialize → byte-equality round-trip closes the
  gap between the signed reconstruction and the file, so unmodelled content cannot survive at all.
  Blast radius was measured before landing it: **37 committed manifest-shaped artifacts** re-checked
  under the stricter rule, **zero rejected** — the tightening breaks nothing already in the tree.
  Disposition: open until #554 merges.

## U-96 — The dogfood PR-body checker ended a section only at the next `### ` heading, and accepted any non-empty checked item as evidence

- Substance: two defects in one predicate. (1) `has_checked_evidence`
  (`scripts/check_dogfood_pr_body.py:107`) searched only for the next literal `### ` heading, so a
  higher-level `## ` — which closes the section in Markdown — did not end it, and a checked item
  under a later unrelated `## ` satisfied the evidence contract. (2) A required heading matched by
  **prefix**, and **any** non-empty checked payload counted as evidence, so a body whose headings all
  carry suffix prose and whose Local / Remote / Evidence sections read only `- [x] x` passed with no
  command, result, artifact, URL or meaningful assertion. Both recomputed by this seat against
  `beeb5e7b2`: the boundary body (`/tmp/garnet-hardening/D1-outside-section-body.md`) and the vacuous
  body (`/tmp/garnet-hardening/H3-vacuous-body.md`) each returned
  `dogfood-pr-body: ok (1 changed files checked)`, exit 0.
- Raised by: **Codex** — crown scope D, blocking finding 1 (section boundary), and the H3 hardening
  seat, H3-01 (vacuous evidence). **Cross-family, both.** Confirmed by: Claude Opus 5, records seat,
  first-hand recomputation of both bodies at the swept head.
- Command: `python3 -I scripts/check_dogfood_pr_body.py --body-file /tmp/garnet-hardening/D1-outside-section-body.md --changed-file .github/workflows/ci.yml; echo "exit=$?"` and the same with
  `/tmp/garnet-hardening/H3-vacuous-body.md`.
- Route: L4 gate hardening. Status: **cure open as PR #551**, branch
  `mission/dogfood-body-checker-hardening`, two commits. `250c9748` closes the section boundary
  (a section now ends at the next heading of the same or higher level), requires exact heading
  equality, and requires evidence tokens in checked items. Codex reviewed that commit
  **cross-family** and returned **REJECT**: a whitespace-only code span plus the negated claims
  `No CI run.` and `No report was recorded.` still passed
  (`/tmp/garnet-hardening/widened-vacuous-pass.md`, which this seat also reproduced green at
  `beeb5e7b2`). `2df9c5c2` cures that bypass with three rules — a code span counts only when its
  stripped content is non-empty, at least two characters, and carries a letter or digit; the two
  widened alternatives must be positive and specific; negation is judged **per clause**, not per
  item, because all seven merged bodies (#540–#546) state the remote line as a positive claim
  followed by "no CI conclusion is claimed in advance" and a per-item guard would have rejected every
  one of them. Disposition: open until #551 merges.

## U-97 — The WV acceptance reporter checked, stat'ed and reopened evidence by path, and read it as newline-translating text

- Substance: `scripts/garnet_wv_acceptance_status.py:84` separately checked, stat'ed and then
  **reopened** the evidence manifest by path, and artifact handling repeated the pattern at line 348.
  Two consequences, both reproduced: a **CRLF** manifest reached `accepted`
  (`wv_crlf_manifest: cr=75 state=accepted ok=True`), and a deterministic **check/use swap**
  redirected the read (`wv_check_use_swap: accepted_source=outside`). The future WV succession
  contract is explicitly inactive, so the CRLF half is a Standards finding rather than a claim that
  the future schema is live.
- Raised by: **Codex**, crown scope D, blocking finding 2 — **cross-family**. Confirmed by:
  Claude Opus 5, records seat, by running Codex's own bounded repro program at the swept head and
  observing both tokens.
- Command: `cd <worktree at beeb5e7b2> && python3 -I /tmp/garnet-crown-repros.py` → the
  `wv_crlf_manifest` and `wv_check_use_swap` lines; surface at
  `sed -n '84,100p' scripts/garnet_wv_acceptance_status.py`.
- Route: L4 gate hardening. Status: **cure open as PR #552**, branch
  `mission/wv-reporter-bytes-and-toctou`, commit `c057b336`. Every evidence file the reporter reads
  — the contract document, the evidence manifest, and each listed artifact — is now lstat'ed, opened
  once (`O_RDONLY|O_CLOEXEC|O_NOFOLLOW` where available, plus a Windows reparse-point check),
  fstat'ed on that descriptor with `(st_dev, st_ino)` bound to the check, read from the same
  descriptor under the size bound, and fstat'ed again after the read; the same bytes feed both the
  SHA-256 and the JSON parse. JSON bytes must be strict UTF-8, BOM-free and LF-only. Artifacts stay
  opaque hashed bytes because native Windows logs may legitimately carry CRLF, so the byte rule
  applies to JSON only, and no canonical-JSON requirement is added because the contract states none.
  Disposition: open until #552 merges.

## U-98 — Ambient Git variables redirect the Base-controlled reporter's repository reads

- Substance: `scripts/garnet_base_controlled_trust_status.py:235` builds the subprocess environment
  by removing credential variables and adding `GIT_NO_REPLACE_OBJECTS` / `GIT_TERMINAL_PROMPT`, but
  **retains** `GIT_DIR`, `GIT_WORK_TREE`, the object-directory variables and the configuration
  variables. Passing `cwd=repo` therefore does not bind the reporter's Git reads to that repository.
  Reproduced: with the reporter pointed at repository A and an ambient `GIT_DIR` set, the run exits 0
  having read repository B (`base_ambient_git_dir: ... redirected=True`; the two repo digests differ
  per run because each reproduction builds fresh temp repositories).
- Raised by: **Codex**, crown scope D, blocking finding 3 — **cross-family**. Confirmed by:
  Claude Opus 5, records seat, by running the bounded repro at the swept head.
- Command: `cd <worktree at beeb5e7b2> && python3 -I /tmp/garnet-crown-repros.py` → the
  `base_ambient_git_dir` line; surface at
  `sed -n '235,245p' scripts/garnet_base_controlled_trust_status.py`.
- Route: L4 gate hardening — an explicit environment allowlist rather than a credential denylist.
  Status: **open**, stacked behind #549/#551/#552. Disposition: open.

## U-99 — The Base-controlled rolling-review adapter can render a credential-bearing exception into public problems

- Substance: `scripts/garnet_base_controlled_trust_status.py:357` passes the token to the transport
  constructor and interpolates the resulting exception **verbatim** into the public problems list.
  Reproduced with a secret marker: `base_adapter_secret_rendered: True problem='Item 2 rolling-review
  adapter dependency failed: constructor included SECRET_DO_NOT_LOG'`.
- Raised by: **Codex**, crown scope D, blocking finding 4 — **cross-family**. Confirmed by:
  Claude Opus 5, records seat, by running the bounded repro at the swept head.
- Command: `cd <worktree at beeb5e7b2> && python3 -I /tmp/garnet-crown-repros.py` → the
  `base_adapter_secret_rendered` line; surface at
  `sed -n '357,375p' scripts/garnet_base_controlled_trust_status.py`.
- Route: L4 gate hardening — redact before rendering, and keep the token out of any string a problem
  can quote. Status: **open**. Disposition: open — this is the one finding in the set whose failure
  mode is disclosure rather than an authority gap, so it should not queue behind the others.

## U-100 — The authenticated-principal projection accepts automation accounts as author, committer and reviewer

- Substance: the gate proves only that author, committer and reviewer ids are positive, well-shaped
  and disjoint. The author/committer projection checks `id` and `login` only
  (`scripts/garnet_trust_kernel_review_status.py:1592-1642`); the review projection likewise ignores
  `user.type` (`:1371-1393`); the common validation only rejects reviewer-id overlap
  (`:1043-1067`). Nothing constrains principal class, organization or team membership, an
  approved-reviewer allowlist, or common control behind two service accounts. Reproduced at the
  swept head with a fake authenticated transport whose author and committer are
  `{id: 999, login: buildbot-service, type: Bot}` and whose distinct reviewer approves the exact
  head: `author_login=buildbot-service author_type=Bot`, `findings=[]`, exit 0. Compounding it, the
  checked-in ruleset declares `required_approving_review_count: 0` and
  `require_code_owner_review: false` (`.github/rulesets/garnet-main.json`, recomputed), so the local
  policy carries no independent review backstop for such a candidate.
- Credential boundary, stated as the raising seat stated it: this is an **executed predicate-level**
  reproduction, not a live GitHub proof that a particular bot or app can submit the review. A live
  account-control or affiliation claim would need authenticated API evidence and was not attempted.
- Raised by: **Codex**, H3 hardening seat, H3-03 — **cross-family**. Confirmed by: Claude Opus 5,
  records seat, by running the reproduction at the swept head.
- Command: `python3 -I /tmp/garnet-hardening/H3-bot-principal-repro.py`
- Route: L4 gate hardening / L1 (a human-principal rule belongs in the succession law, not only in
  the gate). Status: **open**. Disposition: open.

## U-101 — The dogfood gate's implementation and its test are outside the rolling-review trust surface

- Substance: `scripts/check_dogfood_pr_body.py` produces the **required context** "PR dogfood
  evidence" (`.github/workflows/dogfood-readiness.yml:23`, referenced at
  `.github/rulesets/garnet-main.json:87` and in `required-context-producers.json`), but neither it
  nor `scripts/test_check_dogfood_pr_body.py` matches the rolling-review surface: the prefixes are
  `scripts/garnet_` and `scripts/test_garnet_` only, and neither file is in `TRUST_KERNEL_FILES`
  (recomputed at the swept head — both membership tests False). A one-byte change to the gate
  therefore walks through with `trust-kernel touched: False` and `overall: ok`. The required-context
  fingerprint does not close it either: it hashes the workflow YAML and job literal but not the
  transitive files a `run` step invokes (`scripts/garnet_required_context_contract.py:505-537`).
  The omission is **asserted deliberately today** — `test_lookalike_paths_stay_non_sensitive`
  (`scripts/test_check_dogfood_pr_body.py:123-126`) asserts
  `is_sensitive_path("scripts/check_dogfood_pr_body.py")` is False. **Authority-widening: yes.**
- Raised by: **Codex**, H3 hardening seat, H3-02 (blocking) — **cross-family**. Confirmed by:
  Claude Opus 5, records seat, by recomputing the surface membership and the ruleset review counts
  directly from the swept tree.
- Command: `python3 -I -c "import re; s=open('scripts/garnet_trust_kernel_review_status.py').read(); p=re.search(r'TRUST_KERNEL_PREFIXES = \((.*?)\)',s,re.S).group(1); f=re.search(r'TRUST_KERNEL_FILES = \((.*?)\)',s,re.S).group(1); print('check_dogfood in prefixes/files:', 'check_' in p, 'check_dogfood_pr_body' in f)"`
  — the H3 seat's end-to-end form is a one-byte change to the checker in a scratch clone followed by
  `python3 -I scripts/garnet_trust_kernel_review_status.py --format md --base HEAD~1 --head HEAD`,
  which reports `trust-kernel touched: **False**`, exit 0.
- Route: L4 gate hardening — add the checker and its test to the trust-kernel file list, and retire
  the lookalike assertion. Note the ordering trap: the cure moves the gate that the cure's own PR
  merges under, which is human-merge-only under integrity rule 1. Status: **open**.
  Disposition: open. **See U-116** — this entry is the `scripts/` instance of a wider pattern; the
  same enumeration also omits five `garnet-cli/src/**` files that compute and gate the capability
  surface, and the two should be cured together rather than one hole at a time.

## U-102 — The governance gate's failure construction erases completed-work evidence

- Substance: `scripts/garnet_github_governance_gate.py:300` (`_failure`) resets counts, bindings,
  ruleset and completed flags when a late validation fails at `:1135`. Reproduced:
  `governance_late_failure: workflow_count=0 selected_run_count=0 required_check_count=0 bindings=0
  problems=('selected check run must be completed/success',)`. It stays **fail-closed** — the gate
  reds — but everything already established about the run is destroyed in the emission, so a reader
  cannot tell how far the evaluation got.
- Raised by: **Codex**, crown scope D, non-blocking finding 1 — **cross-family**. Confirmed by:
  Claude Opus 5, records seat, by running the bounded repro at the swept head.
- Command: `cd <worktree at beeb5e7b2> && python3 -I /tmp/garnet-crown-repros.py` → the
  `governance_late_failure` line.
- Route: L4 gate hardening — preserve the established facts alongside the problems. Status: **open**.
  Disposition: open.

## U-103 — Distinct transport failures collapse into generic messages in both the governance gate and the rolling reporter

- Substance: `scripts/garnet_github_governance_gate.py:388` discards structured problem codes — an
  HTTP-status fault and a pagination fault both emit `repository transport is incomplete` — and
  `scripts/garnet_trust_kernel_review_status.py:1396` maps **every** thrown exception to
  `transport-failure`, so `TimeoutError` and `RuntimeError` produce the identical line
  `authenticated review enumeration failed closed: transport-failure`. All four outputs reproduced.
  Fail-closed, but undiagnosable.
- Raised by: **Codex**, crown scope D, non-blocking finding 2 — **cross-family**. Confirmed by:
  Claude Opus 5, records seat, by running the bounded repro at the swept head.
- Command: `cd <worktree at beeb5e7b2> && python3 -I /tmp/garnet-crown-repros.py` → the
  `governance_problem_*` and `review_exception_*` lines.
- Route: L4 gate hardening. Status: **open**. Disposition: open — cheap, and it pairs naturally with
  U-102 in one PR.

## U-104 — Checked policy JSON is parsed from decoded text with no byte rule, so CRLF is accepted

- Substance: `scripts/garnet_required_context_contract.py:113` and
  `scripts/garnet_github_governance_gate.py:227` both read through descriptors safely and then pass
  raw **decoded text** to `json.loads`, with no line-ending or BOM rule. Reproduced:
  `required_context_crlf: problems=[] producers=32` and `governance_crlf: problems=[]` — a CRLF
  policy document is accepted on both the required-context and governance paths. The ruleset contract
  does not expressly require canonical source-JSON bytes, so this is Standards-only, not a contract
  violation.
- Raised by: **Codex**, crown scope D, non-blocking finding 3 — **cross-family**. Confirmed by:
  Claude Opus 5, records seat, by running the bounded repro at the swept head.
- Command: `cd <worktree at beeb5e7b2> && python3 -I /tmp/garnet-crown-repros.py` → the
  `required_context_crlf` and `governance_crlf` lines.
- Route: L4 gate hardening — the byte rule #552 adds for WV evidence should be applied here too.
  Status: **open**. Disposition: open.

## U-105 — The dogfood checker's Git subprocess had no timeout

- Substance: `scripts/check_dogfood_pr_body.py:140` invoked `subprocess.check_output` with no
  `timeout` keyword, so a hung `git diff --name-only` would hang the required context indefinitely.
  Reproduced two ways: Codex's `dogfood_git_timeout_kwarg: None`, and this seat's
  `grep -n 'timeout' scripts/check_dogfood_pr_body.py` at the swept head → **no match**, exit 1.
- Raised by: **Codex**, crown scope D, non-blocking finding 4 — **cross-family**. Confirmed by:
  Claude Opus 5, records seat, by direct recomputation on the swept tree.
- Command: `grep -n 'timeout' scripts/check_dogfood_pr_body.py; echo "exit=$?"`
- Route: L4 gate hardening. Status: **cure open as PR #551** — bounded in the same commit
  (`250c9748`) that closes U-96's section boundary. Disposition: open until #551 merges.

## U-106 — The seal predicate embeds environment state: whether `cosign` is on `PATH` changes the seal bytes

- Substance: `garnet-cli/src/cmd/seal.rs:121` calls `cosign_available()`
  (`garnet-cli/src/seal.rs:39`, which execs `cosign version`), and the boolean selects one of two
  `tooling.cosign` strings (`seal.rs:102-105`) that are **inside the predicate**. Two machines at the
  same SHA on the same source therefore emit different seal bytes. Reproduced at the swept head with
  a `PATH` shim (no real cosign on this machine): `cmp` reports the two seals differ at char 1003,
  line 23. The unit test `seal::tests::statement_is_deterministic` (`seal.rs:295`) passes the cosign
  flag explicitly, so it cannot catch this. Practical consequence: a maintainer who has cosign
  installed and legitimately regenerates the flagship seal produces a package the shelf **rejects**,
  because the shelf pins seal bytes — the crown scope C run recorded
  `Minimum Shelf package rejected: seal bytes do not match the trusted flagship`, exit 1. A cosign
  that exits non-zero is treated as absent, byte-identical to the no-cosign seal.
- Raised by: Claude Opus 5, crown scope C (non-blocking N-1) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, first-hand recomputation of the shim divergence — **same-family.
  A cure needs a Codex record.**
- Command: `mkdir -p fakebin && printf '#!/bin/sh\necho "cosign v99"\nexit 0\n' > fakebin/cosign && chmod +x fakebin/cosign && garnet seal tool.garnet --out seal_nocosign.json && PATH="$PWD/fakebin:$PATH" garnet seal tool.garnet --out seal_fakecosign.json && cmp seal_nocosign.json seal_fakecosign.json`
- Route: L8 product (`garnet-cli`) — take the signer's availability out of the predicate, or make the
  seal record it as data rather than as prose that varies. Status: **open**. Disposition: open — and
  worth pairing with U-107, which is the same field read from the other side.

## U-107 — The seal predicate carries no positive signedness field, and the cosign-available branch omits the word UNSIGNED

- Substance: no consumer can be fooled into thinking a signature is present, because **no signature
  ever is** — the predicate's root keys are `_type`, `predicate`, `predicateType`, `subject` in both
  branches and there is no `signature` / `signatures` / `sig` field either way. But the
  `tooling.cosign` note is the **only** signal, and in the cosign-available branch it reads
  "available — sign with: cosign attest …" with no `UNSIGNED` token. The minimum shelf detects
  unsigned status by requiring exactly that token (`garnet-cli/src/minimum_shelf.rs:194-201`), so a
  consumer following the shelf's own rule finds nothing in that branch and could infer signedness
  from silence. Recomputed at the swept head: `grep -c UNSIGNED` gives **1** on the no-cosign seal and
  **0** on the cosign-available seal. The predicate has no schema field stating signature status.
- Raised by: Claude Opus 5, crown scope C (non-blocking N-2) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, first-hand recomputation — **same-family. A cure needs a Codex
  record.**
- Command: `garnet seal tool.garnet --out seal_nocosign.json && PATH="$PWD/fakebin:$PATH" garnet seal tool.garnet --out seal_fakecosign.json && grep -c UNSIGNED seal_nocosign.json seal_fakecosign.json`
- Route: L8 product (`garnet-cli`) — an explicit `"signed": false` or `signature_status` field, so the
  signal is positive rather than inferred from prose. Status: **open**. Disposition: open.

## U-108 — `target_triple` is a hardcoded fallback whose comment describes a build script that does not exist

- Substance: `garnet-cli/src/manifest.rs:399-405` reads
  `option_env!("TARGET").unwrap_or("unknown-target")` under the comment "baked in via the
  `RUSTC_TARGET` env we set up during build". There is **no** `garnet-cli/build.rs` (recomputed:
  `ls garnet-cli/build.rs` → No such file or directory) and no `cargo:rustc-env=TARGET` anywhere in
  the workspace, so the field can never be anything but the literal. Recomputed: a fresh
  `build --deterministic` manifest carries `"target_triple": "unknown-target"`, and so does the
  checked-in flagship fixture `examples/minimum-shelf-flagship/tool.seal.json`. Substantively the
  manifest hashes source and AST only, so platform-independence is arguably correct — the defect is
  the false comment plus a vestigial field that reads like a platform binding it is not, undocumented
  anywhere in `docs/` or `README.md`.
- Raised by: Claude Opus 5, crown scope C (non-blocking N-3) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, first-hand recomputation — **same-family. A cure needs a Codex
  record.**
- Command: `ls garnet-cli/build.rs; grep -rn 'rustc-env=TARGET' . --include='*.rs' --include='*.toml'; grep -o '"target_triple": "[^"]*"' examples/minimum-shelf-flagship/tool.seal.json`
- Route: L8 product (`garnet-cli`) — remove the field, or bind it and correct the comment; either
  way document which. Status: **open**. Disposition: open.

## U-109 — EOL normalization is CRLF-only, and the minimum shelf hashes its three files under two different rules

- Substance: `normalize_source_eol` (`garnet-cli/src/manifest.rs:382-384`) replaces `\r\n` only.
  CR-only (classic-Mac) input therefore diverges in `source_hash` while producing an identical
  `ast_hash` — recomputed at the swept head: `source_hash equal? False`, `ast_hash equal? True`.
  CR-only is out of contract and fails closed, which is the safe direction; CRLF handling **is** the
  intended contract and holds end-to-end (seal bytes, manifest bytes and `verify` all agree across
  LF/CRLF, pinned by `manifest::tests::crlf_and_lf_source_produce_the_same_seal_hashes`). Separately,
  the shelf applies **two different rules** to its own three files
  (`garnet-cli/src/minimum_shelf.rs:59, 69, 73`): the package manifest and the seal go through
  `canonical_text_blake3`, which LF-normalizes, while `tool.garnet` is hashed **raw** via
  `blake3_hex`. Observed in the crown scope C mutation matrix: a CRLF `tool.garnet` is rejected
  (`source bytes do not match the trusted flagship`) while a CRLF `SHELF_PACKAGE.json` and a CRLF
  `tool.seal.json` both pass. The source's CRLF safety therefore rests on `.gitattributes:20`
  (`*.garnet text eol=lf`) rather than on the code. Fail-closed, but asymmetric.
- Raised by: Claude Opus 5, crown scope C (non-blocking N-4) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, first-hand recomputation of the CR-only divergence — **same-family.
  A cure needs a Codex record.**
- Command: `printf '@caps()\ndef main(v) { v * 2 }\n' > lf/tool.garnet && python3 -c "d=open('lf/tool.garnet','rb').read(); open('cr/tool.garnet','wb').write(d.replace(b'\n', b'\r'))" && (cd lf && garnet build --deterministic tool.garnet) && (cd cr && garnet build --deterministic tool.garnet) && python3 -c "import json;a=json.load(open('lf/tool.garnet.manifest.json'));b=json.load(open('cr/tool.garnet.manifest.json'));print('source equal?',a['source_hash']==b['source_hash'],'ast equal?',a['ast_hash']==b['ast_hash'])"`
- Route: L8 product (`garnet-cli`) — one hash rule for all three shelf files, and a decision on
  CR-only recorded rather than left implicit. Status: **open**, recorded as a known partial rather
  than a defect to rush. Disposition: open.

## U-110 — `canonical_text_blake3` hardcodes the label "seal", so a non-UTF-8 package manifest is reported as a seal error

- Substance: `garnet-cli/src/minimum_shelf.rs:229` rejects with the literal string
  `"seal is not valid UTF-8"`, and the function is called for **both** the package manifest (line 59)
  and the seal (line 73). A non-UTF-8 `SHELF_PACKAGE.json` and a non-UTF-8 `tool.seal.json` therefore
  produce one identical message. This is the **only** generalized rejection in the shelf: the crown
  scope C mutation matrix records 21 of 22 mutation classes rejected with a *distinct*, specific
  problem naming the file class and the failure mode — one payload byte, one path name and one
  manifest field each produce a different message, and the strongest case (source tampered, freshly
  resealed with a valid predicate, `sourceBlake3` rebound in the package manifest) is still rejected
  because the manifest bytes themselves are pinned. The label is a one-line cure: thread the existing
  `label` argument through.
- Raised by: Claude Opus 5, crown scope C (non-blocking N-5) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, by direct source read of `minimum_shelf.rs:57-75` and `:227-231` at
  the swept head — **same-family. A cure needs a Codex record.**
- Command: `sed -n '57,75p;227,231p' garnet-cli/src/minimum_shelf.rs`
- Route: L8 product (`garnet-cli`), U-53 class (distinct rejections). Status: **open**.
  Disposition: open — the smallest item in this sweep and the one most worth folding into any other
  `minimum_shelf.rs` change.

## U-111 — `build` and `verify` are edition-blind while `seal`, `caps` and `diff-caps` are edition-aware, and the edition is recorded nowhere

- Substance: `garnet-cli/src/cmd/build.rs:17` and `garnet-cli/src/cmd/verify.rs:18` call
  `parse_source`; `garnet-cli/src/cmd/seal.rs:107` and `garnet-cli/src/cap_manifest.rs:146` call
  `parse_source_with_edition`. The same tree can therefore build but not seal. Recomputed at the
  swept head against a v2.0 project using a v2.0-reserved identifier:
  `garnet seal` → `parse error: 'async' is a reserved word in edition v2.0`, while
  `garnet build --deterministic` succeeds and emits a manifest, and `garnet diff-caps --machine`
  exits 2. Neither artifact records the edition (`grep -c edition` on the manifest → **0**), so an
  `ast_hash` is **not reproducible from source alone** for a v2.0 project without out-of-band
  knowledge of the edition. Bites only v2.0 ("Next") projects using v2.0-reserved identifiers today.
- Raised by: Claude Opus 5, crown scope C (non-blocking N-6) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, first-hand recomputation — **same-family. A cure needs a Codex
  record.**
- Command: `mkdir -p ed_v2 && printf '@caps()\ndef main(async) { async * 2 }\n' > ed_v2/tool.garnet && printf '[project]\nedition = "v2.0"\n' > ed_v2/Garnet.toml && (cd ed_v2 && garnet seal tool.garnet --out seal.json; garnet build --deterministic tool.garnet; grep -c edition tool.garnet.manifest.json)`
- Route: L8 product (`garnet-cli`) — make `build`/`verify` edition-aware and record the edition in the
  manifest and the seal. Status: **open**. Disposition: open.

## U-112 — `minimum_shelf::read_regular` resolves the path three times while the crate already ships an identity-bound reader

- Substance: `garnet-cli/src/minimum_shelf.rs:205-216` performs `symlink_metadata(path)` →
  `fs::metadata(path)` → `fs::read(path)`: three independent path resolutions with **no retained
  handle**. The crate already ships the fix for exactly this — `garnet-cli/src/bound_source.rs`,
  which re-checks the path against a second identity handle before bytes are read, used by
  `cmd/run.rs:396` and `cmd/test.rs:373,412` — and `minimum_shelf.rs` does not reference it
  (recomputed: `grep -c bound_source garnet-cli/src/minimum_shelf.rs` → **0**). Impact is bounded and
  stated as such rather than inflated: whatever bytes finally arrive must still match a compiled-in
  BLAKE3 pin, so a swap wins only by delivering bytes that already hash to the trusted value. The
  residue is that the size check and the regular-file / symlink checks are decided against a
  different resolution than the read. **No race was won**; this is a code-shape finding.
- Raised by: Claude Opus 5, crown scope C (non-blocking N-7) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, by direct source read and the `bound_source` count at the swept head —
  **same-family. A cure needs a Codex record.**
- Command: `sed -n '205,216p' garnet-cli/src/minimum_shelf.rs; grep -c bound_source garnet-cli/src/minimum_shelf.rs`
- Route: L8 product (`garnet-cli`) — use the in-repo reader. Status: **open**. Disposition: open —
  the cure is already written, in the same crate, and unused here.

## U-113 — The enforcement scope names two Declared primitives by identifiers that do not resolve

- Substance: `C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md:37` names the Declared
  (checker-only) row's UUID primitives as `uuid::new_v4` and `uuid::new_v7`. The registry keys are
  `std::uuid::new_v4` and `std::uuid::new_v7`. Recomputed at the swept head: a program whose body is
  `uuid::new_v4()` fails with `runtime error: unresolved path: uuid::new_v4`, exit 1, while
  `std::uuid::new_v4()` returns a UUID with no capability trap, exit 0. The **behavioral** claim
  (checker-only, never runtime-trapped) is correct; only the identifier is wrong. Worth fixing
  because this table is the fence other surfaces quote.
- Raised by: Claude Opus 5, crown scope B (finding B-2) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, first-hand recomputation of both spellings — **same-family. A cure
  needs a Codex record**, and the scope document is a trust-kernel file, so the cure is
  human-merge-only.
- Command: `printf '@caps(time)\ndef main() { uuid::new_v4() }\n' > u1.garnet && garnet run u1.garnet; printf '@caps(time)\ndef main() { std::uuid::new_v4() }\n' > u2.garnet && garnet run u2.garnet; grep -n 'uuid::new_v' C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md`
- Route: L5 public truth / enforcement-scope document. Status: **open**. Disposition: open — pairs
  with U-92, which edits the same document.

## U-114 — The adapter-drift guard in the stdlib bridge is a `debug_assert` that compiles out in release

- Substance: `garnet-interp-v0.3/src/stdlib_bridge.rs:59-65` and `:71-75` guard a missing adapter
  with `debug_assert!(false, ...)` followed by `continue`. In a release build the assert compiles out
  and the binding is **silently skipped**. The consequence is **fail-closed** — the name is simply
  unbound and a call raises an undefined-variable error, never an ungated authority binding — and
  `registry_join_is_total` makes the condition unreachable in a green tree (the crown scope B run
  recorded `cargo test -p garnet-interp` at 446 passed, 0 failed). Recorded for completeness rather
  than as a live defect.
- Raised by: Claude Opus 5, crown scope B (finding B-3) — **same-family**. Confirmed by:
  Claude Opus 5, records seat, by direct source read at the swept head — **same-family. A cure needs
  a Codex record.**
- Command: `sed -n '57,76p' garnet-interp-v0.3/src/stdlib_bridge.rs`
- Route: L8 product (`garnet-interp-v0.3`) — a real error rather than a debug-only assert.
  Status: **open**. Disposition: recorded; lowest priority in this sweep.

## U-115 — The U-82 diagnosis's option D1 conflicts with the R2 law's run-census requirement

- Substance: option D1 of the U-82 diagnosis — a sanctioned "re-run all jobs" on the Base-controlled
  run **after** the carrier approval, so the composite is observable post-approval — collides with
  the adopted R2 law. `C_Language_Specification/GARNET_TRUST_KERNEL_ROLLING_REVIEW.md:154` states:
  "A fresh head-scoped run census MUST show only this CI workflow at attempt 2 and every other
  producer at attempt 1." A sanctioned re-run of the Base-controlled workflow puts a **second**
  producer at attempt 2 on the same head, which that census rejects by construction. The cure that
  actually shipped for U-82 is option A (splitting the composite predicate), not D1, so nothing is
  broken today — but the activation ceremony's own "rerun on the exact activation head" step runs
  into the same wall, and the tension should be on the record **before R2 activates** rather than
  discovered inside the ceremony. The #549 PR body already flags it as "registered separately"; this
  is that registration.
- Raised by: Claude Opus 5, records seat, this sweep — **same-family**, reading the R2 law against
  the U-82 cure lineage. Confirmed by: the law text and the #549 PR body, read directly at the swept
  head; the #549 cure itself carries a **cross-family** Codex CONFIRM bound to `b6f960a3`, but that
  record confirms option A, **not** this tension.
- Command: `sed -n '154,159p' C_Language_Specification/GARNET_TRUST_KERNEL_ROLLING_REVIEW.md; git log --format=%B -1 origin/mission/u82-composite-boundary~1 | grep -n 'freeze-by-construction'`
- Route: L1 (succession law, acts 2–5) and L9 (activation evidence shape). Status: **recorded** — no
  cure, and none is due until R2 activation is scheduled. Disposition: doctrine; the directing seat
  rules which of the two moves, the law or the ceremony.

## U-116 — The rolling gate's trust surface is a hand-maintained enumeration with holes in exactly the load-bearing places

- Substance: `is_trust_kernel()` decides whether a change needs a structured review record, and it
  answers from two hand-maintained lists — `TRUST_KERNEL_PREFIXES` and `TRUST_KERNEL_FILES`
  (`scripts/garnet_trust_kernel_review_status.py:54-87`). Five `garnet-cli/src/**` files that
  compute and gate on the capability surface are **absent** from both. Verified by calling the
  gate's own predicate at the swept head — every one returns **False**:
  `garnet-cli/src/cap_manifest.rs`, `garnet-cli/src/cmd/diff_caps.rs`,
  `garnet-cli/src/cmd/verify_gate.rs` (these three are the capability-surface computation and the
  walk that decides **whether authority widened** — the very walk U-93 is a defect in) and
  `garnet-cli/src/manifest.rs`, `garnet-cli/src/cmd/verify.rs` (manifest verification and Ed25519
  signature checking — the surfaces U-94 and U-95 are defects in). A change to any of them merges
  with `trust_kernel_touched: false` and no review record required.
- **These are holes in a deliberate list, not a decision to exclude the crate.** `TRUST_KERNEL_FILES`
  already names the siblings `garnet-cli/src/cmd/{add,mod,run,test,eval,doctest}.rs`,
  `garnet-cli/src/bound_source.rs`, `garnet-cli/src/bin/garnet.rs` and `garnet-cli/src/lib.rs` —
  all nine return **True** from the same predicate in the same run. Somebody enumerated this crate
  file by file and these five did not get added.
- Same class as **U-101** one level up: there, the gate that produces a required context was outside
  the surface it gates; here, the code that computes the authority verdict is outside the surface
  that reviews it. Both are enumeration drift, and the enumeration is the whole mechanism —
  a hand-maintained allow-list is only as good as its last edit.
- Raised by: **three seats independently, on the same day** — **Codex**, H3 hardening seat, H3-02
  (**cross-family**), for the `scripts/check_*` half recorded as U-101; and two **Claude**
  implementer seats (**same-family**), each of which expected `trust_kernel_touched: true` on its own
  cure and got `false`. Confirmed by: Claude Opus 5, records seat, by calling `is_trust_kernel()`
  on all five claimed holes and all nine covered siblings at the swept head. The convergence of a
  cross-family probe and two independent implementer surprises is what makes this systemic rather
  than a single oversight.
- Command: `python3 -I -c "import importlib.util,sys; spec=importlib.util.spec_from_file_location('g','scripts/garnet_trust_kernel_review_status.py'); g=importlib.util.module_from_spec(spec); sys.modules['g']=g; spec.loader.exec_module(g); [print(g.is_trust_kernel(p), p) for p in ['garnet-cli/src/cap_manifest.rs','garnet-cli/src/cmd/diff_caps.rs','garnet-cli/src/cmd/verify_gate.rs','garnet-cli/src/manifest.rs','garnet-cli/src/cmd/verify.rs','garnet-cli/src/cmd/run.rs','garnet-cli/src/lib.rs']]"`
- Route: gate coverage (L4). **Human-merge-only** — a PR may not modify the gate it merges under
  (integrity rule 1), and this cure is exactly that shape. Status: **cure in preparation**.
  Disposition: open — blocking, and it should carry U-101's `scripts/check_*` half in the same act so
  the enumeration is corrected once rather than twice.

## Amendments without reallocation

- **U-82** — cure open as **PR #549** (branch `mission/u82-composite-boundary`, content `b6f960a3`,
  Codex CONFIRM bound to that content, record `af8c53be`). The diagnosis established a fact worth
  carrying: `evaluate_base_controlled_trust` required `reviewed_head`, `reviewed_tree` and
  `content_digest` to be strings on **every** pull request, while rolling review v2 computes
  `content_digest` only when a trust-kernel path is touched. Every clean non-trust-kernel PR
  therefore arrived with the triple `None` and the composite printed the boundary problem with
  `candidate_policy_ok: true`. The consequence the diagnosis names in its own words: had the
  Base-controlled context been made required as-was, "every non-trust-kernel PR would have been
  blocked by construction (a freeze-by-construction)". So requiring the context **today** is not a
  hardening step — it is a repository-wide freeze. The cure splits the predicate into `boundary_ok`,
  `record_ok` and `untouched_ok` and leaves the problem string unchanged. Open until #549 merges.
- **U-83** — refined. The four caps-invisible `memory::*` natives are a **registry-completeness gap**
  and **not** reachable host authority today. `MemoryBackend::for_kind` constructs purely in-process
  stores, and the episodic store's disk-touching methods are unreachable from Garnet source —
  recomputed at the swept head: `s.save_text("leak.txt")` on a `memory::episodic` handle gives
  `runtime error: EpisodeStore has no method 'save_text'` and no file is written; `load_text` and
  `append_text` behave the same. `grep` across `garnet-interp-v0.3/src` and `garnet-cli/src` for
  those names returns nothing. So the accurate statement is a **latent** surface: it must earn
  registry rows **before** any persistence method is bridged, and the fence belongs in the registry
  rather than in a note. Status unchanged (open); route unchanged (L8).
- **U-70** (reviewer-seat error class) — two instances added from this arc.
  (1) A review prompt bound its verdict to an **uncured parent** because the prompt still named the
  old SHA: the union-page review v2 returned `VERDICT: REJECT — bound to 8f81dee7…` with the sole
  blocking line "reviewed evidence binds `838969a9…`, not its uncured parent `8f81dee…`. All content
  checks pass at `838969a…`" — the cure had landed, the prompt had not moved, and the verdict read
  as a rejection of cured content.
  (2) A claim in a review brief that a Gemini skills directory was **not found** in the source read
  was falsified by a later migration in the same source tree
  (`omarchy-quattro/migrations/1786719479.sh`, which adds `~/.gemini/config/skills`); the union
  page's commit record states plainly that the brief's line "was not true at this source read and was
  not shipped." The class's cure held both times — the opposite seat's recomputation caught each
  before anything false landed. Status unchanged (fenced — active covenant).

## Reconciliation

- Candidates processed: **26 new allocations** (U-91 … U-116), 0 backfills, **3 amendments**
  (U-82, U-83, U-70).
- Census: **73 before** (`U-1`, `U-04` … `U-90`; the `U-1` prose token counted) → **99 after**.
  Collision sweep: no id in U-91 … U-116 occurs in any of the 475 unique advertised trees or in any
  commit message on the 476 advertised refs; re-verified against the advanced `origin/main` after
  #547 merged, still census 73 and still nothing at or above U-91. U-91 and U-92 are fixed by the
  directing seat's prior commit-message use, not allocated freshly.
- Family split, stated because it governs what may be cited: **cross-family (Codex)** — U-96, U-97,
  U-98, U-99, U-100, U-101, U-102, U-103, U-104, U-105 (10). **Same-family (Claude)** — U-91, U-92,
  U-93, U-94, U-95, U-106, U-107, U-108, U-109, U-110, U-111, U-112, U-113, U-114, U-115 (15).
  **Both** — U-116, raised by a Codex probe and by two Claude implementer seats independently (1).
  The same-family set includes the two authority-widening findings of this arc (U-91, U-93), which is
  the uncomfortable shape worth naming: the most serious items here have **no independent record**,
  and every cure to them must earn one.
- Cures in flight at the time of writing: #549 (U-82), #551 (U-96, U-105), #552 (U-97),
  #553 (U-91 and U-92, public-truth half only), #554 (U-94, U-95), #555 (U-93). In preparation:
  U-116 with U-101. Nothing in this sweep is closed; every status above is open.
- Every entry was recomputed by this seat directly at `beeb5e7b2` — the L2 laundering case, the
  VM/interp divergence, the `vendor/` skip and its control, the five-manifest tamper set, the signed
  injected key, all three dogfood bodies, Codex's full bounded repro program, the bot-principal
  reproduction, the cosign shim divergence, the CR-only divergence, the edition split, the two UUID
  spellings, the `save_text` refusal, the pre-cure `garnet caps` aggregate under `vendor/`, and all
  fourteen `is_trust_kernel()` answers — or read directly in a named report and attributed to it.
  No measurement in this file was transcribed from another seat's prose.
- Binary provenance, since every black-box result above used one build: the binary is `garnet 0.8.1`
  built at `fbd64bc5`/`e36c61f9`, not at the swept head. Recomputed rather than assumed —
  `git diff --stat e36c61f9 beeb5e7b2 -- garnet-interp-v0.3/src garnet-vm/src garnet-cli/src
  garnet-stdlib/src garnet-check-v0.3/src garnet-parser-v0.3/src` is **empty**, so the black-box
  behavior binds validly to `beeb5e7b2`.

This record lives under `F_Project_Management/W_TRUST/**`, which is both product-digest-excluded and
an enumerated record-class surface, so this tip moves no frozen pair and buys no ceremony.
