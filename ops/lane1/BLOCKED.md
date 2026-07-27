# Lane 1 · Phase 0 — BLOCKED

- As of: 2026-07-27
- Frozen candidate: `d7430c285fa8620dcf1f0c1cd94e5cc44b98d180` (tree `2e8ce5de…`, product digest `c4b3cf7c…`)
- Fork branch: `Navigata1/garnet` · `mission/l1-reconcile-post-activation` · tip `742d00e` (review request 01 above the candidate)

## Blocked on (three gates, none the implementer's to clear)

1. **Independent delta review** — Codex GPT-5.6 Sol (different machine/model) reviews the frozen candidate `d7430c2`. Verdict lands as `ops/lane1/review/NN-verdict.md` under the fleet-fork identity (never IDC-Trust-Review, per U-30).
2. **Slice 4 — native-Windows WV-6 refresh (F-1).** Implementer machine is macOS arm64; the WV-6 acceptance manifest names **Windows** (`proofs/windows/…`). Must run on a Windows-capable seat at the reviewer-approved head. Currently WV-6 is `partial` (digest `c4b3cf7c` ≠ certified `2f8c9ad8`, U-29) — expected until this refresh re-accepts at the frozen head.
3. **Slices 5–7 — Jon-only.** Verify at the evidence head, open the PR, the ONE record commit ADDED once, IDC-Trust-Review approval at the record head, authenticated gate ok:true, and Jon's squash-merge. Nothing pushed after the record.

## What is done (this session)

- RED baseline `7db0f45`; markers `a55a4e0` (#514 + #517, `verify_repository_landed_markers` clean); why.html Five Theses graft `d7430c2` (capability-scope gate green). Freeze computed; branch pushed; review request 01 pushed. STOP.

## Deferred (Jon-decided) / freeze conflict

- **Reporter / SOTU / denominator refresh → slice 5 (post-Windows).** Not regenerated here: on this pre-Windows tree the producers yield launch_critical 3/6 (50.0%), launch_ledger 3/8 (37.5%) — not the 83.3/62.5 target (gated on WV-6 Windows acceptance) — and `garnet_launch_readiness_status.py` embeds an absolute machine path in `08.source`. Regenerating `ops/mission/state.json` + SOTU would also be a producer-owned, product-digest-affecting change above the frozen head; **Covenant-9 SOTU regeneration is therefore reported-as-conflicting-with-the-freeze, not performed.**

## Exact resume command (Windows seat, slice 4)

```
git config --global core.autocrlf false
git clone https://github.com/Island-Dev-Crew/garnet.git garnet && cd garnet
git remote add fork https://github.com/Navigata1/garnet.git && git fetch fork
git checkout d7430c285fa8620dcf1f0c1cd94e5cc44b98d180
python3 -I scripts/garnet_content_provenance.py  # recompute product digest; STOP if != c4b3cf7c
# then execute the sanctioned WV-6 acceptance procedure end-to-end (all 5 checks), platform=windows
```
