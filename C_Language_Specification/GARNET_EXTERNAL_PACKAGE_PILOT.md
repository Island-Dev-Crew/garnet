# Garnet external package pilot (S77)

Ecosystem maturation: pilot the **external-package** flow end-to-end against the
filesystem registry stub, with the slopsquatting guard in the loop. The runnable
proof is `garnet-registry-stub/tests/external_package_pilot.rs` (runs in the
`cargo test --workspace` matrix on every OS).

## The piloted flow

1. **Publish.** An external package `acme-logger/1.0.0/lib.garnet` is placed into
   a filesystem registry (`<root>/<name>/<version>/<files>`).
2. **Resolve.** `build_index` indexes it; `resolve(index, "acme-logger", "1.0.0")`
   returns its version entry.
3. **Content-address verify.** `verify_package` recomputes the BLAKE3 hash tree
   and checks it against the index — a tampered vendored file fails verification.
4. **Refuse the nonexistent.** Resolving a dependency not in the registry returns
   `NotFound` — a hallucinated name simply does not resolve.
5. **Slopsquatting guard.** `slopguard::nearest` flags a hallucinated *near-miss*
   of a known name — both the separator-confusable vector (`acme_logger` vs
   `acme-logger`) and an edit-distance miss (`acme-loggr`). This directly answers
   the trajectory research's #1 supply-chain threat (19.7% of LLM-suggested
   packages don't exist; attackers pre-register the near-miss).

## Honest scope (do not soften)

- A **LOCAL filesystem registry-stub pilot, NOT a live public ecosystem.** No
  HTTP(S) transport, no publish/auth flow, no SemVer version ranges, no signature
  verification — all deferred (see `GARNET_REGISTRY_v0_1.md`).
- The **slopguard is a deterministic heuristic** (separator-confusable +
  Damerau–Levenshtein), explicitly *"a prompt to verify, not a security
  guarantee"*: "known names" are the local index, not a global ecosystem feed.
- `garnet add` vendors **local** paths only and does not yet load vendored deps
  into the symbol table (`garnet run` is unaffected by vendored bytes today).

The pilot proves the resolution + integrity + slopsquatting-defense *mechanism*
on a local registry; productionizing the transport/publish/SemVer/signature
layers is future work.
