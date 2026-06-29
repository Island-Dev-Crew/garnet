import XCTest
@testable import GarnetStudioKit

/// Behavior tests for the macOS port of the PR #391 shell standard.
/// Static/source-level parity is gated by scripts/test_garnet_macos_studio_shell.py;
/// these tests prove the runtime behavior of each ported row.
final class StudioShellTests: XCTestCase {

    // MARK: Row 1 · Version truth

    func testVersionStampIsSemverShaped() {
        let parts = StudioVersion.release.split(separator: ".")
        XCTAssertEqual(parts.count, 3, "version stamp must be MAJOR.MINOR.PATCH")
        for part in parts {
            XCTAssertNotNil(Int(part), "version component \(part) must be numeric")
        }
        XCTAssertNotEqual(StudioVersion.release, "0.1.0", "the 0.1.0 stamp drift must not return")
    }

    // MARK: Row 4 · Validated settings

    func testSettingsNormalizationClampsTimeouts() {
        let wild = StudioSettings(mode: .power, theme: .dark, commandTimeoutSecs: 999_999, matrixTimeoutSecs: 1)
        let normalized = wild.normalized()
        XCTAssertEqual(normalized.commandTimeoutSecs, StudioSettings.commandTimeoutRange.upperBound)
        XCTAssertEqual(normalized.matrixTimeoutSecs, StudioSettings.matrixTimeoutRange.lowerBound)
        XCTAssertEqual(normalized.mode, .power)
        XCTAssertEqual(normalized.theme, .dark)
    }

    func testCorruptSettingsFileNeverBlocksBoot() {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent("garnet-studio-tests-\(UUID().uuidString)", isDirectory: true)
        let file = dir.appendingPathComponent("settings.json")
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        try? Data("{not valid json at all".utf8).write(to: file)

        let store = StudioSettingsStore(fileURL: file)
        XCTAssertEqual(store.load(), .defaults, "corrupt settings must degrade to defaults")

        try? Data(#"{"mode":"warp-speed","theme":"dark","commandTimeoutSecs":2}"#.utf8).write(to: file)
        let tolerated = store.load()
        XCTAssertEqual(tolerated.mode, .simple, "unknown mode falls back to default")
        XCTAssertEqual(tolerated.theme, .dark, "valid fields survive a partially bad file")
        XCTAssertEqual(
            tolerated.commandTimeoutSecs,
            StudioSettings.commandTimeoutRange.lowerBound,
            "out-of-range timeout clamps, not errors"
        )
    }

    func testSettingsRoundTripPersistsNormalizedValues() {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent("garnet-studio-tests-\(UUID().uuidString)", isDirectory: true)
        let store = StudioSettingsStore(fileURL: dir.appendingPathComponent("settings.json"))
        let saved = store.save(StudioSettings(mode: .power, theme: .light, commandTimeoutSecs: 0, matrixTimeoutSecs: 99_999))
        XCTAssertTrue(saved)
        let loaded = store.load()
        XCTAssertEqual(loaded.mode, .power)
        XCTAssertEqual(loaded.commandTimeoutSecs, StudioSettings.commandTimeoutRange.lowerBound)
        XCTAssertEqual(loaded.matrixTimeoutSecs, StudioSettings.matrixTimeoutRange.upperBound)
    }

    // MARK: Row 5 · Process discipline

    func testRunnerReportsTimedOutAndKillsTheProcess() {
        let start = Date()
        let result = StudioProcessRunner.run(
            executableURL: URL(fileURLWithPath: "/bin/sleep"),
            arguments: ["30"],
            timeoutSeconds: 1
        )
        let elapsed = Date().timeIntervalSince(start)
        XCTAssertTrue(result.timedOut, "a 30s sleep under a 1s ceiling must report timed_out")
        XCTAssertLessThan(elapsed, 10, "the tree kill must end the wait well before the child's natural exit")
        XCTAssertTrue(result.output.contains("timed_out"), "the honest timeout marker must be in the output")
        XCTAssertNotEqual(result.exitCode, 0)
    }

    func testRunnerDrainsLargeOutputWithoutDeadlockAndCapsUI() {
        // 512 KiB of output: far past any pipe buffer (deadlock probe) and past
        // the 64 KiB UI cap (truncation probe).
        let result = StudioProcessRunner.run(
            executableURL: URL(fileURLWithPath: "/bin/sh"),
            arguments: ["-c", "yes garnet | head -c 524288"],
            timeoutSeconds: 30
        )
        XCTAssertFalse(result.timedOut)
        XCTAssertTrue(result.outputTruncatedForUI, "512 KiB must trip the UI payload cap")
        XCTAssertTrue(result.output.contains("output capped for UI display"))
        XCTAssertGreaterThan(result.fullOutput.utf8.count, StudioProcessRunner.uiPayloadByteCap, "full output is preserved for evidence")
    }

    func testRunnerSurfacesDurationAndExitCode() {
        let result = StudioProcessRunner.run(
            executableURL: URL(fileURLWithPath: "/usr/bin/true"),
            arguments: [],
            timeoutSeconds: 10
        )
        XCTAssertEqual(result.exitCode, 0)
        XCTAssertFalse(result.timedOut)
        XCTAssertGreaterThanOrEqual(result.durationSeconds, 0)
    }

    // MARK: Row 6 · Truth surface

    func testTruthSummaryDecodesFieldsAndReportsUnavailableHonestly() throws {
        let dir = FileManager.default.temporaryDirectory
            .appendingPathComponent("garnet-truth-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(
            at: dir.appendingPathComponent("docs"), withIntermediateDirectories: true
        )
        try Data("[workspace.package]\nversion = \"9.9.9\"\n".utf8)
            .write(to: dir.appendingPathComponent("Cargo.toml"))
        try Data(#"{"version":"9.9.9","primitive_count":82,"latest_tag":"v9.9.9","generated_at_commit":"abc1234","workspace_tests":{"passed":1952,"failed":0,"measured_at_commit":"abc1234"}}"#.utf8)
            .write(to: dir.appendingPathComponent("docs/truth.json"))

        guard case .loaded(let fields) = StudioTruthSummary.load(repoRoot: dir) else {
            return XCTFail("expected loaded truth")
        }
        XCTAssertEqual(fields.version, "9.9.9")
        XCTAssertEqual(fields.primitiveCount, 82)
        XCTAssertEqual(fields.latestTag, "v9.9.9")
        XCTAssertEqual(fields.workspaceTests?.passed, 1952, "workspace tests are NESTED in real truth.json — a flat decoder renders a dash forever")
        XCTAssertEqual(fields.generatedAtCommit, "abc1234")

        guard case .unavailable(let reason) = StudioTruthSummary.load(repoRoot: nil) else {
            return XCTFail("expected unavailable without a repo root")
        }
        XCTAssertTrue(reason.contains("unavailable"))
    }

    // MARK: Row 8 · Evidence readers

    func testEvidenceReaderRefusesPathsOutsideRootsAndSymlinks() throws {
        let fm = FileManager.default
        let root = fm.temporaryDirectory
            .appendingPathComponent("garnet-evidence-\(UUID().uuidString)", isDirectory: true)
        let outside = fm.temporaryDirectory
            .appendingPathComponent("garnet-outside-\(UUID().uuidString)", isDirectory: true)
        try fm.createDirectory(at: root, withIntermediateDirectories: true)
        try fm.createDirectory(at: outside, withIntermediateDirectories: true)
        let secret = outside.appendingPathComponent("secret.txt")
        try Data("outside".utf8).write(to: secret)
        let inside = root.appendingPathComponent("report.md")
        try Data("inside".utf8).write(to: inside)
        let link = root.appendingPathComponent("sneaky-link")
        try fm.createSymbolicLink(at: link, withDestinationURL: secret)

        let reader = StudioEvidenceReader(roots: [root])

        XCTAssertEqual(reader.readEvidenceText(at: inside), .success("inside"))
        XCTAssertEqual(reader.readEvidenceText(at: secret), .failure(.outsideRoots))
        XCTAssertEqual(reader.readEvidenceText(at: link), .failure(.isSymlink), "a symlink must not smuggle an outside target")

        switch reader.listEvidenceFiles(under: root) {
        case .success(let names):
            XCTAssertTrue(names.contains("report.md"))
        case .failure(let error):
            XCTFail("listing inside the root must succeed, got \(error)")
        }
        XCTAssertEqual(reader.listEvidenceFiles(under: outside), .failure(.outsideRoots))
    }

    // MARK: Row 2 · Boot bounds

    func testBootSequenceBoundsMatchTheStandard() {
        XCTAssertEqual(StudioBootSequence.minimumMilliseconds, 700)
        XCTAssertEqual(StudioBootSequence.ceilingMilliseconds, 25_000)
        XCTAssertLessThan(
            StudioBootSequence.minimumMilliseconds,
            StudioBootSequence.ceilingMilliseconds,
            "the splash minimum must stay below the hard dismissal ceiling"
        )
    }
}
