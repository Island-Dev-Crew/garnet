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

The generated `src/main.garnet` uses pure functions instead of `spawn
Researcher::new()` so `garnet run src/main.garnet` works immediately on the
current v0.3 interpreter. Treat actor spawning as the next implementation step,
not as first-run scaffolding.

## BoundedMail

The target actor implementation uses `@mailbox(1024)` so incoming `tell` from
anywhere in the system back-pressures at 1024 in-flight messages with
`SendError::Full` rather than unbounded growth. This is Security V2 §3.

## Capability model

```toml
[caps]
allowed = ["time", "fs"]
```

The generated `main` declares `@caps()` because the starter program is pure. If
you extend it to read/write persistent fact files, annotate the I/O function
with `@caps(fs)` — the CapCaps propagator (v3.4.1) will then propagate the
requirement up to `main` at compile time, forcing the `Garnet.toml` `[caps]`
budget to stay honest.

## Run

```sh
garnet run src/main.garnet
```

## Hot-reload

Agent services often need to ship new reasoning logic without
restarting. Use the v3.5 signed hot-reload primitive:

```garnet
let key = load_signing_key()
researcher.reload_signed(target_version = 2, &key, |old| {
  # Paper VI Contribution 6. BLAKE3 schema fingerprint catches type
  # mismatches; zero message loss measured across 1000 cycles.
  migrate_state(old)
})
```

See `Paper_IV_Garnet_Agentic_Systems_v2_1_1.docx` for the full agent-
native architecture and `Paper_VI_Garnet_Novel_Frontiers.md` for the
seven research contributions this template instantiates.
