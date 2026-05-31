# Garnet v0.8 beta gate (S50)

The S41–S50 **v0.8 hardening band** milestone. This is a band-completion
**checkpoint**, not a release. The live status is `scripts/garnet_v0_8_beta_gate.py
--format md`; `--gate` runs in CI and fails unless the band is complete and its
sub-gates hold.

## What the gate checks

1. **Band complete** — the nine implemented hardening slices **S41–S49** are
   `merged` at merge-confidence 5 in `.dogfood/goal.json` (S50 is this gate).
2. **Sub-gates hold** — the band's own anti-rot gates still pass:
   - `garnet_build_proof.py --gate` (S47 cross-OS coverage),
   - `garnet_proof_matrix.py --gate` (S48 evidence anchors).

The beta gate is **OPEN** only when both are true.

## What the v0.8 hardening band shipped (S41–S49)

| slice | capability |
|---|---|
| S41 | async/concurrency contract |
| S42 | typed Result / error policy (over-catch advisory) |
| S43 | docs-as-tests (`garnet doctest`) |
| S44 | LSP safe-mode precision (canonical CheckError severity/code) |
| S45 | slopsquatting guard (`garnet add --registry`) |
| S46 | caps-to-sandbox policy (`garnet sandbox`) |
| S47 | Windows/Linux/macOS build proof + coverage gate |
| S48 | 12-domain / 7-novel proof matrix |
| S49 | AI-PR-review-collapse wedge demo |

## Deferred for v0.8 beta (honest)

- **Runtime sandbox enforcement** — S46 *generates* seccomp/WASI/egress policy
  (`"enforced": false`); it does not enforce. Needs wasmtime / a Linux seccomp
  host.
- **Windows CLI distribution** — only the separate Studio installer exists (S47).
- **LLM advisory tier** — the compiler-as-agent rules tier ships; the LLM tier
  remains pending-infra.
- **Cross-package LSP precision** — S44 deferred the cross-file half to the
  package-resolver line.
- **Empirical Paper VI measurements / mechanized proofs** — S48 is an evidence
  inventory, not proof.

## Honesty anchors (verbatim — not softened)

- "research-grade prototype (v0.x.x) — not production-complete"
- "tracked-slice ledger is complete, but that is not full MIT/productization completion"
- "Paper VI scorecard: 4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra"
- "production allocator path tracked in MEMORY_CORE_ROADMAP.md"
- "human/aesthetic acceptance remains open"

## Not a release

This gate does **not** cut a tag and does **not** claim production readiness.
Only `v0.4.2` and `v0.5.0` are tagged (S31 release-truth); cutting
`v0.8.0-beta` — or any tag — is a **release-truth decision for Jon**, not made by
this slice. The v0.8.0 tag remains planned later in the roadmap.

```sh
python3 scripts/garnet_v0_8_beta_gate.py --format md   # this status (live)
python3 scripts/garnet_v0_8_beta_gate.py --gate        # CI checkpoint (exit 1 if not open)
```
