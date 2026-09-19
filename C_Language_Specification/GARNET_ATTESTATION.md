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
- `tool` — an MCP/tool the pipeline could use (`mcp:filesystem`); comma-join
  several tools. Each key may appear once; duplicate keys are rejected.

The block is **deterministic** (keys sorted), rides inside the same predicate as
the capability manifest and the authorship string, and is therefore diffable
(S37) and signable (`cosign attest`, S51). Together: *who* (`authorship`, S65),
*what pipeline* (`attestation`, S66), and *what authority* (`capability_manifest`,
S35–S38).

## Provenance seal chain (S97)

`garnet seal <file> --provenance-chain` validates the conventional attestation
keys `agent`, `model`, and `prompt_sha256`, then binds them to the current seal's
`source_blake3` and subject `artifact_blake3` (both the normalized-source
hash in seal/v2). The historical seal/v1 subject used the capability-blind
`build_manifest.ast_hash`; current seals bind source directly, so a capability
edit changes the subject. The
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

- `schema: "garnet-provenance-chain-v2"`,
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

## Scope (do not soften)

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
  so the hash is LF/CRLF-stable; normalization is **idempotent on LF**, so the normalized source digest is unchanged for existing
  LF inputs (the seal/v2 envelope is an explicit format change). (2) `.gitattributes` pins `*.garnet text eol=lf` as
  defense-in-depth, so checkouts are LF regardless.
- **Scope.** Only line endings are canonicalized. Other whitespace (indentation,
  trailing spaces) still changes `source_blake3` by design — the AST hash
  (`ast_hash`) is the shape-stable digest; `source_blake3` is the exact-source
  digest modulo line endings.

`scripts/garnet_seal_determinism_status.py --gate` enforces the pin + the
in-code normalization + this documented contract.

## Checked acceptance and versioned source identity (T3, U-118)

`garnet seal` first resolves the source edition, parses, and applies the same
`check_module` fatal/advisory decision as `garnet check`. A fatal diagnostic
returns nonzero before stdout or an output file is written. An existing output
file is left unchanged on that rejection; callers must honor the exit status.
Advisories are printed on stderr and do not prevent sealing.

The current predicate type is `https://garnet-lang.org/attestation/seal/v2`.
Its `subject[0].digest.blake3` is `build_manifest.source_hash`, and its predicate
records `subject_identity: "garnet-source-lf-blake3-v1"`. Source identity uses
UTF-8 source after CRLF-to-LF normalization, with no other whitespace or comment
normalization. The capability-blind AST digest remains in the build manifest
for compatibility; it is not current seal identity. The provenance chain uses
`garnet-provenance-chain-v2` with `artifact_blake3` equal to that source subject.
Neither the digest nor the self-declared chain is a signature. This identity
covers one normalized source file, not the complete project configuration,
dependency closure, selected edition, deployment or runtime. Verification uses
the currently resolved edition and checker; it is not proof that the producer
used identical external configuration. Bundle those inputs separately when
that broader reproducibility claim is required.

`garnet verify app.garnet seal.json` resolves the edition, rechecks the source,
and independently regenerates the complete seal binding: source identity,
build fields, declared capabilities and any provenance chain. Producer tooling
availability and self-declared authorship/attestation are preserved metadata;
verification does not execute cosign or prove their truth. Verification accepts
only the exact compact producer serialization with at most one final LF.
Reformatted, duplicate-key, unknown-field, unsupported-version and tampered
seals fail. Generic seal/v1 input is rejected; recreate it from checked source
with the current producer. Minimum Shelf retains its independently byte-pinned
historical flagship v1 predicate, with no change to its archived evidence.

The two-argument command also accepts existing `garnet-manifest-v1` deterministic
manifests, discriminated by their distinct format. Those retain their previous
field and Ed25519 checks. `--signature` on a seal is rejected with an external
verification instruction; successful seal verification reports content binding
only. `--external-band` belongs to the one-path gate and is rejected on the
two-argument route; `--signature` without an artifact is likewise rejected.

`--caps-baseline <old-source>` now enforces no program-wide declared surface expansion on
both verify routes (S37 aggregate gains or a new wildcard). Per-function gains
already present elsewhere in the program remain diff details, not aggregate
expansion. Missing, unreadable, unparsable, checker-invalid, source-free directories or
incompletely walked baseline/current source returns nonzero. Empty, whitespace
and comment-only source files are valid inputs with no declared capabilities. Omissions name
the walk rules and counts; use explicit source roots that exclude build and
VCS directories. An absent flag retains the pending signal. External reviewer
bands are advisory and do not grant merge permission. Verification is not an
atomic filesystem snapshot: callers must freeze source and baseline while it
runs. Capability comparison covers declared surface only, not all runtime
behavior or bound deltas.
