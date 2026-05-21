# AGENTS.md - Registry Stub Contract

## Scope

Owns the Garnet registry stub v0.1: the `index.json` schema, the
filesystem-backed `build`/`verify` tool, and the content-addressed
resolution helpers consumed by `garnet add --registry`.

## Stable Contracts

- This is a research-grade STUB, not a hosted registry. Claims must stay
  narrower than a production package registry.
- v0.1 is filesystem-backed only. HTTP(S) transport, tarball packaging,
  authentication, and a publish flow must be reported as deferred, not
  implied.
- Content-addressing is BLAKE3 per file. Resolution must verify on-disk bytes
  against the index before vendoring, and must refuse any index `path` that
  canonicalizes outside the registry root (path-traversal guard).
- The index `signature` field is reserved and MUST NOT be presented as
  verified until a signing slice implements verification.
- `index.json` output must be deterministic (sorted keys + file lists) so two
  builds of the same registry produce identical bytes.

## Required Checks

```sh
cargo test -p garnet-registry-stub
cargo build -p garnet-registry-stub --release
cargo run -p garnet-registry-stub -- build examples/registry_stub_fixture
cargo run -p garnet-registry-stub -- verify examples/registry_stub_fixture
```

See `C_Language_Specification/GARNET_REGISTRY_v0_1.md`.
