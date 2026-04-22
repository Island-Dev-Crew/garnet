# Rust and Ruby: a doctoral comparative study and synthesis proposal for Garnet

**Rust and Ruby occupy opposite poles of programming language design — one prioritizing machine-level safety and performance, the other championing programmer happiness and expressive power.** This study finds that their respective weaknesses are precisely mirrored by each other's strengths, creating a rare opportunity for genuine language synthesis. Rust delivers memory safety without garbage collection and performance within **5–10% of C**, while Ruby enables developer velocity unmatched by any compiled language, with idiomatic programs reading nearly as English prose. A reconciliation language — proposed here as **Garnet** — could merge Rust's compile-time ownership with Ruby's expressive syntax through gradual typing, optional ownership annotations, and a dual-mode memory model that defaults to automatic reference counting but allows opt-in zero-cost ownership semantics for performance-critical paths. The total addressable market for developer tooling reached **$6.4–7.6 billion in 2025** and is projected to hit **$15.7 billion by 2031**, suggesting meaningful commercial opportunity for a language that credibly spans systems, web, and scripting domains. What follows is an exhaustive analysis across history, design philosophy, performance characteristics, ecosystem maturity, industry adoption, and a formal synthesis proposal with market viability assessment.

---

## Part I — Rust: safety forged in fire

### A broken elevator and a language "over-engineered for survival"

Rust began in **2006** as the personal side project of **Graydon Hoare**, then a 29-year-old Mozilla employee. The catalyst, as Hoare later told MIT Technology Review, was a malfunctioning elevator in his Vancouver apartment building — software had crashed it, and as he climbed 21 flights of stairs, he resolved to create a language that could produce fast, compact code without memory bugs. He named it after a group of remarkably hardy fungi that are, in his words, **"over-engineered for survival"** (MIT Technology Review, February 2023). Hoare described his creation as **"technology from the past come to save the future from itself,"** drawing inspiration from decades-old languages including CLU, Erlang, Limbo, OCaml, and Haskell.

Mozilla began sponsoring Rust around **2009**, and the project was officially announced at the Mozilla Annual Summit in **July 2010**. The initial git commit, dated June 16, 2010, was authored by Hoare alongside Andreas Gal, Dave Herman, Patrick Walton, and Brendan Eich. Notably, the first Rust compiler was written in OCaml, and the original language actually included a garbage collector — the ownership and borrowing system that defines modern Rust came later.

Hoare stepped down from his "Benevolent Dictator For Life" role in **2013**, and the language evolved under a federated governance structure with an RFC process added in March 2014. By the time **Rust 1.0 stabilized on May 15, 2015**, the language had undergone radical transformation from Hoare's original sketch. Key contributors beyond Hoare included **Niko Matsakis** (type system and borrow checker design), **Patrick Walton** (early core development), **Aaron Turon** (language design rationale), and **Alex Crichton** (Cargo and ecosystem infrastructure).

### The editions: a language that evolves without breaking

Rust's edition system provides opt-in breaking changes on a roughly three-year cadence while maintaining backward compatibility — crates from different editions interoperate seamlessly, and migration is assisted by `cargo fix --edition`.

| Edition | Release | Key Changes |
|---------|---------|-------------|
| **2015** | Rust 1.0 (May 2015) | Initial stable release; backward compatibility guarantee established |
| **2018** | Rust 1.31 (Dec 2018) | `async`/`await` keyword reservation, `impl Trait` syntax, module system overhaul, non-lexical lifetimes |
| **2021** | Rust 1.56 (Oct 2021) | Disjoint capture in closures, `IntoIterator` for arrays, const generics stabilization, new prelude additions |
| **2024** | Rust 1.85 (Feb 2025) | Most comprehensive edition: ownership/lifetime refinements, unsafe Rust guardrails, temporary scope changes, style edition system |

Following Mozilla's COVID-19 pandemic restructuring in August 2020 — which laid off approximately 250 employees, many on the Rust team — the **Rust Foundation** was established on **February 8, 2021** as an independent 501(c)(6) nonprofit. Five founding Platinum Members provided initial funding: **Amazon Web Services, Google, Huawei, Microsoft, and Mozilla**. Meta joined in April 2021. The Foundation committed to a multi-million-dollar annual budget, with **Rebecca Rumbul** serving as executive director and CEO.

### Ownership, borrowing, and the borrow checker — Rust's central innovation

Rust's defining contribution to programming language theory is achieving **memory safety without a garbage collector** through a compile-time ownership system. Three rules govern this system: each value has exactly one owner (a variable binding), ownership can be transferred (moved) which invalidates the original binding, and when the owner goes out of scope the value is deterministically dropped (deallocated). This model is rooted in **affine type theory**, where values can be used at most once.

**Borrowing** extends ownership with reference semantics. Shared references (`&T`) allow multiple simultaneous read-only views, while mutable references (`&mut T`) grant exclusive write access — the fundamental invariant being **"aliasing XOR mutation."** You can have either multiple readers or one writer, but never both simultaneously. **Lifetime annotations** (`'a`) specify how long references remain valid, and while the compiler infers most lifetimes through elision rules, explicit annotations are required in ambiguous cases.

The **borrow checker** — Rust's compile-time analysis pass — enforces these invariants, statically preventing use-after-free, dangling pointers, double frees, and data races. The formal soundness of this system was proven in the landmark **RustBelt** paper (Ralf Jung, Jacques-Henri Jourdan, Robbert Krebbers, Derek Dreyer; POPL 2018, Max Planck Institute for Software Systems), which provided the first machine-checked safety proof for a realistic subset of Rust using the Iris framework for higher-order concurrent separation logic in Coq. The verification actually uncovered and fixed a bug in Rust's standard library.

**Fearless concurrency** emerges directly from the type system. The `Send` trait marks types safe to transfer between threads; `Sync` marks types safe to share references across threads. The compiler automatically derives these bounds, making data races a compile-time error rather than a runtime catastrophe.

### The type system, async complexity, and the "colored functions" problem

Rust's type system combines algebraic data types (enums as sum types), exhaustive pattern matching, trait-based polymorphism with default implementations and coherence rules, and monomorphized generics that achieve zero-cost abstraction through compile-time specialization. The macro system operates at two levels: declarative macros (`macro_rules!`) for pattern-matching code generation, and procedural macros that operate on the abstract syntax tree for derive macros, attribute macros, and function-like macros.

Error handling eschews exceptions entirely. Rust has **no null and no nil** — instead, `Option<T>` (either `Some(T)` or `None`) models the absence of a value, and `Result<T, E>` (either `Ok(T)` or `Err(E)`) models fallible operations. The `?` operator enables ergonomic error propagation, and exhaustive `match` expressions ensure every variant is handled. This design makes it impossible to forget to handle an error.

The async/await model, while powerful, represents Rust's most significant ergonomic challenge. Rust's futures are **zero-cost** — an `async fn` returns a state machine implementing the `Future` trait with no heap allocation — but this creates complexity. The **"colored functions" problem** means async functions can only be called from async contexts. `Pin<T>` ensures self-referential futures don't move in memory, adding conceptual overhead. No async runtime is built in; users must choose between **Tokio** (dominant), async-std, or smol. The distinction between `Send` and `!Send` futures further complicates library design.

### Cargo, crates.io, and a tooling ecosystem that sets the standard

Cargo, Rust's unified build system and package manager, is consistently ranked as one of the most admired developer tools — **71% admiration** in the Stack Overflow 2025 survey, the highest for any cloud development/infrastructure tool. It handles dependency resolution, compilation, testing, documentation generation, benchmarking, and publishing in a single coherent tool.

The **crates.io** registry hosts approximately **250,500+ crates** as of early 2026, published by over **61,730 users or teams**, with peak daily downloads reaching **944.9 million** and monthly traffic of **1.6 petabytes across 11 billion requests**. The broader tooling ecosystem includes **rustup** (toolchain management), **clippy** (700+ lints), **rustfmt** (canonical formatting), **rust-analyzer** (LSP-based IDE support), and **Miri** (an interpreter for detecting undefined behavior in unsafe code).

### Industry adoption: from elevator crashes to trillion-request infrastructure

Rust's industry adoption has accelerated dramatically, driven by the growing recognition that **~70% of security vulnerabilities** in major codebases stem from memory safety issues (per Microsoft and Google data).

**Linux Kernel.** Rust support was merged into Linux **6.1** on October 3, 2022, with approximately 12,500 lines of initial infrastructure. The first Rust-written drivers were accepted in kernel **6.8** (December 2023). By December 2025, Rust in the Linux kernel was declared **no longer experimental**, with over **600,000 lines** of production Rust code across drivers, filesystem abstractions, and subsystem bindings. The **Nova GPU driver** for NVIDIA open-source firmware represents the highest-profile all-Rust driver effort.

**Cloudflare — Pingora replacing NGINX.** Cloudflare built Pingora, a Rust-based HTTP proxy serving over **1 trillion requests per day**, to replace their NGINX infrastructure. Production results: **70% less CPU usage**, **67% less memory**, a **5ms reduction in median TTFB**, and connection reuse improving from 87.1% to **99.92%** — a **160× reduction** in new connections for their largest customer. Open-sourced under Apache 2.0 in February 2024.

**Discord — the famous Go-to-Rust migration.** Discord's February 2020 blog post documented their rewrite of the Read States service. The Go implementation suffered **latency spikes every 2 minutes** (10–40ms) due to forced garbage collection on a large LRU cache. The Rust version eliminated these spikes entirely, with the team reporting: *"Even with just basic optimization, Rust was able to outperform the hyper hand-tuned Go version. This is a huge testament to how easy it is to write efficient programs with Rust."* In 2023, Discord's migration from Cassandra to ScyllaDB + Rust data services yielded p99 read latency improvements from **40–125ms to 15ms** while reducing cluster nodes from **177 to 72**.

**Google Android.** Rust was introduced to Android in 2021, and by 2025, memory safety vulnerabilities had dropped from **76% of total CVEs (2019) to below 20%** — a **1,000× reduction** in memory-safety vulnerability density comparing Rust code to C/C++ code across approximately **5 million lines** of Rust in the Android platform. Google engineer Jeff Vander Stoep noted: *"We adopted Rust for its security and are seeing a 1000x reduction in memory safety vulnerability density compared to Android's C and C++ code."*

**Microsoft.** Azure CTO Mark Russinovich declared at RustConf 2025: *"Every place that we've got untrusted input handling, the mandate is rewrite it in Rust, and any new agents are written in Rust."* Microsoft now has Rust running in the Windows kernel (`win32kbase_rs.sys`), in DWriteCore (~152,000 lines), Win32 GDI (~36,000 lines), and Hyper-V ARM64 emulation.

**Amazon/AWS** built Firecracker (the microVM hypervisor powering Lambda and Fargate) and Bottlerocket entirely in Rust. **Figma** rewrote their multiplayer engine in Rust. **Dropbox** rewrote significant portions of their sync engine, reportedly reducing memory usage by **75%**.

### Where Rust falls short — and what the gaps reveal

Rust's most frequently cited weaknesses are its **steep learning curve** (development velocity typically drops **30–50% during the first 3–6 months** of adoption), **long compile times** (the 2025 Compiler Performance Survey found ~45% of developers who stopped using Rust cited build performance), **async complexity**, and a **smaller talent pool** (approximately 2.3 million developers versus tens of millions for JavaScript or Python).

These gaps explain the appeal of alternatives: **Zig** offers C-like simplicity with comptime metaprogramming and much faster compilation (used by Bun.js). **Go** provides sub-second builds and trivially simple goroutine-based concurrency — though its garbage collector creates the exact latency spikes Discord fled. **Mojo** promises Python compatibility with systems-level performance via MLIR. **Gleam** brings ML-family type safety to the BEAM VM. **Crystal** delivers Ruby-like syntax compiled to native code. Each addresses a specific dimension where Rust demands more from its users than they're willing to give.

---

## Part II — Ruby: a language designed to make programmers happy

### From a chat room in Osaka to global web dominance

Yukihiro "Matz" Matsumoto conceived Ruby on **February 24, 1993**, during an online chat with colleague Keiju Ishitsuka. They chose the name "Ruby" over "Coral" — a tongue-in-cheek nod to Perl (pearl is June's birthstone; ruby is July's). Matz, who had graduated from the University of Tsukuba in 1990 with a degree in Information Science, began implementation immediately and produced his first "Hello World" by August 1993. Ruby 0.95 was publicly released on **December 21, 1995**, announced on Japanese newsgroups.

Matz's motivation was explicitly human-centric. He told the Ruby-Talk mailing list in 1999: *"I was talking with my colleague about the possibility of an object-oriented scripting language. I knew Perl, but I didn't like it really, because it had the smell of a toy language. I wanted a scripting language that was more powerful than Perl, and more object-oriented than Python."* This philosophy crystallized into what Matz calls **designing for "programmer happiness"**: *"For me the purpose of life is partly to have joy. Programmers often feel joy when they can concentrate on the creative side of programming, so Ruby is designed to make programmers happy"* (Artima, 2003).

The often-cited **Principle of Least Surprise** (POLS) is frequently misunderstood. Matz clarified: *"The principle of least surprise is not for you only. The principle of least surprise means principle of least my surprise. And it means the principle of least surprise after you learn Ruby very well"* (Artima, September 29, 2003). Ruby blended elements from Perl (multiple approaches), Smalltalk (pure OOP and message passing), Eiffel (design-by-contract), and Lisp (closures and functional programming).

### Everything is an object — Ruby's radical purity

Ruby implements **pure object-orientation** more completely than almost any mainstream language. Integers, floating-point numbers, `nil`, `true`, `false`, and even classes themselves are objects with methods. The expression `5.times { print "Hello" }` is not syntactic sugar — it's a genuine method call on an integer object. This Smalltalk-derived design means Ruby has no "primitive" types, and all operations are fundamentally message-passing.

**Blocks, Procs, and Lambdas** form Ruby's closure system. Blocks are anonymous code chunks passed to methods using `do...end` or `{}` syntax; `Proc.new` objectifies blocks; lambdas are stricter Procs with arity checking. Matz noted: *"In Ruby closures, I wanted to respect the Lisp culture"* (ruby-lang.org/en/about/).

**Metaprogramming** is where Ruby truly distinguishes itself. `method_missing` intercepts calls to undefined methods, enabling dynamic behavior. `define_method` creates methods at runtime. Open classes allow any class — including core classes like `String` and `Integer` — to be reopened and modified (monkey-patching). `class_eval` and `instance_eval` evaluate code in the context of arbitrary classes or instances. This metaprogramming machinery is the engine behind Rails' famously expressive DSLs: `has_many :posts`, `validates :name, presence: true`, and `before_action :authenticate_user!` are all method calls that dynamically generate behavior.

**Duck typing** governs Ruby's type philosophy: *"If it walks like a duck and quacks like a duck, it's a duck."* Ruby cares about what an object *can do* (responds to which messages), not what class hierarchy it belongs to. **Mixins via modules** replace multiple inheritance — any class implementing `each` can include `Enumerable` to gain `map`, `select`, `reduce`, and dozens of other methods for free.

### Rails: the framework that changed web development

The trajectory of Ruby is inseparable from **Ruby on Rails**, created by **David Heinemeier Hansson (DHH)** while working at 37signals. Extracted from the Basecamp project and released as open source in **July 2004** (with Rails 1.0 following on December 13, 2005), Rails introduced conventions that reshaped the entire web development landscape: **Convention over Configuration** (CoC), **Don't Repeat Yourself** (DRY), the Active Record ORM pattern, and the MVC architecture applied to web applications.

Rails' influence extends far beyond Ruby. It directly inspired **Django** (Python), **Laravel** (PHP), **Phoenix** (Elixir), ASP.NET MVC, Grails (Groovy), and Sails.js (Node.js). The framework now has over **5,000 contributors** on GitHub, serves an estimated **~2.3 million developers**, and powers approximately **667,000 companies** globally. Rails 8.0, released in 2024, champions a "No PaaS Required" philosophy with built-in deployment capabilities.

The broader ecosystem includes **~187,900 gems** on RubyGems.org, with monthly download volume reaching **4.15 billion in April 2025** — a 51% increase from the prior year. The testing culture pioneered by Ruby (RSpec for BDD, Minitest in stdlib, Cucumber for acceptance testing) remains one of the language's enduring contributions to software engineering practice.

### Performance evolution: from "slow but happy" to YJIT's Rust-powered JIT

Ruby's performance journey represents one of the most dramatic improvement arcs in programming language history. The original MRI (Matz's Ruby Interpreter) was a tree-walking interpreter — functional but slow. **YARV** (Yet Another Ruby VM), developed by Koichi Sasada and integrated in Ruby 1.9 (2007), replaced this with a bytecode VM, delivering the first major performance leap.

At RubyConf 2015, Matz announced the **Ruby 3x3** goal: Ruby 3.0 should be three times faster than Ruby 2.0. He later admitted: *"When I first declared 'Ruby3x3' in the conference keynote, many including members of the core team felt 'Matz is a boaster'. In fact, I felt so too. But we did."* The goal was achieved on the **Optcarrot benchmark** (NES emulation), with CPU-intensive code running "solidly three times the speed." Real-world Rails applications saw ~70%+ end-to-end improvement.

The most transformative development has been **YJIT** (Yet Another JIT), developed at Shopify under the leadership of **Maxime Chevalier-Boisvert**. YJIT uses **Lazy Basic Block Versioning** (LBBV), creating specialized code versions based on runtime type information. Critically, YJIT's backend is **written in Rust** — a direct bridge between the two languages under study.

| Ruby Version | YJIT Benchmark Improvement | Production Impact |
|-------------|---------------------------|-------------------|
| 3.1 (Dec 2021) | ~20% faster than interpreter | ~6% on Shopify canary |
| 3.2 (Dec 2022) | **38% faster** (57% cumulative vs. 3.1.3 interpreter) | **10% avg** on Storefront Renderer |
| 3.3 (Dec 2023) | Register allocation optimization | **15% faster** than YJIT 3.2 |
| 3.4 (Dec 2024) | **~92% faster** on headline benchmarks; ~2× on representative suite | 5–7% faster than YJIT 3.3.6 |

On Shopify's production infrastructure during Black Friday/Cyber Monday 2024, YJIT 3.4 handled **80 million requests per minute** (284 million at edge), processing **$11.5 billion in merchant sales**. The Fibonacci benchmark shows the JIT's raw power: interpreter execution of 16.44 seconds drops to **2.20 seconds** with YJIT — a **7.5× speedup** on pure-Ruby compute.

Ruby 3.0 also introduced **Ractors** (actor-model concurrency enabling true parallelism without the GVL) and enhanced **Fibers** (lightweight cooperative concurrency with a Fiber Scheduler for non-blocking I/O). Ractors remain experimental as of Ruby 3.4, with limited real-world adoption due to ecosystem compatibility challenges. The **GVL** (Global VM Lock) continues to limit CRuby to single-threaded Ruby code execution, though I/O operations release the lock and Ractors bypass it entirely.

### Industry adoption: Shopify's billion-dollar bet on Ruby

**Shopify** represents the most significant corporate investment in Ruby's future. Self-described as running **"the biggest Rails app in the world,"** Shopify employs multiple Rails Core team members and Ruby committers, invests nearly **$500,000 annually** in academic Ruby research, and has built YJIT, ZJIT (a next-generation JIT merged upstream), the **Prism parser** (now the default in Ruby 3.4), Ruby LSP, the Trilogy MySQL client, and extensive Sorbet type-checking infrastructure (98% of files typed, 61% of methods with signatures). During BFCM 2025, the platform processed **489 million requests per minute** at edge with **53 million database queries per second**.

**GitHub** runs a **2-million-line Rails monolith** with over **1,000 engineers** deploying 20 times daily. **Stripe** uses Ruby extensively in their payment infrastructure (though not Rails — they built their own framework) and created **Sorbet**, the most mature Ruby type checker. **Basecamp/37signals**, where Rails was born, continues to power Basecamp and Hey.com with Rails.

The type safety landscape in Ruby is **fragmented**. The official **RBS** system (Ruby 3.0+) uses separate `.rbs` signature files, while Stripe's **Sorbet** uses inline annotations. Sorbet is dramatically faster — type-checking Shopify's monolith in **~15 seconds** versus Steep's (the RBS-native checker) approximately **45 minutes**. Matz remains personally opposed to in-language type syntax, making RBS his "olive branch" keeping types out of `.rb` files.

---

## Part III — The chasm between them: a 34-dimension comparison

The following matrix captures the fundamental architectural differences across every significant dimension of language design. Where Rust and Ruby differ most sharply, the greatest opportunities for synthesis exist.

| Dimension | Rust | Ruby |
|---|---|---|
| **Memory model** | Ownership + borrowing + lifetimes; deterministic deallocation; no GC | Garbage collected (mark-and-sweep, generational) |
| **Concurrency** | OS threads, async/await (Tokio), channels; data-race free at compile time | Threads (GVL-limited), Fibers (cooperative), Ractors (experimental) |
| **Type system** | Static, strong, algebraic (enums + structs), trait-based | Dynamic, strong, duck-typed |
| **Type inference** | Local (Hindley-Milner-like); function signatures require annotations | Fully dynamic; optional gradual typing (RBS/Sorbet) |
| **Null handling** | No null; `Option<T>` enforced by compiler | `nil` is an object; any variable can be nil |
| **Error handling** | `Result<T, E>` + `?` operator; no exceptions | Exceptions (`begin`/`rescue`/`ensure`/`raise`) |
| **Pattern matching** | First-class (`match`, `if let`, destructuring) | Added Ruby 3.0 (`case`/`in`); still maturing |
| **Metaprogramming** | Compile-time: procedural macros, derive macros | Runtime: `method_missing`, `define_method`, open classes |
| **Generics** | Monomorphized with trait bounds; zero-cost | Duck typing as implicit generics |
| **OOP support** | Structs + impl + traits; no classical inheritance | Pure OOP; everything is an object |
| **FP support** | Strong: iterators, closures, immutable-by-default | Good: blocks, lambdas, `Enumerable`; mutable-by-default |
| **Inheritance** | None; composition via traits | Single inheritance + module mixins |
| **Package manager** | Cargo (unified build+deps+test+publish) | RubyGems + Bundler |
| **Package count** | ~250,500 crates | ~187,900 gems |
| **Build system** | Cargo (integrated) | Rake |
| **Testing** | Built-in (`#[test]`) | Minitest (stdlib), RSpec (community) |
| **Documentation** | `rustdoc` / docs.rs | RDoc / YARD |
| **Formatter** | rustfmt (canonical) | RuboCop / syntax_tree |
| **Linter** | Clippy (700+ lints) | RuboCop |
| **IDE support** | rust-analyzer, RustRover (JetBrains) | Ruby LSP, RubyMine (JetBrains) |
| **REPL** | Limited (evcxr) | Excellent (irb, pry) |
| **Compile vs. interpret** | AOT compiled (LLVM) | Interpreted bytecode VM; optional JIT (YJIT) |
| **Deployment** | Single static binary | Requires runtime; app servers (Puma) |
| **Binary size** | ~1–10 MB | N/A (interpreter ~25 MB + gems) |
| **Learning curve** | Steep (3–6 months to proficiency) | Gentle (days to weeks) |
| **Community size** | ~2.3M (JetBrains) to ~5M+ (SlashData) | ~3–4M (estimated, stagnant) |
| **Governance** | Rust Foundation (nonprofit); RFC process | Ruby Core team led by Matz; Ruby Central |
| **License** | MIT / Apache 2.0 (dual) | Ruby License (BSD-2-like) + GPL-2.0 |
| **Mobile support** | Limited (FFI, cross-compilation) | Limited (RubyMotion, niche) |
| **WebAssembly** | Excellent (first-class wasm32 target) | Experimental (ruby.wasm) |
| **Embedded/IoT** | Strong (`no_std`, `embedded-hal`) | Minimal (mruby) |
| **Startup time** | ~1–2ms (native binary) | ~50ms (interpreter); 3–15s (Rails boot) |
| **Idle memory** | 10–80 MB (web server) | 60–200 MB (web server) |

---

## Part IV — Performance: the numbers that matter

### Rust versus Ruby across every measurable axis

The performance gap between Rust and Ruby is not a matter of opinion — it is measured in orders of magnitude for compute-intensive work and significant multiples for I/O-bound web workloads.

**TechEmpower Framework Benchmarks (Round 23, February 2025)** — Fortunes test (the most realistic web scenario, involving database queries, HTML templating, and HTTP response construction):

| Language/Framework | Requests/Second | Relative to Rails |
|---|---|---|
| Rust / Actix-web | **320,144** | **7.5×** |
| Go / Fiber | 338,096 | 7.9× |
| Java / Spring | 243,639 | 5.7× |
| Node.js / Express | 78,136 | 1.8× |
| **Ruby / Rails** | **42,546** | **1.0× (baseline)** |
| Python / Django | 32,651 | 0.8× |

**CPU-intensive benchmarks** (programming-language-benchmarks.vercel.app, AMD EPYC 7763):

| Benchmark | Rust | CRuby 3.4 | Ruby + YJIT | Rust : CRuby Ratio |
|---|---|---|---|---|
| n-body (500K) | 18ms / 1.8 MB | 2,832ms / 12.8 MB | 1,044ms / 13.5 MB | **157×** |
| Spectral-norm | 492ms / 2.3 MB | Timeout (>5s) | Timeout (>5s) | **>10×** |
| Hello world (startup) | 1.2ms / 1.8 MB | 50ms / 12.8 MB | 51ms / 13.0 MB | **42×** |

**Memory comparison** — a developer migrating a PostgreSQL JSON streaming endpoint (50K+ rows, 200MB data) from Rails to Actix+sqlx reported: Rails consumed **~2 GB RAM** on Heroku while the Rust version used **~4 MB** — approximately **500× less memory** — and ran on a **10× cheaper** Heroku dyno.

**Rust versus C/C++.** The JetBrains RustRover analysis (December 2025) confirms: *"C and C++ tend to win out, but Rust is often within 5–10% — and beats the older language on some measures."* A Karlson et al. arxiv study found C++ outperforms Rust in matrix multiplication while **Rust beats C++ in merge sort**. PNG decoding benchmarks showed Rust-based memory-safe decoders **"vastly outperformed" C libraries** thanks to efficient concurrency.

**Compile time trade-offs.** Rust's compile times remain its most significant productivity cost. A 40K-line crate takes approximately **5–10 seconds** for incremental rebuilds on modern hardware; large projects (359K lines, 946 dependencies) can take **~26 seconds** incrementally and **15–20 minutes** for clean builds. Go, by contrast, compiles most projects in seconds. The 2025 Rust Compiler Performance Survey confirmed that **~45% of respondents** who stopped using Rust cited build performance as a contributing factor.

### Code in conversation: how the same task looks in both languages

The following side-by-side comparisons illuminate the philosophical chasm. Consider a minimal HTTP server returning JSON:

**Ruby (Sinatra) — 6 lines:**
```ruby
require 'sinatra'
require 'json'

get '/' do
  content_type :json
  { message: "hello" }.to_json
end
```

**Rust (Axum) — 20 lines:**
```rust
use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct Message { message: String }

async fn hello() -> Json<Message> {
    Json(Message { message: "hello".to_string() })
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

Sinatra reads like a configuration file. Axum requires a struct definition, derive macros for serialization, an async runtime, explicit type-safe bindings, and error handling at every await point. The trade-off: Axum serves **250,000–500,000 requests per second** versus Sinatra's ~10,000–50,000.

Error handling provides an even starker contrast. Ruby's exception-based approach keeps the "happy path" clean with `rescue` blocks catching failures. Rust's `Result<T, E>` with pattern matching makes errors **values** — impossible to ignore, exhaustively handled, propagated explicitly via the `?` operator. Neither approach is universally superior; they optimize for different failure modes.

---

## Part V — The Claude Code incident and the TypeScript-to-Rust wave

### What actually happened with Claude Code

The user referenced an "embarrassingly available software leak" and subsequent Rust rewrite of Anthropic's Claude Code. Research reveals a story that is **partially true but importantly misattributed**.

**Claude Code** is Anthropic's agentic coding tool — a terminal-based AI assistant that reads codebases, edits files, runs commands, and handles git workflows. Launched in late 2024 with a broader research preview in February 2025, it reached a run-rate revenue exceeding **$2.5 billion** by February 2026 (CNBC). The codebase comprises approximately **512,000 lines of TypeScript** across ~1,900 files, built with React/Ink, Yoga layout engine, and the Bun runtime, distributed via npm.

**The leak is verified.** On **March 31, 2026**, a **59.8 MB JavaScript source map file** was accidentally bundled into npm package version 2.1.88. This `.map` file referenced a zip archive on Anthropic's R2 cloud storage containing the complete original TypeScript source code. Discovered by Chaofan Shou (an intern at Solayer Labs), the post revealing it accumulated over **21 million views** on X. Boris Cherny, a Claude Code engineer, confirmed it was *"plain developer error, not a tooling bug"* — a failed `.npmignore` configuration. The leaked code revealed internal model codenames, dozens of unreleased feature flags, and a Tamagotchi-style "Buddy" pet system.

**The Rust rewrite, however, was not Anthropic's work.** Following the leak, multiple community developers created independent Rust reimplementations: **Claurst** (a clean-room reimplementation using behavioral specifications), **claux** (a personal rewrite documented in a blog series — ~1,530 lines of Rust versus 512,000 lines of TypeScript for core functionality), and several pull requests to Anthropic's official repository (PR #11583 with 13,125 lines of Rust across 10 crates and 188 tests, PR #11666, PR #41568). **Anthropic rejected all Rust rewrite PRs**, and the official Claude Code remains TypeScript as of April 2026.

### The broader TypeScript-to-Rust infrastructure rewrite is real

While the Claude Code Rust rewrite is a community artifact, the broader trend of JavaScript/TypeScript developer tooling being rewritten in Rust is extensively documented and represents one of the most significant shifts in web development infrastructure:

| Project | Replaces | Performance Gain | Status |
|---------|----------|-----------------|--------|
| **SWC** | Babel | **20× faster** single-thread, **70×** on 4 cores | Stable; used by Next.js, Parcel, Deno |
| **Turbopack** | Webpack | Claims **700× faster** (disputed) | Stable; default in Next.js 16 |
| **Biome** | ESLint + Prettier | **10–100× faster** | Stable v2 |
| **Rolldown** | Rollup + esbuild | **10–30× faster** | 1.0 RC (Jan 2026); default in Vite 7+ |
| **Oxlint** | ESLint | **50–100× faster** | Production-ready |
| **Rspack** | Webpack | **70%+ build time reduction** | Stable 1.0 (ByteDance) |
| **Lightning CSS** | PostCSS/cssnano | **100× faster** | Stable |

Notably, **TypeScript itself** (the compiler) is being rewritten — but in **Go**, not Rust. Anders Hejlsberg announced TypeScript 7 will use a native Go compiler achieving **~10× faster** compilation, with the choice of Go over Rust driven by ownership model mismatch considerations when porting the existing compiler architecture.

---

## Part VI — Prior art in language reconciliation

### Crystal: the closest existing Rust-Ruby synthesis (and why it hasn't broken through)

**Crystal** (crystal-lang.org), created by Ary Borenszweig and Juan Wajnerman at Manas Labs in Buenos Aires starting in **2011**, is the most direct attempt at bridging Ruby's syntax with compiled performance. Crystal features Ruby-inspired syntax (not full compatibility), static type checking via global type inference, nil checking at compile time, and compilation to native code via LLVM. Its concurrency model borrows from Go's CSP with lightweight fibers.

Crystal's performance is remarkable. On CPU-intensive benchmarks, Crystal is typically **10–60× faster than Ruby** and **surprisingly competitive with Rust** — in the binary trees benchmark (input 18), Crystal actually clocked 1,231ms versus Rust's 1,259ms. Currently at version **1.19.1** with ~19,700 GitHub stars, Crystal nonetheless remains niche. The reasons illuminate critical lessons for any language synthesis attempt: **no major corporate backer** (funded via OpenCollective and 84codes sponsorship), a **late 1.0 release** (March 2021, after 10 years of development), **incomplete Windows support**, **slow compile times** from global type inference, and a small ecosystem competing against Go and Rust with vastly larger communities.

### Other bridges across the chasm

**Elixir**, created in 2011 by José Valim (a former Rails Core team member), brings Ruby-inspired syntax to the Erlang BEAM VM, providing battle-tested concurrency, fault tolerance, and the Phoenix framework. It scored **66% admiration** in the Stack Overflow 2025 survey and has been adopted by Discord, Remote.com, and others.

**Mojo**, created by Chris Lattner (LLVM, Swift, MLIR creator), is a Python superset with systems-level performance, built on MLIR. It demonstrates the **progressive disclosure** pattern — write Python-like `def` functions for rapid prototyping, graduate to typed `fn` functions for performance — and claims up to **35,000× faster** than CPython for certain workloads. Mojo's standard library is open-sourced but the compiler remains closed, and the language is pre-1.0.

**Gleam** brings ML-family static typing to the BEAM VM, achieving **70% admiration** in the 2025 Stack Overflow survey (second only to Rust's 72%). Its toolchain is written in Rust.

### Academic foundations for synthesis

The theoretical groundwork for combining static and dynamic typing — **gradual typing** — was established by Siek and Taha in their foundational 2006 paper "Gradual Typing for Functional Languages." Refined criteria were formalized at SNAPL 2015, establishing the "gradual guarantee" (removing type annotations should not break well-typed programs). **Abstracting Gradual Typing** (Garcia, Clark, and Tanter; POPL 2016) provided formal foundations using abstract interpretation, while **Gradual Type Theory** (New, Licata, and Ahmed; POPL 2019) gave an axiomatic account of program equivalence. Industrial implementations include TypeScript, Hack (Facebook), Flow, Python's PEP 484/mypy, and Sorbet for Ruby.

The historical precedents of successful language synthesis are instructive. **TypeScript** succeeded through a superset strategy — any JavaScript is valid TypeScript — preserving the existing ecosystem while adding opt-in types. **Kotlin** beat Scala's expressiveness by prioritizing familiarity; as one analysis noted, *"If a programming language is not immediately understandable, it is not suitable for the mainstream."* **Mojo's** Python import compatibility represents the most ambitious current attempt at backward-compatible language extension.

---

## Part VII — Garnet: a synthesis proposal

### Design philosophy: "Safe by default, fast when needed, joyful always"

Garnet is proposed as a language targeting **three domains simultaneously**: systems/performance (Rust's territory), web/backend (Ruby/Rails territory), and general scripting/automation (Python's territory). The name continues Ruby's gemstone tradition while nodding to Rust's mineral aesthetic — a garnet is a crystal that forms under pressure, strong, multifaceted, and beautiful.

The core design insight is that **most code does not need Rust-level control, but some code desperately does**, and forcing all code into either mode creates unnecessary friction. Garnet addresses this through a **dual-mode architecture**: managed mode (the default, optimized for developer happiness) and safe mode (opt-in, providing Rust-like ownership semantics for performance-critical paths).

### The dual-mode memory model

In **managed mode** (the default), Garnet uses automatic reference counting with cycle detection, similar to Swift's ARC. This eliminates garbage collection pauses while requiring no manual memory management. Variables are inferred, types are optional, and the syntax favors expressiveness. A managed-mode Garnet program should be writable by anyone comfortable with Ruby, Python, or TypeScript.

In **safe mode** (activated per-module with `@safe`), ownership and borrowing rules apply. Lifetime annotations become available. Memory management becomes deterministic with zero overhead. Safe-mode Garnet compiles to the same quality of machine code as Rust. The compiler inserts safe bridging automatically at the boundary between managed and safe modules.

```garnet
# Managed mode — feels like Ruby
def connect(config)
  puts "Connecting to #{config.database} at #{config.host}:#{config.port}"
end

# Safe mode — feels like Rust
@safe
module FastPath
  def process(own data: Buffer) -> own Buffer
    let ref header = data.slice(0, 4)
    let mut body = data.slice(4, -1)
    body.transform!
    data
  end
end
```

### Concurrency: structured async with typed actors

Garnet combines structured concurrency (all spawned tasks are scoped to their parent), `async`/`await` for I/O-bound work backed by a work-stealing event loop, and **typed actors** as a first-class primitive for CPU-bound parallelism — like Ractors but production-ready, with typed message protocols enforced by the compiler. No shared mutable state between actors.

### Metaprogramming: compile-time power with runtime flexibility

The metaprogramming system bridges Rust's compile-time code generation with Ruby's runtime reflection through explicit opt-in. Compile-time macros use a friendlier syntax inspired by Ruby blocks. Runtime reflection (including `method_missing`-style dynamic dispatch) requires an explicit `@dynamic` annotation, making the safety-expressiveness trade-off visible to both humans and static analysis tools.

### Interoperability as a first principle

Drawing from TypeScript's superset strategy and Mojo's Python import capability, Garnet provides native FFI with Rust (safe-mode modules are ABI-compatible), C ABI support for universal interop, Ruby VM embedding for running existing gems during migration, and WebAssembly compilation for browser/edge deployment. A unified CLI (`garnet`) handles build, test, format, lint, document, REPL, and publish — following Cargo's model.

---

## Part VIII — Market viability and commercial opportunity

### The developer tools market is expanding rapidly

The software development tools market reached **$6.41–7.57 billion in 2025** and is projected to grow to **$15.72 billion by 2031** at a 16.12% CAGR (Mordor Intelligence). GitHub Copilot alone generated **$400 million** in revenue in 2025 (248% year-over-year growth), while Cursor AI reached a **$9.9 billion valuation** in June 2025 after a $900 million round. The programming language training market is forecast to grow by **$8.53 billion at 19.31% CAGR** through 2028 (Technavio).

### Where Rust and Ruby stand today

| Metric | Rust | Ruby |
|--------|------|------|
| Stack Overflow 2024 usage (all) | **12.6%** | 5.2% |
| Stack Overflow admiration | **#1 (83%)** | Not in top tier |
| TIOBE rank | ~#13 (↑ rising) | ~#30 (↓ declining) |
| RedMonk rank | #19 | #9 (legacy position) |
| GitHub Octoverse trend | >50% community growth | ~18th, declining |
| Developer population | ~2.3–5M (growing fast) | ~3–4M (stagnant) |
| US salary (average) | $110K–$130K | $115K–$137K |
| Growth trajectory | **Fastest-growing major language** | **Declining steadily** |
| Primary corporate champion | AWS, Microsoft, Google, Cloudflare | Shopify |

The global developer population reached **47.2 million** in Q1 2025 (SlashData), with growth decelerating from 21% (2023–24) to **10%** (2024–25). Rust is the **only language to set a new usage record** in the 2024 JetBrains survey, with **2.27 million developers** (709,000 primary). Ruby, per TIOBE CEO commentary, faces existential pressure: *"There is no need for Ruby anymore"* — a provocative overstatement, but one reflecting Ruby's fall from #8 (2016 peak) to ~#30.

### TAM, SAM, and SOM for Garnet

**TAM (Total Addressable Market):** The combined market for developer tooling, training, consulting, and ecosystem services around a top-10 programming language is estimated at **$1–5 billion**, based on JetBrains' multi-billion-dollar IDE revenue, Vercel's ~$250M ARR built on Next.js/TypeScript infrastructure, and the $130M funding raised by Modular for Mojo.

**SAM (Serviceable Available Market):** For a language targeting the intersection of systems programming, web development, and scripting — the combined developer pools of Rust (~5M), Ruby (~3–4M), Python scripting (~10M subset), and Go (~5M) — the serviceable market for language-adjacent tooling (IDE plugins, cloud hosting, training, consulting) is approximately **$200M–$1B** at steady state.

**SOM (Serviceable Obtainable Market):** A new language realistically achieves 0.5–2% of its target developer population within 5 years if backed by strong corporate sponsorship and a killer application. For Garnet, targeting systems-web-scripting developers: a realistic first 5-year SOM of **$10–50M** in ecosystem revenue (tooling subscriptions, training, enterprise consulting, hosted build services), scaling to **$100–500M** at maturity if the language enters the top 15.

### Competitive landscape and the 5-to-7-year window

Historical data shows new languages typically require **5–10 years** to reach mainstream adoption, with a critical **5–7 year "prove it or lose it" window**. TypeScript took ~8 years (2012→2020), Go ~7 years (2009→2016, catalyzed by Docker/Kubernetes), and Rust ~10 years (2015→2025, still gaining). Swift and Kotlin achieved faster adoption (~3 years) by leveraging captive platform audiences (iOS and Android respectively).

Every successful modern language has had either **major corporate backing** (Go/Google, Swift/Apple, Kotlin/JetBrains+Google, TypeScript/Microsoft) or a **killer application** (Ruby/Rails, Dart/Flutter, Scala/Spark). Crystal's limited adoption despite excellent technical merits demonstrates that technical quality alone is insufficient. Garnet would require: (1) a well-funded foundation or corporate sponsor, (2) a compelling framework or tool that showcases its dual-mode advantage, (3) strategic positioning in the **LLM-assisted development** wave where its semantic clarity could differentiate, and (4) a deliberately gentle onboarding curve with progressive disclosure of safety features.

### LLM-native language design: an emerging competitive advantage

A Garnet-specific opportunity exists in designing syntax optimized for **LLM comprehension and generation**. Current LLMs struggle most with Rust's lifetime annotations and borrow checker interactions — features that require tracking complex state across code blocks. A language that provides Rust-level safety through a more regular, predictable syntax would be easier for AI coding assistants to generate correctly, potentially creating a virtuous cycle: better AI-generated code → faster adoption → more training data → even better AI assistance. This "AI-native" design consideration — syntax regularity, semantic clarity, explicit intent markers — represents a novel axis of language competition that didn't exist before 2023.

### Risk analysis

The primary risks are: (1) **ecosystem bootstrapping** — the cold-start problem of having no libraries, requiring either excellent FFI (Garnet's Rust/C/Ruby interop strategy) or massive initial investment in stdlib; (2) **talent acquisition** — developers must learn a new language, though the dual-mode design allows Ruby/Python developers to start in managed mode and Rust developers to start in safe mode; (3) **corporate commitment fatigue** — the industry has seen too many "next big language" announcements, creating skepticism (Carbon, Val/Hylo, and others remain experimental years after announcement); and (4) **Mojo competition** — Chris Lattner's Mojo addresses a similar gap for the Python-to-systems bridge, and Lattner's track record (LLVM, Swift) gives Mojo significant credibility.

---

## The path forward: what this study reveals

This analysis establishes that Rust and Ruby are not merely different languages but **complementary philosophies** — one optimizing for the machine's needs (safety, performance, correctness), the other for the human's needs (expressiveness, happiness, velocity). Their respective weaknesses map almost exactly to each other's strengths: Rust's steep learning curve corresponds to Ruby's gentle onboarding; Ruby's runtime performance deficit corresponds to Rust's C-competitive execution; Rust's verbosity mirrors Ruby's conciseness; Ruby's type safety gaps mirror Rust's algebraic type system.

The most profound insight from the benchmark data is that **the performance gap is narrowing from Ruby's side** (YJIT, written in Rust, delivers 2–7× speedups) while **the ergonomic gap is narrowing from Rust's side** (each edition simplifies lifetime inference, error handling, and async patterns). They are converging — slowly — but a purpose-built synthesis could accelerate this convergence by decades.

Crystal has proven the concept technically viable (Ruby syntax achieving near-Rust performance) but failed to achieve adoption due to insufficient corporate backing and ecosystem investment. Mojo has proven the progressive-disclosure model viable (Python compatibility + systems performance). TypeScript has proven the superset/gradual-typing strategy viable (preserving the existing ecosystem while adding safety). Garnet's opportunity lies in combining all three precedents: Crystal's proof that Ruby + compiled works, Mojo's progressive disclosure, and TypeScript's ecosystem preservation — while adding Rust's ownership model as an opt-in safety layer and AI-native syntax as a forward-looking differentiator.

The market conditions are favorable: a $15.7 billion developer tools market by 2031, a demonstrated willingness among enterprises to invest in language infrastructure (Shopify's $500K/year in Ruby research, Google's $1M Rust Foundation donation, Microsoft's $10M internal Rust investment), and a growing recognition that **memory safety is a national security concern** (the US White House's February 2024 recommendation for memory-safe languages). A language that makes Rust-level safety accessible to Ruby/Python-level developers would address a genuine, measurable, and increasingly urgent need.

The question is not whether such a language should exist — the evidence overwhelmingly suggests it should — but whether the investment required to build it (estimated at $5–20 million over 5 years for a credible initial ecosystem) can be justified against the risk of joining the growing list of ambitious but ultimately niche language experiments. The answer depends entirely on execution: the right team, the right corporate partners, and above all, the right killer application that makes the dual-mode advantage impossible to ignore.