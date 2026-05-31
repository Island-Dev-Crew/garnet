# S45 Plan — package resolver / slopsquatting guard

Contract: `F_Project_Management/GARNET_v0_8_SLICE_DOGFOOD.md` → S45.
Map: reconciled plan §146 (+ §202-208 threat corroboration) — "package resolver
/ slopsquatting guard (live threat)."
Branch: `codex/s45-slopguard`. Base: `origin/main` @ `ceb8649` (S44).

## Threat
AI-generated code references a *hallucinated* package name that is a near-miss
of a real one; attackers pre-register it. Guard: when a requested name is
unknown but closely resembles a known one, warn before trusting it.

## Deliverables
- `garnet-registry-stub/src/slopguard.rs` (pure): `osa_distance` (Damerau–
  Levenshtein / OSA), `nearest(query, known, max_distance) -> Vec<Suspicion>`
  with `SuspicionKind::{SeparatorConfusable, EditDistance(d)}`, deterministic
  ordering (confusable first, then distance, then name); length-relative
  threshold (`d < longer_len`) suppresses unrelated short names. 6 unit tests.
- `RegistryIndex::known_names()` — the corpus accessor.
- `garnet-cli/cmd/add.rs::run_registry_add`: on `resolve` NotFound for an
  unknown *name* (guard with `index.packages.contains_key`), enrich the error
  with near-miss hints + the slopsquatting caution. Exit code unchanged.

## Dogfood
- Reuse `tests/registry_add.rs` fixtures: `garnet add --registry <dir>
  hello_lbi@0.1.0` (transposition of `hello_lib`) → fails + "did you mean
  `hello_lib`? … slopsquatting". `hello_lib@9.9.9` (version miss) → fails, no
  slop warning.

## End-state / gates
- Full ladder green; CHANGELOG + contract S45 block. Ledger: `s44 → merged(5)`
  advanced this branch; `s45` advance rides with the S46 PR.

## Honest scope
- Registry is a filesystem stub → "known names" = local index, not a global
  feed. Prompt-to-verify heuristic, not a security guarantee.
- No new readiness lane (not mandated).
