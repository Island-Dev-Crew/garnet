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

    func testConverterAssistPlanLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = ConverterAssistPlanScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_converter_assist_plan.py")
    }

    func testConverterAdvisoryBundleLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = ConverterAdvisoryBundleScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_converter_advisory_bundle.py")
    }

    func testConverterAdvisoryReviewLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = ConverterAdvisoryReviewScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_converter_advisory_review.py")
    }

    func testConverterAdvisoryHandoffLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = ConverterAdvisoryHandoffScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_converter_advisory_handoff.py")
    }

    func testMitReadinessLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = MitReadinessScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_mit_readiness_status.py")
    }

    func testMitDemoRouteLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = MitDemoRouteScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_mit_demo_route.py")
    }

    func testMacContinuationLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = MacContinuationScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_mac_side_continuation_status.py")
    }

    func testConverterAssistPlanRunnerBuildsMarkdownCommand() {
        let location = ConverterAssistPlanScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_converter_assist_plan.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = ConverterAssistPlanRunner(
            location: location,
            language: "typescript",
            sourceURL: URL(fileURLWithPath: "/tmp/agent_router.ts")
        )

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_converter_assist_plan.py",
                "--language",
                "typescript",
                "--source",
                "/tmp/agent_router.ts",
                "--format",
                "markdown",
            ]
        )
    }

    func testConverterAdvisoryBundleRunnerBuildsManifestedNoSourceCommand() {
        let location = ConverterAdvisoryBundleScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_converter_advisory_bundle.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = ConverterAdvisoryBundleRunner(
            location: location,
            language: "typescript",
            sourceURL: URL(fileURLWithPath: "/tmp/agent_router.ts"),
            outputDirectoryURL: URL(fileURLWithPath: "/tmp/GarnetStudioAdvisory")
        )

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_converter_advisory_bundle.py",
                "--language",
                "typescript",
                "--source",
                "/tmp/agent_router.ts",
                "--output-dir",
                "/tmp/GarnetStudioAdvisory",
                "--format",
                "markdown",
            ]
        )
        XCTAssertFalse(runner.commandArguments().contains("--include-source"))
    }

    func testConverterAdvisoryReviewRunnerBuildsManifestedReviewCommand() {
        let location = ConverterAdvisoryReviewScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_converter_advisory_review.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = ConverterAdvisoryReviewRunner(
            location: location,
            bundleDirectoryURL: URL(fileURLWithPath: "/tmp/GarnetStudioAdvisory"),
            outputDirectoryURL: URL(fileURLWithPath: "/tmp/GarnetStudioReview")
        )

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_converter_advisory_review.py",
                "--bundle-dir",
                "/tmp/GarnetStudioAdvisory",
                "--output-dir",
                "/tmp/GarnetStudioReview",
            ]
        )
        XCTAssertFalse(runner.commandArguments().contains("--allow-source-included"))
    }

    func testConverterAdvisoryHandoffRunnerBuildsReviewedNoSourceCommand() {
        let location = ConverterAdvisoryHandoffScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_converter_advisory_handoff.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = ConverterAdvisoryHandoffRunner(
            location: location,
            bundleDirectoryURL: URL(fileURLWithPath: "/tmp/GarnetStudioAdvisory"),
            reviewDirectoryURL: URL(fileURLWithPath: "/tmp/GarnetStudioReview"),
            outputDirectoryURL: URL(fileURLWithPath: "/tmp/GarnetStudioHandoff")
        )

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_converter_advisory_handoff.py",
                "--bundle-dir",
                "/tmp/GarnetStudioAdvisory",
                "--review-dir",
                "/tmp/GarnetStudioReview",
                "--output-dir",
                "/tmp/GarnetStudioHandoff",
            ]
        )
        XCTAssertFalse(runner.commandArguments().contains("--allow-source-included"))
        XCTAssertFalse(runner.commandArguments().contains("--include-source"))
    }

    func testMitReadinessRunnerBuildsMarkdownCommand() {
        let location = MitReadinessScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_mit_readiness_status.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = MitReadinessRunner(location: location)

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_mit_readiness_status.py",
                "--format",
                "markdown",
            ]
        )
    }

    func testMitDemoRouteRunnerBuildsManifestedMarkdownCommand() {
        let location = MitDemoRouteScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_mit_demo_route.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = MitDemoRouteRunner(
            location: location,
            outputDirectoryURL: URL(fileURLWithPath: "/tmp/GarnetStudioMitDemoRoute")
        )

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_mit_demo_route.py",
                "--output-dir",
                "/tmp/GarnetStudioMitDemoRoute",
                "--format",
                "markdown",
            ]
        )
    }

    func testMacContinuationRunnerBuildsMarkdownCommand() {
        let location = MacContinuationScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_mac_side_continuation_status.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = MacContinuationRunner(location: location)

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_mac_side_continuation_status.py",
                "--format",
                "markdown",
            ]
        )
    }

    func testStudioAdvisoryBundleEvidenceDirectoryDefaultsToDesktopDogfood() {
        let directory = GarnetStudioEvidenceDirectory(
            homeDirectoryURL: URL(fileURLWithPath: "/Users/example", isDirectory: true)
        ).advisoryBundleDirectory(stamp: "20260516-093000")

        XCTAssertEqual(
            directory.path,
            "/Users/example/Desktop/dogfood/garnet-studio-advisory-bundle-20260516-093000"
        )
    }

    func testStudioMitDemoRouteEvidenceDirectoryDefaultsToDesktopDogfood() {
        let directory = GarnetStudioEvidenceDirectory(
            homeDirectoryURL: URL(fileURLWithPath: "/Users/example", isDirectory: true)
        ).mitDemoRouteDirectory(stamp: "20260516-174000")

        XCTAssertEqual(
            directory.path,
            "/Users/example/Desktop/dogfood/garnet-studio-mit-demo-route-20260516-174000"
        )
    }

    func testStudioAdvisoryReviewEvidenceDirectoryDefaultsToDesktopDogfood() {
        let directory = GarnetStudioEvidenceDirectory(
            homeDirectoryURL: URL(fileURLWithPath: "/Users/example", isDirectory: true)
        ).advisoryReviewDirectory(stamp: "20260516-101500")

        XCTAssertEqual(
            directory.path,
            "/Users/example/Desktop/dogfood/garnet-studio-advisory-review-20260516-101500"
        )
    }

    func testStudioAdvisoryHandoffEvidenceDirectoryDefaultsToDesktopDogfood() {
        let directory = GarnetStudioEvidenceDirectory(
            homeDirectoryURL: URL(fileURLWithPath: "/Users/example", isDirectory: true)
        ).advisoryHandoffDirectory(stamp: "20260516-104500")

        XCTAssertEqual(
            directory.path,
            "/Users/example/Desktop/dogfood/garnet-studio-advisory-handoff-20260516-104500"
        )
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
