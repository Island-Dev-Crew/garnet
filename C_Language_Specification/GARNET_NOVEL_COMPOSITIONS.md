# Garnet Novel Compositions — Paper-VI Contributions, Fused

| Field | Value |
|---|---|
| **Document** | Novel-composition dogfood (descriptive) |
| **Slice** | S20 plus S21/S22 runtime-dispatch extensions |
| **Programs** | `examples/novel_01..05_*.garnet` |
| **Harness** | `scripts/smoke_garnet_novel_compositions.py` (+ `test_*.py`) |
| **Companion** | `scripts/smoke_garnet_studio_domain_matrix.py` (the existing single-concern corpus) |

The v0.5–v0.7 example corpus proves each Paper-VI contribution **in isolation**:
mvp_06 is a deterministic agent pipeline, mvp_11 is a BLAKE3 signed-fingerprint
check, agent_toolbelt_02 is a capability budget, agent_toolbelt_03 is
cognitively-typed memory recall, agent_toolbelt_04 is a release gate. The
interesting question for an agent-native language is what happens when you
**fuse** them. The first three programs do that for modeled Paper-VI patterns;
S21/S22 then extend the same harness to prove newly-dispatched stdlib and
Mnemos-handle surfaces through the CLI.

> **Calibrated-honesty scope.** Like the canonical corpus, these compositions
> **model the patterns deterministically** in managed mode (the proven runnable
> subset — `def`/`match`/`let mut`/`crypto::blake3`/arithmetic). They prove the
> *composition shape* executes and is reproducible; they do **not** stand up the
> live runtimes (actor mailboxes, Mnemos stores, Ed25519 signing) — those remain
> tracked separately (Memory Core roadmap, actor runtime, manifest-sig). The
> value here is demonstrating that the contributions **compose** into coherent
> agentic behaviour, not a claim of production runtime integration.

---

## novel_01 — Capability-budgeted, memory-backed agent

**Fuses:** capability-budget gating · cognitively-typed memory recall · the
researcher→synthesizer→reviewer pipeline.

The agent will only execute its work pipeline for an action that is **both**
within its tool-authority budget **and** supported by recalled prior decisions.
Of three candidate actions, one passes both gates and runs the pipeline (16),
one is denied for exceeding the capability budget, and one is deferred for weak,
stale, unverified memory — final governance score **16**.

**Novel discovery:** capability budgets and a memory of prior decisions are
*independent veto gates* on the same action. Composed, they yield an agent that
is conservative-by-construction: authority alone is insufficient to act, and
recall alone is insufficient to act. This is the seed of *least-authority,
evidence-gated* agent execution — a pattern the standard agentic stack usually
bolts on after the fact, here expressed in the program itself.

## novel_02 — Content-addressed provenance pipeline

**Fuses:** BLAKE3 signed fingerprint · multi-stage pipeline · determinism.

Each pipeline stage extends an append-only lineage string; the BLAKE3 digest
over the whole lineage is the pipeline's **provenance fingerprint**, verified
against an embedded expected hash. Tamper with any stage and the digest changes
and the program raises. Verified fingerprint:
`1f02c414…c325ce`.

**Novel discovery:** because Garnet builds are deterministic, a content hash over
a pipeline's lineage is a *stable identity* for "this exact pipeline produced
this exact artifact." Fusing the signed-fingerprint check (built for hot-reload)
with an ordinary agent pipeline turns it into **tamper-evident build lineage** —
the same primitive secures both code hot-swap and pipeline provenance.

## novel_03 — Multi-signal release-gate quorum

**Fuses:** release-evidence gate · capability budget · BLAKE3 provenance · memory
recall.

A release is **APPROVED** only when a quorum (≥ 3 of 4) of independent signals
agree: CI green, requested tooling within the capability budget, artifact
provenance fingerprint matches, and memory recalls a recent prior green release.
With all four signals go, the verdict is **APPROVED quorum: 4** — and no single
signal can wave a release through.

**Novel discovery:** the four contributions are *orthogonal evidence sources*,
and quorum over them is a governance primitive. This is a concrete, runnable
answer to "how does an agent decide to ship?" that is auditable (every signal is
explicit and deterministic) rather than a black-box judgement — directly
relevant to trustworthy autonomous release in the agentic coding industry.

## novel_04 — Dispatched stdlib pipeline

**Fuses:** first-class iterator combinators · math/cmp standard-library
primitives · base64 content tags.

This program proves the S21 runtime bridge: `core::iter::filter/map/fold`,
`core::math`, `core::cmp`, and `std::base64` execute from Garnet source through
qualified names. It yields deterministic score/tag output rather than only
registry metadata.

**Novel discovery:** once higher-order iterators and content tags are callable
from managed Garnet, a small program can express a reproducible analysis
pipeline whose intermediate computation and final verdict are standard-library
operations, not bespoke demo helpers.

## novel_05 — Stdlib + Mnemos handle pipeline

**Fuses:** `std::json` · `std::regex` · deterministic `std::uuid::new_v5` ·
`std::log` formatting · live `memory::` Mnemos handles.

This program proves the S22 runtime bridge for deterministic surfaces: JSON is
parsed/patched/stringified, regex extracts the signal words, UUIDv5 creates a
stable identity, log formatting records an event, and working/episodic memory
handles carry the data. The expected UUID is
`ee54a926-f375-5759-a5aa-67f7d8528cff`.

**Novel discovery:** the stdlib is no longer just a registry contract; it can
feed live memory handles from Garnet source. That is the first practical shape
of "agent program state" where structured input, text extraction, stable
identity, log records, and recall handles compose without leaving the language.

---

## Why this matters (the story)

Single-feature demos answer "does the feature work?" Compositions answer the
harder, more valuable question: **"do these features combine into something an
agent can be trusted to run?"** The recurring shape across all three —
*independent, deterministic, auditable evidence gates fused into one program* —
is the foundation Garnet offers the agentic and standard coding industries:
capability discipline, content-addressed provenance, typed memory, and
deterministic verdicts that compose rather than conflict. Each program is small,
but the composition is the point.

## Reproduce

```bash
python3 scripts/smoke_garnet_novel_compositions.py   # 5/5 check + deterministic run
python3 -m unittest scripts.test_garnet_novel_compositions
# or individually:
garnet check examples/novel_01_capability_budgeted_memory_agent.garnet
garnet run   examples/novel_05_s22_stdlib_memory_pipeline.garnet
```
