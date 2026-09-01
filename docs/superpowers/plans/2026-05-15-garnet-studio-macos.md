# Garnet Studio macOS Workbench Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Build the first installable Garnet Studio macOS workbench so a user can open an app, verify the bundled Garnet CLI, run examples, launch conversion, and inspect release readiness without starting in Terminal.

**Architecture:** Add a SwiftPM-based SwiftUI macOS app under `apps/garnet-studio-macos` and a repository packaging script under `scripts/`. The app shells out only to a bundled `garnet` resource or a discovered `garnet` executable, writes temporary example/conversion files under the user temp directory, and keeps signing/notarization separate from local `.app`/`.dmg` packaging.

**Tech Stack:** Swift 6.3, SwiftUI/AppKit on macOS 14+, Cargo-built `garnet-cli`, shell packaging with `swift build`, `cargo build`, `codesign --sign -`, and `hdiutil`.

---

## File Structure

- Create `apps/garnet-studio-macos/Package.swift`: SwiftPM package for a macOS executable named `GarnetStudio`.
- Create `apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift`: SwiftUI app, view models, command runner, sample workflows, and workbench UI.
- Create `apps/garnet-studio-macos/Sources/GarnetStudio/Resources/garnet-logo.png`: copy of the existing Garnet logo for bundled resources.
- Create `apps/garnet-studio-macos/Tests/GarnetStudioTests/GarnetStudioTests.swift`: XCTest coverage for command selection, sample catalog, and output classification. The app also keeps a `--self-test` smoke for packaged-bundle verification.
- Create `scripts/package_garnet_studio_macos.sh`: build `garnet`, build the Swift app, assemble `Garnet Studio.app`, ad-hoc sign if possible, and create a local `.dmg`.
- Modify `CURRENT_STATE.md`: add a short current-truth note that Garnet Studio is local-packaged but not notarized.
- Modify `F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md`: add the app-distribution lane as a post-v0.4.2 productization checkpoint.

## Task 1: SwiftPM App Scaffold

**Files:**
- Create: `apps/garnet-studio-macos/Package.swift`
- Create: `apps/garnet-studio-macos/Sources/GarnetStudio/Resources/garnet-logo.png`
- Create: `apps/garnet-studio-macos/Tests/GarnetStudioTests/GarnetStudioTests.swift`

- [x] **Step 1: Create the SwiftPM package**

Add a package with one executable and one test target. Require macOS 14 to keep SwiftUI APIs conservative for current macOS.

- [x] **Step 2: Copy the Garnet logo**

Copy `assets/garnet-logo.png` into the app resources path so the package can embed it without depending on repository-relative paths at runtime.

- [x] **Step 3: Add model tests first**

Write tests for:
- bundled CLI path preference
- fallback CLI path list
- sample catalog contains parse/check/run/convert-facing examples
- command output status maps zero exit to success and nonzero exit to failure

- [x] **Step 4: Run tests to verify red**

Run: `swift test --package-path apps/garnet-studio-macos`
Expected before implementation: compile failure because the app model types are not implemented.

## Task 2: Native Workbench UI and Command Runner

**Files:**
- Create: `apps/garnet-studio-macos/Sources/GarnetStudio/GarnetStudioApp.swift`

- [x] **Step 1: Implement the model types**

Define:
- `GarnetCommandResult`
- `GarnetSample`
- `GarnetSampleCatalog`
- `GarnetCLI`
- `GarnetStudioViewModel`

- [x] **Step 2: Implement safe process execution**

Use `Process`, `Pipe`, temporary files, bounded output display, and no shell interpolation. Commands must pass argument arrays directly to the `garnet` executable.

- [x] **Step 3: Implement a first-run workbench UI**

The UI must open directly into a useful workbench:
- sidebar with `Overview`, `Examples`, `Converter`, `Release`
- CLI health card
- first-run onboarding checklist
- sample selector
- run/check/parse buttons
- converter language picker and source editor
- release/install evidence panel
- output console

- [x] **Step 4: Run tests and build**

Run:
`swift test --package-path apps/garnet-studio-macos`
`swift run --package-path apps/garnet-studio-macos GarnetStudio --self-test`
`swift build --package-path apps/garnet-studio-macos`

Expected: both pass.

## Task 3: Local `.app` and `.dmg` Packaging

**Files:**
- Create: `scripts/package_garnet_studio_macos.sh`

- [x] **Step 1: Build the CLI and Swift executable**

Run Cargo release build for `garnet-cli` and Swift release build for `GarnetStudio`.

- [x] **Step 2: Assemble app bundle**

Create:
`target/macos/Garnet Studio.app/Contents/MacOS/GarnetStudio`
`target/macos/Garnet Studio.app/Contents/Resources/garnet`
`target/macos/Garnet Studio.app/Contents/Resources/garnet-logo.png`
`target/macos/Garnet Studio.app/Contents/Info.plist`

- [x] **Step 3: Ad-hoc sign and create DMG**

Run `codesign --force --deep --sign -` when available. Create `target/macos/GarnetStudio.dmg` with `hdiutil create`.

- [x] **Step 4: Smoke the bundle**

Run:
`target/macos/Garnet Studio.app/Contents/Resources/garnet --version`
`target/macos/Garnet Studio.app/Contents/MacOS/GarnetStudio --self-test`
`target/macos/Garnet Studio.app/Contents/MacOS/GarnetStudio --smoke-test`
`hdiutil verify target/macos/GarnetStudio.dmg`

Expected: CLI version includes `garnet 0.4.2`, app self-test passes, app smoke runs version plus parse/check/run/convert samples through the bundled CLI, and DMG verifies.

## Task 4: Documentation and Dogfood Evidence

**Files:**
- Modify: `CURRENT_STATE.md`
- Modify: `F_Project_Management/DOGFOOD/GARNET_v0_5_DOGFOOD_READINESS_PHASE_LOG.md`

- [x] **Step 1: Update status docs against current truth**

Document that Garnet Studio is a local macOS workbench and local-packaged `.app`/`.dmg` path. Do not claim Developer ID signing, notarization, App Store distribution, iOS, Android, or web/PWA completion.

- [x] **Step 2: Create Desktop dogfood bundle**

Create a timestamped folder under `/Users/idc2.0/Desktop/dogfood` with build logs, Swift test logs, packaging logs, DMG verification, release status, readiness status, and manifest.

- [x] **Step 3: Run verification ladder**

Run:
`swift test --package-path apps/garnet-studio-macos`
`swift run --package-path apps/garnet-studio-macos GarnetStudio --self-test`
`swift build --package-path apps/garnet-studio-macos -c release`
`./scripts/package_garnet_studio_macos.sh`
`cargo fmt --all -- --check`
`git diff --check`
`python3 scripts/test_garnet_readiness_status.py`

Expected: all pass, or failures are fixed before PR.

## Self-Review

- Spec coverage: this plan covers the first macOS app/install surface, core Garnet workflows, local packaging, Linear tracking, and dogfood evidence. It intentionally defers mobile/web/video until the app workbench is real and verified.
- Placeholder scan: no `TBD`, `TODO`, or unspecified implementation steps remain.
- Type consistency: model names are consistent across tests, app implementation, and packaging smoke.
