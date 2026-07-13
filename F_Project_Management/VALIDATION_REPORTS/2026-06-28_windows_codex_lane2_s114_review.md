# Lane 2 - S114 Independent Review Re-Verification

Date: 2026-06-28  
Reviewer lane: Codex on Windows  
Host OS: Windows  
Validation branch: `validation/2026-06-25-codex-s114-review`  
Reviewed commit: `2e2fe843e87be0c8fc9a4745a5bb138fba597d23`  
Evidence bundle: `proofs/validation/s114-review/windows-20260628-lane2/`  
Evidence manifest: `proofs/validation/s114-review/windows-20260628-lane2/MANIFEST.sha256` (160 files)

## Verdict

S114 re-verification mostly HELD on Windows, with one behavior caveat preserved.

The impl-method authority-surface fix held in both the active checkout and a clean clone at the same commit: `@caps(fs)` introduced through an impl method is visible in `garnet caps`, `diff-caps --machine` returns `authority-expanded` with exit 1, and `agent-loop` rejects at the diff-caps stage on both `interp` and `vm` backends before any seal is written.

The new evasion probes also held: nested module functions, impl-inside-module methods, wildcard `@caps(*)`, and mis-cased or unknown caps all produced conservative authority-expansion results where applicable. Runtime traps held for over-depth recursion and proc laundering on both backends.

The S114-FIX-2 residual lanes held for the committed integration target: explicit `cargo test -p garnet-cli --test s114_residual_lanes -- --nocapture` ran 5 tests, 0 failed. Manual dynamic eval/test/doctest probes also trapped undeclared `fs` authority and did not leak the secret marker.

One caveat remains: the manual vendored dependency preload probe emitted the expected capability trap and did not leak the secret marker, but the `garnet run --interp` process exited 0 and still printed `=> 0`. The committed integration test only asserts the trap is surfaced and the secret is not leaked; it does not require nonzero process exit. Classify this as a follow-up on exit semantics, not as a renewed HIGH unless maintainers decide dependency preload errors must make the whole run fail.

## Surface Scores

| Surface | Score | Classification | Evidence |
| --- | ---: | --- | --- |
| Impl-method authority expansion | 5/5 | HELD | `100`, `101`, `102`, `103`; clean clone `401`, `402`, `403` |
| Nested module and impl-inside-module evasion | 5/5 | HELD | `110`, `111`, `115`, `116`; clean clone `406` |
| Wildcard and mis-cased/unknown caps | 5/5 | HELD | `113`, `114`; clean clone `404`, `405` |
| Agent-loop accept/reject/no-seal gate | 5/5 | HELD | `102`, `103`, `120`, `121`; clean clone `402`, `403`, `407`, `408`; target test `204b` |
| `@max_depth` enforcement and invalid bound rejection | 5/5 | HELD | `120`, `121`, `122`, `123`; target test `203b`; clean clone `407`, `408`, `411` |
| Proc laundering | 5/5 | HELD | corrected dynamic probes `130b`, `131b`; clean clone `409`, `410` |
| Active-frame residual lanes: eval/test/doctest | 5/5 | HELD | `140g`, `141`, `142c`; target test `203b`; clean clone `411` |
| Active-frame residual lane: vendored dependency preload | 3/5 | PARTIAL / FOLLOW-UP | `143` surfaced trap and no leak, but exit 0; target test `203b` passes because it does not assert nonzero exit |
| caps-log tail forgery LOW | 5/5 | STILL LOW / REPRODUCED | `300`, `301`, `302`, `303` |
| seal subject digest LOW | 3/5 | NOT REPRODUCED BY THIS PROBE | `310`, `311`, `seal-digest-comparison.json` show differing subject digests and differing predicate caps |

## Confirmed Facts

- Recon read the S114 review package, current kernel review status document, relevant integration tests, and current capability surface implementation before writing this report.
- Active checkout `HEAD` and `origin/main` both resolved to `2e2fe843e87be0c8fc9a4745a5bb138fba597d23`.
- A clean clone from `https://github.com/island-dev-crew/garnet.git` resolved to the same commit and built `garnet-cli` release successfully (`400-fresh-cargo-build-release`, exit 0).
- Corrected integration target runs were explicit `--test` invocations:
  - `203b-cargo-garnet-cli-s114-residual-target`: 5 passed, 0 failed.
  - `204b-cargo-garnet-cli-agent-loop-target`: 6 passed, 0 failed.
  - `205b-cargo-garnet-cli-diff-caps-target`: 9 passed, 0 failed.
  - `206b-cargo-garnet-cli-caps-log-target`: 2 passed, 0 failed.
  - `207b-cargo-garnet-cli-seal-attestation-target`: 3 passed, 0 failed.
- Earlier non-`--test` cargo commands that filtered to zero tests are retained in raw evidence but are not counted as proof.
- Several PowerShell quoting/BOM diagnostic attempts are retained in raw evidence but are not counted as proof: initial `130`/`131`, `140c` through `140f`, `140h`, and `142`/`142b`.
- PowerShell wraps some native stderr as `NativeCommandError`; the recorded exit codes in `commands.jsonl` are the authoritative per-command exits.
- This report makes no Windows or macOS OS-sandbox enforcement claim. All findings here are language/runtime trap and review-surface results on Windows.

## Recommendations

- Treat the impl-method HIGH and S114-FIX-2 residual HIGH as independently re-verified-with-fixes on this Windows lane, pending Jon acceptance.
- Add or update a test if dependency preload is intended to fail the full `garnet run` process when a vendored dependency traps during preload. Current behavior surfaces the trap without leaking data but returns exit 0 in the manual run.
- Reconcile the seal subject digest LOW wording. This run did not reproduce capability-blind subject digests for the simple baseline-vs-net pair; `seal-digest-comparison.json` records different subject digests and different predicate caps. Either the old LOW has been fixed by later changes or this probe is not the same shape as the original finding.
- Keep the caps-log tail-forgery finding as LOW until `caps-log --verify` recomputes the tail entry's `caps_blake3` from `caps` or anchors the current tip.

## Jon-Only Decisions

- Whether public wording may move from self-verified to independently re-verified.
- Whether the dependency-preload exit 0 behavior is acceptable as "trap surfaced / no leak" or must become nonzero.
- Whether the seal subject digest LOW can be closed, revised, or requires the original reproducer to be restored.
- Any release note, public claim, tag, or signed artifact update.
