# Garnet Capability-Manifest Standard Seed (S98)

Status: draft/reference seed. No standards body has adopted this document.

## Purpose

SBOM formats describe dependencies. They do not directly describe the authority a
program asks to exercise. Garnet's S36 capability manifest already exposes the
declared `@caps(...)` surface used by `diff-caps`, `seal`, and the local
capability transparency log. S98 adds a small language-neutral profile around
that surface so the format can be discussed, tested, and eventually donated
without pretending donation has happened.

## Draft Profile

Schema: `capability-manifest/v1`

Required fields:

- `schema`: exactly `capability-manifest/v1`.
- `status`: `draft-reference-seed` until a neutral body accepts ownership.
- `producer`: object with `name` and `manifest_schema`.
- `surface.aggregate`: sorted list of all declared capabilities.
- `surface.entries`: sorted list of declared authority entries.
- `surface.entries[].kind`: currently `function` for Garnet's reference seed.
- `surface.entries[].name`: function/tool identifier in deterministic order.
- `surface.entries[].capabilities`: sorted declared capability list for that entry.
- `surface.entries[].source_span`: `null` until a stable CST/span source map is
  available.
- `surface.wildcard`: true when `@caps(*)` is present in the declared surface.
- `limitations`: machine-readable honesty notes.

The Garnet reference implementation is:

```text
garnet caps --standard-profile <file-or-package>
```

The default `garnet caps <path>` output remains the S36 Garnet-native
`garnet-capability-manifest-v1` envelope for backward compatibility.

## Determinism Rules

- Lists are sorted and deduplicated before serialization.
- Field order is fixed in the reference implementation.
- Local filesystem paths are omitted from the profile so clean clones can produce
  byte-identical profile output for the same source.
- The profile reflects the declared capability surface only. It does not prove
  the absence of undeclared authority, and it does not close the VM enforcement
  gap.

## Honest Scope

This is a draft/reference seed, not an accepted standard. No OWASP, Linux
Foundation, W3C, IETF, or other external body has reviewed, adopted, or endorsed
it. "Reference implementation" means Garnet can emit deterministic profile JSON
and test vectors for discussion; it does not imply a multi-language ecosystem or
production security guarantee.

