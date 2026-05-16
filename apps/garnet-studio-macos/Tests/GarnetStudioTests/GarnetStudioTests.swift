import XCTest
@testable import GarnetStudio

final class GarnetStudioTests: XCTestCase {
    func testCliLocatorPrefersBundledExecutable() {
        let bundled = URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources/garnet")
        let locator = GarnetCLILocator(
            bundleResourceURL: bundled.deletingLastPathComponent(),
            environmentPath: "/usr/local/bin:/opt/homebrew/bin"
        )

        XCTAssertEqual(locator.candidatePaths().first, bundled.path)
    }

    func testCliLocatorFallsBackToCommonDeveloperPaths() {
        let locator = GarnetCLILocator(bundleResourceURL: nil, environmentPath: "/custom/bin:/usr/bin")

        let candidates = locator.candidatePaths()

        XCTAssertTrue(candidates.contains("/custom/bin/garnet"))
        XCTAssertTrue(candidates.contains("/usr/local/bin/garnet"))
        XCTAssertTrue(candidates.contains("/opt/homebrew/bin/garnet"))
    }

    func testSampleCatalogCoversCoreWorkbenchModes() {
        let modes = Set(GarnetSampleCatalog.samples.map(\.mode))

        XCTAssertTrue(modes.contains(.parse))
        XCTAssertTrue(modes.contains(.check))
        XCTAssertTrue(modes.contains(.run))
        XCTAssertTrue(modes.contains(.convert))
    }

    func testStudioSectionsExposeAgenticTests() {
        XCTAssertTrue(StudioSection.allCases.contains(.agentic))
    }

    func testAgenticMatrixLocatorPrefersExplicitRepoRoot() {
        let locator = AgenticDogfoodScriptLocator(
            bundleResourceURL: nil,
            environmentRepoRoot: "/repo",
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/repo")
        XCTAssertEqual(first?.scriptURL.path, "/repo/scripts/run_agentic_dogfood_matrix.py")
    }

    func testAgenticMatrixLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = AgenticDogfoodScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/run_agentic_dogfood_matrix.py")
    }

    func testAgenticMatrixRunnerBuildsStrictDesktopCommand() {
        let location = AgenticDogfoodScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/run_agentic_dogfood_matrix.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = AgenticDogfoodRunner(location: location, garnetBinaryPath: "/repo/target/debug/garnet")

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/run_agentic_dogfood_matrix.py",
                "--garnet-bin",
                "/repo/target/debug/garnet",
                "--copy-to-desktop",
                "--strict",
            ]
        )
    }

    func testAgenticMatrixRunnerBuildsPackagedAppCommand() {
        let location = AgenticDogfoodScriptLocation(
            scriptURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources/scripts/run_agentic_dogfood_matrix.py"),
            repoRootURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true)
        )
        let runner = AgenticDogfoodRunner(
            location: location,
            garnetBinaryPath: "/Applications/Garnet Studio.app/Contents/Resources/garnet",
            appExecutablePath: "/Applications/Garnet Studio.app/Contents/MacOS/GarnetStudio"
        )

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/Applications/Garnet Studio.app/Contents/Resources/scripts/run_agentic_dogfood_matrix.py",
                "--garnet-bin",
                "/Applications/Garnet Studio.app/Contents/Resources/garnet",
                "--app-executable",
                "/Applications/Garnet Studio.app/Contents/MacOS/GarnetStudio",
                "--copy-to-desktop",
                "--strict",
            ]
        )
    }

    func testAgenticMatrixRunnerCanDelegateBinarySelectionToScript() {
        let location = AgenticDogfoodScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/run_agentic_dogfood_matrix.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = AgenticDogfoodRunner(location: location, garnetBinaryPath: nil)

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/run_agentic_dogfood_matrix.py",
                "--copy-to-desktop",
                "--strict",
            ]
        )
    }

    func testAgenticMatrixOutputAcceptsAnyCompleteProbeCount() {
        XCTAssertTrue(AgenticDogfoodRunner.outputProvesCompleteReadiness("readiness=100\npassed=28/28\n"))
        XCTAssertTrue(AgenticDogfoodRunner.outputProvesCompleteReadiness("artifact_dir=/tmp/run\nreadiness=100\npassed=29/29\n"))
    }

    func testAgenticMatrixOutputRejectsPartialOrLowReadinessRuns() {
        XCTAssertFalse(AgenticDogfoodRunner.outputProvesCompleteReadiness("readiness=100\npassed=27/28\n"))
        XCTAssertFalse(AgenticDogfoodRunner.outputProvesCompleteReadiness("readiness=95\npassed=28/28\n"))
        XCTAssertFalse(AgenticDogfoodRunner.outputProvesCompleteReadiness("readiness=100\npassed=twenty-eight/twenty-eight\n"))
    }

    func testAgenticMatrixRunnerFindsBundledResources() {
        let temporary = FileManager.default.temporaryDirectory
            .appendingPathComponent("GarnetStudioTests-\(UUID().uuidString)", isDirectory: true)
        let resources = temporary
            .appendingPathComponent("Garnet Studio.app", isDirectory: true)
            .appendingPathComponent("Contents", isDirectory: true)
            .appendingPathComponent("Resources", isDirectory: true)
        let macOS = resources
            .deletingLastPathComponent()
            .appendingPathComponent("MacOS", isDirectory: true)
        let garnet = resources.appendingPathComponent("garnet")
        let executable = macOS.appendingPathComponent("GarnetStudio")

        do {
            try FileManager.default.createDirectory(at: resources, withIntermediateDirectories: true)
            try FileManager.default.createDirectory(at: macOS, withIntermediateDirectories: true)
            FileManager.default.createFile(atPath: garnet.path, contents: Data(), attributes: [.posixPermissions: 0o755])
            FileManager.default.createFile(atPath: executable.path, contents: Data(), attributes: [.posixPermissions: 0o755])
            defer { try? FileManager.default.removeItem(at: temporary) }

            let location = AgenticDogfoodScriptLocation(
                scriptURL: resources
                    .appendingPathComponent("scripts", isDirectory: true)
                    .appendingPathComponent("run_agentic_dogfood_matrix.py"),
                repoRootURL: resources
            )

            XCTAssertEqual(AgenticDogfoodRunner.checkoutGarnetBinary(for: location), garnet.path)
            XCTAssertEqual(AgenticDogfoodRunner.appBundleExecutable(for: location), executable.path)
        } catch {
            XCTFail("failed to prepare temporary bundle: \(error)")
        }
    }

    func testCommandResultClassifiesExitStatus() {
        let success = GarnetCommandResult(command: "garnet version", exitCode: 0, output: "garnet 0.4.2")
        let failure = GarnetCommandResult(command: "garnet check broken.garnet", exitCode: 1, output: "diagnostic")

        XCTAssertEqual(success.status, .success)
        XCTAssertEqual(failure.status, .failure)
    }
}
