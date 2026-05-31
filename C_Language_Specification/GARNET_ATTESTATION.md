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

## Honest scope (do not soften)

Every field is **self-declared**, **not verified** — the same posture as `@caps`
and `--authored-by`. Garnet does not introspect the model, hash the live prompt,
or enumerate the tools an agent actually invoked; it records what the toolchain
*declares*. An absent `--attest` records **no** attestation block (default shape
unchanged). The value is a truthful, attestable, signable channel for the
declaration; auditing the declaration's accuracy is a process question, out of
scope for the tool. Bringing the *capability* lens to those declared tools is S67.
