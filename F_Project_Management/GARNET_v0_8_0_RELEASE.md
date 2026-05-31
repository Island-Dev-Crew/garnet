# Garnet v0.8.0 release readiness (S60)

> **Version-map correction (2026-05-31, S70).** This document is the **first
> v0.8.0 readiness *checkpoint*** (S60), not a tag slice. Per the corrected
> source of truth ([`GARNET_v0_8_VERSION_MAP.md`](GARNET_v0_8_VERSION_MAP.md)),
> the single `v0.8.0` tag for the whole S30–S80 run is cut at the **S80 cut
> decision** — S60 and S70 are checkpoints, and no tag is shipped before S80. The
> evidence below still stands as the S60-band readiness snapshot.

This is the evidence for the S60 `v0.8.0` readiness checkpoint. The live verdict is
`scripts/garnet_v0_8_0_release_readiness.py --format md`; `--gate` runs in CI.

> **This document and its gate do NOT cut a tag.** Only `v0.4.2` and `v0.5.0` are
> tagged. Cutting `v0.8.0` is a **release-truth decision for Jon**, not made by
> this slice. "READY TO TAG" is a recommendation backed by evidence — not the act.

## Verdict: READY TO TAG (pending Jon)

- **Hardening band (S41–S50): 10/10 merged.** async/concurrency contract, typed
  Result / error policy, docs-as-tests, LSP precision, slopsquatting guard,
  caps-to-sandbox policy, cross-OS build proof, 12-domain/7-novel proof matrix,
  AI-PR-review-collapse wedge, v0.8 beta gate.
- **Adoption band (S51–S59): 9/9 merged.** signed release lanes, one-line install
  check, tree-sitter grammar, VS Code marketplace path, WASM hello-world,
  playground MVP, idiomatic corpus, benchmark campaign, fuzz campaign.
- **Anti-rot sub-gates: 11/11 pass** (the beta gate + every band gate, re-run by
  the release-readiness gate).

## What v0.8.0 would include

The 19 implemented slices S41–S59 — the hardening band's compiler/runtime/LSP/
trust-kernel work and the adoption band's release/editor/web/validation tooling —
each shipped one-PR, CI-green, dogfooded, with an honest contract block.

## Deferred for v0.8.0 (honest)

- Runtime sandbox **enforcement** (S46 generates policy; does not enforce).
- Release-artifact + supply-chain **signing** lanes (S51 lanes 2–3 — GPG/cosign).
- OpenVSX / Marketplace **publish** (S54 — needs `OVSX_TOKEN`/`VSCE_PAT`).
- **WASM build** + browser playground execution (S55/S56 — toolchain absent).
- **LLM advisory tier** (rules tier ships; LLM tier pending-infra).
- Empirical **Paper VI measurements** / mechanized proofs (S48 is an inventory).

## Honesty anchors (verbatim — not softened)

- "research-grade prototype (v0.x.x) — not production-complete"
- "Paper VI scorecard: 4 supported, 2 partial (downgraded honestly), 0 refuted, 1 pending-infra"
- "human/aesthetic acceptance remains open"

## Decision (2026-05-31)

**Jon chose to defer the `v0.8.0` tag and continue the slice train (S61+).** The
readiness gate stays OPEN; the tag remains uncut (only `v0.4.2`/`v0.5.0` are
tagged) and can be cut on Jon's authorization whenever he chooses. Per the
roadmap's 0.8.x arc, tagging is not on the critical path for continuing
development.

## The tag decision (escalated)

Cutting and pushing the `v0.8.0` git tag is **Jon's call** — it is irreversible,
release-strategy-bearing, and reserved by the honesty anchors. When approved, the
mechanical step is `git tag v0.8.0 <commit> && git push origin v0.8.0` (which also
triggers the tag-gated release-asset + VSIX-publish workflows). This slice ships
the readiness gate and stops there.
