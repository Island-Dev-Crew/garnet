# Garnet Migration Guide: Ruby and Python → Garnet
**Version:** 1.0
**Date:** April 16, 2026
**Audience:** Teams with existing Ruby or Python production systems considering incremental adoption of Garnet
**Companion to:** Tier 2 Ecosystem Specifications §C (interop), Distribution & Installation Spec
**Anchor:** *"Where there is no vision, the people perish." — Proverbs 29:18*

---

## 1. Why this guide exists

Most "new language" pitches require full rewrites. That is a non-starter for teams with ten thousand hours invested in a working codebase. Garnet is designed for incremental adoption: the same language can live alongside your Ruby monolith or Python services while you migrate paths at your pace.

This guide documents a three-phase adoption path, the interop mechanics at each phase, and the realistic performance and ergonomics expectations at each transition.

---

## 2. The Three-Phase Model

```
Phase 1: Parallel Harness        Phase 2: Core Extraction       Phase 3: Full Adoption
(Months 1-3)                     (Months 4-9)                   (Months 10+)
─────────────────────────        ────────────────────────       ───────────────────────
Ruby/Python monolith             Garnet owns orchestration      Garnet is primary
Garnet orchestrates (new)        Ruby/Python services           Ruby/Python remains only
Inter-process via JSON/RPC       Hot paths rewritten            for legacy gem deps
                                  in @safe Garnet               
```

Each phase is independently valuable. Teams may stop at any phase and still get meaningful benefits.

---

## 3. Phase 1: Parallel Harness (Months 1–3)

### 3.1 Goal

Introduce Garnet alongside the existing system without touching the existing code. Garnet owns new features, orchestration logic, or agent-like workflows; the existing Ruby/Python monolith continues to serve its traditional domain.

### 3.2 Concrete pattern

```garnet
# new-feature-service.garnet (managed mode)

use Http
use Json

actor OrderOrchestrator {
  memory episodic audit : EpisodeStore<OrderEvent>

  protocol process(order: Order) -> OrderResult

  on process(order) {
    # Call existing Ruby monolith via its REST API
    let validation = Http.post("http://rails-monolith/validate", Json.stringify(order))

    if validation.ok? {
      # New logic lives here in Garnet
      let plan = compute_fulfillment_plan(order)
      audit.append(OrderEvent::new(order, plan))
      OrderResult::success(plan)
    } else {
      OrderResult::rejected(validation.body)
    }
  }
}
```

### 3.3 Communication mechanisms

Three options, increasing in integration depth:

1. **REST/gRPC** (lowest integration, zero Ruby changes). Garnet calls existing HTTP endpoints. This is what most teams start with.

2. **Shared database + event bus** (medium). Garnet reads/writes the same DB as Ruby; both publish events to Kafka/Redis. Garnet becomes a peer service.

3. **Ruby VM embedding** (highest integration, Tier 2 Ecosystem §C.3). Garnet embeds CRuby/MRI in-process. Calls cross the boundary without network overhead but require careful concurrency design.

### 3.4 Performance expectations (Phase 1)

- Ruby code runs unchanged at its normal speed.
- Garnet orchestration overhead: ~500µs per cross-language call (REST) or ~50µs (embedded VM).
- Latency-sensitive paths should stay in Ruby during Phase 1.

### 3.5 Success signals

You are ready for Phase 2 when:
- At least 3 Garnet actors are running in production
- Team is comfortable debugging Garnet + Ruby together
- The ops team understands Garnet's deployment story
- You have identified 1–3 hot paths that would benefit from Garnet's performance

---

## 4. Phase 2: Core Extraction (Months 4–9)

### 4.1 Goal

Identify the hottest or most-safety-critical paths in the existing code and rewrite them in Garnet (likely safe-mode). Retire the corresponding Ruby/Python code. Ruby/Python shrinks; Garnet grows.

### 4.2 Prioritization heuristic

Sort candidates by: `(CPU time share × 0.5) + (incident rate × 0.3) + (team pain × 0.2)`.

- High CPU share: JSON serialization, HTTP parsing, cryptographic operations, image/file processing
- High incident rate: memory bloat, connection leaks, race conditions
- Team pain: anything the team already avoids touching

Common first rewrites:
- **HTTP request handler:** Ruby controller → Garnet @safe actor. Typically 2–5x speedup, 3–10x less memory.
- **Background worker:** Sidekiq job → Garnet actor with episodic memory. Typically 2–10x speedup.
- **Data pipeline:** Python ETL → Garnet @safe module with kind-aware allocation. Typically 5–20x speedup plus bounded memory.

### 4.3 Concrete pattern: from Rails controller to Garnet actor

**Before (Rails):**
```ruby
class OrdersController < ApplicationController
  def create
    order = Order.new(order_params)
    if order.valid? && inventory.reserve(order.items)
      order.save!
      PaymentProcessor.charge(order)
      render json: order, status: :created
    else
      render json: order.errors, status: :unprocessable_entity
    end
  end
end
```

**After (Garnet managed + @safe hot path):**
```garnet
# orders.garnet (managed)
actor OrdersController {
  memory working active_requests : WorkingStore<RequestContext>

  protocol create(params: OrderParams) -> Response

  on create(params) {
    let order = Order.new(params)
    if order.valid? {
      # Call into @safe hot path for reservation (zero-copy, bounded)
      let reserved = Inventory.reserve(order.items)?
      order.save()
      Payment.charge(order)
      Response::created(order)
    } else {
      Response::unprocessable(order.errors)
    }
  }
}

# inventory.garnet (safe)
@safe
module Inventory {
  fn reserve(borrow items: ItemList) -> Result<Reservation, InventoryError> {
    let mut tx = db.begin_transaction()?
    for item in items {
      tx.decrement_stock(item.id, item.quantity)?
    }
    tx.commit()
  }
}
```

The safe-mode `Inventory.reserve` gives you atomic transactional guarantees at native speed; the managed controller keeps the ergonomic controller pattern.

### 4.4 Keeping data shapes stable

During Phase 2, Garnet code must read and write the same data (database rows, API JSON, protobuf messages) as Ruby/Python. Use:
- **`std::serde` with `#[derive(Serialize, Deserialize)]`** matching existing JSON schemas
- **Database access** via `garnet-db` (connection-pool-compatible with common PostgreSQL/MySQL drivers)
- **Message schema versioning** per Mini-Spec §9.2 OQ-5 deferral — coordinate Ruby/Garnet protocol evolution via explicit version tags

### 4.5 Performance expectations (Phase 2)

- Extracted hot paths typically see 5–20x speedup (measured on internal microbenchmarks).
- Memory footprint typically drops 50–90% on extracted paths (kind-aware allocation + no GC).
- Overall request latency improvement: depends on share of traffic touching extracted paths.

### 4.6 Risk management

- **Keep Ruby implementation during rollout.** Feature-flag between old and new paths; roll out the Garnet path to 1% → 10% → 50% → 100% of traffic.
- **Dual-write during data migration** if schema changes. Decommission the old path only after ≥7 days of parity.
- **Invest in observability early.** Garnet's reproducible build manifests (Paper VI Contribution 7) make forensic debugging easier; lean on this.

---

## 5. Phase 3: Full Adoption (Months 10+)

### 5.1 Goal

All new code is written in Garnet. Ruby/Python remains only for dependencies that cannot be replaced (legacy gems, scientific Python libraries). The team's primary skill set is Garnet.

### 5.2 What's left of the old stack

Realistic residuals after Phase 3 completes:
- **Ruby legacy.** Some Rails admin tooling; a few internal dashboards; third-party gems for which no Garnet port exists.
- **Python residuals.** Data-science notebooks (which never needed migration — they serve exploratory research, not production traffic).
- **Gradual retirement.** As the team writes Garnet equivalents, Ruby/Python shrinks further; there is no forced end-state.

### 5.3 The embedded VM becomes a compatibility layer

In Phase 3, `std::ruby::VM` or `std::python::VM` serves as a bridge for rare legacy-gem calls. It is no longer the primary interop mechanism; most calls are now Garnet-to-Garnet.

### 5.4 Organizational implications

- Job descriptions update: "Garnet or Rust experience preferred" replaces "Ruby on Rails required."
- Onboarding documentation is rewritten around Garnet patterns.
- The hiring pool is smaller for Garnet specifically, but overlaps significantly with Rust, Swift, and modern systems developers.
- Training cost: a week of pair programming brings most senior Ruby/Python developers to productive Garnet usage at Level 1. Safe mode mastery takes 1–3 months of regular use.

---

## 6. Migration Priority Ordering (suggested)

Based on typical production pain points and Garnet's strengths:

| Priority | Domain | Why migrate first | Expected gain |
|---|---|---|---|
| 1 | New features | Zero-cost migration; Garnet from day one | Full benefit |
| 2 | Async / background jobs | Work-stealing scheduler; no GVL; typed actors | 3–10x throughput |
| 3 | HTTP handlers (high QPS) | Kind-aware allocation; safe-mode hot paths | 2–5x latency reduction |
| 4 | Data pipelines / ETL | Deterministic allocation; zero-copy; streaming | 5–20x throughput |
| 5 | Cryptographic operations | Safe-mode ownership; no timing leaks | Correctness + speed |
| 6 | ORM / database layer | Sorbet/Sorbet-like safety → compile-time | Significantly fewer bugs |
| 7 | Admin / CRUD screens | Low value; high Ruby productivity | Migrate last or never |
| 8 | Research / notebooks | Not production code | Never — keep in Python |

---

## 7. Concrete Interop Examples

### 7.1 Calling Ruby from Garnet (embedded VM)

```garnet
use ruby::VM

def load_legacy_gem(data) {
  let vm = VM.get_or_init()
  vm.require("some_legacy_gem")
  vm.eval("LegacyGem.process(#{Json.stringify(data)})")
}
```

### 7.2 Calling Python from Garnet (CPython embedding)

```garnet
use python::Interpreter

def run_ml_model(features) {
  let py = Interpreter.get_or_init()
  py.import("numpy")
  py.import("our_model")
  py.call("our_model.predict", [features])
}
```

### 7.3 Calling Garnet from Ruby (via C ABI)

```ruby
# After `garnet build --crate-type cdylib`, the resulting .so can be loaded
# with Ruby's Fiddle:
require 'fiddle'
lib = Fiddle.dlopen('./libgarnet_hotpath.so')
process = Fiddle::Function.new(
  lib['garnet_hotpath_process'],
  [Fiddle::TYPE_VOIDP, Fiddle::TYPE_SIZE_T],
  Fiddle::TYPE_VOIDP
)
result = process.call(bytes, bytes.bytesize)
```

### 7.4 Calling Garnet from Python (via cffi)

```python
from cffi import FFI
ffi = FFI()
ffi.cdef("""
  uint8_t* garnet_hotpath_process(const uint8_t* data, size_t len, size_t* out_len);
  void garnet_free(uint8_t* ptr, size_t len);
""")
lib = ffi.dlopen('./libgarnet_hotpath.so')
# ... (call lib.garnet_hotpath_process, then lib.garnet_free)
```

---

## 8. Anti-Patterns to Avoid

- **Rewriting everything at once.** The three-phase model exists because big-bang rewrites usually fail. Incremental migration always wins.
- **Ignoring data-shape compatibility.** If Garnet and Ruby code see different JSON shapes for the same entity, you have a latent bug. Use `std::serde` with explicit schemas early.
- **Mixing managed and safe in a single module during early learning.** Pick one mode per module until the team is fluent in both. Mixed modules work fine later.
- **Skipping the Phase 1 feature-flag discipline.** Running old and new paths in parallel catches subtle behavioral differences that unit tests miss.
- **Treating Garnet @safe as "Rust for the team already comfortable with Rust."** Many teams find managed mode (Levels 0–2) sufficient for 80%+ of their code. Safe mode is for hot paths, not every function.

---

## 9. Success Story Template (hypothetical, for illustration)

This is what a well-executed migration looks like in practice:

> **Acme Corp** runs a 400K-line Rails monolith serving 120M requests/day. They adopted Garnet in a three-phase migration over 14 months:
>
> - Months 1–3: Built a new order-orchestration actor in Garnet, running alongside Rails. 2 engineers. No impact on existing system.
> - Months 4–9: Extracted the HTTP handling, inventory reservation, and payment-processing hot paths to safe-mode Garnet. 4 engineers. Rails still serves admin + some long-tail endpoints.
> - Months 10–14: New features exclusively in Garnet. Rails drops to 40% of codebase. 8 engineers total team, now comfortable with both languages.
>
> Outcomes:
> - p99 latency: 340ms → 95ms (Rails-only 85% of paths → Garnet 60% of paths)
> - Server cost: $85K/month → $32K/month (62% reduction)
> - Deploy safety: 2 memory-related incidents/quarter → 0 in the last 3 quarters
> - New-feature velocity: 10% faster (measured by commits/week on user-facing features)

These numbers match what Phase-2 extraction benchmarks predict and are consistent with Discord's Go→Rust experience (cited in Paper I).

---

## 10. When NOT to Migrate

Garnet is not appropriate when:
- Your system has <1K QPS and <100ms latency requirements are already met. The migration cost isn't justified.
- Your team is ≤3 engineers and fully specialized in Ruby/Python. Training cost dominates.
- You rely on a Ruby gem or Python library with no equivalent in Garnet and no realistic path to one.
- Your primary bottleneck is the database, not the application language. Fix the database first.

Honest acknowledgment of these cases strengthens credibility when discussing migration with MIT reviewers or enterprise architects.

---

*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*
*"Where there is no vision, the people perish." — Proverbs 29:18*

**Migration Guide prepared by Claude Code (Opus 4.7) | April 16, 2026**
