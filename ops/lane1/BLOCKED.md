# Lane 1 · Phase 0 — BLOCKED

- As of: 2026-07-27 (post verdict-01 cure)
- **Cured frozen candidate:** `48fc7529b1321ce5e54eafd9ac42523a6813f161` (tree `e769ef1f…`, product digest `aa6a65a3…`, 1575 paths) — linear successor atop verdict 01 `f51c936b`, superseding `d7430c2`
- Fork branch: `Navigata1/garnet` · `mission/l1-reconcile-post-activation`
- Verdict 01: APPROVE-WITH-BLOCKERS (`ops/lane1/review/01-verdict.md`); F1+F2 cured in `48fc752`; cure request: `ops/lane1/review/02-request.md`

## Blocked on (none the implementer's to clear)

1. **Verdict 02** — Codex GPT-5.6 Sol delta review of the F1/F2 cure at `48fc752`. Verdict lands as `ops/lane1/review/02-verdict.md` under the fleet-fork identity (never IDC-Trust-Review, U-30).
2. **Slice 4 — native-Windows WV-6 refresh** at the head verdict 02 approves (NOT `d7430c2`). WV-6 remains `partial` until the Windows re-accept binds the approved head's product digest.
3. **Slice 5 — reporter/SOTU/denominator refresh** (post-Windows): publishes **66.7% / 50.0%** per the verdict-01 ruling; **blocked by U-31/F4** (absolute path in `08.source`) which must be cured first via its own reviewed change.
4. **Slices 6–7 — Jon-only:** PR, ONE record commit added once, IDC-Trust-Review approval at the record head, authenticated gate ok:true, squash-merge. Nothing pushed after the record.

## Exact resume command (Windows seat, slice 4 — substitute the verdict-02-approved head)

```
git config --global core.autocrlf false
git clone https://github.com/Island-Dev-Crew/garnet.git garnet && cd garnet
git remote add fork https://github.com/Navigata1/garnet.git && git fetch fork
git checkout <verdict-02-approved-head>
# recompute the product digest via scripts/garnet_content_provenance.py; STOP if it disagrees with the request/verdict value
# then execute the sanctioned WV-6 acceptance procedure end-to-end (all 5 checks), platform=windows
```
