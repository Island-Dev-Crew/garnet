# Garnet capability enforcement scope

**Status: normative scope fence.** This document draws the exact line between
what Garnet's capability system enforces, where, and how. It exists so that no
public sentence claims "universal `@caps` runtime enforcement" — because the
truth is more precise and more defensible than that.

Garnet has **one** capability source of truth: `garnet-stdlib/src/registry.rs`.
Every primitive row carries `RequiredCaps` plus a `Guard` column that names the
*runtime* backstop. The classes below are that column, made public.

## The check-time guarantee, and the edges it is built from

The **CapCaps propagator** (`garnet-check-v0.3/src/caps_graph.rs`,
`check_caps_coverage`) reads primitive capabilities from the same registry and
propagates them transitively **across the call edges it can build**. If a
function reaches a capability-bearing primitive along such a chain without the
chain declaring that capability, **`garnet check` rejects the program**, and the
program entry point must declare its own budget.

The guarantee is real but it is **not universal**, and the bound is the call
graph, not the capability surface (U-91). An edge is built for a call written by
name to a declared `def`, to a method (`MethodByName` unions every
implementation method of that name), or to a registry primitive, inside a
function body the checker walks. **No edge is built** for a callee reached
through a function value — an alias binding, a higher-order parameter, an actor
handler, a map of functions — nor for a call inside a closure body or a string
interpolation, a top-level `let`/`const` initializer, or `method_missing`
dispatch. Where no edge is built the checker is silent. Two further boundaries sit inside the named-chain case. **The body of an
unannotated function is not checked at all**: a lone `def` with no `@caps`
annotation that calls `write_file` reports `0 diagnostics`. And **a wholly
unannotated recursive cycle passes even along a named chain.**

**`garnet run` does not invoke the checker at all.** Checking is a step you run,
not a precondition of running. A program `garnet check` rejects will execute,
and what stops it then is the runtime fence below, which covers only part of the
surface.

Everything below is about what happens **after** check time — at run time and at
the OS boundary — where the guarantees are real but **bounded**, not universal.

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
capability and a program-entry frame whose declared budget covers it. Since the
U-91 cure that is the whole gated surface: `Guard::Gate`, the class with only
the call-chain check, is empty.

**The other 65 registry rows carry no runtime gate at all.** For 63 of them an
undeclared call simply runs: `garnet check` rejects the program, `garnet run`
does not check, and the primitive executes and returns a real value. The
remaining two, `net::tcp_listen` and `net::udp_bind`, are `Binding::Unbridged`
— they have no interpreter binding, so they do not execute either; see the
Unbridged row above. Neither group traps on capability grounds.

Garnet manages frames as follows:

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

- **May say (true):** undeclared OS authority fails `garnet check` **when the
  primitive is reached through a named call chain the propagator can build, from
  a function that carries an annotation** (U-91); all 15 gated primitives additionally require the program entry's own
  declared budget, whichever call edge reached them; `@caps` and `@max_depth`
  trap identically on both backends for the gated surface, with cross-OS trap
  parity recorded as evidence; the `garnet` CLI and the default high-level
  `Interpreter::new()` load/eval/call path are deny-by-default.
- **May not say (overclaim):** that `garnet check` rejects an undeclared use of a
  capability-bearing primitive *however it is reached* — the propagator builds
  named call edges only (U-91); that it rejects every undeclared use *along* a
  named chain — a wholly unannotated recursive cycle passes, and the body of an
  unannotated function is not checked at all; that `garnet test` rejects a
  `@caps()` test that invokes *any* undeclared authority — it rejects one that
  reaches a gated primitive, and passes one that calls a checker-only row; that
  running a
  program is protected by the checker — `garnet run` does not invoke it; that
  the runtime refuses any capability-bearing primitive nothing declares — that
  is true of the 15 gated rows and false of the 65 `Declared` rows, which have
  no runtime gate; "universal `@caps` runtime enforcement"; "no
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
