# Lane 2C Memcheck Evidence Header

- Blocker: Verdict 01 B1, missing leak-disposition evidence.
- Implementer: OpenAI Codex, GPT-5-based model; exact version not exposed.
- Machine: `NUCBOX_M2PRO_S`, Ubuntu WSL2 x86_64 on `/dev/sdd` ext4.
- Tool: Valgrind Memcheck 3.22.0.
- Product boundary: base `efd4f6bae8b3afaba74594e57944b2548142aeae`;
  product `5cd113617acd35307bb028463833a8da2bbd6ad2`, tree
  `85faad1de5a2c47cb632bedea78dfb89d209001a`.
- Quiet state: irrelevant to deterministic leak accounting. No quiet-state
  ritual was performed, no service was stopped for these captures, and no
  quiet window is claimed.
- Base binary provenance: reused original artifact after SHA-256 matched
  `4577447bdfba5163467c48fc59d6444688a094c52df7a9360ffbeaa9f3f00a72`.
- Product binary provenance: reused original artifact after SHA-256 matched
  `0ca1e4e38471ba34ffe51274216a6de144910fb5d0c791a40be7a012bcdb9810`.

Capture command:

```sh
valgrind --tool=memcheck --leak-check=full --show-leak-kinds=all \
  --error-exitcode=99 BINARY CASE 1024
```

The exact loop and binary bindings are in `ops/lane2c/replay_memcheck.sh`.

## Six captures

| Phase | Case | Captured UTC | Definitely lost | Indirectly lost | Possibly lost | Still reachable | Errors |
|---|---|---|---:|---:|---:|---:|---:|
| Before | working-clear | 2026-07-31T06:21:55.9146875Z | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | 0 |
| Before | episodic-drop | 2026-07-31T06:21:58.3440312Z | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | 0 |
| Before | semantic-drop | 2026-07-31T06:22:00.4023913Z | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | 0 |
| After | working-clear | 2026-07-31T06:22:04.5416470Z | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | 0 |
| After | episodic-drop | 2026-07-31T06:22:05.2273137Z | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | 0 |
| After | semantic-drop | 2026-07-31T06:22:05.8433136Z | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 544 B / 1 block | 0 |

## Before-to-after delta

| Case | Definitely lost | Indirectly lost | Possibly lost | Still reachable |
|---|---:|---:|---:|---:|
| working-clear | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks |
| episodic-drop | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks |
| semantic-drop | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks | 0 B / 0 blocks |

The product retains no more exit-time memory than the base in any reported
leak category. This record establishes disposition parity for the three B1
cases; it does not extend the reviewed product claim boundary.
