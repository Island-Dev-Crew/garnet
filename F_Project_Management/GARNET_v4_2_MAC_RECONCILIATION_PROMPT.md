# GARNET v4.2 — Mac Reconciliation Prompt

**Copy the block between the two rulers verbatim into a fresh Claude Code session on the Mac.**
This is the operator-facing prompt; Claude Code will execute it when pasted.

---

You are Claude Code running on my Mac. Two folders on my desktop (or nearby) hold two parallel working states of the Garnet language project, a doctoral research codebase I am preparing for MIT submission. I need you to reconcile them into a single authoritative tree, verify the merged workspace end-to-end, and then walk me through cutting the v0.4.2 tag + release.

Do not guess. Do not invent files. Work from what exists on disk. Ask me before any destructive action.

### The two trees

- `~/Desktop/Garnet Opus 4.7` — the Mac-side work-in-progress. I've been editing here while the Windows build was underway. Contains potentially new docs, bugfixes, or notes that DO NOT exist in the final Windows transfer.
- `~/Desktop/Garnet Opus 4.7 final` — the authoritative Windows transfer. This is where the v4.2 Stage 6 work closed (Phase 6A installer scaffolding, Phase 6B `garnet new`, Phase 6C branding, Phase 6D Linux + Windows verification, the MSI build).

If either folder is not at `~/Desktop/`, ask me where it is before proceeding. Do not crawl the whole disk.

### Numbered checklist (track each visually; report back as each completes)

1. **Locate both trees.** `ls ~/Desktop/"Garnet Opus 4.7"` and `ls ~/Desktop/"Garnet Opus 4.7 final"`. If either is missing, ask me. Do not proceed past step 1 until both resolve.

2. **Read the authoritative handoff first.** Read `~/Desktop/Garnet Opus 4.7 final/Garnet_Final/F_Project_Management/GARNET_v4_2_HANDOFF.md` and `GARNET_v4_2_COMPLETE_PROJECT_STATE.md` cover-to-cover. This is your ground truth for what the project is and what Stage 6 shipped. Then read `GARNET_v4_2_Phase_6D_Linux_VERIFIED.md` and `GARNET_v4_2_Phase_6D_Windows_VERIFIED.md` for the verification-gate shape you will reproduce on macOS.

3. **Produce a recursive file-by-file diff.** Run:
   ```sh
   cd ~/Desktop
   diff -r -q "Garnet Opus 4.7" "Garnet Opus 4.7 final" > reconciliation-diff.txt 2>&1
   wc -l reconciliation-diff.txt
   ```
   Save `reconciliation-diff.txt` at `~/Desktop/reconciliation-diff.txt`. Keep it human-readable. If the diff is over ~2000 lines, additionally produce a **summary file** `~/Desktop/reconciliation-summary.md` grouping differences by top-level directory.

4. **Categorize every difference into three buckets.** Read each diff line and classify:
   - **(a) Mac-only additions to merge FORWARD into final.** Files present in the Mac tree but not in `final/`. Docs I wrote while waiting, new notes, corrections. Merge target: copy into the corresponding path inside `Garnet Opus 4.7 final`.
   - **(b) Final-only authoritative content.** Files present in `final/` but not the Mac tree. These are the Windows-session outputs — Phase 6D handoffs, the built MSI, branding assets, the new three deliverables (`GARNET_v4_2_GITHUB_REPO_LAYOUT.md`, `GARNET_v4_2_MAC_RECONCILIATION_PROMPT.md`, `GARNET_v4_2_COMPLETE_PROJECT_STATE.md`). Authoritative; nothing to do (they already live in `final/`).
   - **(c) Both-modified — both files exist but differ.** This category needs judgment. Do not silently choose a winner. Produce a list at `~/Desktop/reconciliation-needs-review.md` — one entry per file, with the relative path, file sizes + mtimes for both sides, and a first-10-line head of each. Stop and ask me before merging any (c) file.

5. **Apply (a) automatically. Report (c) for review.**
   - For each (a) file: copy it into the corresponding path inside `~/Desktop/Garnet Opus 4.7 final/`. Use `cp -a` to preserve timestamps. Log every copy to `~/Desktop/reconciliation-applied.log`.
   - For each (c) file: do nothing yet. Present the list to me; I will call out which side wins file-by-file, or instruct you to read the diffs and recommend per-file.

6. **After I sign off on the (c) resolutions, declare `~/Desktop/Garnet Opus 4.7 final/` as THE tree.** Rename or ignore `~/Desktop/Garnet Opus 4.7/` — do not delete it; leave it as a safety backup until the v0.4.2 tag is pushed.

7. **Run the verification ladder on the reconciled tree:**
   ```sh
   cd ~/Desktop/"Garnet Opus 4.7 final"/Garnet_Final/E_Engineering_Artifacts
   cargo check --workspace --tests                     # must be clean
   cargo test -p garnet-actor-runtime --release --lib  # 17 pass
   cargo test -p garnet-stdlib        --release        # 74 pass
   cargo test -p garnet-convert       --release        # 85 pass (61 + 24)
   ```
   If any of the four fails, STOP and report the failure verbatim. Do not attempt to fix Rust source code unless the failure is clearly environmental (missing toolchain component, path issue). This is a doctoral-grade codebase; every test counted in the cumulative 1244 was green in prior sessions.

8. **Build the macOS `.pkg`** per `Garnet_Final/F_Project_Management/GARNET_v4_2_Phase_6A_HANDOFF.md` §macOS and `garnet-cli/macos/README.md`. I have Apple Developer ID credentials and a notarytool keychain profile set up.
   ```sh
   # I'll set these before you run the build; verify they're populated.
   echo "APPLE_DEV_ID_INSTALLER=$APPLE_DEV_ID_INSTALLER"
   echo "APPLE_DEV_ID_APP=$APPLE_DEV_ID_APP"
   echo "APPLE_NOTARY_PROFILE=$APPLE_NOTARY_PROFILE"
   # If any are empty, ask me to set them before continuing.

   xcode-select -p                                     # must print a valid Xcode/CLT path
   cd Garnet_Final/E_Engineering_Artifacts/garnet-cli
   ./macos/build-pkg.sh
   ```
   Expected output: `target/macos/garnet-0.4.2-universal.pkg` (or similar; the script is authoritative on the exact filename).

9. **Verify the `.pkg`:**
   ```sh
   PKG="target/macos/garnet-0.4.2-universal.pkg"     # adjust to actual filename
   pkgutil --check-signature "$PKG"                   # expect: status: signed by a developer certificate issued by Apple
   spctl --assess --type install "$PKG"               # expect: accepted
   xcrun stapler validate "$PKG"                      # expect: The validate action worked!
   shasum -a 256 "$PKG"                               # capture for the Release notes
   ```
   All three commands must succeed. If Gatekeeper or the notary service rejects anything, STOP and report the exact message — do not attempt to re-sign or bypass.

10. **Smoke-test the installed binary.** On a sandboxed user account or a freshly-privileged macOS environment:
    ```sh
    sudo installer -pkg "$PKG" -target /
    garnet --version                                   # wordmark + 6-crate version table
    garnet new --template cli /tmp/macos-smoke
    cd /tmp/macos-smoke
    garnet test                                        # 2 starter tests pass
    garnet keygen /tmp/test.key
    garnet build --deterministic --sign /tmp/test.key src/main.garnet
    garnet verify src/main.garnet src/main.garnet.manifest.json --signature
    ```
    Every step must print success. If any fails, capture stderr and stop.

11. **Write `GARNET_v4_2_Phase_6D_macOS_VERIFIED.md`** at `Garnet_Final/F_Project_Management/`, modeled exactly on the structure of `GARNET_v4_2_Phase_6D_Linux_VERIFIED.md` and `GARNET_v4_2_Phase_6D_Windows_VERIFIED.md`. Include: executive summary, the three signature/notary/stapler outputs verbatim, the 6 binary-level gates with PASS/FAIL, SHA-256 of the .pkg, reproduce-the-build command, known issues (if any).

12. **Update `Garnet_Final/F_Project_Management/GARNET_v4_2_HANDOFF.md`** — change the macOS Phase 6D status row from ⏳ to ✅ and add a one-line pointer to the new macOS VERIFIED doc.

13. **Cut the v0.4.2 release.** Follow `Garnet_Final/F_Project_Management/GARNET_v4_2_HANDOFF.md` §PHASE 6E. The abbreviated sequence:
    ```sh
    # Still inside E_Engineering_Artifacts/ if the repo lives there; otherwise at the repo root.
    # Update CHANGELOG.md with the v4.2 entry citing: v4_2_HANDOFF, v3_4_1_HANDOFF, Phase_6A,
    # Phase_6D_Linux_VERIFIED, Phase_6D_Windows_VERIFIED, Phase_6D_macOS_VERIFIED.
    git add -A
    git commit -m "v4.2: Stage 6 close — macOS verified"
    git tag -a v0.4.2 -m "Garnet v0.4.2 — Stage 6 (cross-platform installers + branding)"
    git push origin main
    git push origin v0.4.2   # triggers the Linux-packages GHA workflow + Release upload
    ```

14. **Attach the macOS `.pkg` + Windows `.msi` to the GitHub Release manually.**
    - The Linux workflow auto-attaches `.deb` + `.rpm` + `SHA256SUMS`.
    - You attach `target/macos/garnet-0.4.2-universal.pkg` (just built) and `dist/windows/garnet-0.4.2-x86_64.msi` (from the Windows transfer; SHA-256 `564d302fbaa3d05b16f77dd9d862972cceaed30132994997056f6e82e2d379c4`).
    - Regenerate `SHA256SUMS` to include the .pkg and .msi alongside the .deb + .rpm.

15. **Final report back to me.** Produce a single summary message covering: which (a)/(b)/(c) counts you resolved, the four verification-ladder gate results, the three macOS signature/notary/stapler outputs, the .pkg SHA-256, the 6-gate binary smoke results, the tag-push confirmation, and any anomalies that should block or delay announcing the release.

### Operating rules

- **No destructive actions without asking.** No `rm -rf`, no overwriting files in place without confirmation from me, no `git push --force`, no reset --hard.
- **Read before you edit.** Every handoff in `F_Project_Management/` carries context you cannot reconstruct.
- **Absolute paths in all shell commands.** Spaces in the folder names (`Garnet Opus 4.7`) matter; always quote them.
- **Don't fabricate test counts or SHAs.** If a command's output is surprising, report it verbatim; don't paraphrase.
- **Cite your sources.** When you make a claim about what the project is or what v4.2 shipped, quote the handoff doc and cite its path.
- **If you're stuck, stop and ask.** One blocked step + a clear question is infinitely more useful than five steps of speculation.

Ready? Start at step 1.

---

*(End of prompt to paste.)*

---

## Notes for Jon (not part of the prompt)

- Before pasting, make sure the two desktop folders are named exactly `Garnet Opus 4.7` and `Garnet Opus 4.7 final`. If they're different, either rename them or edit the prompt before pasting.
- The Apple Developer ID + notarytool setup is a one-time prerequisite. If you haven't already:
  ```sh
  xcode-select --install
  xcrun notarytool store-credentials --apple-id <your Apple ID> --team-id <TEAMID>
  ```
  Then set `APPLE_DEV_ID_INSTALLER`, `APPLE_DEV_ID_APP`, `APPLE_NOTARY_PROFILE` in the shell before step 8.
- The GitHub repo `github.com/IslandDevCrew/garnet` should already exist (empty) before step 13's `git push` can succeed. If not, run `gh repo create IslandDevCrew/garnet --public` from inside the reconciled tree first.
- Keep the `Garnet Opus 4.7` backup folder around at least through v0.4.3 — it's your only safety net if anything from the reconciliation was dropped.

*Written by Claude Opus 4.7 (1M) — 2026-04-21.*
