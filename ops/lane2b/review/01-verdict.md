# Lane 2B Review Verdict 01 — raw-byte framing core

reviewer: Claude Code Fable 5 (independent, non-implementer, MacBook Air)
reviewed_head: 0919f7a1b14d6f53a251f3528787b2278331e488
reviewed_tree: 3266d5bd05312368777d57da6fe48e0342b5ad60
request: 01-request.md
swept_at: 2026-07-19T10:29:06Z
machine: MacBook Air, macOS 26.5.1, aarch64-apple-darwin, rust 1.95.0 (rustup), python3 -I
verdict: APPROVE

verified_identity: head/tree/diffstat reproduced: yes
- `git rev-parse 0919f7a1^{tree}` → `3266d5bd05312368777d57da6fe48e0342b5ad60` (matches packet)
- `git diff --stat cede73c0..0919f7a1` → 15 files changed, 1091 insertions(+), 9 deletions(-) (matches packet)
- claimed base `cede73c0` = `git merge-base origin/main <head>` (exact; base is current origin/main)
- head is an ancestor of branch tip `c715a6f2`

differential: python full battery (`for s in scripts/test_*.py; python3 -I`), fresh clone, this machine:
- merge-base cede73c0: 121 pass / 13 fail
- reviewed head 0919f7a1: 121 pass / 13 fail — identical failure set, new-vs-base failures: none
- the 13 failures are pre-existing on main and at least partly environmental here
  (PyYAML 6.0.3 requirement absent, release-asset and launch-readiness singles);
  they reproduce bit-for-bit at the base and are not charged to this delta.
- cargo (functional only): `rustup run 1.95.0 cargo test -p garnet-cli --no-fail-fast`
  at reviewed head → 450 passed, 0 failed on this machine. Packet's specific claims all
  reproduce: raw-byte framing 4/4 (lib mcp_stdio), Tier-1 router 3/3 (lib minimum_shelf),
  existing MCP schema/lifecycle/adversarial 14/14 (mcp_initialize_schema 4,
  mcp_protocol 4, mcp_protocol_adversarial 6).

findings: none blocking.
- N1 (NOTE): `garnet-cli/AGENTS.md` reframes the mcp.rs "honesty fence" (empty
  capabilities / unconditional method-not-found) into "default session keeps them;
  explicit application callback after readiness". This is the lane's declared boundary
  change, is documented, and the executable defaults are preserved (`McpSession::new()`
  → `with_capabilities(json!({}))`; `handle_message` → `|_, _| None` handler → −32601).
  Said explicitly per protocol: the removed doc assertion IS the named feature of this lane.

review questions answered:
1. Lifecycle preservation — YES. Re-read of `handle_request` at the reviewed head:
   id-dedup and REQUEST_ID_LIMIT run before any dispatch; the application handler is
   reachable only in `Phase::Ready` (AwaitInitialize → close −32002; AwaitInitialized
   non-ping → −32002); default capabilities remain `{}`; default handler preserves −32601.
   14/14 pre-existing lifecycle/adversarial tests pass unmodified.
2. Framing fails closed — YES, verified by code read and by running the traps here:
   lone LF and lone CR rejected byte-by-byte; any CR/LF inside the header field forces
   "exactly one Content-Length header"; exact-case `Content-Length: ` prefix; canonical
   digits (no leading zero, ASCII only); 8 KiB header / 1 MiB body caps; `read_exact`
   for bodies (truncation → error); invalid UTF-8 rejected after byte framing; single
   −32700 frame then session termination. Pure `Read`/`Write` byte I/O — no line/text readers.
3. Application surface frozen — YES. `tools/list` + `tools/call` only, tool name exactly
   `garnet.core.double`, params exactly `{name, arguments}`, arguments exactly one key
   `value` as i64 (u64 overflow rejected), interpreter invoked in-process under
   `panic_firewall`, return type must be Int.
4. Tool code before readiness / seal after execution — NO path found. The handler is
   unreachable outside `Phase::Ready`. At this head there is no public process entry at
   all (no `bin/garnet.rs` or `cmd/` change in the diff), matching the packet's statement
   that `--test mcp_stdio` process-level gating is deferred until the sealed package is
   the only constructor.
5. Blockers before the sealed-package/negative-proof slices — none arising from this
   head's content. (A sealing-step portability blocker was found at the Request 02 head;
   see 02-verdict.md F1. It does not originate in this framing core.)

scope: clean. Files touched: `garnet-cli/{AGENTS.md, src/lib.rs, src/mcp.rs,
src/mcp_stdio.rs, src/minimum_shelf.rs}` + `ops/lane2b/**` — all inside the declared
Lane 2B scope. No `.github/workflows`, no gate/reporter logic, no Cargo.lock or sealed
build inputs touched (SA-4 not triggered).
weakening: none in executable checks. All 9 deletions accounted for: 5 are the mcp.rs
refactor lines shown above (behavior-preserving), 4 are the AGENTS.md doc reframe (N1).
provenance: intact — no sealed inputs changed at this head.
evidence: RED-before-implement present and coherent for slice 1
(`02-slice1-red.txt`, base_head 80a93d1d, compile-boundary RED) and slice 2
(`04-slice2-red.txt`, base_head aae301d3). `ops/**/evidence/**` resolves `-text`
via `git check-attr` (verified: `text: unset` on the new lane2b evidence files).

not_verified:
- Windows-native behavior (`_setmode` O_BINARY binary-mode stdio, CRLF-checkout
  autocrlf interactions): machine unsuitable (macOS/arm64). Code reviewed; constants
  and fd usage look correct for the MSVC CRT; not executed.
- timing / wall-clock claims: none made in this packet; machine unsuitable regardless.
