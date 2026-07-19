# Lane 2B Review Verdict 02 — sealed package, ALERT and trust authorization

reviewer: Claude Code Fable 5 (independent, non-implementer, MacBook Air)
reviewed_head: c333db5f83114f6ad0525ba68e97602de95a8503
reviewed_tree: 6dab95d30bebb4cd115faf942aa71b488d9e1a81
request: 02-request.md
swept_at: 2026-07-19T10:29:06Z
machine: MacBook Air, macOS 26.5.1, aarch64-apple-darwin, rust 1.95.0 (rustup), python3 -I
verdict: APPROVE-WITH-BLOCKERS

verified_identity: head/tree/diffstat reproduced: yes
- `git rev-parse c333db5f^{tree}` → `6dab95d30bebb4cd115faf942aa71b488d9e1a81` (matches packet)
- `git diff --stat cede73c0..c333db5f` → 27 files changed, 1826 insertions(+), 9 deletions(-) (matches packet)
- base `cede73c0` = merge-base with origin/main (exact); head is ancestor of tip `c715a6f2`
- post-head commits (`f83051c`, `5e4390a`, `c715a6f`) touch only `ops/lane2b/**`
  (checkpoint/SOTU/request artifacts) — no source or package bytes; verified by
  `git diff --stat c333db5f..c715a6f`. F1 below therefore also holds at the tip.

differential: python full battery, fresh clone, this machine:
- merge-base 121/13 vs reviewed head 121/13 — identical failure set, new-vs-base: none.
  Packet's "Python full battery delta vs exact built base: 0" reproduced.
- cargo functional: `rustup run 1.95.0 cargo test -p garnet-cli --no-fail-fast` at
  reviewed head → 457 passed, 2 FAILED on this machine (packet claims 459/0 on the NUC).
  Both failures are the sealed-flagship positive legs and share one root cause (F1):
  `minimum_shelf_package::sealed_flagship_loads_end_to_end` and
  `mcp_stdio::native_stdio_initialize_list_call_and_error`.
  All six negative package traps 6/6 and the tamper-rejection native test pass here.
- clippy: `rustup run 1.95.0 cargo clippy -p garnet-cli --all-targets -- -D warnings`
  → PASS, reproduced.

findings:
F1 (BLOCKER): the committed sealed flagship is CRLF-checkout-bound, not repo-canonical;
on every LF checkout (macOS, Linux, CI, or Windows with autocrlf=false) the genuine
package is REJECTED and two committed tests fail.
- Exact reproduction (fresh clone, this machine, reviewed head c333db5f):
  `rustup run 1.95.0 cargo test -p garnet-cli --test minimum_shelf_package sealed`
  → `sealed_flagship_loads_end_to_end ... FAILED`, host stderr:
  `Minimum Shelf package rejected: seal build manifest does not match current Garnet`.
- Root cause, verified byte-for-byte:
  `./target/debug/garnet build --deterministic examples/minimum-shelf-flagship/tool.garnet`
  and field-compare against `tool.seal.json` `predicate.build_manifest` → every field SAME
  (source_hash, ast_hash, versions, target_triple="unknown-target", flags) EXCEPT
  `prelude_hash`: local `df4f1648cf79…` vs sealed `652a0ea81975…`.
  `hash_prelude()` hashes `garnet_interp::PRELUDE_SOURCE = include_str!("prelude.rs")`
  raw. `garnet-interp-v0.3/src/prelude.rs` has NO `.gitattributes` eol pin
  (`git check-attr text eol` → unspecified), and the implementer's own boot evidence
  records `core.autocrlf: true` on the NUC — so the sealing binary baked a CRLF prelude
  into the hash, and the committed seal binds that CRLF build manifest. The repo's own
  WIN-S38-001 rule pinned `*.garnet` and LF-normalizes `source_hash`, but the prelude
  (a `.rs` include) was missed.
- Consequence: "the one verified local Shelf package" only verifies on CRLF checkouts of
  the machine class that sealed it. The journal's "two regenerated seals byte-identical"
  claim (07 evidence) holds only on such checkouts. Any Linux CI leg and any Mac/Linux
  clone runs these two committed tests RED, and the planned deterministic Shelf reporter
  cannot be checkout-independent while its subject package is not.
- Cure is the implementer's, not mine; the shape that would satisfy this seat:
  LF-normalize the prelude bytes inside `hash_prelude()` (mirror the WIN-S38-001
  `source_hash` normalization — idempotent for LF checkouts) and/or pin
  `garnet-interp-v0.3/src/prelude.rs` (or `*.rs`) `text eol=lf`; then regenerate the
  seal, update `TRUSTED_SEAL_BLAKE3` + `SHELF_PACKAGE.json` `sealBlake3` (and manifest
  pin), and re-record RED/GREEN for the reseal. RED-record this before WV-6 evidence
  generation (this answers requested decision 5).

N1 (NOTE): deliberate canonicality tolerance — `canonical_text_blake3` CRLF→LF-normalizes
manifest and seal bytes before pinning (source stays exact-byte). Named explicitly per
protocol. Verified bounded by driving the real binary with a CRLF-converted seal: it
passes the normalized byte pin and cannot smuggle content changes (JSON bytes otherwise
identical). Acceptable as Windows-checkout durability; pinning the two JSON files
`eol=lf` alongside the F1 cure would let this tolerance be removed later.

N2 (NOTE): the packet's "sealed / rejection / native stdio: 1/1 / 6/6 / 2/2" is true on
the NUC but is not a portable property of the tree (F1). Not an honesty failure — the
runs were real and the machine was declared — but future packets should not present
machine-bound greens as unqualified.

N3 (NOTE): the committed raw-byte client transcript and WV-6 artifacts are declared
pending (BLOCKED.md resume step 5) — nothing to verify yet. When committed they must
live under a `-text`-fenced path (`ops/**/evidence/**` already resolves `-text`, verified
via `git check-attr`).

independent adversarial checks run here (beyond the committed traps):
- symlinked `tool.garnet` with byte-identical content → REJECTED
  ("source must not be a symlink"), exit 1, zero MCP bytes on stdout.
- CRLF-converted seal → passes the normalized pin as designed (N1), then rejects at
  build-manifest comparison on this machine (F1 path); zero MCP bytes on stdout.
- rejection ordering: package load and full seal verification complete before
  `into_host()`; `from_verified_source` is `pub(crate)` and reachable only from
  `into_host()` and `#[cfg(test)]` modules — no unsealed production constructor.

requested decisions:
1. APPROVE the exact reviewed implementation content — lifecycle layering, bounded Tier 1
   surface, raw-byte framing, pre-host package-rejection ordering, and the
   binary-mode Windows stdio boundary as code — subject to F1 being RED-recorded and
   cured before WV-6. The rejection ordering is verified: rejection happens before host
   construction with zero protocol bytes emitted.
2. W_TRUST companion: AUTHORIZED, bound to exactly protected path
   `garnet-cli/src/bin/garnet.rs` at head `c333db5f83114f6ad0525ba68e97602de95a8503` /
   tree `6dab95d30bebb4cd115faf942aa71b488d9e1a81`, reviewer identity
   "Claude Code Fable 5 (independent reviewer, MacBook Air)", carrier Jon.
   Reviewed content of that path: one dispatch line
   (`"mcp-serve" => cmd::mcp_serve::run(&args[1..]),`) — no other change. The F1 cure
   does not touch this path; if it ever does, the companion is void and re-review is
   required.
3. Deterministic Shelf reporter: AUTHORIZED as new reporter logic, with these binding
   requirements before its RED is written:
   - protected path: `scripts/smoke_garnet_minimum_shelf.py` (the P4-G1 command already
     locked in plan.lock.json); classify it as reporter/trust logic requiring its own
     digest/path-bound review companion in the same rolling-review-v2 form as decision 2.
   - reads committed artifacts only — no network, no refs/pull/*, no machine state, no
     wall-clock content in the verdict body.
   - two runs from two fresh checkouts must emit byte-identical verdict JSON; the
     byte-identity proof must itself be committed evidence.
   - WV-6 marker flips must be produced by this reporter's output alone — never
     hand-edited (current state.json P4 gates correctly remain `pending`).
   - PRECONDITION: F1 cured first; a checkout-dependent package cannot yield a
     checkout-independent reporter verdict.
4. Unsigned in-toto content predicate + compiled Git-reviewed digest roots: ACCEPTABLE
   for this bounded local Shelf claim, given (a) the verifier enforces the honest
   UNSIGNED label, (b) the loader re-derives AST/build/capability bindings rather than
   trusting the predicate, and (c) the claim is scoped to reviewed local content, not
   signer identity. This acceptance is conditional on F1: the compiled digest root must
   be canonical for the repo, not for one checkout convention.
5. Blocker to RED-record before WV-6 evidence generation: F1, as specified above.

scope: clean. All 27 files inside declared Lane 2B scope (Shelf/MCP surfaces incl. the
repo-bundled `examples/minimum-shelf-flagship/**` package, Ring Tier 1, `ops/lane2b/**`,
tests) plus the declared protected-path single line (decision 2). No gate or reporter
LOGIC changed; no `.github/workflows`; no `ops/mission/state.json`.
weakening: none in executable checks (same 9 deletions as Request 01; see 01-verdict).
Canonicality tolerance N1 named explicitly.
provenance: Cargo.lock and sealed build inputs untouched (SA-4 not triggered). The new
sealed artifact itself carries the F1 defect described above.

not_verified:
- Windows-native legs: `_setmode` binary-mode behavior, the NUC's 459/0 and 950-test
  native battery parity, WinError/CreateProcess pre-existing errors — machine unsuitable
  (macOS/arm64). Their delta-vs-base=0 claim is independently corroborated by my own
  base-vs-head battery parity here.
- timing (machine unsuitable): no wall-clock claims were made; none validated.
