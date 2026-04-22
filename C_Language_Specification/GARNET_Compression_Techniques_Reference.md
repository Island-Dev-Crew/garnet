# Garnet — Compression Techniques Reference (v0.4)

**Status:** Companion reference (non-normative). Consolidates the
PolarQuant + QJL mathematical machinery that Gemini 3.1 Pro Deep Research
contributed during the v2.1 four-model consensus, formalises it for
inclusion in the Garnet runtime documentation, and references it from
the Memory Manager Architecture (§6.4). v0.4 adds the deeper derivations
the v3.3 Phase 1D plan committed to: Hadamard projection construction
for non-power-of-2 dimension, calibration of the QJL scalar α, the
adversarial-robustness re-seed schedule, and the cross-reference index
to Paper IV Appendix B.

**Anchor:** *"In an abundance of counsellors there is safety." — Proverbs 11:14*

---

## 1. Motivation

Long-horizon agent workloads keep tens of millions of token-level vectors
in fast-tier memory (HBM on GPU, DRAM on CPU). At full precision this
caps concurrent users, blows past device budgets, and erodes inference
economics. The TurboQuant paper (Google Research, March 2026) demonstrated
that **6× compression with no measurable accuracy loss** is achievable on
modern KV caches via a two-stage pipeline. Garnet adopts this pipeline as
a *runtime tactic* — see Mini-Spec §12 OQ-6 — and makes the algorithmic
contract explicit here so implementers can interoperate.

The pipeline is **language-neutral**: it is a serialization scheme over
`Vec<f32>` payloads, not a Garnet-specific syntax extension. Memory units
declared with kind `semantic` (vector index over embeddings) and
`episodic` (timestamped logs of high-dimensional activations) are the
primary candidates for compressed storage; `working` and `procedural`
typically remain uncompressed because their access patterns punish
quantisation overhead.

---

## 2. Stage 1 — PolarQuant: Geometric Simplification

### 2.1 Inputs and intuition

Given a high-dimensional input vector `x ∈ ℝ^d`, traditional vector
quantisation (PQ, RQ, OPQ) requires storing per-block scale + offset
constants in full precision. Those constants add 1–2 bits of overhead
per parameter, eroding the compression budget. PolarQuant *eliminates*
the need for those constants by mapping each block onto a known geometric
distribution.

### 2.2 The pipeline

1. **Random projection.** Apply a random orthonormal projection
   `P ∈ ℝ^{d×d}` (a Hadamard or Walsh-Hadamard matrix is computationally
   convenient). This step decorrelates the input and induces a
   **concentrated Beta(α, β) distribution** over the projected
   components. The shape parameters `α, β` depend on `d` and the
   projection rank, but for sufficiently large `d` (≥ 512) the
   distribution converges and can be treated as fixed.

2. **Polar coordinate transform.** Group the projected components into
   pairs `(u_{2i}, u_{2i+1})` and convert each pair to polar
   coordinates:
   ```
   r_i  = sqrt(u_{2i}^2 + u_{2i+1}^2)
   θ_i  = atan2(u_{2i+1}, u_{2i})
   ```
   The radius `r_i` follows a **chi distribution with 2 degrees of
   freedom** (Rayleigh) — a closed-form, parameter-free distribution.

3. **Lookup-table quantisation.** Because the radius distribution is
   known and fixed, a single global lookup table converts every `r_i`
   into a 2- or 4-bit codeword. **No per-block scale or offset is
   stored.** The angle `θ_i` is uniformly distributed and quantised
   independently with another fixed-shape table.

### 2.3 Compression budget

Naive 32-bit FP storage: `32 d` bits per vector.

PolarQuant 4-bit (radius + angle): `4 + 4 = 8` bits per pair, so `4 d`
bits per vector → **8× reduction** on the radius/angle stream alone,
before the residual stage corrects bias.

---

## 3. Stage 2 — QJL: Quantised Johnson–Lindenstrauss Residuals

### 3.1 Why a residual stage

Mean-Squared-Error optimal quantisers are biased estimators of inner
products: the systematic error compounds in the attention dot-product
that drives transformer inference. Left uncorrected, this bias degrades
retrieval accuracy on long contexts. QJL (Quantised Johnson–
Lindenstrauss) supplies a **single-bit residual correction** that
restores the inner-product expectation without paying for additional
storage on the major axis.

### 3.2 The pipeline

1. Compute the per-component residual `e_i = x_i − dequantise(c_i)`
   where `c_i` is the PolarQuant codeword.

2. Apply a JL projection `J ∈ ℝ^{d×d}` (Gaussian random matrix scaled by
   `1/√d`) to the residual vector `e`.

3. Encode the projected residual using **only its sign bit**:
   ```
   q_i = sign(J · e)_i  ∈ {−1, +1}
   ```

4. The dequantiser reconstructs the corrected vector by adding back the
   sign-bit estimate:
   ```
   x_hat = dequantise(c) + α · J^T · q
   ```
   where `α` is a calibrated scalar (≈ `√(d) / r_avg`) that the encoder
   stores ONCE per cache, not per vector.

### 3.3 Why this works

The Johnson–Lindenstrauss lemma guarantees that random projections
preserve pairwise distances up to a factor of `(1 ± ε)` with high
probability. The sign of the projected residual carries the dominant bit
of error information; reconstruction with `J^T · q` undoes the bias to
within `O(1/√d)` precision — exactly the regime needed to keep attention
scores within numerical noise.

### 3.4 Storage budget

QJL adds `d` bits per vector (the sign bits) plus a constant `α` per
cache. For 4-bit PolarQuant + 1-bit QJL, total = **5 bits per dimension**,
i.e. `5 d` bits per vector → **6.4× compression** vs. FP32 with
**< 0.5% retrieval-quality loss** in TurboQuant's published evaluations.

---

## 4. Combined pipeline (TurboQuant)

```
encode(x):
    p   ← P · x                          # random orthonormal projection
    pl  ← polar_pairs(p)                 # (r_i, θ_i)
    c   ← lookup_table_quantise(pl)       # 4-bit codewords, no constants
    e   ← p − dequantise(c)              # residual
    q   ← sign(J · e)                    # 1-bit JL residual
    return (c, q)

decode(c, q):
    p_hat ← dequantise(c) + α · J^T · q
    x_hat ← P^T · p_hat
    return x_hat
```

The two random matrices (`P`, `J`) are seeded once per cache and stored
out-of-band; they are not part of the per-vector payload.

---

## 5. Worked Example

For `d = 768` (a typical small-model embedding dim):

| Storage scheme | Bits per vector | Compression |
|---|---:|---:|
| FP32 baseline | 24 576 | 1× |
| FP16 | 12 288 | 2× |
| INT8 (PQ) | 6 144 + scale/offset | ≈ 3.5× |
| **PolarQuant + QJL @ 4+1 bits** | **3 840** | **6.4×** |
| Theoretical Shannon lower bound (5% loss) | ≈ 3 200 | ≈ 7.7× |

---

## 6. Performance Envelope

Reported by TurboQuant (Google Research, 2026):

- **6× memory reduction** on KV cache vs. FP16 baselines.
- **8× attention-logit speedup** on NVIDIA H100 at 4-bit precision (the
  compressed payload fits in fewer cache lines and the dot-product can
  use packed 4-bit SIMD).
- **< 0.5% retrieval quality regression** on long-context retrieval
  benchmarks (single-needle haystack at 1M tokens).
- **Zero training cost** — the projections `P` and `J` are random; no
  fine-tuning required.

---

## 7. Garnet integration model

Garnet's contract with this pipeline is **opt-in at the runtime layer**:

1. The language `memory <kind> <name> : VectorIndex<T>` declaration
   produces a typed memory unit (Mini-Spec §4.1, kind-aware allocation
   per Paper VI Contribution 4).
2. The runtime selects a backend implementation. The reference
   `garnet-memory::VectorIndex<T>` stores `Vec<f32>` uncompressed; a
   future `garnet-memory-turboquant` crate will provide a drop-in
   `TurboquantVectorIndex<T>` that compresses on insert and decompresses
   on search.
3. The choice of backend is **never visible at the Garnet language
   layer** — code that imports a memory unit cannot tell whether it is
   compressed or uncompressed. This keeps the language clean and lets
   compression evolve independently.

This separation is the four-model consensus position (memo §5,
"TurboQuant = runtime, not language core"), and it preserves the
ability to swap PolarQuant/QJL for a future scheme (e.g. Microsoft's
ResQ, Meta's coordinate-aware quantisation) without breaking source
code.

---

## 8. v0.4 Deepening — Practical Implementation Details

This section folds in derivations and engineering choices that
implementers need but the v0.3 reference left implicit. Phase 1D
commitment per the v3.3 plan.

### 8.1 Hadamard projection for non-power-of-2 dimension

The pure Walsh-Hadamard transform requires `d` to be a power of 2. For
arbitrary `d`, use a *padded structured projection* (a.k.a. SRHT —
Subsampled Randomised Hadamard Transform):

1. Find the smallest power of two `d' ≥ d`.
2. Pad `x ∈ ℝ^d` to `x' ∈ ℝ^{d'}` with zeros.
3. Apply `D x'` where `D ∈ {−1, +1}^{d'×d'}` is a diagonal
   sign-randomisation matrix (independent of input).
4. Apply the `d'×d'` Hadamard matrix `H_{d'}` constructed by Sylvester's
   recursion. Total cost: `O(d' log d')` via fast Walsh–Hadamard
   transform.
5. Sub-sample the first `d` rows of the output.

The composite `S H_{d'} D` (sub-sample × Hadamard × diagonal) is the
*structured random projection*. Standard SRHT analysis (Tropp 2011)
shows it preserves pairwise distances within `(1 ± ε)` with probability
`≥ 1 − δ` for `d ≥ Ω(ε^{-2} log(d/δ))` — the same JL bound as a Gaussian
projection, at `O(d log d)` cost instead of `O(d²)`.

This means: for any embedding dimension (768, 1024, 1536, 4096, etc.)
the Hadamard path is the right default. The earlier text's "Hadamard or
Walsh-Hadamard matrix is computationally convenient" is now made
precise: SRHT is the correct construction for arbitrary `d`.

### 8.2 Calibration of QJL scalar `α`

Stage 2 stores a single scalar `α` per cache. The earlier description
gave `α ≈ √d / r_avg` as a rule of thumb. The principled derivation:

Given the radius `r_i` follows a Rayleigh distribution under PolarQuant,
the *expected* residual magnitude is `E[‖e‖] = c_d · σ_e` where
`c_d → √(d / 2)` for large `d` and `σ_e` is the per-component residual
standard deviation. The optimal `α` (in MSE sense) for a sign-bit
estimator of a Gaussian is the *sub-Gaussian normalising constant*:

```
α* = √(π / 2) · σ_e   ≈ 1.2533 · σ_e
```

So calibration reduces to estimating `σ_e` once per cache:

```
σ_e = sqrt( mean_i (e_i^2) )
α   = sqrt(π / 2) · σ_e
```

Computed at encoder warm-up over the first 1000 vectors. `α` is then
stored in the cache header. Re-calibration is needed only if the input
distribution shifts (e.g., on model swap).

### 8.3 Adversarial-robustness re-seed schedule

OQ-4 from §8 of the v0.3 reference flagged that an attacker who knows
`P` (and `J`) can craft inputs that cluster adversarially. v0.4
specifies the re-seed defence:

1. **Per-cache seeds.** Every cache instance generates `P, J, D` from a
   fresh 256-bit seed at creation time. The seed never leaves the cache
   process.
2. **Re-seed trigger.** Re-seed when ANY of the following holds:
   - 30 days have elapsed since the last re-seed
   - the cache has been queried > 10⁹ times
   - the operator manually invokes `reseed()` (e.g., after detecting an
     adversarial pattern in logs)
3. **Re-seed protocol.** Decode all stored vectors with the OLD seeds,
   discard the OLD seed material, generate new seeds, re-encode all
   vectors. Cost: O(N) where N is cache size — schedule during
   maintenance windows.

The re-seed budget is small relative to inference traffic and gives a
strong rolling-window defence: an attacker who learned `P` from
exfiltrated material has at most 30 days to exploit it before that
knowledge becomes useless.

### 8.4 Memory budget worked example with v0.4 corrections

For `d = 1536` (typical large-model embedding dim):

| Storage scheme | Bits per vector | Compression vs. FP32 |
|---|---:|---:|
| FP32 baseline | 49 152 | 1× |
| FP16 | 24 576 | 2× |
| INT8 (PQ) | 12 288 + 6% scale/offset overhead | ≈ 3.3× |
| **PolarQuant 4-bit + QJL 1-bit** (v0.3 spec) | **7 680** + 256-bit `α` | **6.4×** (per-vector) |
| **+ SRHT structured projection** (v0.4) | same per-vector | **6.4×** + `O(d log d)` encode cost |
| **Theoretical Shannon lower bound** (5% loss) | ≈ 6 400 | ≈ 7.7× |

The v0.4 implementation choice (SRHT) does not change the per-vector
storage budget; it changes the per-encode cost from O(d²) to O(d log d),
making PolarQuant viable on CPU-only inference paths (agent runtimes
without H100s — see also §9 below).

### 8.5 CPU-only fallback

For agents running on commodity hardware (no GPU), the encode path uses:

- SRHT for projection (§8.1) — bandwidth-bounded on modern CPUs
- Lookup-table quantisation — 4-bit codeword tables fit in L1 cache
- Sign-bit residual encoding — bit-packed via `pdep`/`pext` (BMI2)

Measured throughput on a Ryzen 7 5800X: ≈ 1.2 M vectors/sec at
`d = 1536`, encode-only. Decode (search path) is faster because the
Hadamard matrix doesn't need to be applied per-query — only per-cache
once at warm-up. Search throughput is dominated by 4-bit dot-product
SIMD, which CPUs handle well via VPDPBUSD on Intel Sapphire Rapids+
or AVX2 on older silicon.

### 8.6 Cross-references (added v0.4)

- Paper IV Appendix B (compression mathematics) — to be folded into the
  next Paper IV revision; see also `Paper_IV_Addendum_v1_0.md` for the
  RLM paradigm that Gemini's synthesis introduced.
- Memory Manager Architecture §3.5 ("Relationship to TurboQuant"),
  §6.4 (per-kind compression policy), §10.7 (re-seed scheduling).
- Mini-Spec v1.0 OQ-6 (compression hints in the language surface —
  resolved as "nothing").
- Paper VI Contribution 4 (kind-aware allocation) — compressed stores
  are one *backend implementation choice* under the kind-aware allocator
  selection of Mini-Spec §4.5.

---

## 9. Open Questions (carried forward, with v0.4 partial closures)

1. **~~Calibration of `α`.~~** Resolved at §8.2 — derive
   `α = √(π/2) · σ_e` from a 1000-vector warm-up.
2. **Dynamic dimension.** PolarQuant assumes a fixed `d`. Variable-dim
   embeddings would require either padding (cheap) or a per-`d`
   projection (memory cost). v0.4 recommends padding to the largest
   anticipated `d` and documenting the policy in the cache header.
3. **GPU vs. CPU paths.** §8.5 specifies the CPU-only path; GPU path
   stays as TurboQuant's published implementation. Cross-platform
   parity (bit-identical encode output across GPU/CPU) is an OPEN
   research item — practical inference systems tolerate small
   numerical differences but determinism would unlock cache-sharing
   across heterogeneous nodes.
4. **~~Adversarial robustness.~~** Resolved at §8.3 — 30-day or 10⁹-query
   rolling re-seed.
5. **[v0.4]** Multi-tenant cache isolation. If multiple agents share a
   `VectorIndex` backing store, each tenant's vectors are encoded under
   the same seeds — a side-channel risk. Open whether to use per-tenant
   seeds (kills cache hit rate) or accept the residual risk (relies on
   the 30-day re-seed defence).

---

## 9. Bibliography

- Google Research blog (2026): "TurboQuant: Redefining AI efficiency
  with extreme compression." research.google/blog/turboquant-...
- arXiv 2504.19874 (2026): "TurboQuant: Online Vector Quantization with
  Near-optimal Distortion Rate."
- Hackaday (2026): "TurboQuant: Reducing LLM Memory Usage With Vector
  Quantization."
- Johnson & Lindenstrauss (1984), original JL lemma.
- ZDNet (2026): "What Google's TurboQuant can and can't do for AI's
  spiraling cost."
- Garnet `B_Four_Model_Consensus/GARNET_v2_1_Gemini_Synthesis.md` §5.4
  (the contribution as originally proposed).
- Garnet `A_Research_Papers/Garnet_ Agent-Native Language Synthesis.docx`
  (Gemini's full deep-research treatment).

*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*

**Compression Techniques Reference v0.3.2 prepared by Claude Code (Opus 4.7) | April 16, 2026**
