# SECURITY.md truth cure — cross-family confirmation record (2026-09-02)

- Branch `mission/security-md-truth-2026-09-02` · base main `fbd64bc514b573a6735b3525f4f3172be14a27d0`
- Lineage: v1 content tip `9120b7d43af9aa2ae476146d1ef3fb71c62a87c5` → Codex **REJECT** (one blocking finding: the `@sandbox` in-scope bullet claimed a declared-not-enforced fence as a vulnerability class) → v2 content tip `73094364bf9f16eac1308fffa5ad685a57eef301` (bullet limited to the applied Linux seccomp boundary; macOS/Windows named-deferred paragraph) → Codex **CONFIRM**, no findings.
- Implementing seat: Claude Fable 5.1 (via a subagent, evidence table in the PR body). Reviewing seat: Codex (codex-cli 0.147.0 via the local wrapper, cross-family, read-only, detached worktree at each tip; L-15 satisfied).
- Path class: `SECURITY.md` is not a rolling-gate trust-kernel trigger (gate `ok: true`, `touched_paths []`); this markdown record is the review artifact. Both verdicts are transported verbatim below; 5 occurrence(s) of a retired word in the reviewing seat's prose are elided and marked; finding content is unchanged.
- Verdict of record: **CONFIRM**, bound to `73094364…`; this record commit is the records-class head move it anticipates.

## Reviewing seat output — v2 (verbatim, elisions marked)

```text
Cross-family-review discipline applied: exact-head/clean-tree preflight, source-by-source recomputation, and final void-on-move verification.

```console
$ git rev-parse HEAD
73094364bf9f16eac1308fffa5ad685a57eef301

$ git status --porcelain
```

No stdout; exit 0.

```console
$ git rev-parse fbd64bc514b573a6735b3525f4f3172be14a27d0
fbd64bc514b573a6735b3525f4f3172be14a27d0

$ git rev-parse origin/main
fbd64bc514b573a6735b3525f4f3172be14a27d0

$ git merge-base fbd64bc514b573a6735b3525f4f3172be14a27d0 HEAD
fbd64bc514b573a6735b3525f4f3172be14a27d0

$ git diff fbd64bc5..HEAD --stat
 SECURITY.md | 27 +++++++++++++++------------
 1 file changed, 15 insertions(+), 12 deletions(-)

$ git diff fbd64bc514b573a6735b3525f4f3172be14a27d0..HEAD --name-status
M	SECURITY.md

$ git log fbd64bc514b573a6735b3525f4f3172be14a27d0..HEAD --oneline
7309436 docs(security): align SECURITY.md with v0.8.1 truth (support window, disowned test figure, enforcement scope, signing facts)
```

```console
$ git tag --sort=-v:refname
v0.8.1
v0.8.0
v0.5.0
v0.4.2
parked/s15-promo-docs
parked/s5-fmt-wip

$ gh release list --repo Island-Dev-Crew/garnet --limit 6
v0.8.1	Latest	v0.8.1	2026-06-07T07:55:45Z
v0.8.0		v0.8.0	2026-05-31T20:55:58Z
Garnet v0.5.0		v0.5.0	2026-05-20T14:19:59Z
Garnet v0.4.2		v0.4.2	2026-05-15T23:38:42Z
```

```console
$ git show origin/main:docs/truth.json | grep -A2 security_test_count
    "security_test_count": "No trusted derivation exists for the historical public '136 security tests' figure (it entered the site undocumented). Re-stamping an unverifiable number would automate drift; the public row is removed/replaced by RB-0d instead."
  },
  "primitive_count": 80,

$ grep -inE '([0-9]+[^[:cntrl:]]*tests?|tests?[^[:cntrl:]]*[0-9]+)' SECURITY.md
79:Security-specific tests were added across four historical hardening layers (v3.3 Layer 1 through v4.0 Layer 4). No current count of those tests is published here: `docs/truth.json` (`omissions.security_test_count`) records that no trusted derivation exists for the historical "136 security tests" figure, so this document does not restate it. The original threat model is documented in [GARNET_v3_3_SECURITY_THREAT_MODEL.md](F_Project_Management/GARNET_v3_3_SECURITY_THREAT_MODEL.md) — a roadmap of 15 hardening patterns, two of which address Garnet-specific threat classes (strategy-miner adversarial training, `Box<dyn Any>` hot-reload type confusion).
```

```console
$ sed -n '66,71p' CLAUDE.md
- **"Enforced" only means a deterministic trap proven by test.** Never call a
  generated policy "enforced"; never call the S114 red-team "independent."
- Preserve the named-deferred fences: `@bounded` (Wasmtime fuel), memory, time,
  `@mailbox`, and macOS/Windows OS-sandbox application remain
  declared-not-enforced; only `@caps` + `@max_depth` are enforced (both
  backends), with seccomp applied on **Linux only**.

$ sed -n '35,43p;85,97p' C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md
| Class | What it means | Where | Members |
|-------|---------------|-------|---------|
| **Declared (checker-only)** | Capability required by the checker; **no runtime gate**. Reachable at run time without a trap once the program type-checks. | `Guard::Declared` (`registry.rs`) | `time::now_ms`, `time::wall_clock_ms`, `time::sleep`; `uuid::new_v4`, `uuid::new_v7` (all require `@caps(time)` at check time only) |
| **Runtime-gated** | `require_capability` in the bridge adapter traps at run time when the active caps frame lacks the capability. **12 primitives.** | `Guard::Gate`; `eval.rs` `require_capability` | `fs::read_file` / `write_file` / `read_bytes` / `write_bytes` / `list_dir`; `net::tcp_connect`; `std::env::get` / `set` / `vars`; `std::process::wait` / `exit_code`; `std::log::to_file` |
| **Entry-gated** | `require_capability` **plus** the S92 program-entry-frame check (anti-laundering). **3 primitives.** | `Guard::GateEntry`; `eval.rs` `require_entry_capability` | `std::process::spawn` / `spawn_args` / `output` |
| **Declared-only, no bridge** | In the checker vocabulary and/or sandbox-policy mapping, but **no runtime enforcement path exists**. | — | `ffi` (checker + manifest + sandbox-policy warning only); `net_internal` (checker vocab + loopback-only in generated sandbox policy; `tcp_connect` always uses strict `NetPolicy::default()`) |
| **Unbridged** | Registry row exists for the CapCaps propagator only; **no interpreter binding at all**. | `Binding::Unbridged` (`registry.rs`) | `net::tcp_listen`, `net::udp_bind` |
| **OS-sandboxed (generated, not self-enforced)** | `garnet sandbox` generates seccomp / WASI / egress policy from aggregate `@caps`. The generator emits `enforced: false`; the policy was applied and trapped on a real **Linux** kernel via an external C reference harness (`tools/seccomp-apply`). macOS / Windows OS-sandbox application is **named-deferred**. | `GARNET_SANDBOX_POLICY.md`, `GARNET_SECCOMP_APPLY.md` | all `@caps` → policy |
| **Caps-invisible** | Host-visible natives with **no capability row at all**. Any "all authority is capability-tagged" claim is false until these earn rows. | `BRIDGE_ONLY` const (`stdlib_bridge.rs`) | `memory::working` / `episodic` / `semantic` / `procedural` |
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
```

```console
$ sed -n '49,61p' SECURITY.md
**In scope:**

- Capability escape — code with `@caps()` successfully invoking a runtime-gated primitive whose capability it lacks (e.g. `fs::read_file`, which requires `@caps(fs)`); the runtime-gated surface is the one listed in [GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md](C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md)
- Manifest signature forgery — a `garnet verify --signature` accepting a tampered signed manifest
- Hot-reload replay — a ReloadKey-signed reload from a stale sequence number being accepted
- StateCert type confusion — a hot-reload surviving a type mismatch via BLAKE3 fingerprint collision or bypass
- Compiler impersonation — a malicious compiler producing a manifest that verifies against a legitimate release pubkey
- Strategy-miner poisoning — adversarial training-time injection into the knowledge graph that survives to runtime
- Path traversal or escape through an OS-sandbox boundary that is actually applied — today that is the Linux seccomp boundary of the reference harness (`garnet sandbox` generates policy; it does not self-enforce it)
- Remote code execution via any network primitive
- Any confidentiality / integrity / availability breach that bypasses a claim from Papers III/V/VI or the v3.4 Security V2 spec

**Declared, not yet enforced** (reports welcome as roadmap findings, not vulnerabilities): `@bounded` (Wasmtime fuel), memory, time, `@mailbox`, and macOS/Windows OS-sandbox application. The repo's operating brief (`CLAUDE.md`) fixes the line: "`@bounded` (Wasmtime fuel), memory, time, `@mailbox`, and macOS/Windows OS-sandbox application remain declared-not-enforced; only `@caps` + `@max_depth` are enforced (both backends), with seccomp applied on **Linux only**." The enforcement scope table says the same of OS sandboxing — "macOS / Windows OS-sandbox application is **named-deferred**" — and limits the runtime claim to "`@caps` and `@max_depth` trap identically on both backends for the gated surface" ([GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md](C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md)). An `Actor::tell` accepting unbounded messages despite `@mailbox(N)` is therefore expected behavior today, not a bypass; file it as a public issue labeled as a roadmap finding. The same applies to `@sandbox` policy on macOS and Windows, where OS-sandbox application is **named-deferred** in the enforcement scope document: an escape there is a roadmap finding, not a vulnerability, until the boundary is applied.
```

```console
$ awk '/^name = "(ed25519-dalek|blake3|sha2)"$/ { print; getline; print }' Cargo.lock
name = "blake3"
version = "1.8.4"
name = "ed25519-dalek"
version = "2.2.0"
name = "sha2"
version = "0.10.9"
```

```console
$ rg -n -C 1 "tags: \['v\*'\]|^  build-packages:|^  macos-cli-tarballs:|^  release:|garnet-sbom-cyclonedx\.tgz|sha256sum \*\.deb \*\.rpm \*\.tar\.gz \*\.tgz|HAS_GPG|ALLOW_UNSIGNED_RELEASE|SHA256SUMS\.asc|release-dist/\*\.deb|release-dist/\*\.rpm|release-dist/\*\.tar\.gz|release-dist/SHA256SUMS$" .github/workflows/linux-packages.yml
19-    branches: [main]
20:    tags: ['v*']
21-  pull_request:
--
36-  # ───────────────────────────────────────────────────────────────────
37:  build-packages:
38-    runs-on: ubuntu-latest
--
100-        #   gpg --batch --yes --detach-sign --armor SHA256SUMS
101:        # which produces SHA256SUMS.asc. Then add it to the upload
102-        # artifact list and the release-asset publish step below.
--
216-  # ───────────────────────────────────────────────────────────────────
217:  macos-cli-tarballs:
218-    runs-on: macos-latest
--
325-  # ───────────────────────────────────────────────────────────────────
326:  release:
327-    if: startsWith(github.ref, 'refs/tags/v')
--
334-      # release ships an UNSIGNED SHA256SUMS (the [retired word elided by the transporting seat] research-grade default).
335:      HAS_GPG: ${{ secrets.GPG_SIGNING_KEY != '' }}
336-    steps:
--
365-          # .tgz extension so the *.tar.gz binary glob never picks up the SBOM.
366:          tar -czf release-dist/garnet-sbom-cyclonedx.tgz -C sbom-tmp .
367-          echo "SBOM files bundled:"; ls -1 sbom-tmp
--
372-          rm -f SHA256SUMS
373:          sha256sum *.deb *.rpm *.tar.gz *.tgz > SHA256SUMS
374-          cat SHA256SUMS
--
376-      - name: Sign SHA256SUMS (activates when GPG_SIGNING_KEY secret is set)
377:        if: env.HAS_GPG == 'true'
378-        env:
--
389-          fi
390:          echo "signed → SHA256SUMS.asc"
391-
--
394-      # unsigned tagged release is BLOCKED unless it is a deliberate, recorded act
395:      # (repo variable ALLOW_UNSIGNED_RELEASE=true), so shipping unsigned is a
396-      # decision, never an accident. [Jon-gated: this is a release-POLICY change.]
397-      - name: Require signed SHA256SUMS (fail-closed)
398:        if: env.HAS_GPG != 'true'
399-        run: |
400:          if [ "${{ vars.ALLOW_UNSIGNED_RELEASE }}" = "true" ]; then
401:            echo "::warning::GPG_SIGNING_KEY absent and ALLOW_UNSIGNED_RELEASE=true — publishing a DELIBERATELY UNSIGNED research-grade release."
402-          else
403:            echo "::error::Tagged release requires a signed SHA256SUMS, but GPG_SIGNING_KEY is not set. Provide GPG_SIGNING_KEY (+ optional GPG_PASSPHRASE), or set repository variable ALLOW_UNSIGNED_RELEASE=true to deliberately ship unsigned."
404-            exit 1
--
410-          files: |
411:            release-dist/*.deb
412:            release-dist/*.rpm
413:            release-dist/*.tar.gz
414:            release-dist/garnet-sbom-cyclonedx.tgz
415:            release-dist/SHA256SUMS
416-          fail_on_unmatched_files: true
--
419-      - name: Attach SHA256SUMS signature (only when signed)
420:        if: env.HAS_GPG == 'true'
421-        uses: softprops/action-gh-release@3bb12739c298aeb8a4eeaf626c5b8d85266b0e65
422-        with:
423:          files: release-dist/SHA256SUMS.asc
424-          fail_on_unmatched_files: true
```

```console
$ rg -n "SHA256SUMS|verify_sha256|SHA256SUMS\.asc|gpg" docs/install.sh
8:# package from GitHub Releases, verifies it against SHA256SUMS, installs it, and
178:verify_sha256() {
202:    _sums_url="${GARNET_CHECKSUM_URL:-${GARNET_BASE_URL}/SHA256SUMS}"
354:    say "fetching SHA256SUMS"
359:    verify_sha256 "$_dest" "$_expected_sha"
```

```console
$ gh release view v0.8.1 --json assets
{"assets":[{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440774492","contentType":"application/octet-stream","createdAt":"2026-06-07T07:55:44Z","digest":"sha256:16d31c507301fbef595971e8b12fd9791b7d150f22fad5839001bf0b5965fa75","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRa9c","label":"","name":"garnet-0.7.0-lsp-mvp-darwin-arm64.vsix","size":1863752,"state":"uploaded","updatedAt":"2026-06-07T07:55:44Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-0.7.0-lsp-mvp-darwin-arm64.vsix"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440774491","contentType":"application/octet-stream","createdAt":"2026-06-07T07:55:44Z","digest":"sha256:60f846a1a2013fd6d826939de962268eed31339c5bcab8a5799fed53f1572778","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRa9b","label":"","name":"garnet-0.7.0-lsp-mvp-linux-x64.vsix","size":2030614,"state":"uploaded","updatedAt":"2026-06-07T07:55:44Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-0.7.0-lsp-mvp-linux-x64.vsix"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775812","contentType":"application/x-redhat-package-manager","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:88b4e9cb255409411a595e38eb086943735a70e824d968548460ff69451ea09d","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRbSE","label":"","name":"garnet-0.8.1-1.x86_64.rpm","size":2582522,"state":"uploaded","updatedAt":"2026-06-07T07:58:27Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-0.8.1-1.x86_64.rpm"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775814","contentType":"application/gzip","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:b89396384fa201652027e5e8f365499b08efa6e5b3c9c45dcd894b492d748c3d","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRbSG","label":"","name":"garnet-0.8.1-aarch64-apple-darwin.tar.gz","size":2697981,"state":"uploaded","updatedAt":"2026-06-07T07:58:27Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-0.8.1-aarch64-apple-darwin.tar.gz"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775816","contentType":"application/gzip","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:83a474995da6a654f855e45ec246f56b3bb7a933e9fd89827da2a4d5157a21ae","downloadCount":2,"id":"RA_kwDOSJv_Zc4aRbSI","label":"","name":"garnet-0.8.1-x86_64-apple-darwin.tar.gz","size":2999427,"state":"uploaded","updatedAt":"2026-06-07T07:58:27Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-0.8.1-x86_64-apple-darwin.tar.gz"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775817","contentType":"application/octet-stream","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:79c1d2b77635b40580fe39427687fad00913aab7bca6e6d0de5e5611f9e875f7","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRbSJ","label":"","name":"garnet-sbom-cyclonedx.tgz","size":116256,"state":"uploaded","updatedAt":"2026-06-07T07:58:27Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-sbom-cyclonedx.tgz"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775815","contentType":"application/x-debian-package","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:ca35ebf881cc1d16f288f850eb767305c590112a05966c6778e5fa3d2a42e0cc","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRbSH","label":"","name":"garnet_0.8.1-1_amd64.deb","size":2401512,"state":"uploaded","updatedAt":"2026-06-07T07:58:28Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet_0.8.1-1_amd64.deb"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775813","contentType":"application/octet-stream","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:00a1ccc254cd6ea6c1f9e875f7","downloadCount":11,"id":"RA_kwDOSJv_Zc4aRbSF","label":"","name":"SHA256SUMS","size":488,"state":"uploaded","updatedAt":"2026-06-07T07:58:27Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/SHA256SUMS"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775847","contentType":"application/pgp-keys","createdAt":"2026-06-07T07:58:29Z","digest":"sha256:d2de154d475fd9f99fe3d5fd8734083e35945ee1f5332be38ad57cc8bb4fd961","downloadCount":6,"id":"RA_kwDOSJv_Zc4aRbSn","label":"","name":"SHA256SUMS.asc","size":228,"state":"uploaded","updatedAt":"2026-06-07T07:58:29Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/SHA256SUMS.asc"}]}
```

```console
$ gh release view v0.8.0 --json assets --jq '.assets[].name'
garnet-0.5.0-1.x86_64.rpm
garnet-0.5.0-aarch64-apple-darwin.tar.gz
garnet-0.5.0-lsp-mvp-darwin-arm64.vsix
garnet-0.5.0-lsp-mvp-linux-x64.vsix
garnet-0.5.0-x86_64-apple-darwin.tar.gz
garnet-0.6.0-lsp-mvp-darwin-arm64.vsix
garnet-0.6.0-lsp-mvp-linux-x64.vsix
garnet-0.7.0-lsp-mvp-darwin-arm64.vsix
garnet-0.7.0-lsp-mvp-linux-x64.vsix
garnet_0.5.0-1_amd64.deb
SHA256SUMS
```

```console
$ gh api -H 'Accept: application/octet-stream' /repos/Island-Dev-Crew/garnet/releases/assets/440775813
ca35ebf881cc1d16f288f850eb767305c590112a05966c6778e5fa3d2a42e0cc  garnet_0.8.1-1_amd64.deb
88b4e9cb255409411a595e38eb086943735a70e824d968548460ff69451ea09d  garnet-0.8.1-1.x86_64.rpm
b89396384fa201652027e5e8f365499b08efa6e5b3c9c45dcd894b492d748c3d  garnet-0.8.1-aarch64-apple-darwin.tar.gz
83a474995da6a654f855e45ec246f56b3bb7a933e9fd89827da2a4d5157a21ae  garnet-0.8.1-x86_64-apple-darwin.tar.gz
79c1d2b77635b40580fe39427687fad00913aab7bca6e6d0de5e5611f9e875f7  garnet-sbom-cyclonedx.tgz
```

```console
$ nl -ba docs/release-signing.md | sed -n '6,14p;27,43p'
     6	> **Verified today / Still open (2026-06-07).** Every release ships a `SHA256SUMS` manifest
     7	> (integrity). **The release signing key is now configured** (`GPG_SIGNING_KEY` is
     8	> set), and the public key is published at
     9	> [`docs/garnet-release-signing.pub.asc`](garnet-release-signing.pub.asc) with
    10	> fingerprint **`04D5 6F91 F038 17DD FFEB  C62A C14D F6E7 1395 6ED1`**. The
    11	> **`v0.8.1` Release (re-cut 2026-06-07) is signed** — it carries
    12	> `SHA256SUMS.asc`. Earlier releases (e.g. `v0.8.0`, `v0.5.0`) predate signing and
    13	> are **unsigned** (research-grade default), **not** tampered. Garnet is a
    14	> research-grade prototype, not production/1.0.
    27	## 2. Authenticity — when the release is signed
    28	
    29	When the maintainer's signing key is configured, the release also attaches
    30	`SHA256SUMS.asc` (a detached GPG signature) and the public key is published at
    31	[`docs/garnet-release-signing.pub.asc`](garnet-release-signing.pub.asc) in this repo.
    32	
    33	```sh
    34	# one-time: import the published public key
    35	gpg --import garnet-release-signing.pub.asc
    36	
    37	# confirm you imported the right key — the fingerprint MUST be:
    38	#   04D5 6F91 F038 17DD FFEB  C62A C14D F6E7 1395 6ED1
    39	gpg --fingerprint jon-isaac@islanddevcrew.com
    40	
    41	# verify the signature over the checksum manifest
    42	gpg --verify SHA256SUMS.asc SHA256SUMS
    43	```
```

```console
$ git show fbd64bc514b573a6735b3525f4f3172be14a27d0:SECURITY.md | sed -n '/^## Reporting a Vulnerability$/,/^### What qualifies as a security issue$/p' | shasum -a 256
37cf6ce6c1748c9ab68e8cc789641a78ee3e538d70ba7afb1f3dc648d390bf07  -

$ sed -n '/^## Reporting a Vulnerability$/,/^### What qualifies as a security issue$/p' SECURITY.md | shasum -a 256
37cf6ce6c1748c9ab68e8cc789641a78ee3e538d70ba7afb1f3dc648d390bf07  -

$ git show fbd64bc514b573a6735b3525f4f3172be14a27d0:SECURITY.md | tail -n 1 | shasum -a 256
5f4015b3c62750eee11b009addbb95993ef9864bf00491f46a2fd52828b2365d  -

$ tail -n 1 SECURITY.md | shasum -a 256
5f4015b3c62750eee11b009addbb95993ef9864bf00491f46a2fd52828b2365d  -
```

```console
$ grep -inE '[retired word elided by the transporting seat]' SECURITY.md
```

No stdout; exit 1.

```console
$ python3 -I scripts/garnet_trust_kernel_review_status.py --base fbd64bc514b573a6735b3525f4f3172be14a27d0 --head HEAD --format json
{
  "schema": "garnet.trust_kernel_review/v2",
  "ok": true,
  "discovery_ok": true,
  "discovery_source": "git",
  "base_commit": "fbd64bc514b573a6735b3525f4f3172be14a27d0",
  "head_commit": "73094364bf9f16eac1308fffa5ad685a57eef301",
  "trust_kernel_touched": false,
  "touched_paths": [],
  "review_record_present": false,
  "review_record_path": null,
  "reviewer": null,
  "reviewer_id": null,
  "reviewer_login": null,
  "reviewed_head": null,
  "reviewed_tree": null,
  "content_digest": null,
  "changed_count": 1,
  "problems": [],
  "review_companion_present": false,
  "companion_paths": [],
  "review_trailer_present": false,
  "trust_kernel_prefixes": [
    "garnet-check-v0.3/src/",
    "garnet-interp-v0.3/src/",
    "garnet-vm/src/",
    "garnet-stdlib/src/",
    "garnet-wasm/src/",
    ".github/actions/",
    ".github/rulesets/",
    ".github/workflows/",
    "scripts/garnet_",
    "scripts/test_garnet_",
    "F_Project_Management/W_TRUST/landed/"
  ],
  "trust_kernel_files": [
    ".github/CODEOWNERS",
    "Cargo.lock",
    "garnet-cli/Cargo.toml",
    "garnet-cli/src/bound_source.rs",
    "garnet-cli/src/cmd/add.rs",
    "garnet-cli/src/cmd/mod.rs",
    "garnet-cli/src/cmd/run.rs",
    "garnet-cli/src/cmd/test.rs",
    "garnet-cli/src/cmd/eval.rs",
    "garnet-cli/src/cmd/doctest.rs",
    "garnet-cli/src/bin/garnet.rs",
    "garnet-cli/src/lib.rs",
    "scripts/garnet_launch_readiness_status.py",
    "scripts/garnet_caps_enforcement_status.py",
    "scripts/garnet_capability_scope_status.py",
    "scripts/garnet_bounded_enforcement_status.py",
    "scripts/garnet_red_team_status.py",
    "docs/why.html",
    "C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md",
    "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json"
  ]
}
```

Final void-on-move/read-only recheck:

```console
$ git rev-parse HEAD
73094364bf9f16eac1308fffa5ad685a57eef301

$ git status --porcelain
```

No stdout; exit 0.

VERDICT: CONFIRM — bound to 73094364bf9f16eac1308fffa5ad685a57eef301

1. Blocking: none.
2. Non-blocking: none.



```

## Reviewing seat output — v1 (verbatim, elisions marked; superseded, retained for lineage)

```text
Author seat: Claude. Reviewer seat: OpenAI Codex. Substantive inspection used a fresh, clean clone.

### Exact binding and diff

```text
$ git rev-parse HEAD
9120b7d43af9aa2ae476146d1ef3fb71c62a87c5

$ git status --porcelain
[no output]

$ git rev-parse fbd64bc514b573a6735b3525f4f3172be14a27d0
fbd64bc514b573a6735b3525f4f3172be14a27d0

$ git diff fbd64bc514b573a6735b3525f4f3172be14a27d0...HEAD --stat
 SECURITY.md | 25 ++++++++++++++-----------
 1 file changed, 14 insertions(+), 11 deletions(-)

$ git log fbd64bc514b573a6735b3525f4f3172be14a27d0..HEAD --oneline
9120b7d docs(security): align SECURITY.md with v0.8.1 truth (support window, disowned test figure, enforcement scope, signing facts)
```

### 1. Supported versions

```text
$ git tag --sort=-v:refname
v0.8.1
v0.8.0
v0.5.0
v0.4.2
parked/s15-promo-docs
parked/s5-fmt-wip

$ gh release list --repo Island-Dev-Crew/garnet --limit 6
v0.8.1	Latest	v0.8.1	2026-06-07T07:55:45Z
v0.8.0		v0.8.0	2026-05-31T20:55:58Z
Garnet v0.5.0		v0.5.0	2026-05-20T14:19:59Z
Garnet v0.4.2		v0.4.2	2026-05-15T23:38:42Z

$ sed -n '1,13p' SECURITY.md
# Security Policy

## Supported Versions

| Version | Status          | Security fixes |
|---------|-----------------|----------------|
| 0.8.1   | Current release | ✅ Yes         |
| 0.8.0   | Previous        | ✅ Yes (critical only) |
| 0.5.0   | Previous        | ❌ No          |
| ≤ 0.4.x | Archived        | ❌ No          |

Garnet follows a forward-compatible security support window: the current release + one prior receive security fixes; older releases require upgrade.
```

### 2. Disowned test count

```text
$ git show origin/main:docs/truth.json | grep -A2 security_test_count
    "security_test_count": "No trusted derivation exists for the historical public '136 security tests' figure (it entered the site undocumented). Re-stamping an unverifiable number would automate drift; the public row is removed/replaced by RB-0d instead."
  },
  "primitive_count": 80,

$ rg -n -i '\b(136|security[- ]specific tests|security tests|test count|tests?)\b' SECURITY.md
79:Security-specific tests were added across four historical hardening layers (v3.3 Layer 1 through v4.0 Layer 4). No current count of those tests is published here: `docs/truth.json` (`omissions.security_test_count`) records that no trusted derivation exists for the historical "136 security tests" figure, so this document does not restate it. The original threat model is documented in [GARNET_v3_3_SECURITY_THREAT_MODEL.md](F_Project_Management/GARNET_v3_3_SECURITY_THREAT_MODEL.md) — a roadmap of 15 hardening patterns, two of which address Garnet-specific threat classes (strategy-miner adversarial training, `Box<dyn Any>` hot-reload type confusion).
```

No affirmative numeric test-count claim remains; `136` appears only as the sourced, expressly disowned historical figure.

### 3. Enforcement scope

```text
$ sed -n '68,71p' CLAUDE.md
- Preserve the named-deferred fences: `@bounded` (Wasmtime fuel), memory, time,
  `@mailbox`, and macOS/Windows OS-sandbox application remain
  declared-not-enforced; only `@caps` + `@max_depth` are enforced (both
  backends), with seccomp applied on **Linux only**.

$ nl -ba SECURITY.md | sed -n '49,62p'
    49	**In scope:**
    50	
    51	- Capability escape — code with `@caps()` successfully invoking a runtime-gated primitive whose capability it lacks (e.g. `fs::read_file`, which requires `@caps(fs)`); the runtime-gated surface is the one listed in [GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md](C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md)
    52	- Manifest signature forgery — a `garnet verify --signature` accepting a tampered signed manifest
    53	- Hot-reload replay — a ReloadKey-signed reload from a stale sequence number being accepted
    54	- StateCert type confusion — a hot-reload surviving a type mismatch via BLAKE3 fingerprint collision or bypass
    55	- Compiler impersonation — a malicious compiler producing a manifest that verifies against a legitimate release pubkey
    56	- Strategy-miner poisoning — adversarial training-time injection into the knowledge graph that survives to runtime
    57	- Path traversal / sandbox escape in `@sandbox` code
    58	- Remote code execution via any network primitive
    59	- Any confidentiality / integrity / availability breach that bypasses a claim from Papers III/V/VI or the v3.4 Security V2 spec
    60	
    61	**Declared, not yet enforced** (reports welcome as roadmap findings, not vulnerabilities): `@bounded` (Wasmtime fuel), memory, time, `@mailbox`, and macOS/Windows OS-sandbox application. The repo's operating brief (`CLAUDE.md`) fixes the line: "`@bounded` (Wasmtime fuel), memory, time, `@mailbox`, and macOS/Windows OS-sandbox application remain declared-not-enforced; only `@caps` + `@max_depth` are enforced (both backends), with seccomp applied on **Linux only**." The enforcement scope table says the same of OS sandboxing — "macOS / Windows OS-sandbox application is **named-deferred**" — and limits the runtime claim to "`@caps` and `@max_depth` trap identically on both backends for the gated surface" ([GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md](C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md)). An `Actor::tell` accepting unbounded messages despite `@mailbox(N)` is therefore expected behavior today, not a bypass; file it as a public issue labeled as a roadmap finding.
    62	

$ nl -ba C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md | sed -n '35,43p;85,97p'
    35	| Class | What it means | Where | Members |
    36	|-------|---------------|-------|---------|
    37	| **Declared (checker-only)** | Capability required by the checker; **no runtime gate**. Reachable at run time without a trap once the program type-checks. | `Guard::Declared` (`registry.rs`) | `time::now_ms`, `time::wall_clock_ms`, `time::sleep`; `uuid::new_v4`, `uuid::new_v7` (all require `@caps(time)` at check time only) |
    38	| **Runtime-gated** | `require_capability` in the bridge adapter traps at run time when the active caps frame lacks the capability. **12 primitives.** | `Guard::Gate`; `eval.rs` `require_capability` | `fs::read_file` / `write_file` / `read_bytes` / `write_bytes` / `list_dir`; `net::tcp_connect`; `std::env::get` / `set` / `vars`; `std::process::wait` / `exit_code`; `std::log::to_file` |
    39	| **Entry-gated** | `require_capability` **plus** the S92 program-entry-frame check (anti-laundering). **3 primitives.** | `Guard::GateEntry`; `eval.rs` `require_entry_capability` | `std::process::spawn` / `spawn_args` / `output` |
    40	| **Declared-only, no bridge** | In the checker vocabulary and/or sandbox-policy mapping, but **no runtime enforcement path exists**. | — | `ffi` (checker + manifest + sandbox-policy warning only); `net_internal` (checker vocab + loopback-only in generated sandbox policy; `tcp_connect` always uses strict `NetPolicy::default()`) |
    41	| **Unbridged** | Registry row exists for the CapCaps propagator only; **no interpreter binding at all**. | `Binding::Unbridged` (`registry.rs`) | `net::tcp_listen`, `net::udp_bind` |
    42	| **OS-sandboxed (generated, not self-enforced)** | `garnet sandbox` generates seccomp / WASI / egress policy from aggregate `@caps`. The generator emits `enforced: false`; the policy was applied and trapped on a real **Linux** kernel via an external C reference harness (`tools/seccomp-apply`). macOS / Windows OS-sandbox application is **named-deferred**. | `GARNET_SANDBOX_POLICY.md`, `GARNET_SECCOMP_APPLY.md` | all `@caps` → policy |
    43	| **Caps-invisible** | Host-visible natives with **no capability row at all**. Any "all authority is capability-tagged" claim is false until these earn rows. | `BRIDGE_ONLY` const (`stdlib_bridge.rs`) | `memory::working` / `episodic` / `semantic` / `procedural` |
    85	## What the public copy may and may not say
    86	
    87	- **May say (true):** undeclared OS authority fails `garnet check`; `@caps` and
    88	  `@max_depth` trap identically on both backends for the gated surface, with
    89	  cross-OS trap parity recorded as evidence; the `garnet` CLI and the default
    90	  high-level `Interpreter::new()` load/eval/call path are deny-by-default.
    91	- **May not say (overclaim):** "universal `@caps` runtime enforcement"; "no
    92	  ambient authority, ever" as a runtime-universal claim; that every third-party
    93	  embedder is forced to use the strict constructor, that the explicit
    94	  `new_permissive()` opt-out does not exist, or that raw public Env/Value/eval
    95	  calls inherit an instance scope they do not enter; that
    96	  `time`/`uuid`/`ffi`/`net_internal`/`memory::*` are runtime-gated; that
    97	  OS-sandbox enforcement holds beyond Linux-seccomp via the reference harness.

$ nl -ba C_Language_Specification/GARNET_SANDBOX_POLICY.md | sed -n '7,20p;26,33p'
     7	## [retired word elided by the transporting seat] scope — generation, not enforcement
     8	
     9	**This slice generates policy; it does not enforce it.** Nothing in `garnet
    10	sandbox` runs a guest under `wasmtime`, applies a seccomp profile to a live
    11	process, or installs an egress firewall. Every emitted policy is marked
    12	`"enforced": false`. Runtime enforcement requires:
    13	
    14	- a `wasmtime` (or other WASI) host to honor the WASI capability set, and
    15	- a Linux kernel + a seccomp loader to honor the syscall profile, and
    16	- a network layer to honor the egress rule.
    17	
    18	These are **out of scope for S46** (and `wasmtime`/`wasm-tools` are absent from
    19	the current build environment; seccomp is Linux-only). The seccomp profile
    20	mirrors the OCI/Docker default-deny shape but is **not** validated against a live
    26	> **Update — the seccomp profile is now proven enforceable on a real kernel.**
    27	> `garnet sandbox`'s `enforced: false` (generation) flag stays [retired word elided by the transporting seat], but the
    28	> *generated* seccomp profile has been **applied and deterministically trapped** on
    29	> a real Linux kernel (the Mac's UTM Debian-12 ARM64 guest): under `@caps(fs)`,
    30	> `socket()` is denied with `EPERM`; under `@caps(fs, net)` it is allowed
    31	> (policy-driven). See `C_Language_Specification/GARNET_SECCOMP_APPLY.md` and the
    32	> reference apply harness `tools/seccomp-apply/`. WASI/`wasmtime` fuel and egress
    33	> enforcement remain deferred; macOS/Windows OS sandboxing remain named-deferred.
```

The three quoted passages in the new paragraph are verbatim modulo Markdown line wrapping. The retained line 57 fails the separate no-deferred-fence requirement.

### 4. Release signing

```text
$ nl -ba .github/workflows/linux-packages.yml | sed -n '17,20p;217,223p;326,328p;335p;358,423p'
    17	on:
    18	  push:
    19	    branches: [main]
    20	    tags: ['v*']
   217	  macos-cli-tarballs:
   218	    runs-on: macos-latest
   219	    strategy:
   220	      fail-fast: false
   221	      matrix:
   222	        target: [aarch64-apple-darwin, x86_64-apple-darwin]
   223	    steps:
   326	  release:
   327	    if: startsWith(github.ref, 'refs/tags/v')
   328	    needs: [smoke-deb, smoke-rpm, macos-cli-tarballs, shellcheck-installer]
   335	      HAS_GPG: ${{ secrets.GPG_SIGNING_KEY != '' }}
   358	      - name: Generate CycloneDX SBOM (attached to the release)
   359	        run: |
   360	          cargo install cargo-cyclonedx --locked --force
   361	          cargo cyclonedx --all --format json
   362	          mkdir -p sbom-tmp
   363	          find . -name '*.cdx.json' -not -path './release-dist/*' -not -path './sbom-tmp/*' \
   364	            -exec cp {} sbom-tmp/ \;
   365	          # .tgz extension so the *.tar.gz binary glob never picks up the SBOM.
   366	          tar -czf release-dist/garnet-sbom-cyclonedx.tgz -C sbom-tmp .
   367	          echo "SBOM files bundled:"; ls -1 sbom-tmp
   368	
   369	      - name: Compose unified SHA256SUMS
   370	        run: |
   371	          cd release-dist
   372	          rm -f SHA256SUMS
   373	          sha256sum *.deb *.rpm *.tar.gz *.tgz > SHA256SUMS
   374	          cat SHA256SUMS
   375	
   376	      - name: Sign SHA256SUMS (activates when GPG_SIGNING_KEY secret is set)
   377	        if: env.HAS_GPG == 'true'
   378	        env:
   379	          GPG_SIGNING_KEY: ${{ secrets.GPG_SIGNING_KEY }}
   380	          GPG_PASSPHRASE: ${{ secrets.GPG_PASSPHRASE }}
   381	        run: |
   382	          cd release-dist
   383	          echo "$GPG_SIGNING_KEY" | gpg --batch --import
   384	          if [ -n "$GPG_PASSPHRASE" ]; then
   385	            gpg --batch --yes --pinentry-mode loopback --passphrase "$GPG_PASSPHRASE" \
   386	              --detach-sign --armor SHA256SUMS
   387	          else
   388	            gpg --batch --yes --detach-sign --armor SHA256SUMS
   389	          fi
   390	          echo "signed → SHA256SUMS.asc"
   391	
   392	      # Fail-closed: a TAGGED release must ship a SIGNED SHA256SUMS. Previously a
   393	      # missing GPG_SIGNING_KEY silently published an UNSIGNED release; now an
   394	      # unsigned tagged release is BLOCKED unless it is a deliberate, recorded act
   395	      # (repo variable ALLOW_UNSIGNED_RELEASE=true), so shipping unsigned is a
   396	      # decision, never an accident. [Jon-gated: this is a release-POLICY change.]
   397	      - name: Require signed SHA256SUMS (fail-closed)
   398	        if: env.HAS_GPG != 'true'
   399	        run: |
   400	          if [ "${{ vars.ALLOW_UNSIGNED_RELEASE }}" = "true" ]; then
   401	            echo "::warning::GPG_SIGNING_KEY absent and ALLOW_UNSIGNED_RELEASE=true — publishing a DELIBERATELY UNSIGNED research-grade release."
   402	          else
   403	            echo "::error::Tagged release requires a signed SHA256SUMS, but GPG_SIGNING_KEY is not set. Provide GPG_SIGNING_KEY (+ optional GPG_PASSPHRASE), or set repository variable ALLOW_UNSIGNED_RELEASE=true to deliberately ship unsigned."
   404	            exit 1
   405	          fi
   406	
   407	      - name: Publish release
   408	        uses: softprops/action-gh-release@3bb12739c298aeb8a4eeaf626c5b8d85266b0e65
   409	        with:
   410	          files: |
   411	            release-dist/*.deb
   412	            release-dist/*.rpm
   413	            release-dist/*.tar.gz
   414	            release-dist/garnet-sbom-cyclonedx.tgz
   415	            release-dist/SHA256SUMS
   416	          fail_on_unmatched_files: true
   417	          generate_release_notes: true
   418	
   419	      - name: Attach SHA256SUMS signature (only when signed)
   420	        if: env.HAS_GPG == 'true'
   421	        uses: softprops/action-gh-release@3bb12739c298aeb8a4eeaf626c5b8d85266b0e65
   422	        with:
   423	          files: release-dist/SHA256SUMS.asc

$ nl -ba docs/install.sh | sed -n '178,198p;200,224p;343,360p'
   178	verify_sha256() {
   179	    _file="$1"
   180	    _expected="$2"
   181	
   182	    if command -v sha256sum >/dev/null 2>&1; then
   183	        _actual="$(sha256sum "$_file" | awk '{print $1}')"
   184	    elif command -v shasum >/dev/null 2>&1; then
   185	        _actual="$(shasum -a 256 "$_file" | awk '{print $1}')"
   186	    else
   187	        err "need sha256sum or shasum to verify $_file"
   188	    fi
   189	
   190	    if [ "$_actual" != "$_expected" ]; then
   191	        err "SHA-256 mismatch for $_file
   192	  expected: $_expected
   193	  got:      $_actual
   194	  refusing to run an unverified installer"
   195	    fi
   196	
   197	    say "SHA-256 verified"
   198	}
   200	lookup_expected_sha256() {
   201	    _asset="$1"
   202	    _sums_url="${GARNET_CHECKSUM_URL:-${GARNET_BASE_URL}/SHA256SUMS}"
   203	    _tmp="$(mktemp_file sums)"
   204	
   205	    try_download "$_sums_url" "$_tmp" || {
   206	        rm -f "$_tmp"
   207	        return 1
   208	    }
   209	    _sha="$(awk -v f="$_asset" '
   210	        {
   211	            name = $2
   212	            sub(/^\*/, "", name)
   213	            base = name
   214	            sub(/^.*\//, "", base)
   215	            if (name == f || base == f) {
   216	                print $1
   217	                exit
   218	            }
   219	        }
   220	    ' "$_tmp")"
   221	    rm -f "$_tmp"
   222	
   223	    [ -n "$_sha" ] || return 1
   224	    printf '%s' "$_sha"
   343	release_install_for_format() {
   344	    _triple="$1"
   345	    _format="$2"
   346	    _asset="$(asset_name "$_triple" "$_format")"
   347	    _url="${GARNET_BASE_URL}/${_asset}"
   348	    _dest="$(mktemp_file "$_asset")"
   349	
   350	    trap 'rm -f "$_dest"' EXIT INT HUP TERM
   351	
   352	    say "detected = ${_triple} / ${_format}"
   353	    say "asset    = ${_asset}"
   354	    say "fetching SHA256SUMS"
   355	    _expected_sha="$(lookup_expected_sha256 "$_asset")" || return 1
   356	
   357	    say "downloading ${_url}"
   358	    try_download "$_url" "$_dest" || return 1
   359	    verify_sha256 "$_dest" "$_expected_sha"
   360	

$ rg -n -i 'gpg|SHA256SUMS\.asc' docs/install.sh
[no output]
```

```text
$ gh release view v0.8.1 --json assets
{"assets":[{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440774492","contentType":"application/octet-stream","createdAt":"2026-06-07T07:55:44Z","digest":"sha256:16d31c507301fbef595971e8b12fd9791b7d150f22fad5839001bf0b5965fa75","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRa9c","label":"","name":"garnet-0.7.0-lsp-mvp-darwin-arm64.vsix","size":1863752,"state":"uploaded","updatedAt":"2026-06-07T07:55:44Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-0.7.0-lsp-mvp-darwin-arm64.vsix"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440774491","contentType":"application/octet-stream","createdAt":"2026-06-07T07:55:44Z","digest":"sha256:60f846a1a2013fd6d826939de962268eed31339c5bcab8a5799fed53f1572778","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRa9b","label":"","name":"garnet-0.7.0-lsp-mvp-linux-x64.vsix","size":2030614,"state":"uploaded","updatedAt":"2026-06-07T07:55:44Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-0.7.0-lsp-mvp-linux-x64.vsix"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775812","contentType":"application/x-redhat-package-manager","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:88b4e9cb255409411a595e38eb086943735a70e824d968548460ff69451ea09d","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRbSE","label":"","name":"garnet-0.8.1-1.x86_64.rpm","size":2582522,"state":"uploaded","updatedAt":"2026-06-07T07:58:27Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-0.8.1-1.x86_64.rpm"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775814","contentType":"application/gzip","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:b89396384fa201652027e5e8f365499b08efa6e5b3c9c45dcd894b492d748c3d","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRbSG","label":"","name":"garnet-0.8.1-aarch64-apple-darwin.tar.gz","size":2697981,"state":"uploaded","updatedAt":"2026-06-07T07:58:27Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-0.8.1-aarch64-apple-darwin.tar.gz"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775816","contentType":"application/gzip","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:83a474995da6a654f855e45ec246f56b3bb7a933e9fd89827da2a4d5157a21ae","downloadCount":2,"id":"RA_kwDOSJv_Zc4aRbSI","label":"","name":"garnet-0.8.1-x86_64-apple-darwin.tar.gz","size":2999427,"state":"uploaded","updatedAt":"2026-06-07T07:58:27Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-0.8.1-x86_64-apple-darwin.tar.gz"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775817","contentType":"application/octet-stream","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:79c1d2b77635b40580fe39427687fad00913aab7bca6e6d0de5e5611f9e875f7","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRbSJ","label":"","name":"garnet-sbom-cyclonedx.tgz","size":116256,"state":"uploaded","updatedAt":"2026-06-07T07:58:27Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet-sbom-cyclonedx.tgz"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775815","contentType":"application/x-debian-package","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:ca35ebf881cc1d16f288f850eb767305c590112a05966c6778e5fa3d2a42e0cc","downloadCount":3,"id":"RA_kwDOSJv_Zc4aRbSH","label":"","name":"garnet_0.8.1-1_amd64.deb","size":2401512,"state":"uploaded","updatedAt":"2026-06-07T07:58:28Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/garnet_0.8.1-1_amd64.deb"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775813","contentType":"application/octet-stream","createdAt":"2026-06-07T07:58:27Z","digest":"sha256:00a1ccc254cd6ea2f99834ef6b9fbe96c54b0ec7b31f33ee68647d1c05541a18","downloadCount":9,"id":"RA_kwDOSJv_Zc4aRbSF","label":"","name":"SHA256SUMS","size":488,"state":"uploaded","updatedAt":"2026-06-07T07:58:27Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/SHA256SUMS"},{"apiUrl":"https://api.github.com/repos/Island-Dev-Crew/garnet/releases/assets/440775847","contentType":"application/pgp-keys","createdAt":"2026-06-07T07:58:29Z","digest":"sha256:d2de154d475fd9f99fe3d5fd8734083e35945ee1f5332be38ad57cc8bb4fd961","downloadCount":6,"id":"RA_kwDOSJv_Zc4aRbSn","label":"","name":"SHA256SUMS.asc","size":228,"state":"uploaded","updatedAt":"2026-06-07T07:58:29Z","url":"https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/SHA256SUMS.asc"}]}

$ gh release view v0.8.0 --json assets --jq '.assets[].name'
garnet-0.5.0-1.x86_64.rpm
garnet-0.5.0-aarch64-apple-darwin.tar.gz
garnet-0.5.0-lsp-mvp-darwin-arm64.vsix
garnet-0.5.0-lsp-mvp-linux-x64.vsix
garnet-0.5.0-x86_64-apple-darwin.tar.gz
garnet-0.6.0-lsp-mvp-darwin-arm64.vsix
garnet-0.6.0-lsp-mvp-linux-x64.vsix
garnet-0.7.0-lsp-mvp-darwin-arm64.vsix
garnet-0.7.0-lsp-mvp-linux-x64.vsix
garnet_0.5.0-1_amd64.deb
SHA256SUMS

$ gpg --show-keys --with-colons docs/garnet-release-signing.pub.asc | rg '^(fpr|pub):'
pub:u:255:22:C14DF6E713956ED1:1780818238:1843890238::u:::scaSCA:::::ed25519:::0:
fpr:::::::::04D56F91F03817DDFFEBC62AC14DF6E713956ED1:

$ curl -fsSL https://github.com/Island-Dev-Crew/garnet/releases/download/v0.8.1/SHA256SUMS
ca35ebf881cc1d16f288f850eb767305c590112a05966c6778e5fa3d2a42e0cc  garnet_0.8.1-1_amd64.deb
88b4e9cb255409411a595e38eb086943735a70e824d968548460ff69451ea09d  garnet-0.8.1-1.x86_64.rpm
b89396384fa201652027e5e8f365499b08efa6e5b3c9c45dcd894b492d748c3d  garnet-0.8.1-aarch64-apple-darwin.tar.gz
83a474995da6a654f855e45ec246f56b3bb7a933e9fd89827da2a4d5157a21ae  garnet-0.8.1-x86_64-apple-darwin.tar.gz
79c1d2b77635b40580fe39427687fad00913aab7bca6e6d0de5e5611f9e875f7  garnet-sbom-cyclonedx.tgz
```

The rewritten section is present-tense and supported by the workflow, installer, key, checksum manifest, and live assets.

### 5. Crypto versions

```text
$ rg -n -A2 '^name = "(ed25519-dalek|blake3|sha2)"$' Cargo.lock
169:name = "blake3"
170-version = "1.8.4"
171-source = "registry+https://github.com/rust-lang/crates.io-index"
--
565:name = "ed25519-dalek"
566-version = "2.2.0"
567-source = "registry+https://github.com/rust-lang/crates.io-index"
--
1943:name = "sha2"
1944-version = "0.10.9"
1945-source = "registry+https://github.com/rust-lang/crates.io-index"
```

### 6. Change isolation and preserved text

```text
$ git diff fbd64bc5..HEAD --stat
 SECURITY.md | 25 ++++++++++++++-----------
 1 file changed, 14 insertions(+), 11 deletions(-)

$ git diff fbd64bc514b573a6735b3525f4f3172be14a27d0..HEAD --name-status
M	SECURITY.md

$ for review_rev in fbd64bc514b573a6735b3525f4f3172be14a27d0 HEAD; do
>   printf '%s contact+timeline ' "$review_rev"
>   git show "${review_rev}:SECURITY.md" | sed -n '/^## Reporting a Vulnerability/,/^### What qualifies as a security issue/p' | shasum -a 256 | awk '{print $1}'
>   printf '%s verse ' "$review_rev"
>   git show "${review_rev}:SECURITY.md" | tail -n 3 | shasum -a 256 | awk '{print $1}'
> done
fbd64bc514b573a6735b3525f4f3172be14a27d0 contact+timeline 37cf6ce6c1748c9ab68e8cc789641a78ee3e538d70ba7afb1f3dc648d390bf07
fbd64bc514b573a6735b3525f4f3172be14a27d0 verse 10df85054ad4f6edb2739b25042fe8561af5753b5a15fd50b977da6c2d30f7f0
HEAD contact+timeline 37cf6ce6c1748c9ab68e8cc789641a78ee3e538d70ba7afb1f3dc648d390bf07
HEAD verse 10df85054ad4f6edb2739b25042fe8561af5753b5a15fd50b977da6c2d30f7f0
```

### 7. Retired vocabulary and rolling gate

```text
$ grep -inE '[retired word elided by the transporting seat]' SECURITY.md
[no output]

$ python3 -I scripts/garnet_trust_kernel_review_status.py --base fbd64bc514b573a6735b3525f4f3172be14a27d0 --head HEAD --format json
{
  "schema": "garnet.trust_kernel_review/v2",
  "ok": true,
  "discovery_ok": true,
  "discovery_source": "git",
  "base_commit": "fbd64bc514b573a6735b3525f4f3172be14a27d0",
  "head_commit": "9120b7d43af9aa2ae476146d1ef3fb71c62a87c5",
  "trust_kernel_touched": false,
  "touched_paths": [],
  "review_record_present": false,
  "review_record_path": null,
  "reviewer": null,
  "reviewer_id": null,
  "reviewer_login": null,
  "reviewed_head": null,
  "reviewed_tree": null,
  "content_digest": null,
  "changed_count": 1,
  "problems": [],
  "review_companion_present": false,
  "companion_paths": [],
  "review_trailer_present": false,
  "trust_kernel_prefixes": [
    "garnet-check-v0.3/src/",
    "garnet-interp-v0.3/src/",
    "garnet-vm/src/",
    "garnet-stdlib/src/",
    "garnet-wasm/src/",
    ".github/actions/",
    ".github/rulesets/",
    ".github/workflows/",
    "scripts/garnet_",
    "scripts/test_garnet_",
    "F_Project_Management/W_TRUST/landed/"
  ],
  "trust_kernel_files": [
    ".github/CODEOWNERS",
    "Cargo.lock",
    "garnet-cli/Cargo.toml",
    "garnet-cli/src/bound_source.rs",
    "garnet-cli/src/cmd/add.rs",
    "garnet-cli/src/cmd/mod.rs",
    "garnet-cli/src/cmd/run.rs",
    "garnet-cli/src/cmd/test.rs",
    "garnet-cli/src/cmd/eval.rs",
    "garnet-cli/src/cmd/doctest.rs",
    "garnet-cli/src/bin/garnet.rs",
    "garnet-cli/src/lib.rs",
    "scripts/garnet_launch_readiness_status.py",
    "scripts/garnet_caps_enforcement_status.py",
    "scripts/garnet_capability_scope_status.py",
    "scripts/garnet_bounded_enforcement_status.py",
    "scripts/garnet_red_team_status.py",
    "docs/why.html",
    "C_Language_Specification/GARNET_CAPABILITY_ENFORCEMENT_SCOPE.md",
    "F_Project_Management/W_TRUST/LANDED_REVIEW_MARKERS.json"
  ]
}
```

Recomputed status: claims 1, 2, 4, 5, 6, and 7 pass; claim 3 fails. Standards axis has no finding. This review is void—not stale—the instant the reviewed head moves.

VERDICT: REJECT — bound to 9120b7d43af9aa2ae476146d1ef3fb71c62a87c5

1. **Blocking — enforcement-scope contradiction.** `SECURITY.md:57` still classifies “Path traversal / sandbox escape in `@sandbox` code” as an unqualified in-scope vulnerability. The governing scope says `garnet sandbox` generates policy but does not self-enforce it, Linux enforcement exists only through the separate reference harness, and macOS/Windows remain named-deferred. Therefore an in-scope bullet still claims a declared/not-self-enforced fence as a vulnerability class. It must be limited to an actually applied Linux seccomp boundary or moved to roadmap-report treatment.

2. **Non-blocking — none.**



```
