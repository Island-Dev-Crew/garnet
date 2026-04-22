# GARNET v3.4 — Security Layer 2 Specification

**Stage:** 2 (P0 Stdlib + Security Layer 2 + First 4 MVPs)
**Date:** April 16, 2026 (Stage 2 Phase 2A-SEC opening)
**Author:** Claude Code (Opus 4.7)
**Status:** Normative implementation specification — gates Stage 2 stdlib work
**Anchor:** *"Set thy heart toward the highway, even the way which thou wentest." — Jeremiah 31:21*

---

## Master Sequencing Rule (RE-STATED)

**No P0 stdlib primitive that touches OS authority — `read_file`, `write_file`, `tcp_connect`, `tcp_listen`, `udp_bind`, `exec`, `now_ms` — ships before its paired security gate is in place.** This rule is non-negotiable per the v3.3 threat model and the user's explicit "no Pegasus laying around" directive.

The four Layer-2 hardening items in this document are the gates. Each is paired with a Stage 2 stdlib subsystem; the stdlib subsystem MUST cite the gate in its PR description and MUST NOT merge without the gate's tests passing.

| Stdlib subsystem | Gate (this doc) | Effort budget |
|------------------|-----------------|---------------|
| Networking (~300 LOC) | NetDefaults + CapCaps | 30h + 15h |
| File I/O (~200 LOC) | CapCaps | (covered by CapCaps 30h) |
| Time/timers (~100 LOC) | CapCaps | (covered) |
| Actor mailbox enhancements | BoundedMail | 8h |
| Build/sign workflow | ManifestSig | 25h |

Total Layer 2 budget: **78 hours** of focused engineering. v3.3 came in at ~45% of its 40h budget because the threat model did the design up front; v3.4 is expected to come in at ~60% (so ~46–55 actual hrs) for the same reason.

---

## §1. CapCaps — Capability-annotated prelude (30h)

### 1.1 Threat closed

Without CapCaps, every Garnet program that imports the prelude (i.e., every program) inherits ambient authority over every primitive in the stdlib. When v3.4 lands `read_file` and `tcp_connect`, that means every random `.garnet` file can read `/etc/passwd` or open a TCP connection to any host. That is the Pegasus default.

CapCaps gates ambient authority at the call graph level: a function MAY only invoke a privileged primitive if the function's capability annotations transitively cover the primitive's required capability.

### 1.2 Surface syntax

```
caps-annotation := "@caps" "(" cap-list ")"
cap-list        := cap ("," cap)*
cap             := "fs" | "net" | "net_internal" | "time" | "proc" | "ffi" | "*" | ident
```

Examples:

```garnet
@caps(fs)
def read_config(path) {
  read_file(path)        # OK — fs cap declared
}

@caps(net)
def fetch_status(host) {
  tcp_connect(host, 80)  # OK — net cap declared
}

@caps()
def pure_compute(x, y) {
  # neither read_file nor tcp_connect is callable here
  x + y
}
```

The `@caps()` form (empty list) explicitly declares "no caps required" — useful for marking a function as pure-computational.

### 1.3 Semantics — the call-graph propagation rule

For every function F declared with `@caps(c₁, …, cₙ)`, the checker MUST verify that every primitive call site within F (or transitively reachable from F's body) requires only capabilities in `{c₁, …, cₙ}`. If a transitive call to a primitive P requiring capability `c` is reachable AND `c ∉ caps(F)`, the checker MUST emit:

```
error E0903: function `F` calls primitive `P` requiring capability `c`,
             but its @caps annotation does not include `c`
   --> src/foo.garnet:42:10
   |
42 |   tcp_connect(host, 80)
   |   ^^^^^^^^^^^ requires @caps(net)
   |
help: add `net` to F's capability set:
   |
36 | @caps(fs, net)        # or @caps(net) if fs not needed
   | ^^^^^^^^^^^^^^
```

### 1.4 The `main` function: declaration of authority

The program's `main` function MUST carry an explicit `@caps(...)` annotation. This is the user's declaration of what authority the program needs from the OS. A program that needs no OS authority writes `@caps()`. A program that does file I/O writes `@caps(fs)`. The annotation propagates from `main` down through the call graph by the §1.3 rule.

Diagnostic on missing `@caps` on `main`:

```
error E0904: `main` function must declare its required capabilities
   --> src/main.garnet:1:1
   |
1  | def main() { ... }
   | ^^^^^^^^^^ missing @caps annotation
   |
help: add `@caps(...)` listing the OS authority this program needs
   |
1  | @caps(fs, net)
1  | def main() { ... }
```

### 1.5 The narrowing rule (`@safe` modules can narrow but not widen)

Per master plan: "Dual-mode twist: `@safe` functions can narrow caps but never widen." The formal rule:

If `@safe` module M imports a managed-mode function `f` annotated `@caps(c₁, …, cₙ)`, and M re-wraps it into a safe function `g` annotated `@caps(c'₁, …, c'ₘ)`, then `{c'} ⊆ {c}` MUST hold (no widening). The compiler emits `error E0905` on widening.

Rationale: a safe wrapper around a less-restricted managed function is a *subset* of authority — that's defensible. A safe wrapper that adds capabilities the underlying managed function never had would be a confusion-of-deputy attack.

### 1.6 Capability set hierarchy

| Cap | Covers | Examples |
|-----|--------|----------|
| `fs` | Read + write of any path | `read_file`, `write_file`, `list_dir`, `stat`, `remove_file` |
| `net` | TCP/UDP outbound + listen | `tcp_connect`, `tcp_listen`, `udp_bind`, `udp_send` |
| `net_internal` | Bypass NetDefaults RFC1918/loopback denylist | NetDefaults exception path |
| `time` | Wall clock + sleep | `now_ms`, `wall_clock_ms`, `sleep` |
| `proc` | Process spawn, signals | `exec`, `kill_process` |
| `ffi` | extern "C" calls | `extern fn …` |
| `*` | Wildcard (DEBUG ONLY; CI rejects in release builds) | All of the above |

`net_internal` is a *strict* subset of `net` — declaring `@caps(net_internal)` does NOT grant `net`. This is the defense against SSRF attacks that need only loopback (e.g., metadata service).

### 1.7 Compiler implementation outline

Files (Rust):

- `garnet-parser-v0.3/src/grammar/annotations.rs` — extend annotation parser to recognize `@caps(...)`. Already handles `@max_depth` / `@fan_out` / `@require_metadata`; same pattern.
- `garnet-check-v0.3/src/caps.rs` — new module. Builds a per-function capability set + a per-primitive required-cap map; runs call-graph propagation.
- `garnet-check-v0.3/src/lib.rs` — invokes `caps::check_program` after the existing borrow-check pass.
- `garnet-interp-v0.3/src/prelude.rs` — labels every primitive with its required cap. Centralized table.

Test plan (~25 tests):
- Happy path: function with required caps calls primitive — accepts
- Negative: function missing cap calls primitive — rejects with E0903
- Transitive: function A → B → primitive — A must have cap; reject if not
- Narrowing: safe wraps managed with subset cap — accepts
- Widening: safe wraps managed with superset cap — rejects with E0905
- main missing @caps — rejects with E0904
- main with `@caps()` calling pure function — accepts
- net_internal does not satisfy net — rejects per §1.6

### 1.8 Sequencing rule (CRITICAL)

**CapCaps MUST land BEFORE Phase 2A merges any networking or fs primitive into the prelude.** PR ordering: (a) CapCaps lands first → (b) NetDefaults lands → (c) Networking stdlib lands → (d) File I/O stdlib lands. Any deviation from this order is a release blocker.

---

## §2. NetDefaults — Secure-by-default networking (15h)

### 2.1 Threat closed

Without NetDefaults, `tcp_connect("169.254.169.254", 80)` succeeds. That's the AWS metadata service. Same for `127.0.0.1`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `fe80::/10`. Without DNS-rebinding defense, `tcp_connect("evil.com", 80)` resolves at validation time to `1.2.3.4` (good), then resolves at connect time to `169.254.169.254` (bad).

NetDefaults blocks both attack classes by default. Programs that genuinely need internal access opt in via `@caps(net_internal)`.

### 2.2 IP denylist (RFC 1918 + cloud metadata + link-local)

The default denylist (IPv4):

| CIDR | Range | Reason |
|------|-------|--------|
| 0.0.0.0/8 | 0.0.0.0–0.255.255.255 | Reserved, "this network" |
| 10.0.0.0/8 | 10.x.x.x | RFC1918 private |
| 100.64.0.0/10 | CGNAT | RFC6598 |
| 127.0.0.0/8 | loopback | RFC1122 |
| 169.254.0.0/16 | link-local + cloud metadata | RFC3927 + AWS/GCP/Azure metadata |
| 172.16.0.0/12 | RFC1918 private | |
| 192.0.0.0/24 | reserved | RFC6890 |
| 192.0.2.0/24 | TEST-NET-1 | |
| 192.168.0.0/16 | RFC1918 private | |
| 198.18.0.0/15 | benchmark | RFC2544 |
| 198.51.100.0/24 | TEST-NET-2 | |
| 203.0.113.0/24 | TEST-NET-3 | |
| 224.0.0.0/4 | multicast | |
| 240.0.0.0/4 | reserved | |
| 255.255.255.255 | broadcast | |

IPv6 denylist:

| CIDR | Reason |
|------|--------|
| ::/128 | unspecified |
| ::1/128 | loopback |
| fc00::/7 | RFC4193 ULA |
| fe80::/10 | link-local |
| ff00::/8 | multicast |
| 2001:db8::/32 | docs / examples |

### 2.3 DNS rebinding defense

Validate the resolved IP at *connect time*, not at *resolve time*. The implementation:

```rust
fn tcp_connect_safe(host: &str, port: u16, caps: &CapSet) -> Result<TcpStream, NetError> {
    let addrs = (host, port).to_socket_addrs()?;  // resolve
    for addr in addrs {
        if !is_allowed(&addr.ip(), caps) {
            return Err(NetError::DeniedByPolicy { ip: addr.ip() });
        }
        // Re-validate at connect time — not just at resolve time
        let stream = TcpStream::connect(addr)?;
        let peer = stream.peer_addr()?.ip();
        if !is_allowed(&peer, caps) {
            // Drop the connection if the peer differs from our validated addr
            return Err(NetError::DeniedByPolicy { ip: peer });
        }
        return Ok(stream);
    }
    Err(NetError::NoAddrs)
}
```

This catches the rebinding pattern where DNS returns one address at resolve time and another at connect time.

### 2.4 UDP amplification cap

UDP responses MUST be ≤ 3× the size of the request that triggered them. This is the Memcached / NTP amplification lesson. Enforced at the per-actor UDP socket layer:

```rust
fn udp_send_response(sock: &UdpSocket, peer: SocketAddr, request_size: usize, response: &[u8]) -> Result<(), NetError> {
    if response.len() > 3 * request_size {
        return Err(NetError::AmpCapExceeded { request_size, response_size: response.len() });
    }
    sock.send_to(response, peer)?;
    Ok(())
}
```

### 2.5 Read/idle timeouts

Default read timeout: 30 seconds. Default idle timeout: 5 minutes. Both configurable via `tcp_set_timeout(sock, ms)`. Slowloris defense.

### 2.6 Listening sockets — accept queue + per-IP cap

Default accept queue depth: 128 (configurable via `tcp_listen(port, opts)` where opts includes `accept_queue_depth`).

Per-IP cap on simultaneous connections: 32 (configurable). Defends against SYN-flood-by-many-distinct-IPs by tracking in-flight per peer.

### 2.7 File layout

New crate: `E_Engineering_Artifacts/garnet-std-net/`

```
garnet-std-net/
├── Cargo.toml
├── src/
│   ├── lib.rs                # public re-exports
│   ├── denylist.rs           # IP CIDR denylist + is_allowed()
│   ├── tcp.rs                # safe wrappers around std::net::TcpStream/Listener
│   ├── udp.rs                # safe wrappers around std::net::UdpSocket
│   ├── timeout.rs            # read/idle timeout enforcement
│   └── policy.rs             # cap-aware policy evaluation
└── tests/
    ├── denylist.rs           # 15 tests covering each CIDR class
    ├── rebinding.rs          # DNS rebinding scenario tests
    ├── amp_cap.rs            # UDP amplification tests
    └── timeout.rs            # slowloris simulation
```

Test plan (~30 tests):
- Each denylisted CIDR: connection refused
- Allowed public IP: connection succeeds (mocked endpoint)
- DNS rebinding: connect-time peer differs from resolve-time → refused
- UDP amp: response > 3× request → refused
- Slowloris: read times out at 30s default
- per-IP cap: 33rd connection from same peer refused
- Cap interaction: `@caps(net_internal)` allows loopback

### 2.8 Sequencing rule

**NetDefaults MUST land BEFORE Networking stdlib MUST land BEFORE any MVP that uses networking (MVPs 5/7/8).**

---

## §3. BoundedMail — Bounded actor mailboxes + backpressure (8h)

### 3.1 Threat closed

v0.3 actor runtime uses unbounded mpsc channels (per `runtime.rs:13`). One misbehaving sender can OOM the receiving actor. For networked actors (post-v3.4), this is a trivial DoS: the attacker just sends.

BoundedMail caps mailbox depth and exposes backpressure via `Result<(), SendError::Full>` on `send()`.

### 3.2 Surface syntax

```
mailbox-annotation := "@mailbox" "(" integer ")"
```

```garnet
@mailbox(1024)             # cap at 1024 in-flight messages
actor RequestHandler {
  protocol handle(req: Request) -> Response

  on handle(req) { ... }
}
```

Default capacity if unannotated: **1024 messages**. Programmers who need more declare it; programmers who need less (memory-constrained agents) can declare `@mailbox(64)`.

### 3.3 Send semantics

`tell(addr, msg)` (fire-and-forget) returns `Result<(), SendError>`:

- `Ok(())` — message accepted into mailbox
- `Err(SendError::Full)` — mailbox at capacity; sender's choice what to do (retry, drop, log)
- `Err(SendError::Closed)` — actor stopped; message dropped

`ask(addr, msg)` (synchronous request/response) returns the same error — if the mailbox is full, the ask fails fast rather than blocking.

`ask_timeout(addr, msg, ms)` adds a timeout on the response channel; combines with backpressure.

### 3.4 Backpressure pattern (recommended)

For senders that can tolerate slow receivers:

```garnet
def send_with_retry(addr, msg, max_retries) {
  let mut attempts = 0
  loop {
    match tell(addr, msg) {
      Ok(()) => return :ok,
      Err(SendError::Full) => {
        attempts += 1
        if attempts >= max_retries { return :dropped }
        sleep(10 * attempts)  # exponential backoff
      },
      Err(SendError::Closed) => return :closed,
    }
  }
}
```

For senders that must drop on overload (always):

```garnet
def send_or_drop(addr, msg) {
  match tell(addr, msg) {
    Ok(()) => :sent,
    Err(_) => :dropped,
  }
}
```

### 3.5 Implementation outline

Files (Rust):

- `garnet-actor-runtime/src/runtime.rs` — replace `mpsc::unbounded_channel` with `mpsc::channel(capacity)`; capacity from actor's `@mailbox(N)` annotation, default 1024
- `garnet-actor-runtime/src/address.rs` — `Addr::tell` and `Addr::ask` change signature: return `Result<_, SendError>` instead of `()`
- `garnet-actor-runtime/src/error.rs` — new `SendError::{Full, Closed}` variants
- Tests added to `garnet-actor-runtime/tests/` covering: bounded fill-up + backpressure, overflow rejection, ask under load, drained capacity recovery

Test plan (~12 tests):
- Bounded mailbox accepts up to N messages
- Send N+1 → SendError::Full
- Receive one → next send succeeds (capacity recovered)
- ask() under saturation returns Full immediately
- @mailbox(64) honors smaller cap
- Default 1024 applied when no annotation
- Closed actor returns SendError::Closed
- Concurrent sends from N senders compete fairly (no starvation)

### 3.6 Sequencing rule

**BoundedMail MUST land BEFORE any MVP that places an actor on a network socket (MVPs 5/7/8).** The sequencing is independent of CapCaps + NetDefaults — those gate the network primitives; BoundedMail gates the actor that consumes from them.

### 3.7 Migration path for v3.3 actor users

Existing v3.3 actor code calling `addr.tell(msg)` will break with the signature change. Migration: either (a) wrap with `let _ = addr.tell(msg)` to accept the result, or (b) handle the error. The v3.3 → v3.4 migration guide MUST list this as the only breaking change.

---

## §4. ManifestSig — Manifest signing + transparency log (25h)

### 4.1 Threat closed

v3.3 deterministic builds produce a BLAKE3-hashed manifest, but anyone with the same compiler version can produce a same-looking manifest. A malicious build of `garnet-cli` with hidden semantics produces a manifest indistinguishable from a legit build.

ManifestSig binds the manifest to a *signing identity* (Ed25519 keypair, OR Sigstore-style keyless via OIDC + transparency log) so a verifier can check "this manifest was produced by an authorized builder."

### 4.2 Two signing modes

**Mode A — Local Ed25519 (offline):** Developer generates a long-lived keypair. `garnet build --deterministic --sign --key=PATH` signs the manifest with that key. `garnet verify --pubkey=PATH manifest.json` verifies.

**Mode B — Sigstore keyless (online):** Developer authenticates via OIDC (GitHub, Google). `garnet build --deterministic --sign --keyless` mints an ephemeral keypair, signs, and publishes the cert + signature to the transparency log. `garnet verify --keyless manifest.json` checks against the log.

Mode A is the default for v3.4 (offline-friendly, no external deps). Mode B is opt-in (requires Sigstore client crate; targeted v3.4.1 or v3.5).

### 4.3 Signature format

Embedded in the manifest:

```json
{
  "version": "1",
  "source_hash": "9ffa...",
  "ast_hash": "b4b0...",
  "prelude_hash": "...",
  "dep_hashes": { ... },
  "manifest_hash": "...",
  "signature": {
    "alg": "ed25519",
    "pubkey": "0x...",
    "sig": "0x...",
    "signed_at": "2026-04-16T12:00:00Z"
  }
}
```

The signature covers a canonical serialization of all fields except `signature` itself.

### 4.4 Verification protocol

```
$ garnet verify --pubkey=trusted-builder.pub manifest.json
✓ Manifest signature matches trusted builder
✓ source_hash matches source tree
✓ ast_hash matches re-parsed AST
✓ prelude_hash matches built-in prelude
✓ dep_hashes match resolved dependencies
✓ Build is deterministically reproducible
```

A verification failure produces a structured error with the failing field named.

### 4.5 Transparency log integration (Mode B, v3.4.1+)

When `--keyless` is used, the signing flow:

1. Garnet client invokes `cosign` (or sigstore-rs library) to mint an ephemeral keypair via OIDC
2. Sign the manifest with the ephemeral key
3. Submit the signature + cert to the Sigstore transparency log (Rekor)
4. Embed the Rekor log entry index in the manifest

Verification:

1. Fetch the Rekor entry by index
2. Verify the signature matches the cert
3. Verify the cert's identity matches the expected signer (e.g., "github user X" or "google org Y")

### 4.6 Implementation outline

Files (Rust):

- `garnet-cli/src/sign.rs` — new module. Mode A Ed25519 signing using `ed25519-dalek` crate; Mode B Sigstore stub for v3.4.1
- `garnet-cli/src/manifest.rs` — extend manifest schema with `signature` field; canonical serialization sketches the signing surface
- `garnet-cli/src/verify.rs` — new module. Loads manifest, verifies signature, verifies all hash fields against current source tree
- `garnet-cli/src/main.rs` — add `garnet sign` and `garnet verify` subcommands

Test plan (~20 tests):
- Sign + verify roundtrip with Mode A
- Sign with one key, verify with different key → fails
- Tamper with source_hash post-sign → verify fails with specific error
- Tamper with sig bytes → verify fails
- Sign without --key flag → error E1001
- Verify without --pubkey or --keyless flag → error E1002
- Forged signature with wrong algorithm → fails
- Mode B stub (v3.4.1): not yet exercised

### 4.7 Sequencing rule

**ManifestSig SHOULD land before v3.4 ships, but is NOT a release blocker for v3.4.0** — it's a v3.4.1 deliverable if v3.4.0 ships on a tight schedule. Reasoning: the threat ManifestSig closes (compiler impersonation) is real but slow-moving; it doesn't block any v3.4 stdlib primitive from being usable.

The other three Layer-2 items (CapCaps, NetDefaults, BoundedMail) ARE release blockers.

---

## §5. Layer 2 Test Tally Forecast

| Item | New tests | Smoke binaries |
|------|-----------|----------------|
| CapCaps | ~25 | 0 |
| NetDefaults | ~30 | 1 (`net_smoke.rs`) |
| BoundedMail | ~12 | 0 |
| ManifestSig | ~20 | 1 (`sign_smoke.rs`) |
| **Total v3.4 Layer 2** | **~87** | **2** |

Combined with Stage 2 stdlib tests (~50) + MVP smoke tests (~40) + integration (~20), v3.4 ships **~200 new tests** on top of v3.3's combined 918 baseline.

---

## §6. Sequencing Rules Summary (CRITICAL)

1. **CapCaps lands FIRST.** All other Layer-2 items reference it; stdlib primitives depend on it.
2. **NetDefaults lands BEFORE Networking stdlib.**
3. **BoundedMail lands BEFORE any networked actor in any MVP.**
4. **Networking stdlib lands BEFORE MVPs 5 (web), 7 (game), 8 (KV).**
5. **File I/O stdlib lands BEFORE MVPs 1 (OS sim), 2 (DB), 3 (compiler bootstrap).**
6. **ManifestSig SHOULD land before v3.4.0; MAY slip to v3.4.1.**

---

## §7. Layer 2 Gate (Stage 2 entry condition)

Stage 2 (P0 stdlib + MVPs) MAY commence once:

- CapCaps tests pass + parser accepts `@caps(...)` annotation
- NetDefaults `denylist.rs` tests pass + DNS-rebinding test green
- BoundedMail `runtime.rs` test for default 1024 cap passes
- ManifestSig tracked in v3.4.1 milestone (acceptable to defer past v3.4.0)

---

## §8. Cross-references

- Mini-Spec v1.0: §16 (single-CLI principle), §10 (recursion guardrails — companion to caps)
- v3.3 Threat Model: 15 patterns, of which 4 are this spec's domain
- v3.3 Security V1: Layer 1 baseline (ParseBudget + KindGuard + StateCert + CacheHMAC + ProvenanceStrategy)
- Master plan: Phase 2A-SEC

---

*Layer 2 specification prepared 2026-04-16 by Claude Code (Opus 4.7) — Stage 2 Phase 2A-SEC opening. Implementation work begins immediately; first land is BoundedMail (smallest), then CapCaps (largest dependency), then NetDefaults, then ManifestSig.*

*"Prepare your minds for action; be sober-minded." — 1 Peter 1:13*
