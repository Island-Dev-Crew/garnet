# Garnet Fleet Report Template

Purpose: each machine/agent pair writes one report so Garnet can consolidate
local drift, proof artifacts, branch state, and machine-specific evidence into a
single repo-visible source of truth before the S131-S200 runway accelerates.

Recommended filename:

```text
F_Project_Management/FLEET_REPORTS/YYYY-MM-DD_<machine>_<agent>.md
```

Examples:

```text
F_Project_Management/FLEET_REPORTS/2026-06-10_macbook-pro_claude-fable.md
F_Project_Management/FLEET_REPORTS/2026-06-10_windows-nuc_codex.md
```

Do not include secrets, private keys, tokens, passwords, full API keys, private
SSH material, or unreduced personal data.

Transport rule: write this report on a dedicated branch named
`fleet/2026-06-10-<machine>-<agent>` and open no PR. Push the branch only if the
MacBook Pro consolidation lane needs to fetch it. The consolidation lane creates
the single S131-S134 source-truth PR.

## Report Header

- Machine:
- Agent/model:
- Date/time:
- OS:
- Hardware:
- Repo path:
- Active user/account:
- Network state:
- Report scope:

## Repo Truth

```sh
git remote -v
git status --short --branch
git rev-parse HEAD
git rev-parse origin/main
git log --oneline -8
git tag --points-at HEAD
git show-ref --tags v0.8.1 || true
gh auth status
gh pr list --repo Island-Dev-Crew/garnet --state open --json number,title,headRefName,author,updatedAt,url
gh release view v0.8.1 --repo Island-Dev-Crew/garnet --json tagName,name,isDraft,isPrerelease,publishedAt,url,targetCommitish
```

Paste summarized results here:

- Current branch:
- HEAD:
- origin/main:
- v0.8.1 tag target:
- Open PRs:
- Release state:
- Local untracked files:
- Local modified files:

## Toolchain Truth

```sh
rustc --version
cargo --version
rustup show active-toolchain
node --version || true
npm --version || true
python3 --version
gh --version | head -1
gpg --version | head -1
java --version 2>&1 | head -5 || true
```

For Mac machines, also report:

```sh
xcodebuild -version 2>/dev/null || true
system_profiler SPHardwareDataType | rg "Model Name|Chip|Memory|Serial" || true
```

For Windows machines, also report:

```powershell
systeminfo | findstr /B /C:"OS Name" /C:"OS Version" /C:"System Type" /C:"Total Physical Memory"
wsl --status
wsl -l -v
```

Paste summarized results here:

- Rust:
- Node/npm:
- Python:
- GitHub CLI:
- GPG:
- Java:
- Tauri/Xcode/Windows/WSL/UTM notes:

## Garnet Verification Snapshot

Run only the checks appropriate for this machine and report exact pass/fail.
If a command is too slow for the lane, say not run and why.

```sh
python3 scripts/check-agent-contracts.py
python3 scripts/garnet_readiness_status.py --format json
python3 scripts/garnet_mit_readiness_status.py --format json
git diff --check
```

Optional when this lane owns implementation:

```sh
cargo fmt --all -- --check
cargo test --workspace --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
```

Paste summarized results here:

- Agent contracts:
- Tracked readiness:
- MIT/productization readiness:
- Diff check:
- Cargo fmt:
- Cargo test:
- Cargo clippy:
- Dogfood-readiness status:

## Local Evidence Inventory

List only real local artifacts that may matter to the source-of-truth merge.
Search beyond the current checkout. Each machine may have built or tested from
multiple repo paths, temporary folders, downloads, or tool-specific workspaces.
Timebox this sweep to 90 minutes per machine unless Jon explicitly extends it.
Record exact roots and patterns searched; do not imply an exhaustive disk audit.

Suggested macOS/Linux sweep:

```sh
find "$HOME/Desktop" "$HOME/Documents" "$HOME/Downloads" /tmp -maxdepth 5 \
  \( -iname '*garnet*' -o -iname '*.vsix' -o -iname '*.tar.gz' -o -iname '*.deb' -o -iname '*.rpm' \) \
  2>/dev/null | sort | head -300
```

Suggested Windows PowerShell sweep:

```powershell
$roots = @("$env:USERPROFILE\Desktop", "$env:USERPROFILE\Documents", "$env:USERPROFILE\Downloads", "$env:TEMP")
Get-ChildItem $roots -Recurse -ErrorAction SilentlyContinue |
  Where-Object { $_.Name -match "garnet|\.vsix$|\.tar\.gz$|\.deb$|\.rpm$|tauri|studio" } |
  Select-Object FullName, Length, LastWriteTime |
  Sort-Object FullName |
  Select-Object -First 300
```

For every local Garnet checkout found, also record:

```sh
git -C <path> status --short --branch
git -C <path> remote -v
git -C <path> log --oneline -5
git -C <path> branch --all --verbose
git -C <path> ls-files --others --exclude-standard | sed -n '1,200p'
```

Do not paste secrets. If an artifact may contain secrets, record only a safe
path hash, redacted filename, and "unsafe until reviewed."

- Desktop dogfood bundles:
- Proof directories:
- Screenshots/videos:
- Release/package artifacts:
- UTM/WSL/VM outputs:
- Claude/Codex handoff docs:
- Relevant downloads:
- Relevant temporary files worth preserving:

For each item:

```text
Path:
Purpose:
Created by:
Date:
Safe to commit? yes/no/unknown
Contains source? yes/no/unknown
Contains secrets? yes/no/unknown
Recommended action:
```

Use one of these verdicts for each item:

- commit: durable project state that should enter the repo.
- archive: evidence worth preserving outside git.
- ignore: generated, duplicate, stale, or not useful.
- duplicate: already represented on `origin/main` or another report.
- unsafe: may contain secrets/private data; do not copy into repo.
- needs Jon: release, tag, signing, credential, or public decision required.

## Drift And Conflicts

- Docs that disagree with current repo truth:
- Release assets or package names that disagree with current version:
- Local files not present on main:
- Main files missing locally:
- Machine-specific evidence that should not be scored globally:
- Claims that need independent verification:

## S129-S200 Recommendations From This Machine

Rank the top local recommendations.

1.
2.
3.
4.
5.

## Safe Next Action

State exactly one recommended next action for this machine.

- Continue:
- Stop:
- Needs Jon:
- Needs another machine:
- Needs independent reviewer:

## Claim Boundaries

Use exact language.

- What this report proves:
- What this report does not prove:
- Any "enforced" claims and their deterministic trap:
- Any self-authored/self-graded claims that need independent review:
