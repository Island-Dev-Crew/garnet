# Garnet model/prompt/tool attestation (S66)

S65 added a flat AI-authorship string (`--authored-by`). S66 adds a **structured
attestation block** to the same `garnet seal` predicate: the model, the prompt,
and the tools behind a build — the supply-chain answer to "what AI pipeline
produced this, and what could it touch?"

## Usage

`garnet seal <file> --attest <key>=<value>` (repeatable) records an
`"attestation"` object in the predicate:

```sh
garnet seal app.garnet \
  --authored-by "ai:claude-opus-4-8" \
  --attest model=claude-opus-4-8 \
  --attest prompt_sha256=abc123… \
  --attest tool=mcp:filesystem
# predicate.predicate.attestation == {"model":"claude-opus-4-8",
#   "prompt_sha256":"abc123…","tool":"mcp:filesystem"}
```

Conventional keys (free-form, not enforced):

- `model` — the generating model (`claude-opus-4-8`),
- `prompt_sha256` — a hash of the prompt (reference, not the prompt itself),
- `tool` — an MCP/tool the pipeline could use (`mcp:filesystem`); repeat or
  comma-join for several.

The block is **deterministic** (keys sorted), rides inside the same predicate as
the capability manifest and the authorship string, and is therefore diffable
(S37) and signable (`cosign attest`, S51). Together: *who* (`authorship`, S65),
*what pipeline* (`attestation`, S66), and *what authority* (`capability_manifest`,
S35–S38).

## Provenance seal chain (S97)

`garnet seal <file> --provenance-chain` validates the conventional attestation
keys `agent`, `model`, and `prompt_sha256`, then binds them to the current seal's
`source_blake3` and subject `artifact_blake3` (`build_manifest.ast_hash`). The
predicate gains a deterministic `"provenance_chain"` object:

```sh
garnet seal app.garnet \
  --authored-by "ai-assisted:gpt-5" \
  --attest agent=win-codex \
  --attest model=gpt-5 \
  --attest prompt_sha256=sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef \
  --provenance-chain
```

The chain records:

- `schema: "garnet-provenance-chain-v1"`,
- `agent`, `model`, and canonical `prompt_sha256`,
- `artifact_blake3` and `source_blake3` from the live seal,
- `chain_blake3`, a deterministic BLAKE3 over the declared chain plus sorted
  attestation pairs,
- `binding_verified: true`,
- `independent_origin_verified: false`.

This is a verification of **binding**, not a claim of independent origin proof.
It proves that the declared agent/model/prompt metadata is present, canonical,
and tied to the artifact currently being sealed. It does not prove that a model
actually executed that prompt, that the named agent produced the file, or that
the declared tool list is complete.

## Honest scope (do not soften)

Every field is **self-declared**, **not verified** — the same posture as `@caps`
and `--authored-by`. Garnet does not introspect the model, hash the live prompt,
or enumerate the tools an agent actually invoked; it records what the toolchain
*declares*. An absent `--attest` records **no** attestation block (default shape
unchanged). The value is a truthful, attestable, signable channel for the
declaration; auditing the declaration's accuracy is a process question, out of
scope for the tool. Bringing the *capability* lens to those declared tools is S67.

## Source-hash determinism — the canonicalization contract (S82)

The seal predicate's `source_blake3` is the **BLAKE3 of the source bytes after
line-ending normalization to LF** (`\r\n` → `\n`). This is the canonicalization
contract:

- **Why.** Hashing raw bytes made the full predicate diverge between an LF
  (Mac/Linux) checkout and a CRLF (Windows `core.autocrlf`) checkout of the *same
  logical source* (WIN-S38-001): the AST subject stayed identical, but
  `source_blake3` — and therefore the predicate digest — changed.
- **The fix (two layers).** (1) `Manifest::build` hashes `normalize_source_eol(source)`
  so the hash is LF/CRLF-stable; normalization is **idempotent on LF**, so existing
  LF seals are unchanged. (2) `.gitattributes` pins `*.garnet text eol=lf` as
  defense-in-depth, so checkouts are LF regardless.
- **Scope.** Only line endings are canonicalized. Other whitespace (indentation,
  trailing spaces) still changes `source_blake3` by design — the AST hash
  (`ast_hash`) is the shape-stable digest; `source_blake3` is the exact-source
  digest modulo line endings.

`scripts/garnet_seal_determinism_status.py --gate` enforces the pin + the
in-code normalization + this documented contract.
