# Garnet Registry v0.1

Status: S13 stub. NOT a production registry.

This document records the v0.6 registry surface introduced for S13. The v0.1
registry is deliberately **filesystem-backed**: the substance is the resolution
loop (index lookup → BLAKE3 content-address verify → vendor), not the
transport. HTTP(S) transport, tarball packaging, authentication, a publish
flow, SemVer ranges, transitive dependency resolution, and signature
verification are all explicitly deferred.

## Layout

A registry is a directory:

```text
<registry>/
  index.json
  <name>/<version>/...package files...
```

Each `<name>/<version>/` directory is a package version — the same on-disk
shape that `garnet add <path>` vendors. Packages are plain directories; v0.1
does not use tarballs.

## index.json schema

```json
{
  "registry_version": "0.1",
  "packages": {
    "<name>": {
      "versions": {
        "<version>": {
          "path": "<name>/<version>",
          "files": [
            { "path": "<relative file>", "blake3": "<lowercase hex>" }
          ],
          "signature": null
        }
      }
    }
  }
}
```

- `registry_version` — `"0.1"`.
- `packages` — map keyed by package name (deterministically ordered).
- `versions` — map keyed by exact version string. v0.1 matches the version
  **exactly**; there are no caret/tilde/range semantics.
- `path` — package directory relative to the registry root.
- `files` — `(relative path, BLAKE3 hex)` for every regular file in the
  package, lexicographically sorted by path. POSIX-style `/` separators.
- `signature` — **reserved**. v0.1 does NOT read or verify it; it exists so a
  future signing slice can populate it without a schema break.

The index is produced by `garnet-registry-stub build <registry>`, which scans
the `<name>/<version>/` directories and hashes every file. Output is
deterministic (`BTreeMap` key ordering + sorted file lists), so two builds of
the same registry produce byte-identical `index.json`.

## Client: `garnet add --registry`

```sh
garnet add --registry <location> <name>@<version>
```

- `<location>` is a filesystem path or a `file://` URL. HTTP(S) is deferred.
- Resolution: load `<location>/index.json`, resolve `<name>@<version>`,
  resolve the package directory (refusing any `path` that escapes the registry
  root — a path-traversal guard), verify every file's BLAKE3 against the index,
  then copy the package into `.garnet/vendor/<name>/`.
- The `[dependencies]` entry written to `Garnet.toml` is registry-shaped:
  `<name> = { registry = "<location>", version = "<version>", vendor =
  ".garnet/vendor/<name>" }`.
- `Garnet.lock` records the dep with a `registry+<location>#<name>@<version>`
  provenance string and the per-file BLAKE3 hashes.

Because the S12 resolver loads `.garnet/vendor/` at `garnet run` time, a
registry-resolved dependency's `use <name>::*` symbols resolve end-to-end:

```sh
cargo run -p garnet-registry-stub -- build examples/registry_stub_fixture
garnet add --registry examples/registry_stub_fixture hello_lib@0.1.0
# src/main.garnet:  use hello_lib::*  ;  def main() { registry_hello() }
garnet run src/main.garnet            # => hi from the registry stub
```

## Tool: `garnet-registry-stub`

```sh
garnet-registry-stub build  <registry-dir>   # generate index.json
garnet-registry-stub verify <registry-dir>   # re-hash and check against index.json
```

`build` is the "publish" side; `verify` re-hashes every package and confirms it
matches the recorded index (catches drift/tampering).

## Integrity

Content-addressing is BLAKE3 over each file. `garnet add --registry` refuses to
vendor a package whose on-disk bytes do not match the index hashes, and the
package-directory resolver refuses any index `path` that canonicalizes outside
the registry root.

## Non-Claims (v0.1)

- No HTTP(S) transport. Filesystem / `file://` only.
- No tarball packaging.
- No authentication, accounts, or publish/upload flow.
- No signature verification (the `signature` field is reserved, unread).
- No SemVer ranges — exact `<name>@<version>` only.
- No multi-registry resolution — one registry per `garnet add` invocation.
- No transitive dependency resolution — a fetched package's own deps are not
  pulled in.
- This is a research-grade stub, not a hosted package registry.
