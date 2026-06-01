# S82 — seal source-hash determinism (LF/CRLF) — closes WIN-S38-001

## Goal
The seal predicate's `source_blake3` hashed raw source bytes, so an LF (Mac/Linux)
checkout and a CRLF (Windows core.autocrlf) checkout of the same logical source
produced different predicates — the reproducible-bundle pillar broken cross-platform.

## Root + fix (two layers)
- Root: `garnet-cli/src/manifest.rs::Manifest::build` → `source_hash: hash_str(source)`
  (`blake3` over raw bytes); the seal predicate's `source_blake3` is this hash.
- Fix 1 (code): `source_hash: hash_str(&normalize_source_eol(source))` where
  `normalize_source_eol` replaces `\r\n`→`\n`. Idempotent on LF, so existing LF
  seals are byte-identical (backward-compatible); only CRLF content now matches its
  LF form. `ast_hash` was already stable (the parser canonicalizes).
- Fix 2 (config): `.gitattributes` pins `*.garnet text eol=lf` as defense-in-depth.
- Contract documented in `C_Language_Specification/GARNET_ATTESTATION.md`.

## What ships
- The manifest EOL-normalization + 2 Rust unit tests (LF↔CRLF same source_hash +
  ast_hash; LF hash == raw blake3, backward-compat).
- `.gitattributes` `*.garnet text eol=lf`.
- `scripts/garnet_seal_determinism_status.py` (+ `--gate`, 5 tests) — pins +
  in-code normalization + documented contract.
- CI agent-contracts; CHANGELOG; this plan; the S82 Windows-proof row updated.

## Verification
- `cargo test -p garnet-cli manifest::tests` → green incl. the new LF/CRLF tests;
  `cargo test --workspace` 0 failed; fmt/diff/clippy clean.
- `python3 scripts/test_garnet_seal_determinism_status.py` → 5 OK; `--gate` rc 0.

## Honest scope (do not soften)
Only line endings are canonicalized; other whitespace still changes `source_blake3`
by design (`ast_hash` is the shape-stable digest). Mac-authored + Mac-unit-tested;
the end-to-end Windows proof (fresh Windows checkout → matching `source_blake3`) is
handed off to the Windows lane (Windows-proof-pending). Must precede provenance work.
