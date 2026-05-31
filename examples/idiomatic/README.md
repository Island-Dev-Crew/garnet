# Idiomatic Garnet corpus

A small, open corpus of programs that show *what good Garnet looks like* — they
follow the policies the v0.8 hardening band established, and each one
`garnet check`s to **0 diagnostics** (fully clean, not even a non-fatal advisory)
and runs deterministically. Proven by `scripts/garnet_idiomatic_corpus.py`.

| File | Idiom |
|---|---|
| [`typed_errors.garnet`](typed_errors.garnet) | **Typed rescue** (S42 error policy): `rescue e: AppError` names the exception type — never a catch-all, which would draw the `check.over_catch` advisory. |
| [`state_machine.garnet`](state_machine.garnet) | **Exhaustive `match`** over a finite enum, with **named `@caps`** on every function — no catch-all arm, no undeclared authority. |

These complement the broader example sets: the 12-domain proof matrix
(`smoke_garnet_studio_domain_matrix.py`), the Paper-VI novel compositions
(`smoke_garnet_novel_compositions.py`), and the docs-as-tests demonstrator
(`documented_math.garnet`). This corpus is about *style*; it is a
discipline/idiom showcase, not a performance or coverage claim.
