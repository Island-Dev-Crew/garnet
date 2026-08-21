# WV-6 terminal freeze 6 ceremony record

## Seat and frozen boundary

- Seat: OpenAI Codex on the native-Windows NUC.
- Checkout: `C:\garnet-freeze6-20260821-a7fbf88b`.
- Cold-start process count: `348`; pre-custody counts remained below the
  directing baseline.
- OneDrive was user-declared paused; the clone and raw capture root are outside
  OneDrive.
- `core.autocrlf=false`.
- Commit identity: `Jon Isaac <Navigata1@gmail.com>`.
- Main base: `1d765cdb2e69bc097cd33db30f9919ad8e969208` / tree
  `a7de550d8f853c6a50d69f87d3fb6d5919cfef29`.
- Frozen A3 boundary: `ba9fa6fe3b3581541cd66ef36334a7235e8e699e`.
- Frozen tree: `e78afada66baad26a20cc967e7141bbbd57ae084`.
- Natively computed pair:
  `449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee / 1643`.

The reporter and an independent implementation over
`git --no-replace-objects ls-tree -r -z` produced the same pair. The containing
commit is the commit that first introduces this record; this file does not
embed its future commit SHA.

## Occasioning changes

### A1 — additive ruleset evolution

Commit `b955f997f7f47868502aaa5c4ac38468a8b7e45a` is the clean carry of
`7b01726fd1ab9ca9194c9203b9d6d418d9d922b1`. It binds the live additive
`dismissal_restriction {enabled: false, allowed_actors: []}` and
`required_reviewers: []` fields without weakening `_strict_equal` or the
governance posture.

### A2 — external registry yank and reversal

Commit `415667a39e2e49f981e9848b21ab59f6a3b0044e` cleanly carries the exact
`arrayref@0.3.9` cargo-deny exception; commit
`7ddb5911095fbbb631545294b89a5f3b4aada869` widens only its documented
removal condition. The exception must be removed when the `blake3` requirement
moves or when the registry yank is reversed. The reviewing seat observed that
reversal upstream on 2026-08-20, so the exception is presently inert and is
retained as a timestamped, exact-version, non-widening allowance. Global
`yanked = "deny"` and `Cargo.lock` remain unchanged.

### A3 — floating-toolchain lint activation and pin cure

Rust/Clippy 1.98.0 activated `chunks_exact_to_as_chunks` under `-D warnings`.
Commit `096626fdafe390c3b112fce49337cb660ea434ac` adopts the compiler's
`as_chunks::<2>().0` form after the existing odd-length guard, pins the
existing Clippy workflow action input to exact Rust `1.98.0`, and moves the
producer-derived required-context semantic and binding digests. A root
`rust-toolchain.toml` was not added because the checked-in MSRV contract owns
that repository-wide surface; the existing workflow action input is the
canonical Clippy toolchain surface.

The source change is in the browser-package producer's input census. The
checked-in producer rebuilt twice and reported `reproducible: true`; the JS and
Wasm artifact hashes stayed byte-identical while provenance moved at commit
`52bd88b82e29be6878e0995069ff4bb14b26ba30`. The strict Playwright journey
passed in 2280 ms over six committed requests, and its refreshed proof is the
frozen A3 boundary `ba9fa6fe3b3581541cd66ef36334a7235e8e699e`.

Final Clippy 1.98.0 capture, exit 0, SHA-256
`7eb9ce338994f5f25ab807d29c862e5b5fa6f1cf9cfec6685c5c77152bfc1d59`:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.50s
```

## Local verification

- Rust 1.98.0 workspace: 2192 passed, zero failed, 6 ignored; capture SHA-256
  `e46e21c512682bdad26c96efb859332c1f18d5f61a2a17c39c12fa5c5fe41435`.
- `garnet-memory`: 121 passed, zero failed, 4 stress tests ignored.
- Governance 33/33; identity 6/6; transport 24/24; base-controlled 4/4;
  activation 10/10.
- Required-context contract 14 passed with 2 skipped; evaluator 13/13.
- Wasm readiness 13/13 and live gate green with no blockers.
- MSRV 25/25 and gate green; action integrity 12 passed with 1 skipped and
  gate green.
- Agent contracts 24 plus 6/6.
- `cargo fmt --all -- --check` exited 0.
- Native WV predicates: 3/3, 2/2, 1/1, and 6/6.

## Supersession with preservation

State: `SUPERSEDED-WITH-PRESERVATION`.

- Superseded reviewed head:
  `218047425fd6871d6cb3ad526ef77e3f4df4c669`.
- Superseded reviewed tree:
  `9cbd7be6810f1f2852d4908fecd64cd66f75fa9c`.
- Superseded pair:
  `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
- Preserved manifest SHA-256:
  `e4f5ebdf9f3936765bf20070837f7953538d6a60024da03f755cc87e8c792579`.
- Complete preserved bundle:
  `proofs/windows/launch-verification/wv6-terminal-freeze-20260821/superseded-218047425fd6871d6cb3ad526ef77e3f4df4c669/`.
- Raw moved predecessor copy:
  `C:\garnet-freeze6-capture-20260821-a7fbf88b\superseded-live-working-copy-218047425fd6871d6cb3ad526ef77e3f4df4c669`.

| Preserved file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `a595cd0895a05427316fee734812adfc20e8b429f1b72a9ecb62401b651268a9` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `e4f5ebdf9f3936765bf20070837f7953538d6a60024da03f755cc87e8c792579` |

The live six-file bundle was moved intact to the raw capture root only after
the producer-censused copy matched byte-for-byte. The producer then emitted
once into the vacant live destination.

## Fresh live bundle

| Live file | Bytes | SHA-256 |
|---|---:|---|
| `f1-canonical-reseal.txt` | 2801 | `02a948c903fec3f02a79f831f9e086a7ed11d8bb48976ed09ef512971fa0a6a7` |
| `mcp-session.input.hex` | 1073 | `64b2f3e15489b4f06e57a0c51afce960beda3edc9c8009d5f3a530d5d9f92638` |
| `mcp-session.output.hex` | 1797 | `15423fa5ba9697f59f877ea0a9afb3587230fc4d7c3182cee82ce709bbce4799` |
| `minimum-shelf-status.json` | 2070 | `14fe6fbb2a9738c0ec57fe3ff9f38ceef6b7cdf28df70f347bf8fd816e389600` |
| `reporter-cross-checkout.txt` | 1660 | `a4f33e467349bf84a6c9fe04e7ddd486df335a96f2edda0b2cb371787d26425f` |
| `WV_ACCEPTANCE.json` | 1978 | `3e6fa03be73c6b467d882153d9de48cd766df32c86e61d4414905cf053cee0ad` |

## Acceptance succession

1. `2cb25d0b47f55c9dd987bf69fc8a97dade5a4767ef0beda0abc8616808dddc0c / 1605`.
2. `fd96e6d910180f5e33999fbd693ea211e336389a13535930d89b2a870ff54727 / 1606`.
3. `1b45387599223adbe8766ae2e04ddc70e4ecc359712a4867ef1973dac9bd42e4 / 1629`.
4. `8ea996129074e2e70c5ad2f9abd6082c85f7021e943dc017d3a9ba2859ed3ac7 / 1634`.
5. `32f3975537470cb1788acfb3365725c4fea208a049e20fb90b1c314b719cba06 / 1637`.
6. `056a153920a9147b5f703e482c8d8b5098347ebd0a3f64d36a1d2c9247b13edc / 1640`.
7. `449ba9b7aa948cb6fe5e1320385025ea563bc9e5c5ac69cc5ae6b6670bb2a9ee / 1643`.

All predecessors remain intact.

## Superseded never-merged boundaries

- PR #525 is closed with no merge commit. Its A1-only boundary pair
  `0513edb99391ad4dfe75dffe3618c8c001f877aa4d4f738709c4b506ec06b425 / 1643`
  and record tip `b31f273022d4e2b411f9650b5123543e7accfb41` never reached main and remain
  outside acceptance succession.
- PR #526 had no merge commit at this ceremony's read. Its A1+A2 boundary pair
  `573523661f3569d925a05f4c95549582d765212c6ef650871432d35c00c12675 / 1643`
  and record tip `a88813360f1d550a4d209a4cea441fdb9cba1bd6` never reached main and remain
  outside acceptance succession. Jon owns closing that superseded PR after
  this successor opens.

## Head-versus-tip boundary

The pin paths, fresh bundle, preservation copy, and these three records are
the established post-acceptance record class. The accepted pair remains bound
to A3 head `ba9fa6fe3b3581541cd66ef36334a7235e8e699e`. No structured review record,
approval, merge, tag, release, carrier readback, or token action is part of
this ceremony.
