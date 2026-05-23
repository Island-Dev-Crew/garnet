# PRD C — S17: Stdlib Expansion + Layer Policy + `@stability` Annotation

| Field | Value |
|---|---|
| **Slot** | win-opus (Claude Code Opus 4.7 1M Max, Windows) |
| **Slice** | S17 |
| **Status** | not-started → planned → in-progress → review-ready → dogfood-passing → merged |
| **PR count** | 1 (or 2 if you choose to split spec doc + impl) |

---

## Goal

Codify Garnet's **five-layer stdlib model**. Expand the prelude + `std::` from
~23 primitives to ~50. Add a first-class **`@stability(...)`** annotation that
the compiler enforces.

## Why Windows Opus

Spec-heavy work + careful checker integration. Opus 4.7's architectural reasoning
fits the design-first nature of this slice. The Layer Policy document is
load-bearing — it will be scrutinized by sharp reviewers (MIT-style); precision
matters more than velocity here.

## Owned crates (writable)

- `garnet-stdlib` — primary expansion target
- `garnet-check-v0.3` — add `@stability` annotation enforcement (small, scoped to
  the new attribute)

## Read-only crates

- `garnet-parser-v0.3` — just add `@stability` to the attribute parser if needed.
  **May require Handoff Request** to mac-opus depending on attribute parsing depth.
- `garnet-interp-v0.3` — consumes stdlib; must keep working
- `garnet-vm` — consumes stdlib; must keep working
- `garnet-cli` — no changes
- `garnet-lsp` — no changes
- `garnet-cst` — no changes

## Dependencies

- **None upstream**. This slice can start immediately.
- **Possible Handoff**: if `@stability(...)` requires new attribute syntax that
  the parser doesn't already handle, file a Handoff Request to `mac-opus`
  (garnet-parser owner).

---

## Implementation Plan

### 1. Layer Policy spec document

`C_Language_Specification/GARNET_STDLIB_LAYER_POLICY.md` — formal document.

Required sections:

**§1 Five-layer model**
- **Layer 0 (`core::`)** — always available, no `@caps` ever, compute only
- **Layer 1 (`std::`)** — bundled, capability-gated, conservative API
- **Layer 2 (`@garnet-lang/*`)** — official packages, versioned independently
- **Layer 3 (`community/*`)** — registry-endorsed first 10–20
- **Layer 4 (`*`)** — anyone publishes

**§2 Promotion criteria** (when does an `@garnet-lang/*` package graduate to `std::`?)
- Two minor releases at `@stability(stable)` without breaking changes
- ≥80% test coverage as measured by the standard reporter
- A documented use case in the official examples
- A vote of confidence from the maintainer (single human gate during v0.x)

**§3 Deprecation policy**
- Mark `@stability(deprecated)` with a `migration` hint
- Emit warnings on use for two minor releases
- Remove only at next major

**§4 Stability semantics**

| Tier | Meaning | Breaking changes allowed? |
|---|---|---|
| `stable` | API contract held for the major version | No, until next major |
| `experimental` | API may change between minor versions | Yes, between minor |
| `frozen` | No further changes; will be deprecated | No, never |
| `deprecated` | Scheduled for removal | No new uses |

**§5 First-order principle** (the defensible reviewer answer)
> Capability surface + spec volatility = layer assignment. JSON: zero caps,
> RFC 8259 stable → `std::`. LLM client: `@caps(net)`, provider APIs change
> quarterly → `@garnet-lang/`. Different blast radii, different layers.

### 2. `@stability` annotation

Syntax (already valid attribute form):

```garnet
@stability(stable)
def split(s: String, sep: String) -> Array<String> { ... }

@stability(experimental)
def http_get(url: String) -> Result<Response> @caps(net) { ... }

@stability(frozen)
@migration("use std::time::now_ms instead")
def time_ms() -> Int { ... }

@stability(deprecated)
def old_fn() { ... }
```

**Parser support**: confirm the attribute machinery handles `stability(...)`.
If yes, no parser changes. If no, file Handoff Request to `mac-opus`.

**Checker enforcement** in `garnet-check-v0.3`:
- A function or primitive lacking explicit `@stability` annotation → **warning**
  (not error — backwards compat)
- A caller references something marked `@stability(experimental)` without
  opting in via `@uses(experimental)` on the caller → **warning**
- A caller references something marked `@stability(deprecated)` → **warning**
  with migration hint if provided
- A caller references something marked `@stability(frozen)` → **info** only
  (frozen is allowed to use, just won't grow)

### 3. Annotate every existing stdlib primitive

Read all current primitives from `garnet-stdlib/src/registry.rs`. Add explicit
`@stability(...)`:

- Primitives that have been in 2+ minor releases → `@stability(stable)`
- Primitives added in the current release → `@stability(experimental)`

### 4. Stdlib expansion to ~50 primitives

**Layer 0 additions** (no caps, in `core::`):

| Module | Functions |
|---|---|
| `core::iter` | `map`, `filter`, `fold`, `zip`, `take`, `drop`, `collect`, `chain`, `enumerate` |
| `core::result` | `ok`, `err`, `map`, `and_then`, `or_else`, `unwrap_or` |
| `core::option` | `some`, `none`, `map`, `and_then`, `unwrap_or` |
| `core::cmp` | `min`, `max`, `ordering`, `clamp` |
| `core::math` | `abs`, `sqrt`, `pow`, `floor`, `ceil`, `round` |

**Layer 1 additions** (capability-gated, in `std::`):

| Module | Functions | Caps |
|---|---|---|
| `std::env` | `get`, `set`, `vars` | **new `@caps(env)`** |
| `std::process` | `spawn`, `wait`, `exit_code` | `@caps(proc)` |
| `std::json` | `parse`, `stringify`, `get`, `set` | none — pure |
| `std::regex` | `compile`, `match`, `find_all`, `replace` | none — pure |
| `std::uuid` | `new_v4`, `new_v5`, `new_v7` | `@caps(time)` for v4/v7, none for v5 |
| `std::base64` | `encode`, `decode` | none — pure |
| `std::log` | `info`, `warn`, `error`, `debug` | none for format, `@caps(fs)` for file sinks |

All new primitives ship with `@stability(experimental)` for v0.7; promote to
`stable` in v0.8 if no breaking change required.

### 5. New capability: `@caps(env)`

Add to `garnet-check-v0.3`'s capability set alongside `fs`, `net`, `time`, `proc`.

**Backwards compat**: existing programs without `@caps(env)` cannot call
`std::env` functions. This is the desired behavior — explicit opt-in.

### 6. New readiness lane

`scripts/garnet_stdlib_layer_gate.py`:

- Total primitives by layer
- % of primitives with explicit `@stability` annotation
- Deprecated primitives with their removal target version
- Hook into `garnet_mit_readiness_status.py` as a new lane

---

## Dogfood block (verification)

```bash
cargo build -p garnet-stdlib -p garnet-check-v0.3 --release
cargo test -p garnet-stdlib -p garnet-check-v0.3 --no-fail-fast
python3 scripts/garnet_stdlib_layer_gate.py

# verify:
#   - total ≥ 50 primitives
#   - ≥ 95% with explicit @stability
#   - layer-policy doc exists at GARNET_STDLIB_LAYER_POLICY.md

# backwards-compat check:
garnet check examples/mvp_01_*.garnet
# verify: no unexpected diagnostics on existing examples
```

---

## Out of scope

- Implementing Layer 2 packages (S18's job — mac-codex).
- Adding the LLM-tier compiler-as-agent (S19's job — mac-codex).
- Migrating existing stdlib internals to use the new `core::iter` / `core::result`
  APIs (refactor slice for v0.8).
- Building a documentation site for the stdlib (v0.8 work).

---

## Coordination

- **If `@stability` requires parser work** beyond what's already supported, STOP
  and file a Handoff Request to `mac-opus`.
- The new `@caps(env)` and `@caps(proc)` capabilities should be added to the
  central capability set in `garnet-check-v0.3`. mac-codex (S19) will add
  `@caps(net)` enforcement on LLM clients — coordinate via the ledger if cap
  set definitions conflict.
- mac-codex (S18) will publish first five Layer-2 packages under `@garnet-lang/`.
  **The Layer Policy doc you write IS the spec they code against.** Notify
  mac-codex via the Shared Messages section of the ledger when your doc is
  draft-complete (even before your full PR lands).

---

## Honest accounting hooks

- "Stdlib expanded from 23 to ~50 primitives in v0.7; many are
  `@stability(experimental)` and may evolve in v0.8."
- "`@stability` enforcement is at warning level, not error level, for backwards
  compat. Error-level enforcement is v0.8 work."
- "Layer 2 packages (LLM client, HTTP client, etc.) are NOT bundled with v0.7
  binaries; they ship via the registry as `@garnet-lang/*` packages."

---

## Done criteria

- [ ] PR merged with green CI.
- [ ] `AGENT_COORDINATION_LEDGER.md` updated: win-opus / S17 / MERGED.
- [ ] Stdlib has ≥ 50 primitives.
- [ ] ≥ 95% of primitives have explicit `@stability(...)` annotations.
- [ ] `garnet_stdlib_layer_gate.py` reports its first numbers.
- [ ] `GARNET_STDLIB_LAYER_POLICY.md` exists and is referenced from
  `CURRENT_STATE.md`.
- [ ] `CHANGELOG.md` updated under `[Unreleased]`.
