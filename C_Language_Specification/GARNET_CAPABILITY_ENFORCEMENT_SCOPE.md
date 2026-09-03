# Garnet capability enforcement scope

**Status: normative honesty fence.** This document draws the exact line between
what Garnet's capability system enforces, where, and how. It exists so that no
public sentence claims "universal `@caps` runtime enforcement" — because the
truth is more precise and more defensible than that.

Garnet has **one** capability source of truth: `garnet-stdlib/src/registry.rs`.
Every primitive row carries `RequiredCaps` plus a `Guard` column that names the
*runtime* backstop. The classes below are that column, made public.

## The one universal guarantee (check time)

The **CapCaps propagator** (`garnet-check-v0.3/src/caps_graph.rs`,
`check_caps_coverage`) reads primitive capabilities from the same registry and
propagates them transitively across the call graph. If any function reaches a
capability-bearing primitive without the calling chain declaring that
capability, **`garnet check` rejects the program**, and the program entry point
must declare its own budget. This check-time property is the universal one: it
holds for the whole registered capability surface, on every backend, before any
code runs.

Everything below is about what happens **after** check time — at run time and at
the OS boundary — where the guarantees are real but **bounded**, not universal.

## Capability kinds

The checker vocabulary is a closed set of **8** kinds
(`garnet-check-v0.3/src/capset.rs`): `*` (wildcard), `env`, `ffi`, `fs`, `net`,
`net_internal`, `proc`, `time`. The stdlib registry constructs rows requiring
`fs`, `net`, `time`, `proc`, `env`.

## Enforcement classes

| Class | What it means | Where | Members |
|-------|---------------|-------|---------|
| **Declared (checker-only)** | Capability required by the checker; **no runtime gate**. Reachable at run time without a trap once the program type-checks. | `Guard::Declared` (`registry.rs`) | `time::now_ms`, `time::wall_clock_ms`, `time::sleep`; `uuid::new_v4`, `uuid::new_v7` (all require `@caps(time)` at check time only) |
| **Runtime-gated (call chain only)** | `require_capability` alone: traps when no *active* frame declares the capability. **0 primitives — this class is empty (U-91).** The union is satisfied by ANY active frame, so a helper that declares the capability satisfied it for an entry point that did not. | `Guard::Gate` | *(none)* |
| **Entry-gated** | `require_capability` **plus** the program-entry-frame check, so the PROGRAM ENTRY's declared budget must cover the capability regardless of which call edge reached the primitive. **15 primitives — the whole gated surface.** S92 introduced this for the three subprocess surfaces; U-91 extended it to the rest. | `Guard::GateEntry`; `eval.rs` `require_entry_capability` | `fs::read_file` / `write_file` / `read_bytes` / `write_bytes` / `list_dir`; `net::tcp_connect`; `std::env::get` / `set` / `vars`; `std::process::wait` / `exit_code` / `spawn` / `spawn_args` / `output`; `std::log::to_file` |
| **Declared-only, no bridge** | In the checker vocabulary and/or sandbox-policy mapping, but **no runtime enforcement path exists**. | — | `ffi` (checker + manifest + sandbox-policy warning only); `net_internal` (checker vocab + loopback-only in generated sandbox policy; `tcp_connect` always uses strict `NetPolicy::default()`) |
| **Unbridged** | Registry row exists for the CapCaps propagator only; **no interpreter binding at all**. | `Binding::Unbridged` (`registry.rs`) | `net::tcp_listen`, `net::udp_bind` |
| **OS-sandboxed (generated, not self-enforced)** | `garnet sandbox` generates seccomp / WASI / egress policy from aggregate `@caps`. The generator emits `enforced: false`; the policy was applied and trapped on a real **Linux** kernel via an external C reference harness (`tools/seccomp-apply`). macOS / Windows OS-sandbox application is **named-deferred**. | `GARNET_SANDBOX_POLICY.md`, `GARNET_SECCOMP_APPLY.md` | all `@caps` → policy |
| **Caps-invisible** | Host-visible natives with **no capability row at all**. Any "all authority is capability-tagged" claim is false until these earn rows. | `BRIDGE_ONLY` const (`stdlib_bridge.rs`) | `memory::working` / `episodic` / `semantic` / `procedural` |

The `(0 Gate, 15 GateEntry)` split is pinned by
`gate_count_matches_the_audited_runtime_backstop` and the exact member list by
`entry_gates_are_the_whole_gated_surface` in `registry.rs`, and the
checker-only-vs-gated behavior is pinned by
`guard_column_matches_runtime_backstop_behavior` in `stdlib_bridge.rs`
(Declared-with-caps prims such as `time::*` and `uuid` v4/v7 must **not**
caps-trap — checker-only by design, S90 scope).

## Runtime-trap scope (the fence that matters most)

Runtime capability trapping applies to the **15 entry-gated** host-authority
primitives. Each one requires both a live call-chain frame declaring the
capability and a program-entry frame whose declared budget covers it. Garnet
manages frames as follows:

- **Managed (`def`) functions** push a caps frame per call; **program entry**
  additionally installs an entry frame.
- With **no active frame**, a gated host primitive is refused whenever either
  the interpreter instance is strict or the process-global `STRICT_NO_FRAME`
  latch is set (`eval.rs`). `Interpreter::new()` is **strict by default** and
  wraps its load/eval/call operations in that per-instance strict scope.
- The `garnet` binary also sets the one-way process-global latch at startup, so
  **the CLI is deny-by-default on every lane** even if internal code creates an
  explicitly permissive interpreter.
- **Library / embedder callers** using `Interpreter::new()` through its
  high-level load/eval/call methods receive the same strict default.
  `Interpreter::new_permissive()` is the explicit legacy opt-out for trusted
  harnesses and deliberate integrations; it cannot override a process whose
  global latch has already been set.
- **Low-level Rust host APIs are not an embedder sandbox.** `Interpreter::global`,
  `Value::NativeFn`, and `eval::call_value` are public. A host that extracts and
  invokes a native outside an `Interpreter` method also executes outside that
  instance's strict scope. The 2026-07-14 post-acceptance delta review reopened
  condition #5 until no-frame denial covers this raw path by default. Therefore
  the current claim is *strict-by-default on the high-level Interpreter API*,
  not "every public Rust call path is deny-by-default."

Pure computation and the checker-only class (`time::*`, `uuid` v4/v7) are never
runtime-trapped. VM/interpreter parity for the gated surface (including the S92
entry gate through `--vm`) is asserted by the enforcement-status gates and the
scope-parity tests.

## What the public copy may and may not say

- **May say (true):** undeclared OS authority fails `garnet check`; `@caps` and
  `@max_depth` trap identically on both backends for the gated surface, with
  cross-OS trap parity recorded as evidence; the `garnet` CLI and the default
  high-level `Interpreter::new()` load/eval/call path are deny-by-default.
- **May not say (overclaim):** "universal `@caps` runtime enforcement"; "no
  ambient authority, ever" as a runtime-universal claim; that every third-party
  embedder is forced to use the strict constructor, that the explicit
  `new_permissive()` opt-out does not exist, or that raw public Env/Value/eval
  calls inherit an instance scope they do not enter; that
  `time`/`uuid`/`ffi`/`net_internal`/`memory::*` are runtime-gated; that
  OS-sandbox enforcement holds beyond Linux-seccomp via the reference harness.

The two currently-published bounded enforcement claims (test-runner entry
authority; VM/interpreter scope parity) live in `docs/why.html` and are the only
`enforced:` claims on the public site. This scope table is what keeps that set
from growing by accident.
