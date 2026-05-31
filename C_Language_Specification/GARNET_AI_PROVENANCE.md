# Garnet AI-authorship provenance (S65)

As AI writes more code, "who (or what) wrote this?" becomes a supply-chain fact
worth attesting — the same way capabilities (S35–S38) are. S65 makes AI-authorship
provenance a **first-class, attestable declaration** recorded in the `garnet seal`
in-toto predicate.

## The model

`garnet seal <file> --authored-by <provenance>` records an `"authorship"` field
in the predicate:

```sh
garnet seal app.garnet --authored-by "ai:claude-opus-4-8"
# predicate.predicate.authorship == "ai:claude-opus-4-8"
```

The provenance string is free-form and conventionally `kind:detail`:

- `ai:<model>` — authored by an AI model (e.g. `ai:claude-opus-4-8`),
- `ai-assisted:<model>` — human-driven with AI assistance,
- `human:<who>` — human-authored.

Because it lives in the seal predicate, AI-authorship is **diffable, reviewable,
and signable** (`cosign attest`, S51) exactly like the capability surface — it
travels with the artifact, not in a side channel.

## Honest scope (do not soften)

This is a **self-declared** provenance fact, **not AI-detection**. Garnet does
not (and cannot reliably) infer whether code was AI-written; `--authored-by`
records what the author/toolchain *declares*, the same posture as `@caps`
(declared authority, not inferred). An absent `--authored-by` records **no**
authorship claim — silence is honest, not an implicit "human". The value is a
truthful, attestable channel for the declaration; verifying the declaration's
accuracy is out of scope (and a social/process question, not a tool guarantee).

Model/prompt/tool attestation (S66) extends this same seal field with richer
structure; MCP/tool capability declarations (S67) bring the capability lens to
the tools an agent uses.
