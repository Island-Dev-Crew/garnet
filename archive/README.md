# `archive/` — historical sources retained for audit trail only

Nothing in this directory is part of the Garnet workspace build. The crates
here are excluded from `Cargo.toml` (`[workspace] exclude = [...]`) and are
not packaged, tested, published, or documented as part of the active
project. They are kept in-tree because each represents a numbered
engineering-ladder rung that newer crates supersede, and removing them
would erase the lineage that the spec and papers reference.

## Contents

### `history/`

Root-level milestone and bundle-index files from earlier Garnet phases. These
files are preserved for audit trail and academic provenance, but they are not
current repository navigation. Start at [`../CURRENT_STATE.md`](../CURRENT_STATE.md)
for live truth.

### `examples/`

Historical example drafts retained for domain ambition and audit trail. Active
runtime proof lives in [`../examples/`](../examples), not in this archive.

### `garnet-parser-v0.2/`

The original Mini-Spec v0.2 parser (Rung 2). Flat-file layout, covers
only memory unit declarations (§2.1) and actor declarations (§4.1).
Superseded in full by [`garnet-parser-v0.3/`](../garnet-parser-v0.3) which
implements all 90 EBNF productions of the Mini-Spec v0.3 / v1.0 grammar.

Use this only as a reference for "what the v0.2 parser shipped"; do not
build or extend it.
