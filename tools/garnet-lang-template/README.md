# {{package_name}}

`{{package_name}}` is a Layer-2 Garnet package template for official `@garnet-lang/*` packages.

Status: template only. Packages generated from this scaffold are not part of the bundled `std::` surface and must declare their stability in `Garnet.toml` before publication.

## Contract

- Layer: 2 (`@garnet-lang/*`)
- Stability: `experimental` until the package earns promotion under `C_Language_Specification/GARNET_STDLIB_LAYER_POLICY.md`
- Capability ceiling: declare every OS authority in package metadata and on any function that actually exercises that authority
- Source-level `@stability(...)`: pending parser annotation support; keep the manifest/docs declaration authoritative for v0.7

## Dogfood

```bash
garnet check garnet/lib.garnet
garnet run tests/smoke.garnet
```
