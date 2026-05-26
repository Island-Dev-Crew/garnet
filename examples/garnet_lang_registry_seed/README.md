# Garnet Lang Registry Seed

This is a local filesystem registry seed for S18. It is not the public
`github.com/garnet-lang/*` publication lane.

The seed exists so the first five Layer-2 package surfaces can be checked
against the current registry stub and resolver before the external GitHub org
is available.

Rebuild the index after package edits:

```bash
cargo run -p garnet-registry-stub -- build examples/garnet_lang_registry_seed
cargo run -p garnet-registry-stub -- verify examples/garnet_lang_registry_seed
```
