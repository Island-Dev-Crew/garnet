# PRD D — S18 (First Five Layer-2 Packages) + S19 (Compiler-as-Agent LLM Tier)

| Field | Value |
|---|---|
| **Slot** | mac-codex (Codex Desktop GPT-5.5 Pro Extra High Fast, macOS) |
| **Slices** | **S18** (sequence first) + **S19** (after S18 mid-stage or in parallel late) |
| **Status** | not-started → planned (waiting for S17) → in-progress → review-ready → dogfood-passing → merged |
| **PR count** | 6 in main repo (5 stub commits + 1 suggest-llm crate) + 5 external repo creations |

---

## Goal

- **S18**: Publish the first five Layer-2 packages under
  `github.com/garnet-lang/`. These are the registry-seed content that demonstrates
  the ecosystem actually works.
- **S19**: Ship the LLM-backed tier of compiler-as-agent. The seam is already in
  S10's `suggest.rs` — you're filling in the `LlmClient` trait and three impls.

## Why Mac Codex

S18 is implementation-bulk with sequential dependencies (five packages, each
with README + CHANGELOG + working code + dogfood test). GPT-5.5 Pro's thoroughness
fits. S19 is implementation-bulk too — the seam is designed; you're filling in
known interfaces.

## Owned crates / repos (writable)

**For S18 (external)**: `github.com/garnet-lang/*` — new repos under a new GitHub
org. Five repos:
- `http-client`
- `llm`
- `cli`
- `test-property`
- `log`

**For S19 (in main repo)**: NEW crate `garnet-suggest-llm/`, feature-flagged
(behind `llm` cargo feature). Not built by default.

## Read-only crates

- `garnet-check-v0.3` — after S17 lands, `@stability` annotations available
- `garnet-stdlib` — after S17 lands, `@stability` annotations available
- All other crates

## Dependencies

- **HARD for S18**: S17 must MERGE first. S18 packages annotate their primitives
  with `@stability(...)` per the Layer Policy doc S17 produces. Watch for
  `win-opus / S17 / MERGED` in the ledger before starting S18 substantive work.
- **Soft for S19**: S17 should land before S19's `LlmClient` trait declares its
  stability tier. If S17 hasn't landed yet, ship S19 with TODO comments noting
  the stability tier and circle back.

---

## Implementation Plan — S18

### 1. Bootstrap `github.com/garnet-lang/` org

- Create the org if it doesn't exist. **Jon's manual step** — prompt via the
  Shared Messages section of the ledger if the org isn't already created.
- Standard org settings: dual-license MIT OR Apache-2.0, `CODE_OF_CONDUCT.md`
  (Contributor Covenant), `SECURITY.md` template.

### 2. Repo template

In main garnet repo, add `tools/garnet-lang-template/` — a scaffold:

```
tools/garnet-lang-template/
├── README.md          (calibrated-honesty voice template)
├── LICENSE-MIT
├── LICENSE-APACHE
├── CHANGELOG.md       (template with [Unreleased] section)
├── Garnet.toml        (with @stability tier declared)
├── garnet/            (src dir)
│   └── lib.garnet
└── tests/
    └── smoke.garnet   (at least one dogfood test)
```

### 3. Publish five packages

Each repo created independently. Each ships a working v0.1.0.

#### `garnet-lang/http-client` — `@caps(net)`

API surface:
- `get(url) -> Result<Response>`
- `post(url, body) -> Result<Response>`
- `put(url, body) -> Result<Response>`
- `delete(url) -> Result<Response>`
- Headers, body (string + bytes), status code, response body access
- Wraps `reqwest` via a thin FFI shim until self-hosted HTTP arrives (v0.8+)
- `@stability(experimental)` for v0.7

#### `garnet-lang/llm` — `@caps(net)` — **the differentiator**

```garnet
@stability(experimental)
trait LlmClient {
    @caps(net)
    fn complete(prompt: String, max_tokens: Int) -> Result<String, LlmError>
}
```

Three implementations:
- `AnthropicClient` — wraps Anthropic API
- `OpenAiClient` — wraps OpenAI API
- `OllamaClient` — local model via Ollama HTTP

Helpers:
- `Prompt::new("Hello, {name}!").bind("name", "world")` — template helper
- `complete_json::<T>(prompt) -> Result<T, LlmError>` — structured output

#### `garnet-lang/cli` — no caps

Argument parser (clap-equivalent for Garnet):
- Sub-commands, flags, positional args, env vars, help generation
- `@stability(experimental)`

#### `garnet-lang/test-property` — no caps

Property-based testing primitives (proptest-equivalent):
- Generators for primitives, collections, structs
- Shrinking
- `@stability(experimental)`

#### `garnet-lang/log` — mixed caps

Structured logging:
- `info!`, `warn!`, `error!`, `debug!`, `trace!`
- Sinks: stderr (no caps), file (`@caps(fs)`), JSON
- `@stability(experimental)`

### 4. Registry publish

Each package, once shipped, gets added to the registry stub (S13's `index.json`)
via:

```bash
garnet add --registry garnet-lang/<name>
```

### 5. Smoke test

A single Garnet program that imports all five packages, uses one primitive from
each, runs successfully via `garnet run`:

`examples/mvp_18_all_official_packages.garnet`

---

## Implementation Plan — S19

### 1. NEW crate `garnet-suggest-llm/` (in main repo, feature-flagged)

```toml
[features]
default = []
llm = []
```

- Depends on `garnet-check-v0.3` for the `Suggestion` type.
- Depends on `garnet-lang/llm` package for the `LlmClient` trait.

### 2. API surface

```rust
/// Additive — does not replace the deterministic suggest_for_module.
pub fn suggest_for_module_with_llm(
    module: &Module,
    history: Option<&CompilationHistory>,
    client: &dyn LlmClient,
) -> Vec<Suggestion>
```

### 3. Behavior

1. First, run the deterministic rules-tier (S10).
2. Then, build a prompt that includes:
   - The full source.
   - The deterministic rules-tier findings as ground truth — *the LLM is told
     "the deterministic analyzer already found N issues; here they are; suggest
     additional improvements the deterministic analyzer can't catch."*
   - The Compilation History (from `.garnet-cache/episodes.log` — Paper VI Exp 3).
3. LLM responds with additional suggestions.
4. Suggestions are tagged `@stability(non-deterministic)`.

### 4. CLI integration

```bash
garnet check --suggest                         # S10 only (deterministic)
garnet check --suggest --llm anthropic         # S10 + LLM tier
garnet check --suggest --llm anthropic --llm-budget 100000
```

Without `--llm`: behavior identical to S10. With `--llm`: deterministic
findings + LLM findings, **clearly separated** in output.

### 5. Reproducibility log

`.garnet-cache/llm-suggest-log.jsonl`. Per call:

```json
{
  "prompt_hash": "blake3:...",
  "model": "claude-opus-4-7",
  "temperature": 0.2,
  "response": "...",
  "suggestions_emitted": [...],
  "timestamp": "2026-..."
}
```

### 6. Capability discipline

- `LlmClient` trait declares `@caps(net)`.
- Capability propagates transitively to any function that uses the client.
- `garnet check --suggest --llm` itself opts the invocation into `@caps(net)`.

### 7. Determinism non-claim

CI determinism gate (S9) does **NOT** run with `--llm`. This must be explicit
in code and in CHANGELOG. Add a CI check that errors if a determinism job is
ever spawned with `--llm` in its command.

### 8. Token budget

- Default: 50K tokens per `--suggest --llm` run
- `--llm-budget <N>` to override
- `--llm-budget unbounded` for explicit opt-out (logs a warning)

### 9. Paper VI Exp 3 wiring

Add `benchmarks/paper_vi_exp3_compiler_as_agent/`:
- `codebase_versions/` — 10 snapshots of an evolving project
- `run_stateless.sh`, `run_history_aware.sh`
- `aggregate.py`, `analyze.py` per the Paper VI protocol

Shipping the harness is in scope. **Running the experiment** to produce h₃a /
h₃b / h₃c results is a separate v0.7.1 task — note this honestly.

---

## Dogfood block (verification)

### S18

```bash
# Each package repo: cd into it, run its own CI
for pkg in http-client llm cli test-property log; do
    (cd ../garnet-lang-$pkg && cargo test)
done

# In main repo:
garnet add --registry garnet-lang/http-client
garnet add --registry garnet-lang/llm
garnet add --registry garnet-lang/cli
garnet add --registry garnet-lang/test-property
garnet add --registry garnet-lang/log
garnet run examples/mvp_18_all_official_packages.garnet
```

### S19

```bash
cargo build --features llm -p garnet-suggest-llm --release

# Set ANTHROPIC_API_KEY or OPENAI_API_KEY or OLLAMA_HOST.
garnet check --suggest --llm anthropic examples/mvp_03_*.garnet
# Expected: deterministic suggestions + non-deterministic LLM suggestions,
# clearly labeled, each with stability tag.
# Reproducibility log appears at .garnet-cache/llm-suggest-log.jsonl.

# Verify the log is real:
tail -1 .garnet-cache/llm-suggest-log.jsonl | python3 -m json.tool
```

---

## Out of scope

- Self-hosted HTTP server in Garnet (the http-client wraps reqwest via FFI;
  pure-Garnet HTTP is v0.8 work).
- LLM streaming responses (v0.7 ships `complete()` only; streaming is v0.8).
- Vector store integration (separate Layer-2 package, deferred to v0.7.1).
- Provider-specific edge features (function calling, tools, vision). v0.7 ships
  `complete()` only.
- **Running** Paper VI Exp 3 to produce h₃ results — the harness ships in
  v0.7; running the experiment is v0.7.1.

---

## Coordination

- **S18 cannot start substantive work until S17 / MERGED in ledger.** Watch for
  win-opus's MERGED entry.
- **S19's `LlmClient` trait** is shared with `garnet-lang/llm` package (S18).
  Define it once in the Layer-2 package and re-export from `garnet-suggest-llm`.
- `garnet-check-v0.3` is **read-only** for you. If you need a new diagnostic
  surface, file a Handoff Request to win-opus.

---

## Honest accounting hooks

- "Five Layer-2 packages ship in v0.7 as `@stability(experimental)`."
- "S19's LLM tier is non-deterministic; the determinism CI gate does not run
  with `--llm`. This is explicit by design and noted in CHANGELOG."
- "Streaming, function calling, and vision are NOT in v0.7; they're v0.8 work."
- "Paper VI Experiment 3 (compiler-as-agent time-to-fix) harness ships in v0.7;
  running the experiment to produce h₃ results is a separate v0.7.1 task."

---

## Done criteria

### S18
- [ ] Five package repos created under `garnet-lang/`, each with green CI.
- [ ] Each package published to registry stub `index.json`.
- [ ] `examples/mvp_18_all_official_packages.garnet` runs end-to-end.
- [ ] `AGENT_COORDINATION_LEDGER.md` updated: mac-codex / S18 / MERGED.

### S19
- [ ] `garnet-suggest-llm` crate merged, feature-flagged.
- [ ] All three provider impls (Anthropic, OpenAI, Ollama) working.
- [ ] Reproducibility log writes to `.garnet-cache/llm-suggest-log.jsonl`.
- [ ] Paper VI Exp 3 harness in place at `benchmarks/paper_vi_exp3_compiler_as_agent/`.
- [ ] CI check prevents determinism job from spawning with `--llm`.
- [ ] `AGENT_COORDINATION_LEDGER.md` updated: mac-codex / S19 / MERGED.
- [ ] `CHANGELOG.md` updated under `[Unreleased]`.
