# Lane 1 · Phase 0 — BLOCKED

- As of: 2026-07-28 (post verdict-08 cross-family APPROVE of U-35 at `7ad4385`; slice 4 DONE — WV-6 accepted 5/5, native-Windows evidence rebound at `f3876c5`; U-31 cure packet pushed as request 09)
- **Cured frozen candidate:** `72ae0246fb448ce33d689b1b80eb783497a7f215` (tree `3c98ba05…`, product digest `99c3f270…`, 1578 paths) — linear successor atop verdict 02 `14a5e45`, superseding `4f5ebb8` (← `48fc752` ← `d7430c2`)
- Fork branch: `Navigata1/garnet` · `mission/l1-reconcile-post-activation`
- Verdict 01 cured in `48fc752`; ceremony-ruled reasons #2/#10 cured in `4f5ebb8`; verdict 02 (`14a5e45`) F1 (--evidence layer fragment) + F2 (build/seal-vs-check, source-confirmed) cured in `72ae024` (request 04). **U-32 registered** (build/seal accept what check rejects — own lane, pre-Lane-4). Ceremony ruling on verdict 02's whole-page units: DEFERRED to Lane 3 as **U-33** (launch-blocking; ONE coherent claim-ledger pass; pre-launch + pre-CRA) — no more cure rounds in this lane; `72ae024` STANDS. Addendum: `ops/lane1/review/04-request-addendum-u33.md`.

## Resolved since the last update

- **U-35: CURED and APPROVED.** Verdict 08 (Codex GPT-5.6 Sol, cross-family verdict of record) APPROVED the cure at exact head `7ad4385`; verdict 07 (Claude Fable 5) stands as same-family corroboration. `ops/lane1/` is digest-excluded; the pair is `e89cb299…/1544` and review artifacts no longer move it.
- **Slice 4: DONE.** WV-6 is `accepted` 5/5 at the approved head; the native-Windows evidence rebind landed at `f3876c5` (proofs/** only, digest-inert).

## Blocked on (none the implementer's to clear)

1. **U-31 RULING — request 09 pending.** Slice-5 precursor: `08.source` stamps a machine-absolute path into the digest-included `ops/lane0/evidence/08-launch-readiness.json`. RED recorded first (`ops/lane1/evidence/93-u31-machine-path-red.md`): seat divergence at the same commit (`equal_without_source: True`) and would-be certified-tree divergence `824e1e8f…` vs `b232031b…`. Packet `ops/lane1/review/09-request.md` proposes the cure (Option A: repo-relative POSIX producer path) with the emitted-not-consumed determination and four pre-agreed traps. **No cure lands before the ruling** (gate-logic-adjacent, freeze ACTIVE).
2. **Slice 5 — reporter/SOTU/denominator refresh** (after the U-31 cure lands): the ruled landed values are **66.7% / 50.0%** (verdict-01 ruling). NOTE the request-09 §6 falsified-expectation report: today's producer regenerates to 3/6 = 50.0% and 3/8 = 37.5%; reaching 4/6 additionally requires the slice-5 shelf-gate reflection of the recorded WV-6 acceptance plus matching closeout expectation updates — slice-5-reviewed changes, not part of the U-31 cure.
3. **Slices 6–7 — Jon-only:** PR, ONE record commit added once, IDC-Trust-Review approval at the record head, authenticated gate ok:true, squash-merge. Nothing pushed after the record.

## Exact resume command (any implementer seat, after the U-31 ruling)

```
git config --global core.autocrlf false
git clone https://github.com/Island-Dev-Crew/garnet.git garnet && cd garnet
git remote add fork https://github.com/Navigata1/garnet.git && git fetch fork
git checkout mission/l1-reconcile-post-activation   # RE-FETCH THE TIP AT WAKE — verdicts land while lanes sleep
# read ops/lane1/review/ for the highest-numbered verdict; if it rules on request 09, implement ONLY the authorized cure surface with RED-turns-green
# recompute the product pair via scripts/garnet_content_provenance.py; STOP unless it equals e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f / 1544 before the cure commit
```

(The slice-4 Windows resume block is retired: WV-6 is accepted 5/5 and the
rebind landed at `f3876c5`.)
