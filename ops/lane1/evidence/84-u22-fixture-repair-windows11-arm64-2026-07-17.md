# U-22 — Windows-native canonical fixture bytes

Date: 2026-07-17  
Scope: Lane 1 successor pre-flight only  
Guest: Windows 11 Pro build 26200, ARM64-based UTM machine  
Python: CPython 3.14.5 ARM64 with PyYAML 6.0.3  
Git default: `core.autocrlf=true`

## Finding

Test fixtures written with `Path.write_text(... "\n" ...)` acquired CRLF on
Windows, and temporary Git repositories inherited the machine-wide
`core.autocrlf=true`. Canonical-byte and clean-worktree gates rejected those
fixtures correctly. The gates are not defective.

The recorded RED at successor parent `5f083b807583d6faa8e9b030b700fa6f271b9bc5`
was:

| Suite | Windows result before repair |
|---|---:|
| rolling review v2 | 108/110; 2 failures |
| cross-OS policy manifest | 6/10; 4 failures |
| workflow action integrity | 5/13; 8 failures |
| governance activation ceremony | 4/10; 6 failures |

Source transcript and totals: the pre-flight files
`/private/tmp/battery-branch-windows11-arm64-pyyaml603.log` and
`/private/tmp/battery-branch-windows11-arm64-pyyaml603.json` on the host.

## Bounded repair

Only fixture code changed:

- temporary Git repositories now run `git config core.autocrlf false`
  immediately after `git init`;
- fixture text is encoded explicitly and written with `Path.write_bytes`, so
  every intended newline is the literal byte `0a`;
- no production gate, canonicality predicate, path classification, or
  platform skip changed;
- no CRLF-tolerance path was added.

The Windows acceptance checkout was created with
`git -c core.autocrlf=false clone` so committed canonical inputs remained exact.
The guest-wide Git default remained `core.autocrlf=true`, and every temporary
fixture repo therefore still needed its explicit setup override.

## Focused GREEN

| OS / architecture | rolling review | cross-OS manifest | action integrity | ceremony |
|---|---:|---:|---:|---:|
| Windows 11 / ARM64 UTM | 110/110 | 10/10 | 13/13 | 10/10 |
| Debian 12 / aarch64 UTM | 110/110 | 10/10 | 13/13 | 10/10 |
| macOS / arm64 host | 110/110 | 10/10 | 13/13 | 10/10 |

These VM results are pre-flight only. The replacement PR's x86_64 CI matrix
remains the acceptance arbiter.

