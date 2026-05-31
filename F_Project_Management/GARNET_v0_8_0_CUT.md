# Garnet v0.8.0 cut readiness — the whole S30–S80 run (S80)

This is the evidence for the **single `v0.8.0` tag** that cuts the entire S30–S80
completion run (per `GARNET_v0_8_VERSION_MAP.md`). The live verdict is
`scripts/garnet_v0_8_0_cut_readiness.py --format md`; `--gate` runs in CI.

> **This document and its gate do NOT cut a tag.** Cutting `v0.8.0` is a
> **release-truth decision for Jon**, not made by this slice. Only `v0.4.2` and
> `v0.5.0` are tagged today. "READY TO CUT" is evidence-backed advice — not the act.

## Verdict: READY TO CUT (pending Jon)

The aggregator confirms, in one verdict:

1. **Ledger** — every slice **S31..S79 is merged** (49/50; S80 is this decision).
2. **Foundation / hardening / adoption (S41–S59)** — the S60 release-readiness
   gate passes (it re-runs the band gates + 11 anti-rot sub-gates).
3. **Native-interop + provenance (S61–S70)** — merged; the FFI / Rust-FFI / C-ABI
   / WASI / provenance / attestation / MCP-caps / transparency-log Rust proofs run
   in the cross-OS `cargo test` matrix.
4. **Validation runway (S71–S79)** — **11/11 runway gates pass**: Paper VI Exp 3,
   self-hosted parser seed, VM/interp parity (33/33), safe-subset spec,
   formal-verification feasibility, stdlib promotion, external-package pilot,
   governance/RFC, positioning reframe (+ S69 LLM-suggest, S70 version-map).

## What v0.8.0 is (honest)

A **research-grade-prototype milestone** capturing the S30–S80 trust-kernel +
validation work: the capability surface → manifest → diff-caps → seal → sandbox
policy → transparency-log spine, the native-interop authority boundary, the
agent-authorship provenance, the self-hosting seed, backend parity, and the
governance/positioning reframe. It is **not** a production or 1.0 release.

## Deferred for v0.8.0 (honest)

- Runtime sandbox **enforcement** (S46 generates policy; does not enforce).
- External supply-chain **signing** lanes (GPG/cosign — credentials absent).
- OpenVSX / Marketplace **publish** (needs `OVSX_TOKEN`/`VSCE_PAT`).
- **WASM build** + browser playground execution (wasm32 toolchain absent).
- **LLM advisory tier** (rules tier ships; LLM tier pending-infra).
- Empirical **Paper VI** measurements / mechanized proofs (h₃a is a 6.5% partial;
  the formal-verification path is a feasibility study, not a proof).

## The tag decision (escalated)

Cutting and pushing the `v0.8.0` git tag is **Jon's call** — it is the release of
the whole completion run, irreversible, and reserved by the honesty anchors. When
approved, the mechanical step is `git tag -a v0.8.0 <commit> -m "..." && git push
origin v0.8.0` (which also triggers the tag-gated release-asset + VSIX-publish
workflows). This slice ships the cut-readiness gate and stops there.
