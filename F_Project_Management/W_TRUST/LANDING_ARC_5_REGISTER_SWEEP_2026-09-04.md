# Landing Arc 5 register sweep — 2026-09-04 (post-merge arc: #556, #554 and the seven-round #553 review)

Records lane (L2), swept at `origin/main` = `b20869d04ea72cc9481a1122d8b215cb7d6ae543`
(`b20869d`, "manifest(verify): compare every field and refuse unknown keys (#554)").
Every entry below was raised inside the 2026-09-03/04 review and audit rounds and carries the
command that reproduces it.

**Review family is stated per entry and it is load-bearing.** U-117 and U-120 were raised by the
OpenAI Codex cross-family seat during pull-request review and reproduced by the implementing
seat before registration. U-118, U-119 and U-121 were raised by Claude seats (an audit agent or
the implementing seat itself) and are **same-family** standing findings until a Codex record
touches them; nothing here may be cited as independently reviewed on that basis alone.

- Sweep seat: Claude Fable 5.1, records lane, macOS, worktree branched from `origin/main`.
- Sweep date: 2026-09-04.
- Authority: the directing seat's standing delegation of 2026-09-01.

## Collision sweep

- swept-at: 2026-09-04, after `git fetch --prune` on both remotes.
- source: 481 advertised refs — every `origin` and `fork` branch head except `fork/main` per
  the boot fence — deduplicated. No hand-listing.
- pattern: `git grep -I -hoE 'U-[0-9]+([^0-9]|$)' <tree>`, then `grep -oE 'U-[0-9]+' | sort -u`,
  with the prose token `U-910` (the anchor explanation in the arc-4 sweep) excluded.
- result **before**: the distinct token set runs `U-1`, `U-04` … `U-116` — **census 99**.
  **No occurrence at or above U-117 exists in any swept tree.**
- result **after** this file: `U-1`, `U-04` … `U-121` — **census 104** (99 + 5 allocations).

## U-117 — CapCaps propagator: a primitive reached only through a call-graph cycle is not reported, annotated or not

- **Class:** checker defect (pre-existing), incomplete transitive propagation.
- **Family:** raised by Codex (review v5 of #553), reproduced by Claude.
- **Statement:** `garnet check` reports `0 diagnostics` for a program in which `a` declares
  `@caps(fs)` and calls `write_file` and `b`; `b` declares `@caps()` and calls `a`; and a
  `@caps()` `main` calls `b`. Remove the `a -> b` edge and two `fs` diagnostics appear. The
  cycle causes the callers to be memoised with empty transitive sets: `caps_graph.rs:540-584`
  returns empty on gray nodes and computes no SCC fixed point. The previously registered
  boundary (U-91) named only a *wholly unannotated* cycle; the boundary is any cycle.
- **Reproduce:**
  ```sh
  printf '@caps(fs)\ndef a() {\n  b()\n  write_file("/tmp/never.txt", "x")\n}\n@caps()\ndef b() {\n  a()\n}\n@caps()\ndef main() {\n  b()\n}\n' > cyc.gn
  garnet check cyc.gn      # 3 functions checked, 4 boundary call sites, 0 diagnostics
  ```
- **Status:** open. The public surfaces and the normative fence were bounded to "named, acyclic
  chain from an annotated function" in #553. The checker cure is a separate slice on the trust
  surface (`garnet-check-v0.3/src`) with a red-first test; it is not registered as cured here.

## U-118 — `garnet seal` in-toto subject digest is the shape-stable AST hash and collides across different `@caps(...)` declarations

- **Class:** product finding, attestation binding.
- **Family:** raised by a Claude audit agent, reproduced by the implementing Claude seat.
- **Statement:** `subject[].digest.blake3` is filled from `build.ast_hash`
  (`garnet-cli/src/seal.rs`), which `manifest.rs` computes as
  `hash_str(&stable_ast_repr(module))` — a structural repr that writes `Str(...)` for every
  literal and never sees `@caps(...)`. Two sources, one `@caps()` returning `"Hello, world!"`
  and one `@caps(fs, net, proc, ffi)` returning a different literal, produce the identical
  subject digest `ca1a3a33df39d4119c12cb5a1e80f0ed…`. The exact-source digest exists in
  `predicate.source_blake3` and is line-ending-normalized.
- **Reproduce:**
  ```sh
  garnet seal a.garnet --out a.json; garnet seal b.garnet --out b.json
  jq -r '.subject[0].digest.blake3' a.json b.json     # identical
  jq -r '.predicate.source_blake3' a.json b.json      # differ
  ```
- **Status:** open. A verifier reading an in-toto Statement treats `subject.digest` as the
  artifact identity; here it is invariant under a capability change. Public copy that says the
  Statement "binds the source digest" must show `predicate.source_blake3` or say `subject` is a
  shape hash (the RF-03 front-door handoff already does). Whether `subject` should carry the
  source digest is a product decision.

## U-119 — "cross-family review" is a process convention, not a gate-checked predicate

- **Class:** governance property.
- **Family:** raised by a Claude audit agent; confirmed by the implementing Claude seat.
- **Statement:** `grep -c family` returns `0` across every `scripts/garnet_*_status.py`. The
  rolling-review gate proves the reviewer principal is well-formed and disjoint from the
  commit authors; it does not and cannot prove that a different model family performed the
  review. Step 2 of the published acceptance sequence therefore describes a convention the
  project keeps, not a predicate the machine enforces.
- **Reproduce:** `grep -c family scripts/garnet_trust_kernel_review_status.py` → `0`.
- **Status:** open. Candidate cure: a `review_family` field in a v3 record schema that the gate
  requires to be present and unequal to the implementer family; until then public copy must
  not imply machine enforcement of the family boundary.

## U-120 — the Minimum Shelf trust root is pinned to the CLI version, so every version bump is a reseal ceremony

- **Class:** design coupling, surfaced by the 0.8.2 bump.
- **Family:** raised by Codex (review v1 of the rebased #550), reproduced and cured by Claude.
- **Statement:** `Manifest::from_module` (`garnet-cli/src/manifest.rs:74-75`) derives
  `parser_version` and `interp_version` from `CARGO_PKG_VERSION`, and
  `garnet-cli/src/minimum_shelf.rs:182-185` compares the complete build manifest, so the 0.8.2
  CLI refused the committed 0.8.1-sealed flagship: `sealed_flagship_loads_end_to_end` and
  `native_stdio_initialize_list_call_and_error` went red. The cure is a reseal through the
  owning procedure — new seal, rebound `SHELF_PACKAGE.json`, three `TRUSTED_*_BLAKE3`
  constants, the smoke pins, and an appended evidence file — never a relaxed validator.
- **Reproduce:** at the pre-reseal #550 head, `cargo test -p garnet-cli --test
  minimum_shelf_package` → 1 failed.
- **Status:** cured for 0.8.2 in #550 (`ops/lane2b/evidence/11-f2-version-bump-reseal-green.txt`).
  The coupling itself stands: until the pin is decoupled from the version, every future bump
  repeats the ceremony.

## U-121 — review records are append-only by gate rule; a void record cannot be removed, only superseded

- **Class:** governance property, documented so it is not "cleaned up".
- **Family:** raised by the implementing Claude seat while rebasing #550.
- **Statement:** after a rebase voids a record (its `base_commit` no longer matches the
  discovered merge-base), removing that file is itself a rolling-gate violation:
  `structured review record history is append-only: record changed or was removed at <sha>`.
  The void record stays byte-identical in the tree as historical provenance and a new record
  is added beside it. A branch that has been rebased therefore carries its void records
  permanently.
- **Reproduce:** `git rm` a `.review.json` on a branch and run
  `python3 -I scripts/garnet_trust_kernel_review_status.py --base origin/main --head HEAD`.
- **Status:** property, not a defect. The rolling-review contract should state it.
