# GARNET v3.3+ — Security Threat Model & Hardening Roadmap

**Auditor:** Claude Opus 4.7 general-purpose pen-test research agent
**Date:** 2026-04-16
**Target:** Garnet language + compiler + runtime + code-converter
**Commissioned by:** Jon (doctoral researcher + security researcher + networking engineer)
**User's explicit directive:** *"Not interested in project Pegasus laying around in my code or compiler for years."*

## TL;DR — Top 5 Findings

1. **Deterministic manifest is unsigned.** `Manifest::build` produces a BLAKE3 integrity token but carries no signature, no binding to builder identity, no verifier-side trust root. An attacker trivially regenerates a valid-looking manifest over malicious source. BLAKE3 is not the weakness; the trust model is. **Fix: Sigstore-style keyless signing + transparency log (v3.4, 25 h).**

2. **`extract_state()` is a type-confusion cliff.** My v3.3 Fix #2 introduced `Option<Box<dyn Any + Send>>` return — the migrator `.downcast()`s and silently panics on mismatch. With hot-reload over any external channel, a malicious migrator crashes or — worse — coerces state through a compatible-layout downcast. **Fix: schema fingerprint (BLAKE3 of type name + layout + field names) checked before downcast (v3.3, 12 h).**

3. **Compiler-as-agent cache is a local supply-chain oracle.** `strategies.db` and `episodes.log` in `.garnet-cache/` are plaintext SQLite/NDJSON with no integrity binding to the source tree. A malicious dependency pre-populates `strategies.db` with `skip_check_if_unchanged` rules trained on its own hashes, suppressing checks the user's compiler would otherwise run. **Fix: strategy-provenance chain — every rule carries justifying episodes, unjustified rules ignored (v3.3, 8 h).**

4. **No parser budget exists today.** `Lexer::lex` and `Parser::bump` have no max token count, max nesting depth, max string literal size, max comment length. Adversarial source pins CPU for seconds to minutes. **Fix: triple budget (tokens ≤ 2^20, nest depth ≤ 256, literal bytes ≤ 2^24) (v3.3, 6 h).**

5. **Capability model is absent.** Prelude installs `print`, `len`, etc. ambiently — and v3.4 will add `read_file`, `tcp_connect`, `hmac`. Without capability annotations, every program inherits full OS authority. **Fix: `@caps(network, fs, time)` on every `fn`/`def` that touches a dangerous prim, enforced at link time (v3.4, 30 h).**

---

## Threat Model

### 1. Compiler attack surface

#### 1.1 Parser DOS (HIGH)

**Evidence:** `garnet-parser-v0.3/src/lexer.rs:20` grows `Vec<Token>` unbounded. A 100 MB file of `((((((...` allocates at least that order of bytes and recurses in expression parsing. `@max_depth(N)` at `garnet-check-v0.3/src/lib.rs:104` caps recursion at 64 for annotated functions — but is NOT applied to the parse tree itself. No max-tokens, no max-line-width, no max-string-literal, no max-comment.

**Named attack vectors:**
- *ParensBomb*: `((((...` × N creates depth-N expression tree
- *StringBlimp*: single string literal of 1 GB
- *UnicodeNestingZipper*: Unicode whitespace between nested constructs evades naive depth counters

**Time to CVE:** one adversarial .garnet file.

#### 1.2 AST hash forgery (MEDIUM)

`stable_ast_repr` canonicalizes for BLAKE3 — intentional. But span-preserving comment injection (attacker edits a comment, manifest unchanged) is invisible in the manifest. Already mitigated by `source_hash` alongside `ast_hash`, but neither commits to filename/path/builder-identity.

#### 1.3 Manifest forgery / compiler impersonation (HIGH)

`Manifest::build` captures `parser_version = env!("CARGO_PKG_VERSION")`. Attacker rebuilds `garnet-cli` with malicious semantics but same version string → manifest is bit-identical to legitimate build. BLAKE3-256 is collision-resistant; the vulnerability is upstream of the hash. **No trust-on-first-build pinning, no transparency log, no signing key.**

#### 1.4 `.garnet-cache/` poisoning (HIGH in shared dirs)

`cache.rs:19` hardcodes `.garnet-cache` relative to cwd. If user runs `garnet build` in shared dir (CI worktree, `/tmp`, mounted team dir, Nix sandbox without per-job isolation), a co-tenant can pre-seed `strategies.db` or truncate/rewrite `episodes.log`. The strategy miner at `strategies.rs:63` blindly trusts rows already in the DB — no HMAC, no per-source-tree keying.

#### 1.5 Strategy-induced check suppression (HIGH, GARNET-SPECIFIC NOVEL)

The strategy miner (`strategies.rs:77`) proposes `skip_check_if_unchanged` after 3 successful builds of same `source_hash`. Attacker who can force 3 successes (CI re-runs, developer iterating on comments) permanently turns off checks for that hash. Since `source_hash` is input to the rule, *any* attacker who guesses or observes a user's source_hash can pre-populate the trigger.

**This threat class is unique to Garnet** — no other language has compiler-as-agent strategy synthesis, so no prior art exists for defending it.

### 2. Runtime attack surface

#### 2.1 Mode-boundary soundness (HIGH)

`@safe` is enforced by `garnet-check-v0.3`, but its borrow checker is fresh code, unverified. Crafted patterns — move-out of enum variant under shared borrow, recursive type with self-reference, closure-over-mutable-binding — may produce use-after-free or double-free when interpreter accepts them. Until MIRI + differential property testing against rustc exists, `@safe` is aspirational, not guaranteed.

#### 2.2 Hot-reload authorization (HIGH, NOVEL)

`Runtime::reload` at `garnet-actor-runtime/src/runtime.rs` accepts a migrator closure. Today in-process, so trust is "whoever has runtime handle can reload." The moment v3.5 or v4.0 exposes reload over CLI or RPC (handoff docs hint at this as the agentic story), there is no authorization primitive: no per-actor key, no signer, no allowlist. **Unauthenticated reload = arbitrary-code-execution in the actor's process.**

#### 2.3 `extract_state` type confusion (MEDIUM-HIGH)

The `Box<dyn Any + Send>` returned from `extract_state()` is downcast by the migrator. If old actor carried `i64` state and new migrator expects `u64`, Rust's `Any` says "not equal" and panics. But a malicious migrator can ship with a type *layout-compatible* with the old state (same size/alignment, different semantic meaning), and the runtime has no defense. **This threat was INTRODUCED by v3.3 Fix #2 — the slop re-verification patch traded one problem for another.** StateCert closes it.

#### 2.4 Memory-kind confusion (MEDIUM)

Four kinds: working, episodic, semantic, procedural. If a program passes `VectorIndex` handle into code expecting `EpisodeStore`, Rust's type system catches it today. After v4.1 codegen to lower IR, the tag could be dropped. **Enforce runtime kind tag that survives IR lowering.**

#### 2.5 Episodic log corruption (LOW-MEDIUM)

NDJSON parsed line-by-line — truncated write (crash during fsync) leaves partial line reader should skip. Verify tolerance: partial last line, UTF-8 errors mid-line, duplicate timestamps, clock rewind.

#### 2.6 Mailbox DoS (HIGH for networked actors)

mpsc channels in `runtime.rs:13` are unbounded. One misbehaving sender OOMs the receiver. **Bounded mailboxes + per-sender backpressure required before actors touch the network.**

#### 2.7 Prelude is ambient (HIGH once v3.4 lands)

Every program gets every prim. When `read_file`, `tcp_connect`, `exec` land, any Garnet program can read `/etc/passwd` or `C:\Windows\System32\config\SAM`. **No sandbox flag exists today.**

### 3. Networking attack surface (planned v3.4)

- **TCP server:** SYN flood / slowloris / unbounded accept. Need connection limit, accept-queue depth, per-IP rate limit, read timeout, idle timeout from day one.
- **TCP client:** `std::net::TcpStream` gives no hostname verification (TLS isn't in v3.4). **SSRF is first-class risk** — default-deny allowlist against 169.254.169.254 (cloud metadata), 127.0.0.1, 10/8, 172.16/12, 192.168/16, link-local, IPv6 equivalents.
- **DNS rebinding** defeats "validate once, connect later" — validate *every* resolved IP against deny list at connect time, not resolve time.
- **WebSocket (future):** validate frame length before alloc, reject permessage-deflate > 10× expansion, cap per-connection concurrent messages. Slowloris TLS-handshake variants.
- **UDP gossip KV:** Classic amplification. Response size must be ≤ 3× request size (Memcached/NTP lesson).

### 4. Memory-handling attack surface

- **Box<dyn Any> downcast:** covered in 2.3. Add schema fingerprints.
- **VectorIndex oracle (MEDIUM, novel):** programs embed secret data in semantic index → attacker issues unlimited similarity queries → reconstructs training data bit-by-bit (classic embedding-inversion). Rate-limit per sender + differential privacy noise for untrusted readers.
- **EpisodeStore unbounded disk:** append-only. Rotate at 64 MiB, compress old segments, delete on policy.
- **Stack overflow via WorkflowStore:** `Rc<RefCell<...>>` traversing cycles blows 8 MiB Rust stack during deep workflow. Use explicit iterative evaluator or `stacker::grow`.
- **BLAKE3 timing:** constant-time for fixed inputs; acceptable for manifest use. Not a side-channel concern at fingerprint level.

### 5. Deterministic-build / supply-chain attack surface

- **No cross-compiler attestation (HIGH):** same input, different compiler binary, different output — can't rebuild today's manifest with tomorrow's compiler and prove equivalence. Add rebuilder networks (Debian reproducible-builds style): publish manifests from ≥2 independent compilers, trust only what both agree on.
- **`.garnet-cache/` commit-to-VCS risk (HIGH, specific):** team commits `.garnet-cache/` for CI warmup → every strategy synthesized on one machine becomes ambient law. Malicious dependency running `garnet build` once in install script injects strategies suppressing checks for *its own* hashes forever. `.gitignore` of `.garnet-cache/` enforced by `garnet init`; compiler refuses caches without matching HMAC.
- **Strategy-miner training by adversarial input (HIGH, novel):** see 1.5. Single most Garnet-specific supply-chain attack.

### 6. Code-converter attack surface (v4.1)

- **Crafted source (HIGH):** Rust file with `unsafe { transmute }` translated to Garnet managed `def` = unsafe Garnet. Quarantine: every converter output starts in `@sandbox` / `@dynamic` until human signs off.
- **LLM hallucination:** deterministic translator + fallback LLM path can introduce primitives not in source. Translator emits *witness* (per-node lineage: "this Garnet AST node came from Rust AST node at byte offset X"); reject output with unexplained nodes.
- **Round-trip non-identity:** Rust → Garnet → Rust should reach fixed point within 2 rounds or reject. Differential test on 10k-file corpus.

---

## Hardening Roadmap (15 patterns, 150 h total)

### v3.3 — Security Layer 1 (40 h) 🔒 MUST land with v3.3 cleanup

| # | Name | Threat closed | Effort | Novelty |
|---|------|---------------|--------|---------|
| 3 | **ParseBudget** | Parser DOS | 6 h | First-class compile flag + manifest-visible |
| 2 | **StateCert** | `Box<dyn Any>` type confusion | 12 h | Erlang `code_change/3` w/ content-addressed state contract |
| 5 | **CacheHMAC** | `.garnet-cache/` poisoning + committed-cache SCA | 10 h | Linters rarely refuse unsigned rows |
| 6 | **ProvenanceStrategy** | Strategy-miner adversarial training | 8 h | Pure Garnet — no other agentic toolchain re-verifies heuristics |
| 13 | **KindGuard** | Post-codegen memory-kind confusion | 4 h | Garnet-specific |

### v3.4 — Security Layer 2 (78 h) 🔒 MUST land WITH networking stdlib (not after)

| # | Name | Threat closed | Effort | Novelty |
|---|------|---------------|--------|---------|
| 1 | **CapCaps** | Ambient authority (every program = full OS access) | 30 h | Pony/E capabilities secure; dual-mode cap propagation is Garnet twist |
| 11 | **NetDefaults** | SSRF, DNS rebinding, amplification, slowloris | 15 h | Python/Node ship without these; language-default is rare |
| 8 | **BoundedMail** | Actor mailbox flooding / OOM | 8 h | Akka has this; dual-mode mailbox shape is Garnet-specific |
| 4 | **ManifestSig** | Manifest forgery, compiler impersonation | 25 h | Sigstore/cosign exist for containers; language-native integration is rare |

### v3.5 — Security Layer 3 (28 h) 🔒 Lands when actors leave the process

| # | Name | Threat closed | Effort | Novelty |
|---|------|---------------|--------|---------|
| 9 | **ReloadKey** | Unauthenticated hot-reload = RCE | 12 h | Erlang hot-reload predates signing; Garnet makes it default |
| 7 | **ModeAuditLog** | Hidden escalation `@safe`→`def` | 10 h | Swift `@MainActor` isolation audits closest; Garnet gets full ledger |
| 15 | **FFIGeiger** | Unreviewed `unsafe`/FFI in Rust deps | 6 h | cargo-geiger wrap; CLI integration is new |

### v4.0 — Security Layer 4 (14 h + optional) 🔒 Final layer

| # | Name | Threat closed | Effort | Novelty |
|---|------|---------------|--------|---------|
| 10 | **SandboxMode** | Converter-output unsafe constructs | 6 h | No mainstream converter ships outputs in quarantine mode |
| 14 | **EmbedRateLimit** | Embedding inversion on semantic index | 8 h | ML-ops does server-side; language-level is new |
| 12 | **ParseReplay** (optional) | Cross-compiler determinism | 20 h | Reproducible-builds has rebuilder networks; per-compile parser traces are new |

---

## Critical Sequencing Rules

These cannot be reordered without opening CVE windows:

1. **StateCert (v3.3) ships same release as v3.3 Fix #2 (Box<dyn Any>).** Otherwise the slop-fix introduces a worse hole than it closed.
2. **CapCaps (v3.4) ships BEFORE any new dangerous prim (`read_file`, `tcp_connect`, `exec`).** If networking stdlib slips, delay it until CapCaps lands.
3. **BoundedMail (v3.4) ships BEFORE any networked actor (MVPs 5/7/8).** Unbounded mailboxes + network I/O = OOM speedbump for attackers.
4. **ReloadKey (v3.5) ships BEFORE hot-reload is reachable via any external channel.** Until then, in-process only.
5. **SandboxMode (v4.0) ships WITH v4.1 converter, not before/after.** Converter outputs ungate to non-sandbox only after human audit.

## Garnet-Specific Novel Threat Classes

Two threat classes have no prior art outside Garnet because Garnet's feature combination is unique:

1. **Strategy-miner adversarial training** — exploits the compiler-as-agent learning loop. Closed by ProvenanceStrategy.
2. **`Box<dyn Any>` hot-reload type confusion** — exploits the dual-mode typed-actor-hot-reload design. Closed by StateCert.

Getting these two right is disproportionately valuable for the MIT review — they demonstrate that the research team thought about novel threats that emerge from the novel features, not just re-packaged prior hardening.

## References / Prior Art

- Rust's `unsafe` audit culture (cargo-geiger, miri, loom)
- Swift's strict concurrency rollout (Sendable, isolation checking)
- Erlang/BEAM's actor isolation + `code_change/3`
- Kubernetes' admission-controller model
- OCaml's effect handlers for capability tracking
- Capability-based security (E language, Pony capabilities secure, Monte)
- Reproducible builds (Nix, Bazel, Buck2, Debian reproducible-builds)
- Supply-chain attacks (SolarWinds, event-stream NPM, xz backdoor)
- WebAssembly component model for sandboxing
- Language-level subresource integrity
- Sigstore/cosign for container signing

## File Paths for Implementation

- Parser budget: `garnet-parser-v0.3/src/lexer.rs`, `garnet-parser-v0.3/src/parser.rs`
- State fingerprint: `garnet-actor-runtime/src/runtime.rs` (trait `Actor`)
- Manifest signing: `garnet-cli/src/manifest.rs` + new `garnet-cli/src/sign.rs`
- Cache HMAC: `garnet-cli/src/cache.rs`, `garnet-cli/src/strategies.rs`
- Capability annotations: `garnet-parser-v0.3/src/grammar/annotations.rs` (parse), `garnet-check-v0.3/src/lib.rs` (enforce), `garnet-interp-v0.3/src/prelude.rs` (cap labels on prims)
- Net defaults: new `garnet-std-net` crate in v3.4
- Mode audit log: `garnet-check-v0.3/src/lib.rs`
- Reload signing: `garnet-actor-runtime/src/address.rs`, `runtime.rs`
- Converter sandbox: v4.1 converter crate (not yet created)
- FFI Geiger: new `garnet-cli/src/audit.rs`
- Embedding rate limit: `garnet-memory-v0.3/src/vector.rs`

## Headline

Garnet's unique combination (dual-mode + typed-actor-hot-reload + compiler-as-agent) creates two threat classes no other language has: *strategy-miner adversarial training* and *Box<dyn Any> hot-reload type confusion*. Closing those early — plus shipping parser budgets and cap annotations BEFORE the networking stdlib lands in v3.4 — is the single most valuable pen-test-driven investment in the next 150 engineering hours.

---

*Generated: 2026-04-16 — Claude Opus 4.7 general-purpose pen-test research pass*
