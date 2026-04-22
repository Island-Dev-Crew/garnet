# GARNET v4.2 — GitHub Repository Layout Design

**Purpose:** Define the canonical layout for `github.com/IslandDevCrew/garnet`, the DNS + Pages wiring for `garnet-lang.org`, the relationship between the repo and the parent company site `islanddevcrew.com`, and the exact `git` command sequence that converts the current Windows workspace into the initial commit on `main`.
**Date:** 2026-04-21
**Author:** Claude Opus 4.7 (1M) — design agent
**Scope:** Repository structure, parent-site integration, DNS, Pages, initial push. No code changes. No cargo builds.
**Anchor:** *"Every wise woman buildeth her house: but the foolish plucketh it down with her hands." — Proverbs 14:1*

---

## 1. Executive summary

The existing Windows workspace root at:

```
D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\E_Engineering_Artifacts\
```

becomes the **repository root** when pushed to `github.com/IslandDevCrew/garnet`. The research corpus currently parallel to `E_Engineering_Artifacts/` (at `Garnet_Final/{A_Research_Papers,B_Four_Model_Consensus,C_Language_Specification,D_Executive_and_Presentation,F_Project_Management}/`) **moves into `research/` at the repo root** rather than staying under a `Garnet_Final/` subdirectory. Justification in §3.

`garnet-lang.org` resolves to GitHub Pages serving `docs/index.html` via the repo's `docs/CNAME`. Parent site `islanddevcrew.com` carries an R&D page that links to the repo, the website, and the latest release — the parent site does not mirror the repo, it points at it. Full DNS + Pages + parent-site wiring in §4–6. Initial commit sequence in §7.

---

## 2. Full directory tree (repository root after conversion)

```
garnet/                                               <- github.com/IslandDevCrew/garnet  (repo root)
├── .github/
│   ├── workflows/
│   │   └── linux-packages.yml                        <- already present; builds .deb + .rpm, publishes on v* tags
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   ├── feature_request.md
│   │   └── security_advisory.md                      <- redirects to private Security Advisory
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── CODEOWNERS
│   ├── FUNDING.yml                                   <- optional; IDC sponsor link
│   └── dependabot.yml
├── .cargo/
│   └── config.toml                                   <- PIN: do not commit host-specific target
│                                                        triple; keep target-dir only, or move to
│                                                        host ~/.cargo/ (see §7.4 "cargo config
│                                                        hygiene")
├── .gitignore
├── .gitattributes                                    <- LF normalization for .sh/.rs/.toml;
│                                                        binary for .png/.pdf/.pptx/.docx/.ico/.bmp
├── CHANGELOG.md                                      <- v4.2 entry at top cites all Stage 6 handoffs
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── LICENSE                                           <- dual MIT OR Apache-2.0
├── LICENSE-APACHE
├── LICENSE-MIT
├── README.md                                         <- (already present) hero + install + quickstart
├── SECURITY.md
├── WORKSPACE_README.md                               <- crate map + rung status
├── FAQ.md
├── Cargo.toml                                        <- workspace manifest
├── Cargo.lock                                        <- committed (binary crates)
├── rust-toolchain.toml                               <- pin to stable; optional
│
├── garnet-parser-v0.3/                               <- crate (Rung 2.1)
├── garnet-interp-v0.3/                               <- crate (Rung 3, 22-primitive stdlib bridge)
├── garnet-check-v0.3/                                <- crate (Rung 4, CapCaps call-graph)
├── garnet-memory-v0.3/                               <- crate (Rung 5, four-kind refs)
├── garnet-actor-runtime/                             <- crate (Rung 6, signed hot-reload)
├── garnet-stdlib/                                    <- crate (OS I/O + capability metadata)
├── garnet-cli/                                       <- crate (top-level `garnet` binary)
│   ├── assets/
│   │   ├── garnet-logo.png                           <- 1024×1024 JPEG-in-PNG source logo
│   │   └── README.md                                 <- ImageMagick commands for MSI assets
│   ├── wix/
│   │   ├── main.wxs
│   │   ├── Dialog.bmp                                <- generated locally; committed
│   │   ├── Banner.bmp
│   │   ├── Garnet.ico
│   │   ├── License.rtf
│   │   └── README.md
│   ├── macos/
│   │   ├── build-pkg.sh
│   │   ├── distribution.xml
│   │   ├── resources/
│   │   │   ├── background.png
│   │   │   ├── welcome.html
│   │   │   ├── conclusion.html
│   │   │   └── LICENSE.txt
│   │   └── README.md
│   ├── linux/
│   │   ├── garnet-actor.service
│   │   └── README.md
│   ├── windows/
│   │   ├── sandbox-smoke.wsb
│   │   ├── smoke-test.cmd
│   │   └── README.md
│   ├── man/
│   │   └── garnet.1
│   ├── templates/
│   │   ├── cli/
│   │   ├── web-api/
│   │   └── agent-orchestrator/
│   ├── src/
│   └── Cargo.toml
├── garnet-convert/                                   <- Rust/Ruby/Python/Go → Garnet migration tool
├── garnet-parser/                                    <- historical v0.2; excluded from workspace
├── xtask/                                            <- repo-local dev tasks (7×-consistency, etc.)
│
├── installer/
│   └── sh.garnet-lang.org/
│       ├── install.sh                                <- universal one-liner
│       └── README.md
│
├── examples/                                         <- 10 MVP programs + 3 real-world
│   ├── README.md
│   ├── mvp_01_os_simulator.garnet
│   ├── mvp_02_relational_db.garnet
│   ├── … (mvp_03 through mvp_10)
│   ├── agentic_log_analyzer.garnet
│   ├── multi_agent_builder.garnet
│   ├── safe_io_layer.garnet
│   └── safe_io_layer.garnet.manifest.json
│
├── dist/                                             <- release artifacts (committed for audit trail;
│   │                                                   the GHA Release workflow is authoritative)
│   ├── SHA256SUMS
│   ├── linux/
│   │   ├── garnet_0.4.2-1_amd64.deb
│   │   └── garnet-0.4.2-1.x86_64.rpm
│   └── windows/
│       └── garnet-0.4.2-x86_64.msi
│
├── docs/                                             <- GitHub Pages root
│   ├── CNAME                                         <- contains the single line: garnet-lang.org
│   ├── index.html                                    <- landing site (already present)
│   ├── assets/                                       <- images, css, js the site pulls
│   └── .nojekyll                                     <- disables Jekyll; serves files as-is
│
└── research/                                         <- MIGRATED from Garnet_Final/ siblings
    ├── README.md                                     <- NEW: "what lives here + reviewer's quickstart"
    ├── papers/                                       <- was A_Research_Papers/
    │   ├── GARNET-The-Reconciliation-of-Rust-and-Ruby.md
    │   ├── GARNET-The-Reconciliation-of-Rust-and-Ruby.pdf
    │   ├── Paper_I_Rust_Deep_Dive_Updated.docx
    │   ├── Paper_II_Ruby_Deep_Dive_Updated.docx
    │   ├── Paper_III_Garnet_Synthesis_v2_1.{md,docx,pdf}
    │   ├── Paper_IV_Garnet_Agentic_Systems_v2_1_1.docx
    │   ├── Paper_IV_Addendum_v1_0.md
    │   ├── Paper_V_Garnet_Formal_Grounding_v1_0.docx
    │   ├── Paper_V_Addendum_v1_0.md
    │   ├── Paper_VI_Garnet_Novel_Frontiers.md
    │   ├── Paper_VI_Empirical_Validation_Protocol.md
    │   ├── Paper_VI_v4_0_Revisions.md
    │   └── Paper_VII_Implementation_Ladder_and_Tooling.md
    ├── consensus/                                    <- was B_Four_Model_Consensus/
    │   ├── GARNET_v2_1_Gemini_Synthesis.md
    │   └── GARNET_v2_1_Four_Model_Consensus_Memo.md
    ├── spec/                                         <- was C_Language_Specification/
    │   ├── GARNET_v1_0_Mini_Spec.md                  <- canonical normative source of truth
    │   ├── GARNET_v0_3_Formal_Grammar_EBNF.md
    │   ├── GARNET_Compiler_Architecture_Spec.md
    │   ├── GARNET_Memory_Manager_Architecture.md
    │   ├── GARNET_Distribution_and_Installation_Spec.md
    │   ├── GARNET_Tier2_Ecosystem_Specifications.md
    │   ├── GARNET_Benchmarking_and_Evaluation_Plan.md
    │   ├── GARNET_Migration_Guide_Ruby_Python.md
    │   ├── GARNET_Academic_Submission_Strategy.md
    │   ├── GARNET_Compression_Techniques_Reference.md
    │   ├── v4_1_Converter_Architecture.md
    │   └── GARNET_v0_{2,3}_Mini_Spec*.md              <- historical
    ├── presentation/                                 <- was D_Executive_and_Presentation/
    │   ├── GARNET_v2_2_Executive_Overview.md
    │   ├── GARNET_v4_2_DX_Comparative_Paper.docx
    │   ├── GARNET_v4_2_DX_Comparative_Deck.pptx
    │   ├── Garnet_v2_2_Deck.pptx
    │   ├── Garnet_Research_Portal_v2_1.html
    │   ├── garnet-website.html
    │   └── assets/
    └── management/                                   <- was F_Project_Management/
        ├── _CANONICAL_DELIVERABLES_INDEX.md
        ├── GARNET_PROJECT_HANDOFF.md
        ├── GARNET_v2_2_Master_Execution_Protocol.md
        ├── GARNET_v2_7_HANDOFF.md
        ├── GARNET_v3_0_HANDOFF.md  …  GARNET_v3_5_HANDOFF.md
        ├── GARNET_v3_4_1_HANDOFF.md
        ├── GARNET_v4_0_HANDOFF.md
        ├── GARNET_v4_0_PAPER_VI_EXECUTION.md
        ├── GARNET_v4_0_PERFORMANCE_BENCHMARKS.md
        ├── GARNET_v4_1_HANDOFF.md
        ├── GARNET_v4_2_HANDOFF.md
        ├── GARNET_v4_2_Phase_6A_HANDOFF.md
        ├── GARNET_v4_2_Phase_6D_Linux_VERIFIED.md
        ├── GARNET_v4_2_Phase_6D_Windows_VERIFIED.md
        ├── GARNET_v4_2_Phase_6D_macOS_VERIFIED.md         <- written post-macOS verify
        ├── GARNET_v4_2_Developer_Comprehension_Study_Protocol.md
        ├── GARNET_v4_2_STAGE6_KICKOFF.md
        ├── GARNET_v4_2_BOOT.md
        ├── VERIFICATION_LADDER_v4_0.md
        ├── GARNET_v4_2_GITHUB_REPO_LAYOUT.md               <- THIS FILE
        ├── GARNET_v4_2_MAC_RECONCILIATION_PROMPT.md
        ├── GARNET_v4_2_COMPLETE_PROJECT_STATE.md
        └── codex-5.4-pass/                                 <- directory kept intact
```

---

## 3. Why `research/` at the repo root (not `Garnet_Final/`)

### Options considered

**Option A — keep `Garnet_Final/` as a top-level directory inside the repo.**
Pro: zero path rewrites. Con: the name "Garnet_Final" is a session-artifact name specific to the build folder; to an outside reader it reads like a backup or a draft stamp. GitHub renders repo file-trees alphabetically and the word "Garnet_Final" at the top of a repo named `garnet` is visual noise.

**Option B — promote the four `Garnet_Final/` children (A/B/C/D/F) to repo root without rename.**
Pro: preserves existing filenames in the handoffs. Con: mixes research docs with the Cargo workspace; the repo root already has Cargo.toml, LICENSE, README, eight crate directories, `docs/`, `dist/`, `installer/`, `examples/`, `.github/`. Adding five more top-level directories ballooned to 20+ makes orientation harder for a first-time visitor.

**Option C (CHOSEN) — consolidate under `research/` at the repo root with renamed children.**
Pro: single top-level entry point for "everything that is research, not code or build output". Lowercase `research/` matches the convention already set by `examples/`, `installer/`, `docs/`, `dist/`. The renamed children (`papers/`, `consensus/`, `spec/`, `presentation/`, `management/`) read naturally. Reviewers arriving from the MIT submission link land on `README.md` → the repo's existing README hero + quickstart → the research corpus one click away at `research/`.

Con: breaks all existing intra-handoff links of the form `F_Project_Management/GARNET_v4_2_HANDOFF.md`. Mitigation: the conversion is a bulk `git mv` (§7 step 2) followed by a single `sed`-style find/replace pass in the handoff markdown to rewrite paths — the canonical list of rewrites is in §3.1 below. README.md's own `Documentation` section already uses the `../A_Research_Papers/` / `../C_Language_Specification/` / `../F_Project_Management/` patterns; those ten occurrences get rewritten to `research/papers/` / `research/spec/` / `research/management/` in the same pass.

### 3.1. Canonical path rewrites for the conversion commit

| Old path prefix (inside handoffs + README) | New path prefix |
|--------------------------------------------|-----------------|
| `Garnet_Final/A_Research_Papers/` | `research/papers/` |
| `../A_Research_Papers/` | `../research/papers/` (from README in repo root → just `research/papers/`) |
| `Garnet_Final/B_Four_Model_Consensus/` | `research/consensus/` |
| `Garnet_Final/C_Language_Specification/` | `research/spec/` |
| `../C_Language_Specification/` | `research/spec/` |
| `Garnet_Final/D_Executive_and_Presentation/` | `research/presentation/` |
| `../D_Executive_and_Presentation/` | `research/presentation/` |
| `Garnet_Final/E_Engineering_Artifacts/` | `` (repo root — these references drop the prefix entirely) |
| `Garnet_Final/F_Project_Management/` | `research/management/` |
| `../F_Project_Management/` | `research/management/` |

A small helper script (`xtask/rewrite-paths.sh`, one-shot, deletable after) performs the rewrite. The handoff corpus is markdown only; there are no compiled code paths that depend on the `Garnet_Final/` layout (verified: `grep -r "Garnet_Final" E_Engineering_Artifacts/**/*.rs` returns zero hits — the layout name only appears in docs and the `.garnetup` hints of the universal installer, which the installer itself does not parse).

**All link-path rewrites are confined to markdown files.** No Rust source, no Cargo.toml, no WiX XML, no shell script references `Garnet_Final/` by path.

---

## 4. Parent company site integration — `islanddevcrew.com/r-and-d/garnet/`

Island Development Crew's site at `islanddevcrew.com` carries an R&D section. The parent site does NOT mirror the repo; it **points at** it. The intended reader flow:

```
islanddevcrew.com  →  /r-and-d/  →  /r-and-d/garnet/  →  garnet-lang.org  +  github.com/IslandDevCrew/garnet
```

### 4.1. Recommended URL shape on the parent site

| Parent-site URL | Target |
|-----------------|--------|
| `islanddevcrew.com/r-and-d/` | Index page listing every IDC research project; Garnet is one card. |
| `islanddevcrew.com/r-and-d/garnet/` | Garnet project card expanded. 3-paragraph summary + hero logo + primary-CTA button linking to `garnet-lang.org` + secondary buttons to repo, research papers, latest release. |
| `islanddevcrew.com/r-and-d/garnet/papers/` *(optional)* | Copy of the `research/papers/` PDF tree hosted on the parent site for people who would never click through to a code-hosting site. Can be a `<link rel="canonical">` pointing back at `github.com/IslandDevCrew/garnet/tree/main/research/papers/`. |

### 4.2. Links that go on the parent-site card

These are the four URLs `islanddevcrew.com/r-and-d/garnet/` should publish:

1. **Language home:** `https://garnet-lang.org`
2. **Source:** `https://github.com/IslandDevCrew/garnet`
3. **Latest release:** `https://github.com/IslandDevCrew/garnet/releases/latest`
4. **Research corpus:** `https://github.com/IslandDevCrew/garnet/tree/main/research`

Plus one install badge:

```html
<a href="https://sh.garnet-lang.org" rel="noopener">
  <code>curl --proto '=https' --tlsv1.2 -sSf https://sh.garnet-lang.org | sh</code>
</a>
```

### 4.3. What NOT to duplicate on `islanddevcrew.com`

- The README — it's the repo's job and it already renders beautifully on github.com.
- The install instructions — they live in README.md + FAQ.md; the parent site should link, not copy.
- The research papers' full text — link to the canonical GitHub blobs; PDFs can be optionally re-hosted but should carry `<link rel="canonical">`.

The parent site's job is **wayfinding**: "here is a project we've produced; here is where to go next." The repo + `garnet-lang.org` carry the content.

---

## 5. DNS + GitHub Pages wiring for `garnet-lang.org`

### 5.1. What is already in the repo

- `E_Engineering_Artifacts/docs/index.html` — the landing site (will be at `docs/index.html` at the repo root post-conversion).
- `E_Engineering_Artifacts/docs/CNAME` — expected to contain the single line `garnet-lang.org` (one-line text file, no trailing newline issues).

### 5.2. GitHub Pages configuration

At `github.com/IslandDevCrew/garnet` → **Settings** → **Pages**:

| Setting | Value |
|---------|-------|
| Source | Deploy from a branch |
| Branch | `main` / `docs` |
| Custom domain | `garnet-lang.org` |
| Enforce HTTPS | ON (enable once the certificate provisions; usually ~10 min) |

Add `.nojekyll` (empty file) at `docs/.nojekyll` to skip Jekyll preprocessing — the site is hand-authored HTML, no Liquid templating.

### 5.3. DNS at the registrar for `garnet-lang.org`

At the domain registrar (Cloudflare / Namecheap / wherever `garnet-lang.org` was purchased):

**Apex domain (`garnet-lang.org` → GitHub Pages):** add four A records to GitHub's Pages IPs:

```
A   @   185.199.108.153
A   @   185.199.109.153
A   @   185.199.110.153
A   @   185.199.111.153
AAAA @  2606:50c0:8000::153
AAAA @  2606:50c0:8001::153
AAAA @  2606:50c0:8002::153
AAAA @  2606:50c0:8003::153
```

**`www` subdomain:**

```
CNAME  www   islanddevcrew.github.io.
```

(CNAME apex is not RFC-legal; the A/AAAA apex + CNAME www pattern is the standard GitHub Pages recipe.)

**Optional subdomains** (future but planning-only right now):

| Subdomain | Points at | Status |
|-----------|-----------|--------|
| `docs.garnet-lang.org` | mdBook build of the language reference | v5.x roadmap (post-MIT) |
| `sh.garnet-lang.org` | CDN host serving `installer/sh.garnet-lang.org/install.sh` as the default file | pre-first-release setup; Cloudflare Workers or Netlify Functions |
| `pkg.garnet-lang.org` | apt/dnf repo for `apt install garnet` / `dnf install garnet` | v4.3 roadmap |

### 5.4. Verification after DNS propagation

```sh
dig +short garnet-lang.org           # should return the four Pages IPs
dig +short www.garnet-lang.org CNAME # should return islanddevcrew.github.io.
curl -I https://garnet-lang.org      # expect HTTP/2 200 + GitHub headers
```

Propagation is typically <1 h after DNS edits + <10 min after toggling "Enforce HTTPS".

---

## 6. Island Development Crew's parent-site build — what URLs to put on the R&D page

Copy-pasteable R&D card content for the parent site:

```html
<article class="rd-card" id="garnet">
  <img src="/r-and-d/garnet/garnet-logo.png" alt="Garnet" width="120">
  <h2>Garnet — Rust rigor. Ruby velocity. One coherent language.</h2>
  <p>
    A dual-mode, agent-native programming language platform. Managed mode feels Ruby-like;
    safe mode feels Rust-like. The mode boundary auto-bridges errors and ARC-affine semantics.
    First-class memory primitives (working / episodic / semantic / procedural) for agent
    cores. Typed actors with bounded mailboxes and Ed25519 signed hot-reload. Compiler-as-agent
    that learns from its own compilation history. Doctoral research project; seven research
    papers + four addenda; Mini-Spec v1.0 (60-plus normative pages).
  </p>
  <ul class="rd-links">
    <li><a href="https://garnet-lang.org"                                                rel="noopener">Language home</a></li>
    <li><a href="https://github.com/IslandDevCrew/garnet"                                rel="noopener">Source on GitHub</a></li>
    <li><a href="https://github.com/IslandDevCrew/garnet/releases/latest"                rel="noopener">Latest release</a></li>
    <li><a href="https://github.com/IslandDevCrew/garnet/tree/main/research/papers"      rel="noopener">Research papers</a></li>
    <li><a href="https://github.com/IslandDevCrew/garnet/tree/main/research/spec"        rel="noopener">Language specification</a></li>
  </ul>
  <pre class="install-oneliner">curl --proto '=https' --tlsv1.2 -sSf https://sh.garnet-lang.org | sh</pre>
  <p class="rd-status">Status: v4.2 research-grade. MIT submission in preparation.</p>
</article>
```

Whatever design system the parent site uses, the five links + the install one-liner are the stable wayfinding contract.

---

## 7. Initial push — `git init` through `git push origin main`

This section converts the existing Windows workspace into the initial commit on `main`.

### 7.1. Pre-flight

Working directory: `D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\E_Engineering_Artifacts`

Confirm no `.git/` exists yet:

```powershell
test-path .git   # expect: False
```

Stage a **staged copy** of the workspace in a path without spaces/parentheses so git and toolchain paths stay simple. Recommended: `C:\Users\IslandDevCrew\garnet-repo\`.

```powershell
robocopy "D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\E_Engineering_Artifacts" `
         "C:\Users\IslandDevCrew\garnet-repo" `
         /E /XD target .git `
         /XF *.log
```

Note: `/XD target` excludes cargo's build output (4-8 GB); `/XD .git` is a defensive guard (there is no `.git/` on source, but if the user has experimented this ensures a clean init).

Then copy the research corpus into the staged repo as `research/`:

```powershell
robocopy "D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\A_Research_Papers"    "C:\Users\IslandDevCrew\garnet-repo\research\papers"       /E
robocopy "D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\B_Four_Model_Consensus" "C:\Users\IslandDevCrew\garnet-repo\research\consensus"    /E
robocopy "D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\C_Language_Specification" "C:\Users\IslandDevCrew\garnet-repo\research\spec"      /E
robocopy "D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\D_Executive_and_Presentation" "C:\Users\IslandDevCrew\garnet-repo\research\presentation" /E
robocopy "D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\F_Project_Management" "C:\Users\IslandDevCrew\garnet-repo\research\management"    /E
copy    "D:\Projects\New folder\Garnet (1)\GARNET\Garnet\Opus-Gpt-Xai-Opus-Gemini-Opus\Garnet_Final\_CANONICAL_DELIVERABLES_INDEX.md" "C:\Users\IslandDevCrew\garnet-repo\research\"
```

### 7.2. `.gitignore`

Create `C:\Users\IslandDevCrew\garnet-repo\.gitignore`:

```gitignore
# Cargo build output
target/
**/target/
C:/garnet-build/target/

# Host-specific cargo config (this workspace pins target=x86_64-pc-windows-msvc which leaks
# into Linux bind-mounts; consider moving the pin to the host ~/.cargo/ and gitignoring the
# workspace-local one entirely).
# .cargo/config.toml      <- DO NOT BLANKET-GITIGNORE; we want the workspace config, just not
#                            host-specific pins. Keep committed but see §7.4.

# IDE / editor
.vscode/
.idea/
*.swp
*.swo

# Backups
*.bak
*.orig

# Signing keys (MUST NOT be committed)
*.pfx
*.p12
*.key
garnet-release-signing.key

# Local build artifacts
**/*.rs.bk
Cargo.lock.bak

# OS cruft
.DS_Store
Thumbs.db

# Local secrets
.env
.env.local
```

### 7.3. `.gitattributes`

Create `C:\Users\IslandDevCrew\garnet-repo\.gitattributes`:

```gitattributes
# Auto-detect text files, normalize to LF on commit
* text=auto eol=lf

# Explicit LF for code/config
*.rs   text eol=lf
*.toml text eol=lf
*.md   text eol=lf
*.sh   text eol=lf
*.yml  text eol=lf
*.yaml text eol=lf
*.garnet text eol=lf
*.wxs  text eol=lf
Makefile text eol=lf

# Explicit CRLF for Windows-only files
*.cmd  text eol=crlf
*.bat  text eol=crlf
*.ps1  text eol=crlf
*.wsb  text eol=crlf

# Binary — never try to diff or normalize
*.png  binary
*.jpg  binary
*.ico  binary
*.bmp  binary
*.pdf  binary
*.docx binary
*.pptx binary
*.xlsx binary
*.rtf  binary
*.msi  binary
*.deb  binary
*.rpm  binary
*.pkg  binary
*.zip  binary
*.tar  binary
*.gz   binary
```

### 7.4. Cargo config hygiene

The existing `E_Engineering_Artifacts/.cargo/config.toml` pins:

```toml
[build]
target = "x86_64-pc-windows-msvc"
target-dir = "C:/garnet-build/target"
```

For the repo, this is wrong in two ways:
1. `target = "x86_64-pc-windows-msvc"` breaks Linux CI — documented in the Phase 6D Linux handoff.
2. `target-dir = "C:/garnet-build/target"` is host-specific.

Recommended action **before first commit**: edit the workspace `.cargo/config.toml` down to nothing host-specific — i.e. delete it entirely — and move both pins to the user's `~/.cargo/config.toml` (global, not committed). GitHub Actions' Linux + macOS runners pick up the natural target; Windows developers set their own global cargo config.

Alternative (if keeping the file committed): replace both pins with a commented example and document the "move to global" guidance in `WORKSPACE_README.md`.

### 7.5. The command sequence

From `C:\Users\IslandDevCrew\garnet-repo\`:

```bash
# 1. Initialize the repo and set the default branch to main (GitHub's modern default).
git init -b main

# 2. Configure identity for this repo if not globally set.
git config user.name  "Island Development Crew"
git config user.email "jon@islanddevcrew.com"

# 3. Rewrite Garnet_Final/... references in research/management/*.md and README.md.
#    Run once. Commit the results as part of the same initial commit.
find research/management README.md FAQ.md WORKSPACE_README.md -type f -name '*.md' -print0 \
  | xargs -0 sed -i \
      -e 's|Garnet_Final/A_Research_Papers/|research/papers/|g' \
      -e 's|Garnet_Final/B_Four_Model_Consensus/|research/consensus/|g' \
      -e 's|Garnet_Final/C_Language_Specification/|research/spec/|g' \
      -e 's|Garnet_Final/D_Executive_and_Presentation/|research/presentation/|g' \
      -e 's|Garnet_Final/E_Engineering_Artifacts/||g' \
      -e 's|Garnet_Final/F_Project_Management/|research/management/|g' \
      -e 's|../A_Research_Papers/|research/papers/|g' \
      -e 's|../C_Language_Specification/|research/spec/|g' \
      -e 's|../D_Executive_and_Presentation/|research/presentation/|g' \
      -e 's|../F_Project_Management/|research/management/|g'

# 4. Stage everything respecting .gitignore.
git add -A

# 5. Sanity-check what's staged — should be many thousand files, no target/, no *.pfx.
git status --short | head -n 40
git status --short | wc -l

# 6. The initial commit.
git commit -m "Initial commit: Garnet v4.2 (Stage 6 close)

Canonical repo layout for github.com/IslandDevCrew/garnet. Carries:
- 8 Rust crates (parser, interp, check, memory, actor-runtime, stdlib, cli, convert)
- 10 MVP examples + 3 real-world
- research/ corpus: 7 papers + 4 addenda, Mini-Spec v1.0, 11 spec docs,
  4-model consensus, DX comparative paper + deck, every v3.x-v4.2 handoff
- docs/ landing site + CNAME for garnet-lang.org
- installer/ universal shell installer
- .github/workflows/ Linux packaging + Release upload on v* tags
- dist/ Linux .deb + .rpm + Windows .msi built in Phase 6D

Verification status at this commit:
- Linux .deb + .rpm: VERIFIED end-to-end in Docker
- Windows binary + .msi: VERIFIED on real Windows (MSVC)
- Windows .msi signing + macOS .pkg: pending user credentials

Cumulative tests: 1244 across workspace.
"

# 7. Create the remote and push.
#    Precondition: the GitHub repo github.com/IslandDevCrew/garnet exists
#    (create via 'gh repo create IslandDevCrew/garnet --public --source=.
#    --description="Rust rigor. Ruby velocity. One coherent language." --homepage=https://garnet-lang.org')
#    OR via the web UI at https://github.com/new.

git remote add origin git@github.com:IslandDevCrew/garnet.git
git push -u origin main
```

Expected push size (after `target/` exclusion + repo-root consolidation): **~150-250 MB** — mostly the `.docx` / `.pptx` / `.pdf` research deliverables and the pre-built installers in `dist/`. If GitHub rejects individual files > 100 MB, the offender is almost always a `.pkg` / `.msi` built from an earlier session; remove from `dist/` and let the Releases upload carry those assets.

### 7.6. Post-push tasks

1. **GitHub → Settings → Pages** — set source to `main`/`docs`, custom domain `garnet-lang.org`, wait for cert, toggle HTTPS enforcement.
2. **GitHub → Settings → General** — confirm repo is public, description + website are populated, topics added (`programming-language`, `rust`, `ruby`, `compiler`, `language-design`, `actor-model`, `agent-native`, `research`).
3. **GitHub → Settings → Actions → General** — allow Actions; the `.github/workflows/linux-packages.yml` workflow runs on push and validates.
4. **DNS at the registrar** — publish the A/AAAA/CNAME records from §5.3.
5. **Parent site** — push the R&D card content from §6 to `islanddevcrew.com/r-and-d/garnet/`.
6. **First tagged release** — once macOS `.pkg` closes + Windows `.msi` is signed:

```bash
# Update research/management/CHANGELOG.md entry for v4.2 pointing at all five Phase 6 handoffs.
git add research/management/CHANGELOG.md CHANGELOG.md
git commit -m "CHANGELOG: v4.2 — Stage 6 close"
git tag -a v0.4.2 -m "Garnet v0.4.2 — Stage 6 (cross-platform installers + branding)"
git push origin main
git push origin v0.4.2    # triggers the Release workflow; auto-uploads .deb + .rpm + SHA256SUMS
# Manually attach the signed .msi + signed+notarized .pkg to the Release on GitHub.
```

---

## 8. Branching strategy after initial commit

Short version: **trunk-based, with short-lived feature branches.**

- `main` is always green under the GHA workflow. No merging if CI is red.
- Feature work happens on branches named `topic/<short-description>` or `fix/<short-description>`.
- PRs require at least 1 approving review (self-review ok while project is solo) and CI green.
- Tagged releases are cut from `main` at the point the GHA workflow + manual cross-platform verification both pass.

The repo is currently single-maintainer; the branch protection rules can be relaxed. But the CI-green and self-review gates should be enforced as habit — they are the only mechanism catching platform-specific regressions between the Linux CI and the Windows/macOS local verifications.

---

## 9. What lives where — reviewer's wayfinding table

| A reviewer asks… | Take them to… |
|------------------|---------------|
| "What is Garnet?" | `README.md` (repo root) |
| "Can I install it?" | `README.md` §Install → `installer/sh.garnet-lang.org/install.sh` |
| "What's the language spec?" | `research/spec/GARNET_v1_0_Mini_Spec.md` |
| "Where are the research papers?" | `research/papers/` |
| "How do I verify a signed release?" | `SECURITY.md` + `garnet-cli/src/manifest.rs` + `garnet verify --signature` |
| "What did v4.2 add?" | `research/management/GARNET_v4_2_HANDOFF.md` + this file |
| "Is it tested on Linux?" | `research/management/GARNET_v4_2_Phase_6D_Linux_VERIFIED.md` |
| "Is it tested on Windows?" | `research/management/GARNET_v4_2_Phase_6D_Windows_VERIFIED.md` |
| "How do I migrate Ruby code?" | `research/spec/GARNET_Migration_Guide_Ruby_Python.md` + `garnet convert ruby foo.rb` |
| "Where do I file a bug?" | `.github/ISSUE_TEMPLATE/bug_report.md` |
| "Where do I report a security issue?" | `SECURITY.md` (private GitHub Security Advisory) |
| "Can I contribute?" | `CONTRIBUTING.md` + `CODE_OF_CONDUCT.md` |
| "What's the roadmap?" | `research/management/GARNET_v4_2_HANDOFF.md` §v4.2 → POST-MIT ROADMAP |
| "Where's the website source?" | `docs/` |

---

## 10. Open questions for the maintainer

1. **Research papers as `.docx` / `.pptx` in the repo vs. PDF-only.** GitHub renders `.md` + `.pdf` natively. `.docx` and `.pptx` must be downloaded to open. Recommend: keep the editable sources in-repo (reviewers expect access to originals), but ensure a PDF companion ships alongside every `.docx` / `.pptx` so in-browser review works. Currently only Paper III has all three (`.md`, `.docx`, `.pdf`); Papers I, II, IV, V have only `.docx`. Generating PDFs from those is a one-time pandoc pass that can precede or follow the first tagged release.

2. **`dist/` in the repo vs. Releases-only.** The current design commits pre-built `.deb` / `.rpm` / `.msi` in `dist/` for audit trail. GitHub Releases are authoritative; the in-repo copies provide an always-accessible download path for reviewers who cannot reach Releases (rare). Tradeoff: ~5-8 MB added to every clone. Recommend: keep `dist/` in `main` for v4.2 only, then switch to Release-only for v4.3+.

3. **Whether to promote the research papers out of the code repo into a sibling `IslandDevCrew/garnet-research` repo.** Clean separation; tidier code repo; two-repo cognitive overhead. Recommend: keep together for MIT submission (single-URL reviewer experience), re-evaluate after v0.4.2 is tagged and cited externally.

---

*"The plans of the diligent lead surely to abundance." — Proverbs 21:5*

*Written by Claude Opus 4.7 (1M) at the v4.2 repo-layout design close — 2026-04-21.*
