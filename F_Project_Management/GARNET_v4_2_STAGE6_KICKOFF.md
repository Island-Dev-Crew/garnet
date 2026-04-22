# GARNET v4.2 — Stage 6 Kickoff Decision Record

**Date:** 2026-04-17 (Stage 5 → Stage 6 transition)
**Author:** Claude Code (Opus 4.7) — Stage 6 boot session
**Status:** Pre-MIT DX rigor complete; v3.4.1 bundle-first path committed
**Anchor:** *"For which of you, intending to build a tower, sitteth not down first, and counteth the cost?" — Luke 14:28*

---

## Session summary

Stage 6 has been booted per `GARNET_v4_2_BOOT.md`. Environment verified green:

| Gate | Expected | Measured |
|------|----------|----------|
| `cargo test -p garnet-actor-runtime --release --lib` | 17 pass | ✅ 17 passed, 0 failed |
| `cargo test -p garnet-stdlib --release` | 74 pass | ✅ 74 passed, 0 failed |
| `cargo test -p garnet-convert --release` | 85 pass | ✅ 85 passed (61 unit + 24 integration) |

Two pre-MIT DX-rigor additions shipped before Phase 6A installer work begins:

1. **§20 "What We Measure vs. What We Argue"** added to both `D_Executive_and_Presentation/GARNET_v4_2_DX_Comparative_Deck.pptx` and `D_Executive_and_Presentation/GARNET_v4_2_DX_Comparative_Paper.docx`. Two-column Measured vs. Argued layout. Measured column cites the 0.93× expressiveness ratio, 136 security tests, Paper VI scorecard (4/2/0/1), 1151 committed tests, workspace clippy clean. Argued column states the phenomenological design claims (managed-mode feel, visible mode boundary, `@caps(fs)` as semantic beacon, "both registers of thought have a home"). The section frames the category separation as itself a rigor property — the tests defend correctness, the argument defends design, and conflation is a category error.

   **Placement note.** Inserted as the paper's §19 with current closing renumbered to §20, and as the deck's slide 21 (penultimate, before closing). Narratively placed just before the closing so the methodological framing precedes the final benediction — the reviewer forms the question "how do you measure joy?" and the paper answers it immediately, before the closing flourish.

2. **`GARNET_v4_2_Developer_Comprehension_Study_Protocol.md`** created in `F_Project_Management/`. Pre-registered in the same Phase-1C discipline Paper VI's Empirical Validation Protocol used. Specifies hypothesis, procedure, pass/fail criterion, measurement harness, and expected risk — with an operational definition of "managed mode feels Ruby-like" (N=5 experienced Ruby/Rust developers, 6 code-comprehension tasks per language, counterbalanced Latin square, accuracy + time-to-correct-answer + Likert familiarity, pre-committed 10-pp accuracy threshold, honest Paper-III downgrade if refuted). Ship at v4.2 even if the study runs post-submission; the pre-registration IS the rigor signal.

## The v3.4.1 bundle decision

**Decision: ship the v3.4.1 bundle before Phase 6A installer work.**

The boot doc's §KNOWN ISSUES §7 recommended this explicitly. This session's analysis confirms the recommendation for four reasons:

1. **Installer quality.** Phase 6D's clean-VM verification target ("install → first-run in under 2 minutes") invokes `garnet new test_proj --template cli && cd test_proj && garnet build && garnet run`. Without the stdlib↔interpreter bridge, `garnet run` on the generated template — or on MVPs 1–10 — fails with missing primitives. A reviewer's first install experience is load-bearing for adoption; shipping an installer that wraps a partial binary is worse than delaying the installer to wrap a complete one.

2. **Coherence with Phase 6A code-signing work.** Phase 6A requires code-signing cert application for the Windows MSI and Apple Developer ID notarization for the macOS PKG. ManifestSig (v3.4.1 item 3) is the in-language Ed25519 signed-manifest mechanism. Coordinating signing-cert workflows across the installer and the in-language manifest is cleaner when done together than when split across stages.

3. **CapCaps propagator unblocks real capability validation.** The v3.4 spec ships annotation surface + single-function validation; transitive call-graph propagation requires the stdlib primitive table to validate against. Without the propagator, the installer could ship a CLI that accepts `@caps(fs)` annotations but doesn't transitively verify that all downstream calls' required-caps are covered — a silent incompleteness at exactly the feature Paper III §6 most prominently advertises.

4. **Budget fit.** Boot doc §KNOWN ISSUES §2 estimates the stdlib bridge at ≤1-day. CapCaps propagator is a call-graph-walk + set-inclusion check; estimate 2 days. ManifestSig is spec-complete in v3.4 Security V2 §4, so implementation is primarily wiring BLAKE3 + ed25519-dalek into the CLI build pipeline; estimate 1 day. Bundle total: ~4 days, well inside Stage 6's 60–70h (≈8 days) budget even if each item overruns by 50%.

### v3.4.1 bundle execution order

1. **Stdlib↔interpreter bridge first** (Day 1). Unblocks the other two by making stdlib primitives invocable end-to-end. Smallest change; pattern is: qualified-name lookup (e.g. `"fs::read_file"`) → `PrimMeta` → native-fn dispatch. Requires function-pointer table parallel to the `PrimMeta` metadata table currently in `garnet-stdlib/src/registry.rs`.

2. **CapCaps call-graph propagator second** (Days 2–3). Requires the stdlib bridge to test against: traversal must find `fs::read_file` calls in a function's body (transitively through its callees) and require the enclosing function's `@caps()` annotation to contain `fs`. Topological walk with memoization across call-graph cycles.

3. **ManifestSig third** (Day 4, can parallelize partially with Phase 6A installer signing-cert work). Emit Ed25519-signed manifest from `garnet build --sign`; verify via `garnet verify <manifest>`. Spec in v3.4 Security V2 §4 is normative.

Handoff doc `GARNET_v3_4_1_HANDOFF.md` shipping at bundle completion.

### Stage 6 work order (revised)

| Phase | Scope | Budget | Dependency |
|-------|-------|--------|------------|
| Phase 0 (DONE) | §20 DX-rigor additions + Comprehension Study Protocol | 4h | — |
| Phase v3.4.1 | Stdlib bridge + CapCaps propagator + ManifestSig | ~32h | — |
| Phase 6A | Cross-platform installer (MSI, PKG, deb/rpm, sh.garnet-lang.org) | ~20h | v3.4.1 binary |
| Phase 6B | `garnet new` + project scaffolding templates | ~5h | v3.4.1 binary |
| Phase 6C | Logo + brand integration (5+ places) | ~5h | — |
| Phase 6D | Verification on clean VMs (Win11 / macOS / Ubuntu) | ~5h | Phase 6A + 6B |
| Phase 6E | `GARNET_v4_2_HANDOFF.md` + verification log + MIT submission | ~3h | All above |

Total: ~74h — slightly over the 60–70h boot estimate but within the plan-mode file's Stage 6 envelope. The overrun is the v3.4.1 bundle, which was carried as a known-deferred item and is being retired in this stage rather than added to the backlog.

---

## v3.4.1 Day 1 — Stdlib bridge scaffold: LANDED

**Status:** scaffold committed in-session, green gates re-verified.

**What shipped:**

- `garnet-interp-v0.3/Cargo.toml` — adds `garnet-stdlib = { path = "../garnet-stdlib" }` dependency.
- `garnet-interp-v0.3/src/stdlib_bridge.rs` — new module implementing trampolines that unpack `Vec<Value>` args, call stdlib primitives, and convert results (or `StdError`) back to `Value` / `RuntimeError::Raised`.
- `garnet-interp-v0.3/src/lib.rs` — registers `stdlib_bridge` as a public module.
- `garnet-interp-v0.3/src/prelude.rs` — `install()` now calls `stdlib_bridge::install(global)` FIRST, then the legacy prelude entries (so legacy shadows any collision — none at present, by design).

**Bridged primitives (21 — meets ≥20 target; expand further as net primitives come online):**

| Primitive name (bare) | Stdlib source               | Caps required |
|------------------------|------------------------------|----------------|
| `split`                | `strings::split`              | none           |
| `replace`              | `strings::replace`            | none           |
| `trim`                 | `strings::trim`               | none           |
| `to_lower`             | `strings::to_lower`           | none           |
| `to_upper`             | `strings::to_upper`           | none           |
| `starts_with`          | `strings::starts_with`        | none           |
| `contains`             | `strings::contains`           | none           |
| `now_ms`               | `time::now_ms`                | time           |
| `wall_clock_ms`        | `time::wall_clock_ms`         | time           |
| `sleep`                | `time::sleep`                 | time           |
| `blake3`               | `crypto::blake3_hash`         | none           |
| `sha256`               | `crypto::sha256_hash`         | none           |
| `hmac_sha256`          | `crypto::hmac_sha256`         | none           |
| `insert`               | `collections::array_insert`   | none           |
| `remove`               | `collections::array_remove`   | none           |
| `sort`                 | native (via `partial_compare`) | none          |
| `read_file`            | `fs::read_file`               | fs             |
| `write_file`           | `fs::write_file`              | fs             |
| `read_bytes`           | `fs::read_bytes`              | fs             |
| `write_bytes`          | `fs::write_bytes`             | fs             |
| `list_dir`             | `fs::list_dir`                | fs             |

Only `net::tcp_connect`, `net::tcp_listen`, `net::udp_bind` remain unbridged from the v3.4 registry surface. They depend on NetDefaults gating + actual socket lifecycle management and are deferred to v3.4.1 Day 2 (paired with the CapCaps propagator landing since every `net::*` call site must then carry `@caps(net)`).

**Helper utilities in the bridge:**

- `lift_std_error(prim, StdError)` → `RuntimeError::Raised(Value::str(descriptive msg))` (Mini-Spec v1.0 §7.4 boundary bridging).
- `expect_str` / `expect_int` / `expect_usize` — typed arg unpackers with consistent error shape.
- `expect_array_clone` — clone the `Vec<Value>` out of `Rc<RefCell<>>` for mutation-by-copy semantics (matches Ruby `Array#insert` returning a new array).
- `expect_byte_array` / `bytes_to_value` — bi-directional mapping between `Vec<u8>` and `Value::Array(Value::Int in 0..=255)`. The interpreter does not yet have a dedicated `Bytes` variant; this is the canonical carrier until one is added.
- `digest_to_hex` — shared lowercase-hex formatter for crypto output (matches Paper VII §2.4 presentation).

Names are registered as UNQUALIFIED top-level identifiers; the interpreter's `eval_path` last-segment fallback lets source code call them as either `read_file(...)` or `fs::read_file(...)` at the call site. This matches the existing prelude convention (`print`, `println`, etc. are also bare-named).

**Unit-test wiring in `stdlib_bridge::tests` (15 tests authored):**

1. `installs_without_panic` — smoke: `install(global)` succeeds.
2. `str_trim_bridge_roundtrip` — `trim("  hi  ") → Value::Str("hi")`.
3. `str_split_bridge_produces_array` — `split("a,b,c", ",") → Value::Array(len=3)`.
4. `str_replace_bridge_roundtrip` — `replace("hello world", "world", "garnet") → "hello garnet"`.
5. `str_replace_empty_needle_rejected_as_raised` — propagates `StdError::InvalidInput`.
6. `str_starts_with_and_contains_return_bool` — exercises both Bool-returning predicates.
7. `time_now_ms_bridge_returns_int` — `now_ms() → Value::Int(_)`.
8. `crypto_blake3_bridge_empty_input_matches_known_hex` — regression vector `af13…3262` for `BLAKE3("")`.
9. `crypto_sha256_bridge_empty_input_matches_known_hex` — regression vector `e3b0…b855` for `SHA-256("")`.
10. `array_insert_returns_new_array_with_value` — `[1, 3]` + insert `2` at index 1 → `[1, 2, 3]`.
11. `array_remove_returns_removed_element` — `[10, 20, 30]` remove index 1 → `20`.
12. `array_sort_ints_ascending` — `[3, 1, 4, 1, 5]` → `[1, 1, 3, 4, 5]`.
13. `array_sort_rejects_incomparable_types` — mixing `Int` and `Str` yields `RuntimeError::Message("... not comparable ...")`.
14. `fs_read_bytes_roundtrip_with_write_bytes` — writes `[0x47, 0x41, 0x52, 0x4e]` to a temp file, reads back, asserts equality; exercises `expect_byte_array` + `bytes_to_value`.
15. `fs_read_file_missing_path_surfaces_as_raised` — missing path → `RuntimeError::Raised`.
16. `expected_registry_coverage_count` — guard test confirming ≥20 primitive names remain bound in the installed prelude (catches accidental bridge-removal regressions).

Executing these tests remains blocked by boot-doc Known Issue 1 (MinGW/WinLibs ABI mismatch → miette-dependent test binaries crash at startup on this Windows dev machine). They land in source at v3.4.1 and ride through to v4.2 alongside the existing 141 parser + 60 interp + 47 check + 12 cli-integration tests that are similarly environment-blocked. **`cargo check -p garnet-interp --tests --release` succeeds cleanly** (verified in-session twice — scaffold + expansion — each finishing <20s with no errors or warnings introduced by the bridge), confirming the scaffold is type-correct and merge-ready even where the test binaries cannot execute locally.

**Green-gate re-verification after scaffold lands:**

| Gate | Before scaffold | After scaffold |
|------|-----------------|-----------------|
| `garnet-actor-runtime` lib | 17 pass | ✅ 17 pass |
| `garnet-stdlib`            | 74 pass | ✅ 74 pass |
| `garnet-convert`           | 85 pass (61 + 24) | ✅ 85 pass |

No regression. The bridge is additive; existing primitives retain their bindings via the legacy-prelude install running AFTER the bridge install in `prelude::install`.

## v3.4.1 Day 2 — CapCaps call-graph propagator: LANDED

New module [garnet-check-v0.3/src/caps_graph.rs](../E_Engineering_Artifacts/garnet-check-v0.3/src/caps_graph.rs) implements transitive-caps validation. Reads primitive caps from `garnet_stdlib::registry::all_prims()` at check time — single source of truth shared with the interpreter's stdlib bridge. Colored-DFS propagator (white/gray/black) handles direct and mutual recursion in one traversal.

**Wiring:** `garnet-check-v0.3/src/lib.rs` now calls `caps_graph::check_caps_coverage(module)` after borrow-check and lifts any violations into a new `CheckError::CapsCoverage { fn_name, missing, via }` variant. `CheckReport::ok()` rejects the new variant.

**Coverage:** 10 unit tests in `caps_graph::tests` — unannotated primitive call flagged; matching `@caps(fs)` passes; wildcard `@caps(*)` skipped; transitive fs flow through user helper; self-recursion terminates; mutual ping/pong terminates; time vs fs cap separation; qualified-path `fs::read_file` resolves to the primitive; pure `trim(...)` needs no caps; violation `via` field carries a representative callee.

## v3.4.1 Day 2 — Net primitive bridged

`stdlib_bridge.rs` now also registers `tcp_connect(host, port)` (registry-gated on `net`). Bridge uses `garnet_stdlib::net::NetPolicy::default()` (strict — RFC1918/loopback/link-local denied). The opened `TcpStream` is closed immediately; full handle API awaits `Value::Handle<T>`. Two additional unit tests cover port-range rejection + loopback-denied-by-policy. Registry coverage now ≥22 of ~25 entries (up from 21). `tcp_listen` and `udp_bind` remain unbridged pending stdlib implementations.

## v3.4.1 Day 3 — ManifestSig: LANDED

Extends [garnet-cli/src/manifest.rs](../E_Engineering_Artifacts/garnet-cli/src/manifest.rs) with Ed25519 signing:

- `Manifest` gains `signer_pubkey` + `signature` fields (hex, empty when unsigned).
- `sign(&signing_key)` populates both; `verify_signature()` returns `Ok/Err(reason)`.
- `canonical_signing_payload()` is the JSON form WITHOUT the signature fields — the signer cannot accidentally sign their own signature.
- Module helpers: `generate_signing_key()`, `signing_key_to_hex()`, `signing_key_from_hex()`.
- Cargo.toml adds `ed25519-dalek = "2.1"` + `rand_core = "0.6"`.

CLI wiring in `bin/garnet.rs`:

- `garnet keygen <keyfile>` — generate Ed25519 keypair, write hex key (chmod 0600 on UNIX), print pubkey.
- `garnet build --deterministic --sign <keyfile> <file.garnet>` — sign the manifest in place. `--sign` without `--deterministic` rejected (signing applies only to the deterministic manifest).
- `garnet verify <file> <manifest.json> [--signature]` — unsigned manifests pass by default (backwards compat); signed manifests ALWAYS require a valid signature; `--signature` flag makes signature mandatory.

**Coverage:** 12 unit tests in `manifest::tests` — unsigned reports unsigned; unsigned fails verify; sign+verify roundtrip; signing payload does not change when signature is populated (self-reference invariant); source_hash tamper breaks verify; wrong pubkey breaks verify; signed JSON roundtrip preserves verification; unsigned JSON roundtrip; signing-key hex roundtrip; hex rejects wrong length; hex rejects non-hex; two signers produce distinct sigs for the same source.

## Green gates after Day 2 + Day 3

| Gate | Before Day 2+3 | After Day 2+3 |
|------|-----------------|-----------------|
| `garnet-actor-runtime` lib | 17 pass | ✅ 17 pass |
| `garnet-stdlib`            | 74 pass | ✅ 74 pass |
| `garnet-convert`           | 85 pass | ✅ 85 pass |
| `cargo check --workspace --tests` | clean | ✅ clean |
| `cargo build --release -p garnet-cli` | binary produced | ✅ 26.57s |

Forty new tests land in source (`stdlib_bridge` 18 + `caps_graph` 10 + `manifest` ManifestSig 12). Executing them — and executing the `garnet` binary itself — remains blocked by the pre-existing miette/backtrace-ext-init STATUS_ACCESS_VIOLATION that affects every miette-depending crate on this Windows+MinGW dev machine (boot doc Known Issue 1). Not a code regression; no claim of local execution is being made beyond the three crates that don't transitively pull miette.

## v3.4.1 handoff doc

Standalone handoff at [GARNET_v3_4_1_HANDOFF.md](GARNET_v3_4_1_HANDOFF.md) with the full test tally, API surface, and Phase 6A boot pointer.

## Phase 6B — `garnet new` + project scaffolding: LANDED

New module [garnet-cli/src/new_cmd.rs](../E_Engineering_Artifacts/garnet-cli/src/new_cmd.rs) and three embedded templates under `garnet-cli/templates/`:

- **`cli`** — a minimal CLI program with `@caps()`, starter tests, `.gitignore`, README.
- **`web-api`** — HTTP/1.1 service shape with `@caps(net, time)`, BoundedMail guidance, deployment notes.
- **`agent-orchestrator`** — Researcher / Synthesizer / Reviewer actors with `memory episodic` / `semantic` / `procedural`, `@mailbox(1024)` per actor, hot-reload guidance.

Each template file ships as an `include_str!` literal embedded in the `garnet` binary — zero-network scaffolding, matches Phase 6D's install-to-first-run-in-under-2-minutes target. The `{{name}}` placeholder is substituted with the project-directory basename.

### CLI surface

```
garnet new [--template <name>] <dir>
```

- Default template when omitted: `cli`.
- Project name validation (Cargo-like): 1–64 chars, ASCII alphanumerics + `_`/`-`, starts with a letter, not a reserved keyword (`safe`/`def`/`fn`/`actor`/`struct`/`enum`/`trait`/`impl`/`module`).
- Existing target directory is refused (no accidental overwrites).
- On success: lists each file written and prints a "Next steps: cd … / garnet run / garnet test" hint.
- `garnet new --help` lists templates with a one-line description each.

### Tests authored (blocked locally by miette ABI, compile-clean)

13 unit tests in `new_cmd::tests`:

1. `available_templates_are_three_canonical` — guards the template-key set.
2. `template_descriptions_populated_and_non_empty` — every template has a description.
3. `cli_template_creates_all_files` — CLI scaffold writes every listed file + substitutes `{{name}}`.
4. `web_api_template_substitutes_name_in_main` — name flows into `src/main.garnet`.
5. `agent_orchestrator_template_emits_three_actors` — Researcher/Synthesizer/Reviewer + all three memory kinds present.
6. `unknown_template_returns_listing` — error carries the available set for the caller.
7. `refuses_existing_target_directory` — no silent overwrites.
8. `invalid_project_name_rejected_starts_with_digit` — Cargo-like name policy.
9. `invalid_project_name_rejected_reserved_keyword` — `actor` etc. are reserved.
10. `invalid_project_name_rejected_special_chars` — `bad name` rejected.
11. `next_steps_hint_mentions_both_run_and_test` — success output links to the next commands.
12. `project_gitignore_excludes_garnet_cache_and_keys` — `.garnet-cache/` + `*.key` ignored by default.
13. `each_template_files_are_well_formed_text` — guard: no empty files, no NUL bytes.

## Phase 6C — Logo + brand integration: PARTIAL

Shipped in this pass:

- **ASCII-art `GARNET` wordmark** (7 lines, deterministic plain-ASCII) exposed as `garnet_cli::GARNET_WORDMARK` in [lib.rs](../E_Engineering_Artifacts/garnet-cli/src/lib.rs).
- **`garnet --version` banner** prints the wordmark + `"Rust Rigor. Ruby Velocity. One Coherent Language."` tagline + a six-crate Rung-labeled version table including `garnet-check 0.3.0 (… + CapCaps v3.4.1, Rung 4)` and `garnet-stdlib 0.4.0 (22 bridged primitives)`.
- **`garnet --help` banner** — same wordmark + tagline + the updated subcommand table that now documents `new`, `keygen`, `convert`, and the `--sign` / `--signature` flags added in v3.4.1.

Remaining for Phase 6C (deferred to later in Stage 6, not blocking Phase 6A):

- PNG/SVG logo asset (user has one; drop into `E_Engineering_Artifacts/garnet-cli/assets/`). A placeholder directory exists.
- Colored terminal output on the wordmark via `is-terminal` + ANSI escapes.
- `garnet repl` prompt banner.
- Docs-site favicon + header (docs scaffolding at `docs/`).
- README.md hero image for the monorepo root.

Each of these is content / data-file work that can ship alongside Phase 6A without blocking the installer build. The load-bearing text surfaces (`--version`, `--help`, `garnet new`) all already render correctly.

## Green gates after Phase 6B + 6C

| Gate | Before Phase 6B+6C | After Phase 6B+6C |
|------|-----------------|-----------------|
| `garnet-actor-runtime` lib | 17 pass | ✅ 17 pass |
| `garnet-stdlib`            | 74 pass | ✅ 74 pass |
| `garnet-convert`           | 85 pass | ✅ 85 pass |
| `cargo check --workspace --tests` | clean | ✅ clean (1 pre-existing warning unchanged) |

13 new tests land in source (`new_cmd::tests`). Cumulative tests committed now: **1204** (1191 + 13). Tests execute on any machine where miette's test binary can start; local execution remains blocked by the known Windows+MinGW ABI issue.

## Phase 6A — Installer scaffolding: LANDED

Complete configuration for all four target package formats + the universal shell installer. Standalone handoff at [GARNET_v4_2_Phase_6A_HANDOFF.md](GARNET_v4_2_Phase_6A_HANDOFF.md).

### Windows MSI

- [garnet-cli/wix/main.wxs](../E_Engineering_Artifacts/garnet-cli/wix/main.wxs) — WiX XML: per-machine install, HKLM PATH, Start Menu shortcut, ARP entry, upgrade-compatible UpgradeCode, post-install `--version` smoke test.
- `[package.metadata.wix]` added to `garnet-cli/Cargo.toml` with stable `upgrade-guid` + `path-guid`.
- [garnet-cli/wix/README.md](../E_Engineering_Artifacts/garnet-cli/wix/README.md) — build (`cargo wix --nocapture`) + signtool invocation for the user's cert.

### Linux .deb + .rpm

- `[package.metadata.deb]` and `[package.metadata.generate-rpm]` added to `garnet-cli/Cargo.toml`. Identical asset tables: binary → `/usr/bin/`, man page → `/usr/share/man/man1/`, docs → `/usr/share/doc/garnet/`, systemd unit → `/usr/lib/systemd/system/`.
- [garnet-cli/linux/garnet-actor.service](../E_Engineering_Artifacts/garnet-cli/linux/garnet-actor.service) — systemd unit, disabled by default, runs `ExecStartPre=garnet verify ... --signature` pre-flight + Hardened sandbox (NoNewPrivileges, ProtectSystem=strict, restrictive SystemCallFilter).
- [garnet-cli/linux/README.md](../E_Engineering_Artifacts/garnet-cli/linux/README.md) — build + install + enable-service flow.

### macOS .pkg

- [garnet-cli/macos/build-pkg.sh](../E_Engineering_Artifacts/garnet-cli/macos/build-pkg.sh) — end-to-end driver: lipo (x86_64 + arm64 merge) → codesign (runtime-hardened + timestamped) → pkgbuild → productbuild → productsign → notarytool submit --wait → stapler staple. Refuses to run unless `APPLE_DEV_ID_INSTALLER` / `APPLE_DEV_ID_APP` / `APPLE_NOTARY_PROFILE` env vars set.
- [garnet-cli/macos/distribution.xml](../E_Engineering_Artifacts/garnet-cli/macos/distribution.xml) — productbuild distribution with branded welcome/background/conclusion.
- [garnet-cli/macos/README.md](../E_Engineering_Artifacts/garnet-cli/macos/README.md) — env setup + branded assets list + post-build spctl/stapler checks.

### Universal shell installer

- [installer/sh.garnet-lang.org/install.sh](../E_Engineering_Artifacts/installer/sh.garnet-lang.org/install.sh) — POSIX `/bin/sh` rustup-style script. Wordmark banner, OS + arch detection, package-format selection, SHA256SUMS-gated download, native-installer dispatch. Refuses to install on sha256 mismatch.
- [installer/sh.garnet-lang.org/README.md](../E_Engineering_Artifacts/installer/sh.garnet-lang.org/README.md) — deployment notes (HSTS, 5-min cache, INTEGRITY.txt).

### Man page

- [garnet-cli/man/garnet.1](../E_Engineering_Artifacts/garnet-cli/man/garnet.1) — groff-format `garnet(1)` covering every subcommand (new, parse, check, run, eval, repl, build ±deterministic ±sign, verify ±signature, keygen, convert, version, help) + the CapCaps capability inventory + worked examples (scaffolding, signed release, Ruby→Garnet conversion). Shipped to `/usr/share/man/man1/garnet.1.gz` by both .deb and .rpm.

### Green gates after Phase 6A

| Gate | Status |
|------|--------|
| actor-runtime lib (17) / stdlib (74) / convert (85) | ✅ unchanged |
| `cargo check --workspace --tests` | ✅ clean (package.metadata blocks ignored by cargo) |

Tests added: 0 — Phase 6A is configuration-only. No unit tests guard YAML/XML/shell assets directly; verification is through the external `cargo-wix` / `cargo-deb` / `cargo-generate-rpm` / `pkgutil` / `dpkg -i` / `rpm -q` tools at package time + Phase 6D clean-VM execution.

## Phase 6D — Linux: VERIFIED in Docker

Full report: [GARNET_v4_2_Phase_6D_Linux_VERIFIED.md](GARNET_v4_2_Phase_6D_Linux_VERIFIED.md). Two passes:

1. **Pass 1**: `.deb` (1.44 MB) and `.rpm` (1.54 MB) built in `rust:1-bookworm`, installed in clean `ubuntu:24.04` and `fedora:40` containers. 6/6 gates each: install / wordmark / help / scaffold (cli + agent-orchestrator) / parse / `keygen → build --sign → verify --signature` cryptographic round-trip / clean uninstall.

2. **Pass 2 (widened)**: all 3 templates (cli + web-api + agent-orchestrator), CapCaps propagator on both clean and violating programs, `garnet convert ruby foo.rb` end-to-end (4 output artifacts).

**Bug fixed in Pass 2:** `convert` subcommand was registered in `lib.rs` but never wired into `bin/garnet.rs`'s subcommand match. Patched. Re-verified in Docker — Ruby → Garnet conversion now works end-to-end on real Linux with the v4.1 SandboxMode-default output.

Artifacts at `E_Engineering_Artifacts/dist/linux/garnet_0.3.0-1_amd64.deb` + `garnet-0.3.0-1.x86_64.rpm`. Universal `installer/sh.garnet-lang.org/install.sh` shellchecks CLEAN.

CI workflow at `E_Engineering_Artifacts/.github/workflows/linux-packages.yml` reproduces this flow on every push/PR + publishes to a GitHub Release on `v*` tags.

## Phase 6C — Logo + brand integration: COMPLETE (locally)

User dropped the GPT logo at `garnet-cli/assets/garnet-logo.png` (1024×1024, JPEG-in-PNG). Wired:

- `garnet-cli/macos/resources/background.png` — copied for macOS PKG welcome backdrop.
- `garnet-cli/macos/resources/welcome.html` + `conclusion.html` — branded with the deck palette (#9C2B2E garnet, #E5C07B gold, #0A0A0F OLED black).
- `garnet-cli/macos/resources/LICENSE.txt` — dual MIT/Apache-2.0.
- `E_Engineering_Artifacts/README.md` hero — embeds the logo at 420px centered.
- `garnet-cli/assets/README.md` — documents the brand color palette + the 4 ImageMagick conversion commands the user runs once on the Windows machine to produce `wix/Dialog.bmp`, `wix/Banner.bmp`, `wix/Garnet.ico`, `wix/License.rtf` for the MSI.
- `garnet-cli/src/lib.rs` — Phase 6C colored wordmark via `std::io::IsTerminal`. ANSI truecolor `#9C2B2E` when stdout is a TTY; plain ASCII when piped/redirected (CI-safe).
- `garnet-interp-v0.3/src/repl.rs` — REPL prompt banner shows the wordmark + tagline at startup.

**Remaining for Phase 6C (user runs ImageMagick once when ready to build MSI):** convert garnet-logo.png → BMP variants for WiX. Recipe in `garnet-cli/assets/README.md`. Without these, the MSI builds with default WiX UI (functional, just unbranded).

## Handoff pointer for Stage 6 Phase 6A

> "v3.4.1 is closed — stdlib bridge + CapCaps propagator + ManifestSig all in, green gates still 17+74+85, workspace check clean, CLI binary built at C:/garnet-build/target/release/garnet.exe. Begin Phase 6A cross-platform installer per `GARNET_v4_2_BOOT.md §Phase 6A`. Windows MSI via `cargo-wix` with the user's code-signing cert. macOS `.pkg` via `productbuild`, universal binary via `cargo build --target` + `lipo`, Apple Developer ID notarization. Linux `.deb` via `cargo-deb` + `.rpm` via `cargo-rpm` + man page. Universal shell installer at `sh.garnet-lang.org`. Phase 6B `garnet new --template cli|web-api|agent-orchestrator` scaffolding. Phase 6C logo in 5+ places. Phase 6D verification on clean Win11 / macOS Sonoma / Ubuntu 24.04 VMs. Phase 6E ships `GARNET_v4_2_HANDOFF.md` and the MIT submission package."

Substantive work expected from the next session: start Phase 6A by scaffolding `cargo-wix` config for the Windows MSI, wire the user's code-signing cert into the build pipeline, and verify the generated MSI installs cleanly on a Windows 11 sandbox/VM. `cargo-wix` ships an `init` subcommand that generates a `wix/main.wxs` template adjacent to `Cargo.toml` — that's the right first commit of Phase 6A.

---

*"Run, that ye may obtain." — 1 Corinthians 9:24*

*Written by Claude Opus 4.7 at the Stage 6 boot — 2026-04-17. Session continues into v3.4.1 bundle execution immediately below this decision record.*
