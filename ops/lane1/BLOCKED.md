# Lane 1 · Phase 0 — BLOCKED

- As of: 2026-07-27 (post verdict-04 APPROVE + ceremony-authorized WV pin rebind)
- **Cured frozen candidate:** `72ae0246fb448ce33d689b1b80eb783497a7f215` (tree `3c98ba05…`, product digest `99c3f270…`, 1578 paths) — linear successor atop verdict 02 `14a5e45`, superseding `4f5ebb8` (← `48fc752` ← `d7430c2`)
- Fork branch: `Navigata1/garnet` · `mission/l1-reconcile-post-activation`
- Verdict 01 cured in `48fc752`; ceremony-ruled reasons #2/#10 cured in `4f5ebb8`; verdict 02 (`14a5e45`) F1 (--evidence layer fragment) + F2 (build/seal-vs-check, source-confirmed) cured in `72ae024` (request 04). **U-32 registered** (build/seal accept what check rejects — own lane, pre-Lane-4). Ceremony ruling on verdict 02's whole-page units: DEFERRED to Lane 3 as **U-33** (launch-blocking; ONE coherent claim-ledger pass; pre-launch + pre-CRA) — no more cure rounds in this lane; `72ae024` STANDS. Addendum: `ops/lane1/review/04-request-addendum-u33.md`.

## Blocked on (none the implementer's to clear)

1. **Next verdict** — Codex GPT-5.6 Sol review of the ceremony-authorized WV pin rebind `f1ec569` (request 05). Verdict 04 APPROVED content head `72ae024`; the rebind (4 constants + 4 PROOF.json mirrors, recomputed 5d3e7f72…/1581) makes slice 4 mechanically reachable. Verdict lands under the fleet-fork identity (never IDC-Trust-Review, U-30).
2. **Slice 4 — native-Windows WV-6 refresh (NUC)** at the head the NEXT verdict approves (verdict 04's approval predates the rebind). WV-6 is `partial` with four expected old-manifest boundary findings until the NUC regenerates evidence.
3. **Slice 5 — reporter/SOTU/denominator refresh** (post-Windows): publishes **66.7% / 50.0%** per the verdict-01 ruling; **blocked by U-31** (absolute path in `08.source`, re-reproduced in verdict 02 F5) which must be cured first via its own reviewed change.
4. **Slices 6–7 — Jon-only:** PR, ONE record commit added once, IDC-Trust-Review approval at the record head, authenticated gate ok:true, squash-merge. Nothing pushed after the record.

## Exact resume command (Windows seat, slice 4 — substitute the verdict-approved head; NO head is approved yet)

```
git config --global core.autocrlf false
git clone https://github.com/Island-Dev-Crew/garnet.git garnet && cd garnet
git remote add fork https://github.com/Navigata1/garnet.git && git fetch fork
git checkout <verdict-approved-head>
# recompute the product digest via scripts/garnet_content_provenance.py; STOP if it disagrees with the request/verdict value
# then execute the sanctioned WV-6 acceptance procedure end-to-end (all 5 checks), platform=windows
```
