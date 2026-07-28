# Lane 1 · Phase 0 — Review Request 09 (U-31 cure PROPOSAL: normalize `08.source`)

- Date: 2026-07-28 (UTC ~03:20Z)
- Implementer: Claude Code Opus 5 (`claude-opus-5`) — Jon's macOS seat
  (macOS 26.5 / Darwin 25.5.0 / arm64), fresh-fetched `autocrlf=false` clone,
  fleet-fork identity Jon Isaac `<Navigata1@gmail.com>`
- Independent reviewer sought: Codex GPT-5.6 Sol — cross-family verdict of record
- Merge authority: Jon (IslandDevCrew) only; review carrier: IDC-Trust-Review only;
  the implementer is neither
- Packet head: this request sits directly above RED commit `5a06c29`
  (`ops/lane1/evidence/93-u31-machine-path-red.md`), on tip lineage
  `f3876c5` (slice-4 WV-6 rebind) ← `a1a0c41` ← `173e822` (verdict 07) …
- **NOTHING IS IMPLEMENTED.** This is a gate-logic-adjacent trust-kernel change
  (`scripts/garnet_*`, freeze ACTIVE); per ceremony it requires a ruling before
  any cure lands. RED is recorded first; the cure below is a proposal.

## Current state (verified this session)

| field | value |
|-------|-------|
| HEAD | `f3876c5a78beb31d6cfe8cc5a115bf264af8008f` (+ RED `5a06c29` + this request) |
| tree at `f3876c5` | `006f06f2b3af451d292116de5e59a14735d4ec6b` |
| product_content_sha256 | `e89cb2996def7eec8e419dac235d55f985734e84be6f991c565da35d46feb64f` |
| product path count | 1544 |
| truth floor on main | lane0 closeout PASS (22/22, ledger 37) · MSRV PASS (1.95, 16/16) · frozen backlog PASS (ok:true) · rolling-review PASS (exit 0) |
| WV-6 | `accepted`, 5/5, bound to `e89cb299…/1544` at reviewed head `72ae0246` (slice 4 DONE) |

`ops/lane1/**` is digest-excluded (U-35, verdict 08), so the RED commit and
this request move no product byte — recomputed after each commit, pair
unchanged.

## 1. The finding as registered

- **Verdict 01 F4** (U-31 candidate): two same-commit checkouts produced
  different `08-launch-readiness.json` bytes; sole difference `source`
  (absolute checkout path). Blocking prerequisite for slice 5.
- **Verdict 02 F5**: reproduced; `equal_without_source=true`.
- **BLOCKED.md item 3**: slice 5 (reporter/SOTU/denominator refresh) "blocked
  by U-31 … which must be cured first via its own reviewed change."

Cause, single line — `scripts/garnet_launch_readiness_status.py:509`:

```python
source=str(Path(__file__).resolve()),
```

`ops/lane0/evidence/08-launch-readiness.json` is digest-INCLUDED (`ops/lane0/`
is not in `FROZEN_MUTABLE_PREFIXES`, 31 tracked paths). Regenerating the
reporter therefore stamps seat-specific bytes into the certified tree.

## 2. RED evidence (committed first, `5a06c29`)

Full record: `ops/lane1/evidence/93-u31-machine-path-red.md`. Load-bearing facts:

1. **Committed today:** `08.source` =
   `/private/tmp/garnet-l0-truth-freeze-231aefa/scripts/garnet_launch_readiness_status.py`
   — the truth-freeze seat's absolute path, already in the certified tree.
2. **This seat would commit:** `/Users/IDC2.5/garnet-lane1-fresh/scripts/…`
   (clone A, sha256 `3f902587…`).
3. **A second checkout of the same commit on the same machine** (clone B,
   different path) yields `<scratch>/garnet-B/scripts/…`
   (sha256 `2440a196…`). `diff` = exactly one hunk (`source`);
   `equal_without_source: True`.
4. **Digest poison, quantified** (independent replay reproduces the pinned
   baseline `e89cb299…/1544` exactly, then substitutes only the 08 blob):
   - seat A commits its regeneration → `824e1e8f…/1544`
   - seat B commits its regeneration → `b232031b…/1544`
   Two seats performing the identical lawful act certify **different trees**.
5. **Axis separation:** committed-vs-regenerated differs in exactly two hunks:
   `source` (U-31) and `live_wasm_playground.blockers` `[3]→[]` (real
   dependency drift since freeze base `231aefa`; state stays `remaining`).
   The drift is lawful slice-5 content, not part of U-31.

## 3. Is `08.source` consumed, or only emitted?

Determination performed exactly as for the `REVIEWED_TREE_PRODUCT` pair —
sweep every reader, classify each:

| reader | what it requires of 08 | reads `source`? |
|---|---|---|
| `scripts/garnet_lane0_closeout_status.py` (sole file consumer, `_require_fields` at ~1607) | `schema == garnet.launch_readiness/v1`, `recommendation == HOLD`, `launch_ready == false`, exact gate (id,state) tuple, derivation `3/6` and `3/8` | **NO** |
| reporter `--gate` mode | never reads the committed artifact at all (gates on freshly computed `launch_ready` only) | NO |
| `.github/**` (CI) | zero references to the artifact or reporter | NO |
| `scripts/test_garnet_launch_readiness_status.py:59` | `status.source.endswith("garnet_launch_readiness_status.py")` — on the in-memory object, suffix only | suffix only |

Contrast, same closeout file: for `09-mit-readiness.json` it **requires**
`source == "committed-truth"` — the repo already distinguishes a load-bearing
source (09) from an emitted-only source (08).

**Ruling requested on this determination: `08.source` is EMITTED, not
gate-consumed. It is not load-bearing.** The only reader anywhere is a
suffix-only unit assertion that survives any cure that keeps the script name
in the value.

## 4. Proposed cure — options and recommendation

### Option A (RECOMMENDED): emit the repository-relative POSIX path

```python
# scripts/garnet_launch_readiness_status.py:509 — the only behavior line touched
source=Path(__file__).resolve().relative_to(REPO_ROOT).as_posix(),
```

yielding the constant value `scripts/garnet_launch_readiness_status.py`.

Argued from the reporter's own semantics, not convenience:

1. The field names the **producing program**. Inside a certified tree a
   program's identity IS its repo-relative path; the absolute prefix names the
   *seat*, and the seat is nowhere else part of this reporter's semantics.
2. The reporter already made machine-independence an explicit design
   decision: the promo-snapshot pin comment says its purpose is that
   "regeneration is machine-independent", and `validate_evidence_base`
   rejects `-dirty` values for the same reason. Line 509 is the last
   seat-dependent byte, contradicting the reporter's own stated intent.
3. The sole existing consumer (unit suffix assertion) stays green unchanged.
4. `.as_posix()` guarantees byte-identity from the Windows NUC seat —
   `str()` of a relative `Path` on Windows would emit backslashes and
   re-introduce a seat-dependent byte, so this is load-bearing for this lane.
5. In-repo precedent: S31-PR2 normalized the MIT reporter's `source` for
   exactly this disease ("Emit only committed-truth lanes and normalize the
   source field").

### Option B: omit the field

Deterministic, but silently changes the `garnet.launch_readiness/v1` shape
without a schema bump, breaks the unit test, and erases producer provenance
that costs nothing to keep. Weaker on the reporter's own terms — the schema
promised a source; deleting it is a semantics change dressed as a cleanup.

### Option C: stable producer identifier (e.g. `"committed-truth"`)

The 09 precedent's literal does not transplant: for the MIT reporter,
`source` classifies **data provenance** (committed-truth lanes vs
machine-local evidence lanes). 08's payload is entirely reporter-derived at
run time — labeling it `committed-truth` would be false, and any other
invented token is just Option A with the truthful token (the script path)
replaced by an arbitrary one. Collapses into Option A.

### Proposed cure slice (for the ruling; NOT implemented)

| # | surface | change |
|---|---|---|
| 1 | `scripts/garnet_launch_readiness_status.py` | line 509 only, per Option A |
| 2 | `scripts/test_garnet_launch_readiness_status.py` | +1 focused trap: `source` equals exactly `scripts/garnet_launch_readiness_status.py`, is not absolute, contains no backslash (keeps the existing suffix assertion) |
| 3 | `ops/lane1/**` | cure RED-turns-green record, request/verdict traffic |

**Explicitly NOT proposed:** regenerating `08-launch-readiness.json` in the
cure slice (that is slice 5's act, lawful only after this cure lands);
touching `FROZEN_MUTABLE_PREFIXES` (see trap d); touching the closeout, the
freeze reporter, schema id, any other field, or the sibling emitters (§7).

## 5. Mandatory traps the cure must pass (pre-agreed here)

- **(a) Seat-independence:** regenerate from two different clone paths on the
  same machine at the same commit → byte-identical artifact. Simulated in the
  RED: normalizing `source` alone already converges both seats to sha256
  `e44eb5b2…`, blob `44cae251…`. The cure slice must reproduce this live with
  the real emitter, and SHOULD additionally record one Windows-seat
  regeneration hash before slice 5 relies on it (the `.as_posix()` clause).
- **(b) Real change still moves the artifact:** a genuine readiness change
  must still change bytes. Unit form: `build_status` over mutated
  `Dependencies` yields different bytes. Live form already observed: the
  `live_wasm_playground.blockers` drift moves the artifact independently of
  `source`.
- **(c) No other field's semantics change:** full-JSON diff of pre-cure vs
  post-cure regeneration on the same seat = exactly one changed line
  (`source`). Simulation confirms A-cured differs from A at line 3 only.
  Schema id stays `garnet.launch_readiness/v1`; human/markdown renders never
  printed `source` and must remain byte-identical.
- **(d) Digest determinism with `ops/lane0/` INCLUDED:** recompute the pair
  after cured regeneration from two clone paths → one value. At this tip with
  today's dependency truth that value is `3aa7ecc6…/1544` (determinism
  demonstration, NOT a slice-5 landed pin — slice 5 adds its own reviewed
  content). `ops/lane0/` must remain in the included set (31 paths) and
  `FROZEN_MUTABLE_PREFIXES` must remain exactly the four authorized literals —
  the existing tuple-exactness trap already fails any extension. Curing the
  reporter, not the gate, is the entire point: extending the exclusion to
  `ops/lane0/` would be curing a gate to unblock ourselves and is DENIED in
  advance by this packet.

## 6. Slice-5 consequence — exact denominator statement

- **Today's producer, regenerated at this tip:** gate tuple unchanged →
  derivations remain **3/6 accepted (50.0%) launch-critical** and **3/8
  (37.5%) ledger**. The closeout pins exactly these (`launch_critical != 3 or
  launch_ledger != 3` → finding; `EXPECTED_LAUNCH_GATES` exact-tuple check).
  The U-31 cure + regeneration alone changes NO denominator and stays
  closeout-green (bytes change: `source` + the blockers drift; states do not).
- **The ruled landed values:** the denominator ruling of record — verdict 01,
  "supported reconciled values are `4/6 = 66.7%` and `4/8 = 50.0%` … Slice 5
  must publish `66.7% / 50.0%` unless another gate is independently closed"
  (the tasking attributes this to verdict-04; verdict 04 contains no
  denominator text — the ruling lives in `01-verdict.md`, reaffirmed nowhere
  else; recorded here as found).
- **Falsified-expectation report (not reconciled):** today's producer CANNOT
  yield 4/6 by regeneration. `minimum_sealed_shelf` is hardcoded
  `state="manual-deferred"` ("never reporter-derived machine truth") and the
  readiness reporter consumes no WV acceptance state at all — even though
  WV-6 is now `accepted` 5/5 bound to `e89cb299…/1544`. The fourth accepted
  gate can only arrive via a slice-5 change that makes the shelf gate reflect
  the recorded WV-6 acceptance (an S114-style recorded-acceptance read is the
  in-repo pattern), plus the matching closeout expectation updates
  (`EXPECTED_LAUNCH_GATES` shelf state; `3/6→4/6`, `3/8→4/8`; the WV-6
  pending-evidence expectations). Those are reporter/gate semantics changes
  belonging to slice 5's own review — NOT smuggled into this cure. If anyone
  publishes 66.7%/50.0% from a regeneration under today's producer, that
  number is unsupported: the producer yields 50.0%/37.5%.

## 7. Sibling observations (registrar notes; OUT of this cure's scope)

- `garnet_converter_advisory_handoff.py:158` and
  `garnet_converter_advisory_review.py:152` emit
  `source=str(Path(__file__).resolve().parents[1])` (absolute repo root) —
  same pattern, but they write only to caller-supplied `--output-dir`; no
  committed artifact in the certified tree carries their output today. Flag
  for the frozen backlog, no action here.
- `09-mit-readiness.json` is already cured (S31-PR2) and its `source` IS
  load-bearing (`committed-truth`), enforced by the closeout.

## STOP

Per ceremony this implementer now stops. Requested from the reviewer:

1. a ruling on the §3 consumed-vs-emitted determination;
2. a ruling on the cure option (§4; Option A recommended) and the exact
   authorized diff surface;
3. confirmation or amendment of the §5 traps, including the Windows-seat
   hash requirement;
4. acknowledgment of the §6 falsified-expectation report so slice 5's scope
   is sized with the shelf-gate reflection in view.

No cure lands before the ruling. The NUC/slice-4 state is untouched; the
product pair after this packet remains `e89cb299…/1544`.
