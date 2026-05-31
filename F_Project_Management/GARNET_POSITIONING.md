# Garnet positioning — the honest reframe (S79)

This is the canonical messaging for the website and deck. It leads with the
**integration + agent-code thesis** rather than pillar-by-pillar novelty, makes
**diff-caps** the headline, and **concedes precedent honestly** — per the
trajectory research's positioning recommendations.

## The one-sentence thesis

> Garnet is an **evidence-native language for code that agents write and humans
> accept**: it makes a function's authority, resource bounds, and provenance
> *machine-checkable and diffable*, so an AI-authored change can be reviewed by
> its capability surface instead of line by line.

## Lead with integration, not pillars

The individual pillars are **well-precedented**, and the prior art is often
stronger — say so:

| Pillar | Stronger prior art | Garnet's honest stance |
|---|---|---|
| Capability annotations (`@caps`) | Austral linear capabilities; E/Monte ocap; Koka effects | Pragmatic, legible middle ground; borrow rigor (S74 proposes a linear/effect mode). |
| Bounded execution (`@bounded`) | Wasmtime fuel; eBPF verifier | Language-level *contract*; lower enforcement to a fuel target (wrap, don't rebuild). |
| Signed provenance (`seal`) | Sigstore + SLSA + in-toto + Go sumdb | Coherent packaging; emit in-toto predicates + SBOM, don't ask for a bespoke format. |

**The genuine novelty is the *combination* — capability annotations + bounded
execution + sealed provenance + capability diffing — for code agents write.** No
existing language combines all of these for that workflow.

In one line: **Garnet's bet is the integration, not the parts** — the combination
targeted at agent-authored code.

## The headline: diff-caps

`diff-caps` answers, in one screen, **"what new authority does this change
grant?"** — across a dependency update or an agent's PR. API-surface diffs
(cargo-semver-checks, go-apidiff) and permission diffs (Android) exist; a
*capability/authority* diff as an acceptance gate has **no cross-language
equivalent**. It directly attacks the review bottleneck — the binding constraint
on accepting AI-written code (Zig banned AI PRs over *review*, not generation).

## The "why a new language?" answer (Lattner's challenge)

Checkable, diffable capability/bound/provenance **evidence** needs language-level
constructs a Rust/Python linter can't fully provide — the dual-mode grammar and
`@bounded` semantics are the parts that genuinely require a language. Be ready to
prove that, and concede everything else to existing tooling.

## Honest scope (do not soften)

- Capability security, bounded execution, and signed provenance are
  **well-precedented**; this reframe says so plainly rather than claiming
  pillar-by-pillar novelty.
- "diff-caps is the headline" is a **positioning** claim about novelty and fit,
  not a production-readiness or 1.0 claim. Garnet remains a research-grade
  prototype (v0.x).
- `scripts/garnet_positioning_status.py --gate` checks that this doc and the
  landing page both carry the integration thesis + the diff-caps headline + the
  precedent concession, so the messaging cannot silently drift back to
  pillar-first marketing.
