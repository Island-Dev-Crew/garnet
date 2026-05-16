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

    func testAgenticMatrixRunnerBuildsStrictDesktopCommand() {
        let location = AgenticDogfoodScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/run_agentic_dogfood_matrix.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = AgenticDogfoodRunner(location: location, garnetBinaryPath: "/repo/target/debug/garnet")

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "python3",
                "/repo/scripts/run_agentic_dogfood_matrix.py",
                "--garnet-bin",
                "/repo/target/debug/garnet",
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
                "python3",
                "/repo/scripts/run_agentic_dogfood_matrix.py",
                "--copy-to-desktop",
                "--strict",
            ]
        )
    }

    func testCommandResultClassifiesExitStatus() {
        let success = GarnetCommandResult(command: "garnet version", exitCode: 0, output: "garnet 0.4.2")
        let failure = GarnetCommandResult(command: "garnet check broken.garnet", exitCode: 1, output: "diagnostic")

        XCTAssertEqual(success.status, .success)
        XCTAssertEqual(failure.status, .failure)
    }
}
