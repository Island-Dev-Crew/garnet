# {{name}}

A Researcher / Synthesizer / Reviewer orchestrator, generated
with `garnet new --template agent-orchestrator`.

## Shape

Three roles, each mapped to a first-class memory kind in the full agent
architecture:

| Actor        | Memory kind  | Access pattern                       |
|--------------|--------------|---------------------------------------|
| Researcher   | episodic     | append-only reasoning trace          |
| Synthesizer  | semantic     | vector-indexed fact retrieval        |
| Reviewer     | procedural   | versioned decision workflow          |

The `memory episodic` / `memory semantic` / `memory procedural` keywords
are Paper VI Contribution 4 — "kind-aware allocation as a language-level
declaration." The runtime picks the allocator; the author declares intent.

The generated `src/main.garnet` now uses managed actor addresses directly:
`spawn Researcher`, `spawn Synthesizer`, and `Reviewer.spawn(2)`. The starter
project exercises `ask`, `try_tell`, `drain`, `mailbox_size`, and actor-local
memory stores while remaining runnable offline with `garnet run`.

## BoundedMail

The managed interpreter supports bounded actor mailboxes through
`Actor.spawn(capacity)`. This template creates the reviewer with capacity `2`
and proves backpressure with `try_tell`: messages over capacity are rejected
instead of growing the mailbox without bound. The full OS-thread actor runtime
bridge remains a later production milestone; this starter is the executable
reference semantics.

## Capability model

```toml
[caps]
allowed = ["mem"]
```

The generated `main` declares `@caps(mem)`: the three actors each own a
`memory` tier (`episodic trace`, `semantic facts`, `procedural workflow`), and
a memory declaration is store construction under the `mem` capability (ADR
0011) — `garnet check` charges `main` for every declared tier, and at run time
the load traps before any store is built unless `main` declares `mem`. The
program touches no file, network, process or clock authority. If you extend it
to read or write fact files, annotate the I/O function with `@caps(fs)` and
declare `fs` on `main` too: `garnet check` reports a `main` whose named,
acyclic calls reach `fs` without declaring it, and at run time the file
primitives trap unless `main` declares it. Nothing reads the `[caps]` list yet;
update it by hand so it documents the budget.

## Run

```sh
garnet run src/main.garnet
```

## Tests

```sh
garnet test
```

The generated tests cover the actor pipeline, bounded mailbox backpressure, and
the bridged BLAKE3 primitive used to fingerprint episodic entries.

See `Paper_IV_Garnet_Agentic_Systems_v2_1_1.docx` for the full agent-native
architecture and `Paper_VI_Garnet_Novel_Frontiers.md` for the research
contributions this template is moving toward.
