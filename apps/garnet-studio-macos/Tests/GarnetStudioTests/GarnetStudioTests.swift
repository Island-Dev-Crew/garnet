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

    func testAncestorWalkTerminatesAtRootAndStaysBounded() {
        // Regression: the ancestor walk (ancestorRoots) once looped forever on
        // Foundation versions where `URL("/").deletingLastPathComponent()` keeps
        // prepending "../" instead of converging (macOS 15 / Darwin 24). On the
        // CI runner that spun to a 26GB footprint and was OOM-killed; locally
        // (macOS 26) it converged, so it passed. The walk must now terminate,
        // reach the filesystem root, stay bounded, and never emit a "../" root.
        let locator = AgenticDogfoodScriptLocator(
            bundleResourceURL: nil,
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/a/b/c/d/e/f/g", isDirectory: true)
        )

        let locations = locator.candidateLocations()

        XCTAssertFalse(locations.isEmpty)
        XCTAssertLessThan(locations.count, 64, "the ancestor walk must be bounded")
        XCTAssertTrue(
            locations.contains { $0.repoRootURL.path == "/" }, "the walk must reach the root")
        for location in locations {
            XCTAssertFalse(
                location.repoRootURL.path.contains(".."),
                "a non-converging '../' root must never appear")
        }
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

    func testConverterProviderOptionsLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = ConverterProviderOptionsScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_converter_llm_feasibility.py")
    }

    func testConverterStatusLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = ConverterStatusScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_converter_status.py")
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

    func testMitDeckOutlineLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = MitDeckOutlineScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_mit_deck_outline.py")
    }

    func testMitDeckPreviewLocatorPrefersBundledScriptBeforeAmbientCheckout() {
        let locator = MitDeckPreviewScriptLocator(
            bundleResourceURL: URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources", isDirectory: true),
            environmentRepoRoot: nil,
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )

        let first = locator.candidateLocations().first

        XCTAssertEqual(first?.repoRootURL.path, "/Applications/Garnet Studio.app/Contents/Resources")
        XCTAssertEqual(first?.scriptURL.path, "/Applications/Garnet Studio.app/Contents/Resources/scripts/garnet_mit_deck_preview.py")
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

    func testConverterProviderOptionsRunnerBuildsManifestedNoSourceCommand() {
        let location = ConverterProviderOptionsScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_converter_llm_feasibility.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = ConverterProviderOptionsRunner(
            location: location,
            outputDirectoryURL: URL(fileURLWithPath: "/tmp/GarnetStudioProviderOptions")
        )

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_converter_llm_feasibility.py",
                "--output-dir",
                "/tmp/GarnetStudioProviderOptions",
                "--format",
                "markdown",
            ]
        )
        XCTAssertFalse(runner.commandArguments().contains("--include-source"))
    }

    func testConverterStatusRunnerBuildsMarkdownCommand() {
        let location = ConverterStatusScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_converter_status.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = ConverterStatusRunner(location: location)

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_converter_status.py",
                "--format",
                "markdown",
            ]
        )
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

    func testMitDeckOutlineRunnerBuildsManifestedMarkdownCommand() {
        let location = MitDeckOutlineScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_mit_deck_outline.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = MitDeckOutlineRunner(
            location: location,
            outputDirectoryURL: URL(fileURLWithPath: "/tmp/GarnetStudioMitDeckOutline")
        )

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_mit_deck_outline.py",
                "--output-dir",
                "/tmp/GarnetStudioMitDeckOutline",
                "--format",
                "markdown",
            ]
        )
    }

    func testMitDeckPreviewRunnerBuildsManifestedHtmlCommand() {
        let location = MitDeckPreviewScriptLocation(
            scriptURL: URL(fileURLWithPath: "/repo/scripts/garnet_mit_deck_preview.py"),
            repoRootURL: URL(fileURLWithPath: "/repo", isDirectory: true)
        )
        let runner = MitDeckPreviewRunner(
            location: location,
            outputDirectoryURL: URL(fileURLWithPath: "/tmp/GarnetStudioMitDeckPreview")
        )

        XCTAssertEqual(
            runner.commandArguments(),
            [
                "env",
                "PYTHONDONTWRITEBYTECODE=1",
                "python3",
                "/repo/scripts/garnet_mit_deck_preview.py",
                "--output-dir",
                "/tmp/GarnetStudioMitDeckPreview",
                "--format",
                "html",
            ]
        )
    }

    func testDeckPreviewSmokeUsesExplicitOutputDirectory() {
        let directory = URL(fileURLWithPath: "/tmp/GarnetStudioDeckPreviewSmoke", isDirectory: true)

        XCTAssertEqual(
            GarnetStudioSelfTest.deckPreviewSmokeOutputDirectory(
                environment: ["GARNET_STUDIO_DECK_PREVIEW_SMOKE_OUTPUT_DIR": directory.path]
            ).path,
            directory.path
        )
    }

    func testDeckPreviewSmokeRunsReporterAndRequiresManifestedOutputs() throws {
        let temp = URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true)
            .appendingPathComponent("GarnetStudioDeckPreviewSmokeTest-\(UUID().uuidString)", isDirectory: true)
        let repo = temp.appendingPathComponent("repo", isDirectory: true)
        let scripts = repo.appendingPathComponent("scripts", isDirectory: true)
        let output = temp.appendingPathComponent("output", isDirectory: true)
        try FileManager.default.createDirectory(at: scripts, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: temp) }

        let reporter = scripts.appendingPathComponent("garnet_mit_deck_preview.py")
        let source = """
        #!/usr/bin/env python3
        from pathlib import Path
        import sys

        out = Path(sys.argv[sys.argv.index("--output-dir") + 1])
        out.mkdir(parents=True, exist_ok=True)
        digest = "dc51b8c96c2d745df3bd5590d990230a482fd247123599548e0632fdbf97fc22"
        names = ["garnet-mit-deck-preview.html", "garnet-mit-deck-preview.json", "garnet-mit-deck-outline.md"]
        for name in names:
            (out / name).write_text("ok\\n", encoding="utf-8")
        (out / "MANIFEST.sha256").write_text("".join(f"{digest}  ./{name}\\n" for name in names), encoding="utf-8")
        """
        try source.write(to: reporter, atomically: true, encoding: .utf8)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: reporter.path)

        let locator = MitDeckPreviewScriptLocator(
            bundleResourceURL: nil,
            environmentRepoRoot: repo.path,
            currentDirectoryURL: repo
        )

        XCTAssertEqual(
            GarnetStudioSelfTest.runDeckPreviewSmoke(locator: locator, outputDirectoryURL: output),
            0
        )
    }

    func testDeckPreviewSmokeRejectsStaleManifest() throws {
        let temp = URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true)
            .appendingPathComponent("GarnetStudioDeckPreviewSmokeManifestTest-\(UUID().uuidString)", isDirectory: true)
        let repo = temp.appendingPathComponent("repo", isDirectory: true)
        let scripts = repo.appendingPathComponent("scripts", isDirectory: true)
        let output = temp.appendingPathComponent("output", isDirectory: true)
        try FileManager.default.createDirectory(at: scripts, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: temp) }

        let reporter = scripts.appendingPathComponent("garnet_mit_deck_preview.py")
        let source = """
        #!/usr/bin/env python3
        from pathlib import Path
        import sys

        out = Path(sys.argv[sys.argv.index("--output-dir") + 1])
        out.mkdir(parents=True, exist_ok=True)
        names = ["garnet-mit-deck-preview.html", "garnet-mit-deck-preview.json", "garnet-mit-deck-outline.md"]
        for name in names:
            (out / name).write_text("ok\\n", encoding="utf-8")
        (out / "MANIFEST.sha256").write_text("".join(f"{'0' * 64}  ./{name}\\n" for name in names), encoding="utf-8")
        """
        try source.write(to: reporter, atomically: true, encoding: .utf8)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: reporter.path)

        let locator = MitDeckPreviewScriptLocator(
            bundleResourceURL: nil,
            environmentRepoRoot: repo.path,
            currentDirectoryURL: repo
        )

        XCTAssertEqual(
            GarnetStudioSelfTest.runDeckPreviewSmoke(locator: locator, outputDirectoryURL: output),
            11
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

    func testStudioMitDeckOutlineEvidenceDirectoryDefaultsToDesktopDogfood() {
        let directory = GarnetStudioEvidenceDirectory(
            homeDirectoryURL: URL(fileURLWithPath: "/Users/example", isDirectory: true)
        ).mitDeckOutlineDirectory(stamp: "20260516-181500")

        XCTAssertEqual(
            directory.path,
            "/Users/example/Desktop/dogfood/garnet-studio-mit-deck-outline-20260516-181500"
        )
    }

    func testStudioMitDeckPreviewEvidenceDirectoryDefaultsToDesktopDogfood() {
        let directory = GarnetStudioEvidenceDirectory(
            homeDirectoryURL: URL(fileURLWithPath: "/Users/example", isDirectory: true)
        ).mitDeckPreviewDirectory(stamp: "20260516-184500")

        XCTAssertEqual(
            directory.path,
            "/Users/example/Desktop/dogfood/garnet-studio-mit-deck-preview-20260516-184500"
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

    func testStudioProviderOptionsEvidenceDirectoryDefaultsToDesktopDogfood() {
        let directory = GarnetStudioEvidenceDirectory(
            homeDirectoryURL: URL(fileURLWithPath: "/Users/example", isDirectory: true)
        ).providerOptionsDirectory(stamp: "20260516-212500")

        XCTAssertEqual(
            directory.path,
            "/Users/example/Desktop/dogfood/garnet-studio-provider-options-20260516-212500"
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
