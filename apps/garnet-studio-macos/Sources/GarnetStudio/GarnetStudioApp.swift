import Foundation
import SwiftUI

enum GarnetCommandStatus: Equatable {
    case success
    case failure
}

struct GarnetCommandResult: Equatable {
    let command: String
    let exitCode: Int32
    let output: String

    var status: GarnetCommandStatus {
        exitCode == 0 ? .success : .failure
    }
}

enum GarnetSampleMode: String, CaseIterable, Identifiable {
    case parse = "Parse"
    case check = "Check"
    case run = "Run"
    case convert = "Convert"

    var id: String { rawValue }
}

struct GarnetSample: Identifiable, Equatable {
    let id: String
    let title: String
    let subtitle: String
    let mode: GarnetSampleMode
    let language: String?
    let filename: String
    let source: String
}

enum GarnetSampleCatalog {
    static let samples: [GarnetSample] = [
        GarnetSample(
            id: "mvp-os-run",
            title: "Run a canonical MVP",
            subtitle: "Cooperative scheduler smoke from the current runnable examples.",
            mode: .run,
            language: nil,
            filename: "mvp_01_os_simulator.garnet",
            source: """
            def run_scheduler(ticks) {
              let mut tick = 0
              let mut ready = 3
              let mut completed = 0
              while tick < ticks {
                if ready > 0 {
                  completed += 1
                  ready -= 1
                } else {
                  ready = 3
                }
                tick += 1
              }
              completed
            }

            @caps()
            def main() {
              let completed = run_scheduler(12)
              println("garnet studio completed:", completed)
              completed
            }
            """
        ),
        GarnetSample(
            id: "safe-check",
            title: "Check safe-mode ownership",
            subtitle: "Shows the checker path without needing a terminal.",
            mode: .check,
            language: nil,
            filename: "safe_mode_check.garnet",
            source: """
            @safe
            def score(value) {
              let owned = value
              owned + 1
            }
            """
        ),
        GarnetSample(
            id: "parse-actor-shape",
            title: "Parse an agent-facing shape",
            subtitle: "A small actor-like source sample for parser exploration.",
            mode: .parse,
            language: nil,
            filename: "agent_shape.garnet",
            source: """
            struct BuildTask {
              id: Int,
              name: String,
            }

            def describe(task) {
              task.name
            }
            """
        ),
        GarnetSample(
            id: "convert-python",
            title: "Convert Python into Garnet",
            subtitle: "Use the migration assistant and inspect the generated checklist.",
            mode: .convert,
            language: "python",
            filename: "sample.py",
            source: """
            def route_weight(path):
                if path == "/":
                    return 10
                if path == "/health":
                    return 20
                return 1
            """
        ),
    ]
}

struct GarnetCLILocator {
    let bundleResourceURL: URL?
    let environmentPath: String

    init(
        bundleResourceURL: URL? = Bundle.main.resourceURL,
        environmentPath: String = ProcessInfo.processInfo.environment["PATH"] ?? ""
    ) {
        self.bundleResourceURL = bundleResourceURL
        self.environmentPath = environmentPath
    }

    func candidatePaths() -> [String] {
        var paths: [String] = []
        if let bundleResourceURL {
            paths.append(bundleResourceURL.appendingPathComponent("garnet").path)
        }
        for directory in environmentPath.split(separator: ":") {
            paths.append(String(directory) + "/garnet")
        }
        paths.append(contentsOf: [
            "/usr/local/bin/garnet",
            "/opt/homebrew/bin/garnet",
            "/usr/local/garnet/bin/garnet",
        ])
        return Array(NSOrderedSet(array: paths)) as? [String] ?? paths
    }

    func locate(fileManager: FileManager = .default) -> String? {
        candidatePaths().first { candidate in
            fileManager.isExecutableFile(atPath: candidate)
        }
    }
}

struct GarnetCLI {
    let executablePath: String

    func run(arguments: [String], workingDirectory: URL? = nil) -> GarnetCommandResult {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: executablePath)
        process.arguments = arguments
        process.currentDirectoryURL = workingDirectory

        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = outputPipe

        let command = ([executablePath] + arguments).joined(separator: " ")
        do {
            try process.run()
            process.waitUntilExit()
            let data = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""
            return GarnetCommandResult(command: command, exitCode: process.terminationStatus, output: output)
        } catch {
            return GarnetCommandResult(command: command, exitCode: 127, output: error.localizedDescription)
        }
    }
}

struct AgenticDogfoodScriptLocation: Equatable {
    let scriptURL: URL
    let repoRootURL: URL
}

struct ConverterAssistPlanScriptLocation: Equatable {
    let scriptURL: URL
    let repoRootURL: URL
}

struct ConverterAdvisoryBundleScriptLocation: Equatable {
    let scriptURL: URL
    let repoRootURL: URL
}

struct ConverterAdvisoryReviewScriptLocation: Equatable {
    let scriptURL: URL
    let repoRootURL: URL
}

struct ConverterAdvisoryHandoffScriptLocation: Equatable {
    let scriptURL: URL
    let repoRootURL: URL
}

struct MitReadinessScriptLocation: Equatable {
    let scriptURL: URL
    let repoRootURL: URL
}

struct MacContinuationScriptLocation: Equatable {
    let scriptURL: URL
    let repoRootURL: URL
}

struct ConverterAssistPlanScriptLocator {
    let bundleResourceURL: URL?
    let environmentRepoRoot: String?
    let currentDirectoryURL: URL

    init(
        bundleResourceURL: URL? = Bundle.main.resourceURL,
        environmentRepoRoot: String? = ProcessInfo.processInfo.environment["GARNET_REPO_ROOT"],
        currentDirectoryURL: URL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
    ) {
        self.bundleResourceURL = bundleResourceURL
        self.environmentRepoRoot = environmentRepoRoot
        self.currentDirectoryURL = currentDirectoryURL
    }

    func candidateLocations() -> [ConverterAssistPlanScriptLocation] {
        var locations: [ConverterAssistPlanScriptLocation] = []

        if let environmentRepoRoot, !environmentRepoRoot.isEmpty {
            let root = URL(fileURLWithPath: environmentRepoRoot, isDirectory: true)
            locations.append(location(forRepoRoot: root))
        }

        if let bundleResourceURL {
            let script = bundleResourceURL
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_converter_assist_plan.py")
            locations.append(ConverterAssistPlanScriptLocation(scriptURL: script, repoRootURL: bundleResourceURL))
        }

        for root in ancestorRoots(from: currentDirectoryURL) {
            locations.append(location(forRepoRoot: root))
        }

        var seen: Set<String> = []
        return locations.filter { location in
            let key = location.scriptURL.path
            if seen.contains(key) {
                return false
            }
            seen.insert(key)
            return true
        }
    }

    func locate(fileManager: FileManager = .default) -> ConverterAssistPlanScriptLocation? {
        candidateLocations().first { location in
            fileManager.fileExists(atPath: location.scriptURL.path)
        }
    }

    private func location(forRepoRoot root: URL) -> ConverterAssistPlanScriptLocation {
        ConverterAssistPlanScriptLocation(
            scriptURL: root
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_converter_assist_plan.py"),
            repoRootURL: root
        )
    }

    private func ancestorRoots(from start: URL) -> [URL] {
        var roots: [URL] = []
        var cursor = start.standardizedFileURL
        while true {
            roots.append(cursor)
            let parent = cursor.deletingLastPathComponent()
            if parent.path == cursor.path {
                break
            }
            cursor = parent
        }
        return roots
    }
}

struct ConverterAdvisoryBundleScriptLocator {
    let bundleResourceURL: URL?
    let environmentRepoRoot: String?
    let currentDirectoryURL: URL

    init(
        bundleResourceURL: URL? = Bundle.main.resourceURL,
        environmentRepoRoot: String? = ProcessInfo.processInfo.environment["GARNET_REPO_ROOT"],
        currentDirectoryURL: URL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
    ) {
        self.bundleResourceURL = bundleResourceURL
        self.environmentRepoRoot = environmentRepoRoot
        self.currentDirectoryURL = currentDirectoryURL
    }

    func candidateLocations() -> [ConverterAdvisoryBundleScriptLocation] {
        var locations: [ConverterAdvisoryBundleScriptLocation] = []

        if let environmentRepoRoot, !environmentRepoRoot.isEmpty {
            let root = URL(fileURLWithPath: environmentRepoRoot, isDirectory: true)
            locations.append(location(forRepoRoot: root))
        }

        if let bundleResourceURL {
            let script = bundleResourceURL
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_converter_advisory_bundle.py")
            locations.append(ConverterAdvisoryBundleScriptLocation(scriptURL: script, repoRootURL: bundleResourceURL))
        }

        for root in ancestorRoots(from: currentDirectoryURL) {
            locations.append(location(forRepoRoot: root))
        }

        var seen: Set<String> = []
        return locations.filter { location in
            let key = location.scriptURL.path
            if seen.contains(key) {
                return false
            }
            seen.insert(key)
            return true
        }
    }

    func locate(fileManager: FileManager = .default) -> ConverterAdvisoryBundleScriptLocation? {
        candidateLocations().first { location in
            fileManager.fileExists(atPath: location.scriptURL.path)
        }
    }

    private func location(forRepoRoot root: URL) -> ConverterAdvisoryBundleScriptLocation {
        ConverterAdvisoryBundleScriptLocation(
            scriptURL: root
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_converter_advisory_bundle.py"),
            repoRootURL: root
        )
    }

    private func ancestorRoots(from start: URL) -> [URL] {
        var roots: [URL] = []
        var cursor = start.standardizedFileURL
        while true {
            roots.append(cursor)
            let parent = cursor.deletingLastPathComponent()
            if parent.path == cursor.path {
                break
            }
            cursor = parent
        }
        return roots
    }
}

struct ConverterAdvisoryReviewScriptLocator {
    let bundleResourceURL: URL?
    let environmentRepoRoot: String?
    let currentDirectoryURL: URL

    init(
        bundleResourceURL: URL? = Bundle.main.resourceURL,
        environmentRepoRoot: String? = ProcessInfo.processInfo.environment["GARNET_REPO_ROOT"],
        currentDirectoryURL: URL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
    ) {
        self.bundleResourceURL = bundleResourceURL
        self.environmentRepoRoot = environmentRepoRoot
        self.currentDirectoryURL = currentDirectoryURL
    }

    func candidateLocations() -> [ConverterAdvisoryReviewScriptLocation] {
        var locations: [ConverterAdvisoryReviewScriptLocation] = []

        if let environmentRepoRoot, !environmentRepoRoot.isEmpty {
            let root = URL(fileURLWithPath: environmentRepoRoot, isDirectory: true)
            locations.append(location(forRepoRoot: root))
        }

        if let bundleResourceURL {
            let script = bundleResourceURL
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_converter_advisory_review.py")
            locations.append(ConverterAdvisoryReviewScriptLocation(scriptURL: script, repoRootURL: bundleResourceURL))
        }

        for root in ancestorRoots(from: currentDirectoryURL) {
            locations.append(location(forRepoRoot: root))
        }

        var seen: Set<String> = []
        return locations.filter { location in
            let key = location.scriptURL.path
            if seen.contains(key) {
                return false
            }
            seen.insert(key)
            return true
        }
    }

    func locate(fileManager: FileManager = .default) -> ConverterAdvisoryReviewScriptLocation? {
        candidateLocations().first { location in
            fileManager.fileExists(atPath: location.scriptURL.path)
        }
    }

    private func location(forRepoRoot root: URL) -> ConverterAdvisoryReviewScriptLocation {
        ConverterAdvisoryReviewScriptLocation(
            scriptURL: root
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_converter_advisory_review.py"),
            repoRootURL: root
        )
    }

    private func ancestorRoots(from start: URL) -> [URL] {
        var roots: [URL] = []
        var cursor = start.standardizedFileURL
        while true {
            roots.append(cursor)
            let parent = cursor.deletingLastPathComponent()
            if parent.path == cursor.path {
                break
            }
            cursor = parent
        }
        return roots
    }
}

struct ConverterAdvisoryHandoffScriptLocator {
    let bundleResourceURL: URL?
    let environmentRepoRoot: String?
    let currentDirectoryURL: URL

    init(
        bundleResourceURL: URL? = Bundle.main.resourceURL,
        environmentRepoRoot: String? = ProcessInfo.processInfo.environment["GARNET_REPO_ROOT"],
        currentDirectoryURL: URL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
    ) {
        self.bundleResourceURL = bundleResourceURL
        self.environmentRepoRoot = environmentRepoRoot
        self.currentDirectoryURL = currentDirectoryURL
    }

    func candidateLocations() -> [ConverterAdvisoryHandoffScriptLocation] {
        var locations: [ConverterAdvisoryHandoffScriptLocation] = []

        if let environmentRepoRoot, !environmentRepoRoot.isEmpty {
            let root = URL(fileURLWithPath: environmentRepoRoot, isDirectory: true)
            locations.append(location(forRepoRoot: root))
        }

        if let bundleResourceURL {
            let script = bundleResourceURL
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_converter_advisory_handoff.py")
            locations.append(ConverterAdvisoryHandoffScriptLocation(scriptURL: script, repoRootURL: bundleResourceURL))
        }

        for root in ancestorRoots(from: currentDirectoryURL) {
            locations.append(location(forRepoRoot: root))
        }

        var seen: Set<String> = []
        return locations.filter { location in
            let key = location.scriptURL.path
            if seen.contains(key) {
                return false
            }
            seen.insert(key)
            return true
        }
    }

    func locate(fileManager: FileManager = .default) -> ConverterAdvisoryHandoffScriptLocation? {
        candidateLocations().first { location in
            fileManager.fileExists(atPath: location.scriptURL.path)
        }
    }

    private func location(forRepoRoot root: URL) -> ConverterAdvisoryHandoffScriptLocation {
        ConverterAdvisoryHandoffScriptLocation(
            scriptURL: root
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_converter_advisory_handoff.py"),
            repoRootURL: root
        )
    }

    private func ancestorRoots(from start: URL) -> [URL] {
        var roots: [URL] = []
        var cursor = start.standardizedFileURL
        while true {
            roots.append(cursor)
            let parent = cursor.deletingLastPathComponent()
            if parent.path == cursor.path {
                break
            }
            cursor = parent
        }
        return roots
    }
}

struct MitReadinessScriptLocator {
    let bundleResourceURL: URL?
    let environmentRepoRoot: String?
    let currentDirectoryURL: URL

    init(
        bundleResourceURL: URL? = Bundle.main.resourceURL,
        environmentRepoRoot: String? = ProcessInfo.processInfo.environment["GARNET_REPO_ROOT"],
        currentDirectoryURL: URL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
    ) {
        self.bundleResourceURL = bundleResourceURL
        self.environmentRepoRoot = environmentRepoRoot
        self.currentDirectoryURL = currentDirectoryURL
    }

    func candidateLocations() -> [MitReadinessScriptLocation] {
        var locations: [MitReadinessScriptLocation] = []

        if let environmentRepoRoot, !environmentRepoRoot.isEmpty {
            let root = URL(fileURLWithPath: environmentRepoRoot, isDirectory: true)
            locations.append(location(forRepoRoot: root))
        }

        if let bundleResourceURL {
            let script = bundleResourceURL
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_mit_readiness_status.py")
            locations.append(MitReadinessScriptLocation(scriptURL: script, repoRootURL: bundleResourceURL))
        }

        for root in ancestorRoots(from: currentDirectoryURL) {
            locations.append(location(forRepoRoot: root))
        }

        var seen: Set<String> = []
        return locations.filter { location in
            let key = location.scriptURL.path
            if seen.contains(key) {
                return false
            }
            seen.insert(key)
            return true
        }
    }

    func locate(fileManager: FileManager = .default) -> MitReadinessScriptLocation? {
        candidateLocations().first { location in
            fileManager.fileExists(atPath: location.scriptURL.path)
        }
    }

    private func location(forRepoRoot root: URL) -> MitReadinessScriptLocation {
        MitReadinessScriptLocation(
            scriptURL: root
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_mit_readiness_status.py"),
            repoRootURL: root
        )
    }

    private func ancestorRoots(from start: URL) -> [URL] {
        var roots: [URL] = []
        var cursor = start.standardizedFileURL
        while true {
            roots.append(cursor)
            let parent = cursor.deletingLastPathComponent()
            if parent.path == cursor.path {
                break
            }
            cursor = parent
        }
        return roots
    }
}

struct MacContinuationScriptLocator {
    let bundleResourceURL: URL?
    let environmentRepoRoot: String?
    let currentDirectoryURL: URL

    init(
        bundleResourceURL: URL? = Bundle.main.resourceURL,
        environmentRepoRoot: String? = ProcessInfo.processInfo.environment["GARNET_REPO_ROOT"],
        currentDirectoryURL: URL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
    ) {
        self.bundleResourceURL = bundleResourceURL
        self.environmentRepoRoot = environmentRepoRoot
        self.currentDirectoryURL = currentDirectoryURL
    }

    func candidateLocations() -> [MacContinuationScriptLocation] {
        var locations: [MacContinuationScriptLocation] = []

        if let environmentRepoRoot, !environmentRepoRoot.isEmpty {
            let root = URL(fileURLWithPath: environmentRepoRoot, isDirectory: true)
            locations.append(location(forRepoRoot: root))
        }

        if let bundleResourceURL {
            let script = bundleResourceURL
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_mac_side_continuation_status.py")
            locations.append(MacContinuationScriptLocation(scriptURL: script, repoRootURL: bundleResourceURL))
        }

        for root in ancestorRoots(from: currentDirectoryURL) {
            locations.append(location(forRepoRoot: root))
        }

        var seen: Set<String> = []
        return locations.filter { location in
            let key = location.scriptURL.path
            if seen.contains(key) {
                return false
            }
            seen.insert(key)
            return true
        }
    }

    func locate(fileManager: FileManager = .default) -> MacContinuationScriptLocation? {
        candidateLocations().first { location in
            fileManager.fileExists(atPath: location.scriptURL.path)
        }
    }

    private func location(forRepoRoot root: URL) -> MacContinuationScriptLocation {
        MacContinuationScriptLocation(
            scriptURL: root
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("garnet_mac_side_continuation_status.py"),
            repoRootURL: root
        )
    }

    private func ancestorRoots(from start: URL) -> [URL] {
        var roots: [URL] = []
        var cursor = start.standardizedFileURL
        while true {
            roots.append(cursor)
            let parent = cursor.deletingLastPathComponent()
            if parent.path == cursor.path {
                break
            }
            cursor = parent
        }
        return roots
    }
}

struct AgenticDogfoodScriptLocator {
    let bundleResourceURL: URL?
    let environmentRepoRoot: String?
    let currentDirectoryURL: URL

    init(
        bundleResourceURL: URL? = Bundle.main.resourceURL,
        environmentRepoRoot: String? = ProcessInfo.processInfo.environment["GARNET_REPO_ROOT"],
        currentDirectoryURL: URL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath, isDirectory: true)
    ) {
        self.bundleResourceURL = bundleResourceURL
        self.environmentRepoRoot = environmentRepoRoot
        self.currentDirectoryURL = currentDirectoryURL
    }

    func candidateLocations() -> [AgenticDogfoodScriptLocation] {
        var locations: [AgenticDogfoodScriptLocation] = []

        if let environmentRepoRoot, !environmentRepoRoot.isEmpty {
            let root = URL(fileURLWithPath: environmentRepoRoot, isDirectory: true)
            locations.append(location(forRepoRoot: root))
        }

        if let bundleResourceURL {
            let script = bundleResourceURL
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("run_agentic_dogfood_matrix.py")
            locations.append(AgenticDogfoodScriptLocation(scriptURL: script, repoRootURL: bundleResourceURL))
        }

        for root in ancestorRoots(from: currentDirectoryURL) {
            locations.append(location(forRepoRoot: root))
        }

        var seen: Set<String> = []
        return locations.filter { location in
            let key = location.scriptURL.path
            if seen.contains(key) {
                return false
            }
            seen.insert(key)
            return true
        }
    }

    func locate(fileManager: FileManager = .default) -> AgenticDogfoodScriptLocation? {
        candidateLocations().first { location in
            fileManager.fileExists(atPath: location.scriptURL.path)
        }
    }

    private func location(forRepoRoot root: URL) -> AgenticDogfoodScriptLocation {
        AgenticDogfoodScriptLocation(
            scriptURL: root
                .appendingPathComponent("scripts", isDirectory: true)
                .appendingPathComponent("run_agentic_dogfood_matrix.py"),
            repoRootURL: root
        )
    }

    private func ancestorRoots(from start: URL) -> [URL] {
        var roots: [URL] = []
        var cursor = start.standardizedFileURL
        while true {
            roots.append(cursor)
            let parent = cursor.deletingLastPathComponent()
            if parent.path == cursor.path {
                break
            }
            cursor = parent
        }
        return roots
    }
}

struct ConverterAssistPlanRunner {
    let location: ConverterAssistPlanScriptLocation
    let language: String
    let sourceURL: URL

    func commandArguments() -> [String] {
        [
            "env",
            "PYTHONDONTWRITEBYTECODE=1",
            "python3",
            location.scriptURL.path,
            "--language",
            language,
            "--source",
            sourceURL.path,
            "--format",
            "markdown",
        ]
    }

    func run() -> GarnetCommandResult {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = commandArguments()
        process.currentDirectoryURL = location.repoRootURL

        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = outputPipe

        let command = (["/usr/bin/env"] + (process.arguments ?? [])).joined(separator: " ")
        do {
            try process.run()
            process.waitUntilExit()
            let data = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""
            return GarnetCommandResult(command: command, exitCode: process.terminationStatus, output: output)
        } catch {
            return GarnetCommandResult(command: command, exitCode: 127, output: error.localizedDescription)
        }
    }
}

struct ConverterAdvisoryBundleRunner {
    let location: ConverterAdvisoryBundleScriptLocation
    let language: String
    let sourceURL: URL
    let outputDirectoryURL: URL

    func commandArguments() -> [String] {
        [
            "env",
            "PYTHONDONTWRITEBYTECODE=1",
            "python3",
            location.scriptURL.path,
            "--language",
            language,
            "--source",
            sourceURL.path,
            "--output-dir",
            outputDirectoryURL.path,
            "--format",
            "markdown",
        ]
    }

    func run() -> GarnetCommandResult {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = commandArguments()
        process.currentDirectoryURL = location.repoRootURL

        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = outputPipe

        let command = (["/usr/bin/env"] + (process.arguments ?? [])).joined(separator: " ")
        do {
            try process.run()
            process.waitUntilExit()
            let data = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""
            return GarnetCommandResult(command: command, exitCode: process.terminationStatus, output: output)
        } catch {
            return GarnetCommandResult(command: command, exitCode: 127, output: error.localizedDescription)
        }
    }
}

struct ConverterAdvisoryReviewRunner {
    let location: ConverterAdvisoryReviewScriptLocation
    let bundleDirectoryURL: URL
    let outputDirectoryURL: URL

    func commandArguments() -> [String] {
        [
            "env",
            "PYTHONDONTWRITEBYTECODE=1",
            "python3",
            location.scriptURL.path,
            "--bundle-dir",
            bundleDirectoryURL.path,
            "--output-dir",
            outputDirectoryURL.path,
        ]
    }

    func run() -> GarnetCommandResult {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = commandArguments()
        process.currentDirectoryURL = location.repoRootURL

        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = outputPipe

        let command = (["/usr/bin/env"] + (process.arguments ?? [])).joined(separator: " ")
        do {
            try process.run()
            process.waitUntilExit()
            let data = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""
            return GarnetCommandResult(command: command, exitCode: process.terminationStatus, output: output)
        } catch {
            return GarnetCommandResult(command: command, exitCode: 127, output: error.localizedDescription)
        }
    }
}

struct ConverterAdvisoryHandoffRunner {
    let location: ConverterAdvisoryHandoffScriptLocation
    let bundleDirectoryURL: URL
    let reviewDirectoryURL: URL
    let outputDirectoryURL: URL

    func commandArguments() -> [String] {
        [
            "env",
            "PYTHONDONTWRITEBYTECODE=1",
            "python3",
            location.scriptURL.path,
            "--bundle-dir",
            bundleDirectoryURL.path,
            "--review-dir",
            reviewDirectoryURL.path,
            "--output-dir",
            outputDirectoryURL.path,
        ]
    }

    func run() -> GarnetCommandResult {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = commandArguments()
        process.currentDirectoryURL = location.repoRootURL

        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = outputPipe

        let command = (["/usr/bin/env"] + (process.arguments ?? [])).joined(separator: " ")
        do {
            try process.run()
            process.waitUntilExit()
            let data = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""
            return GarnetCommandResult(command: command, exitCode: process.terminationStatus, output: output)
        } catch {
            return GarnetCommandResult(command: command, exitCode: 127, output: error.localizedDescription)
        }
    }
}

struct MitReadinessRunner {
    let location: MitReadinessScriptLocation

    func commandArguments() -> [String] {
        [
            "env",
            "PYTHONDONTWRITEBYTECODE=1",
            "python3",
            location.scriptURL.path,
            "--format",
            "markdown",
        ]
    }

    func run() -> GarnetCommandResult {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = commandArguments()
        process.currentDirectoryURL = location.repoRootURL

        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = outputPipe

        let command = (["/usr/bin/env"] + (process.arguments ?? [])).joined(separator: " ")
        do {
            try process.run()
            process.waitUntilExit()
            let data = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""
            return GarnetCommandResult(command: command, exitCode: process.terminationStatus, output: output)
        } catch {
            return GarnetCommandResult(command: command, exitCode: 127, output: error.localizedDescription)
        }
    }
}

struct MacContinuationRunner {
    let location: MacContinuationScriptLocation

    func commandArguments() -> [String] {
        [
            "env",
            "PYTHONDONTWRITEBYTECODE=1",
            "python3",
            location.scriptURL.path,
            "--format",
            "markdown",
        ]
    }

    func run() -> GarnetCommandResult {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = commandArguments()
        process.currentDirectoryURL = location.repoRootURL

        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = outputPipe

        let command = (["/usr/bin/env"] + (process.arguments ?? [])).joined(separator: " ")
        do {
            try process.run()
            process.waitUntilExit()
            let data = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""
            return GarnetCommandResult(command: command, exitCode: process.terminationStatus, output: output)
        } catch {
            return GarnetCommandResult(command: command, exitCode: 127, output: error.localizedDescription)
        }
    }
}

struct GarnetStudioEvidenceDirectory {
    let homeDirectoryURL: URL

    init(homeDirectoryURL: URL = FileManager.default.homeDirectoryForCurrentUser) {
        self.homeDirectoryURL = homeDirectoryURL
    }

    func advisoryBundleDirectory(stamp: String = GarnetStudioEvidenceDirectory.timestamp()) -> URL {
        homeDirectoryURL
            .appendingPathComponent("Desktop", isDirectory: true)
            .appendingPathComponent("dogfood", isDirectory: true)
            .appendingPathComponent("garnet-studio-advisory-bundle-\(stamp)", isDirectory: true)
    }

    func advisoryReviewDirectory(stamp: String = GarnetStudioEvidenceDirectory.timestamp()) -> URL {
        homeDirectoryURL
            .appendingPathComponent("Desktop", isDirectory: true)
            .appendingPathComponent("dogfood", isDirectory: true)
            .appendingPathComponent("garnet-studio-advisory-review-\(stamp)", isDirectory: true)
    }

    func advisoryHandoffDirectory(stamp: String = GarnetStudioEvidenceDirectory.timestamp()) -> URL {
        homeDirectoryURL
            .appendingPathComponent("Desktop", isDirectory: true)
            .appendingPathComponent("dogfood", isDirectory: true)
            .appendingPathComponent("garnet-studio-advisory-handoff-\(stamp)", isDirectory: true)
    }

    static func timestamp(date: Date = Date()) -> String {
        let formatter = DateFormatter()
        formatter.calendar = Calendar(identifier: .gregorian)
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.timeZone = TimeZone(secondsFromGMT: 0)
        formatter.dateFormat = "yyyyMMdd-HHmmss"
        return formatter.string(from: date)
    }
}

struct AgenticDogfoodRunner {
    let location: AgenticDogfoodScriptLocation
    let garnetBinaryPath: String?
    let appExecutablePath: String?

    init(
        location: AgenticDogfoodScriptLocation,
        garnetBinaryPath: String?,
        appExecutablePath: String? = nil
    ) {
        self.location = location
        self.garnetBinaryPath = garnetBinaryPath
        self.appExecutablePath = appExecutablePath
    }

    static func checkoutGarnetBinary(for location: AgenticDogfoodScriptLocation, fileManager: FileManager = .default) -> String? {
        let debugBinary = location.repoRootURL
            .appendingPathComponent("target", isDirectory: true)
            .appendingPathComponent("debug", isDirectory: true)
            .appendingPathComponent("garnet")
            .path
        if fileManager.isExecutableFile(atPath: debugBinary) {
            return debugBinary
        }
        let bundledBinary = location.repoRootURL.appendingPathComponent("garnet").path
        return fileManager.isExecutableFile(atPath: bundledBinary) ? bundledBinary : nil
    }

    static func appBundleExecutable(for location: AgenticDogfoodScriptLocation, fileManager: FileManager = .default) -> String? {
        let executable = location.repoRootURL
            .deletingLastPathComponent()
            .appendingPathComponent("MacOS", isDirectory: true)
            .appendingPathComponent("GarnetStudio")
            .path
        return fileManager.isExecutableFile(atPath: executable) ? executable : nil
    }

    static func outputProvesCompleteReadiness(_ output: String) -> Bool {
        var sawReadiness100 = false
        var sawCompleteProbeCount = false

        for rawLine in output.split(whereSeparator: \.isNewline) {
            let line = rawLine.trimmingCharacters(in: .whitespacesAndNewlines)
            if line == "readiness=100" {
                sawReadiness100 = true
            }

            guard line.hasPrefix("passed=") else {
                continue
            }
            let value = line.dropFirst("passed=".count)
            let parts = value.split(separator: "/", omittingEmptySubsequences: false)
            guard parts.count == 2,
                  let passed = Int(parts[0]),
                  let total = Int(parts[1]),
                  total > 0,
                  passed == total
            else {
                continue
            }
            sawCompleteProbeCount = true
        }

        return sawReadiness100 && sawCompleteProbeCount
    }

    func commandArguments(copyToDesktop: Bool = true, strict: Bool = true) -> [String] {
        var arguments = ["env", "PYTHONDONTWRITEBYTECODE=1", "python3", location.scriptURL.path]
        if let garnetBinaryPath, !garnetBinaryPath.isEmpty {
            arguments.append(contentsOf: ["--garnet-bin", garnetBinaryPath])
        }
        if let appExecutablePath, !appExecutablePath.isEmpty {
            arguments.append(contentsOf: ["--app-executable", appExecutablePath])
        }
        if copyToDesktop {
            arguments.append("--copy-to-desktop")
        }
        if strict {
            arguments.append("--strict")
        }
        return arguments
    }

    func run(copyToDesktop: Bool = true, strict: Bool = true) -> GarnetCommandResult {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = commandArguments(copyToDesktop: copyToDesktop, strict: strict)
        process.currentDirectoryURL = location.repoRootURL

        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = outputPipe

        let command = (["/usr/bin/env"] + (process.arguments ?? [])).joined(separator: " ")
        do {
            try process.run()
            process.waitUntilExit()
            let data = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""
            return GarnetCommandResult(command: command, exitCode: process.terminationStatus, output: output)
        } catch {
            return GarnetCommandResult(command: command, exitCode: 127, output: error.localizedDescription)
        }
    }
}

@MainActor
final class GarnetStudioViewModel: ObservableObject {
    @Published var selectedSection: StudioSection = .overview
    @Published var selectedSample: GarnetSample = GarnetSampleCatalog.samples[0]
    @Published var sourceText: String = GarnetSampleCatalog.samples[0].source
    @Published var converterLanguage: String = "python"
    @Published var output: String = "Run a health check or pick a sample to start."
    @Published var lastStatus: GarnetCommandStatus?
    @Published var cliPath: String?
    @Published var agenticMatrixPath: String?
    @Published var assistPlanPath: String?
    @Published var advisoryBundlePath: String?
    @Published var advisoryReviewPath: String?
    @Published var advisoryHandoffPath: String?
    @Published var mitReadinessPath: String?
    @Published var macContinuationPath: String?

    private let locator: GarnetCLILocator
    private let matrixLocator: AgenticDogfoodScriptLocator
    private let assistPlanLocator: ConverterAssistPlanScriptLocator
    private let advisoryBundleLocator: ConverterAdvisoryBundleScriptLocator
    private let advisoryReviewLocator: ConverterAdvisoryReviewScriptLocator
    private let advisoryHandoffLocator: ConverterAdvisoryHandoffScriptLocator
    private let mitReadinessLocator: MitReadinessScriptLocator
    private let macContinuationLocator: MacContinuationScriptLocator

    init(
        locator: GarnetCLILocator = GarnetCLILocator(),
        matrixLocator: AgenticDogfoodScriptLocator = AgenticDogfoodScriptLocator(),
        assistPlanLocator: ConverterAssistPlanScriptLocator = ConverterAssistPlanScriptLocator(),
        advisoryBundleLocator: ConverterAdvisoryBundleScriptLocator = ConverterAdvisoryBundleScriptLocator(),
        advisoryReviewLocator: ConverterAdvisoryReviewScriptLocator = ConverterAdvisoryReviewScriptLocator(),
        advisoryHandoffLocator: ConverterAdvisoryHandoffScriptLocator = ConverterAdvisoryHandoffScriptLocator(),
        mitReadinessLocator: MitReadinessScriptLocator = MitReadinessScriptLocator(),
        macContinuationLocator: MacContinuationScriptLocator = MacContinuationScriptLocator()
    ) {
        self.locator = locator
        self.matrixLocator = matrixLocator
        self.assistPlanLocator = assistPlanLocator
        self.advisoryBundleLocator = advisoryBundleLocator
        self.advisoryReviewLocator = advisoryReviewLocator
        self.advisoryHandoffLocator = advisoryHandoffLocator
        self.mitReadinessLocator = mitReadinessLocator
        self.macContinuationLocator = macContinuationLocator
        self.cliPath = locator.locate()
        self.agenticMatrixPath = matrixLocator.locate()?.scriptURL.path
        self.assistPlanPath = assistPlanLocator.locate()?.scriptURL.path
        self.advisoryBundlePath = advisoryBundleLocator.locate()?.scriptURL.path
        self.advisoryReviewPath = advisoryReviewLocator.locate()?.scriptURL.path
        self.advisoryHandoffPath = advisoryHandoffLocator.locate()?.scriptURL.path
        self.mitReadinessPath = mitReadinessLocator.locate()?.scriptURL.path
        self.macContinuationPath = macContinuationLocator.locate()?.scriptURL.path
    }

    func select(sample: GarnetSample) {
        selectedSample = sample
        sourceText = sample.source
        converterLanguage = sample.language ?? converterLanguage
        selectedSection = sample.mode == .convert ? .converter : .examples
    }

    func runHealthCheck() {
        run(arguments: ["version"])
    }

    func runSelectedSample() {
        switch selectedSample.mode {
        case .parse:
            runSource(arguments: ["parse"], filename: selectedSample.filename)
        case .check:
            runSource(arguments: ["check"], filename: selectedSample.filename)
        case .run:
            runSource(arguments: ["run"], filename: selectedSample.filename)
        case .convert:
            runConverter()
        }
    }

    func runParse() {
        runSource(arguments: ["parse"], filename: "studio-input.garnet")
    }

    func runCheck() {
        runSource(arguments: ["check"], filename: "studio-input.garnet")
    }

    func runProgram() {
        runSource(arguments: ["run"], filename: "studio-input.garnet")
    }

    func runConverter() {
        guard ["python", "ruby", "rust", "go"].contains(converterLanguage) else {
            output = "\(converterLanguage) is planned-only today. Use Assist Plan for deterministic migration evidence without claiming active conversion."
            lastStatus = .failure
            return
        }
        let ext = fileExtension(for: converterLanguage)
        runSource(arguments: ["convert", converterLanguage], filename: "studio-input.\(ext)")
    }

    func runConverterAssistPlan() {
        guard let location = assistPlanLocator.locate() else {
            output = "No converter assist-plan script found. Open Garnet Studio from a source checkout or use the packaged app resources."
            lastStatus = .failure
            return
        }

        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("GarnetStudioAssist-\(UUID().uuidString)", isDirectory: true)
        do {
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            let file = directory.appendingPathComponent("studio-input.\(fileExtension(for: converterLanguage))")
            try sourceText.write(to: file, atomically: true, encoding: .utf8)
            output = "Planning Garnet-aware migration evidence..."
            let result = ConverterAssistPlanRunner(
                location: location,
                language: converterLanguage,
                sourceURL: file
            ).run()
            apply(result: result)
            assistPlanPath = location.scriptURL.path
        } catch {
            output = "Failed to prepare converter assist plan input: \(error.localizedDescription)"
            lastStatus = .failure
        }
    }

    func runConverterAdvisoryBundle() {
        guard let location = advisoryBundleLocator.locate() else {
            output = "No converter advisory-bundle script found. Open Garnet Studio from a source checkout or use the packaged app resources."
            lastStatus = .failure
            return
        }

        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("GarnetStudioAdvisory-\(UUID().uuidString)", isDirectory: true)
        let bundleDirectory = GarnetStudioEvidenceDirectory().advisoryBundleDirectory()
        do {
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            let file = directory.appendingPathComponent("studio-input.\(fileExtension(for: converterLanguage))")
            try sourceText.write(to: file, atomically: true, encoding: .utf8)
            output = "Building provider-neutral advisory bundle..."
            let result = ConverterAdvisoryBundleRunner(
                location: location,
                language: converterLanguage,
                sourceURL: file,
                outputDirectoryURL: bundleDirectory
            ).run()
            apply(result: result)
            if result.status == .success {
                output += "\nBundle output: \(bundleDirectory.path)"
            }
            advisoryBundlePath = location.scriptURL.path
        } catch {
            output = "Failed to prepare converter advisory bundle input: \(error.localizedDescription)"
            lastStatus = .failure
        }
    }

    func runConverterAdvisoryReview() {
        guard let bundleLocation = advisoryBundleLocator.locate() else {
            output = "No converter advisory-bundle script found. Open Garnet Studio from a source checkout or use the packaged app resources."
            lastStatus = .failure
            return
        }
        guard let reviewLocation = advisoryReviewLocator.locate() else {
            output = "No converter advisory-review script found. Open Garnet Studio from a source checkout or use the packaged app resources."
            lastStatus = .failure
            return
        }

        let stamp = GarnetStudioEvidenceDirectory.timestamp()
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("GarnetStudioAdvisoryReview-\(UUID().uuidString)", isDirectory: true)
        let evidence = GarnetStudioEvidenceDirectory()
        let bundleDirectory = evidence.advisoryBundleDirectory(stamp: stamp)
        let reviewDirectory = evidence.advisoryReviewDirectory(stamp: stamp)
        do {
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            let file = directory.appendingPathComponent("studio-input.\(fileExtension(for: converterLanguage))")
            try sourceText.write(to: file, atomically: true, encoding: .utf8)
            output = "Building and reviewing provider-neutral advisory evidence..."
            let bundleResult = ConverterAdvisoryBundleRunner(
                location: bundleLocation,
                language: converterLanguage,
                sourceURL: file,
                outputDirectoryURL: bundleDirectory
            ).run()
            if bundleResult.status != .success {
                apply(result: bundleResult)
                advisoryBundlePath = bundleLocation.scriptURL.path
                return
            }
            let reviewResult = ConverterAdvisoryReviewRunner(
                location: reviewLocation,
                bundleDirectoryURL: bundleDirectory,
                outputDirectoryURL: reviewDirectory
            ).run()
            apply(result: reviewResult)
            if reviewResult.status == .success {
                output += "\nBundle output: \(bundleDirectory.path)"
                output += "\nReview output: \(reviewDirectory.path)"
            }
            advisoryBundlePath = bundleLocation.scriptURL.path
            advisoryReviewPath = reviewLocation.scriptURL.path
        } catch {
            output = "Failed to prepare converter advisory review input: \(error.localizedDescription)"
            lastStatus = .failure
        }
    }

    func runConverterAdvisoryHandoff() {
        guard let bundleLocation = advisoryBundleLocator.locate() else {
            output = "No converter advisory-bundle script found. Open Garnet Studio from a source checkout or use the packaged app resources."
            lastStatus = .failure
            return
        }
        guard let reviewLocation = advisoryReviewLocator.locate() else {
            output = "No converter advisory-review script found. Open Garnet Studio from a source checkout or use the packaged app resources."
            lastStatus = .failure
            return
        }
        guard let handoffLocation = advisoryHandoffLocator.locate() else {
            output = "No converter advisory-handoff script found. Open Garnet Studio from a source checkout or use the packaged app resources."
            lastStatus = .failure
            return
        }

        let stamp = GarnetStudioEvidenceDirectory.timestamp()
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("GarnetStudioAdvisoryHandoff-\(UUID().uuidString)", isDirectory: true)
        let evidence = GarnetStudioEvidenceDirectory()
        let bundleDirectory = evidence.advisoryBundleDirectory(stamp: stamp)
        let reviewDirectory = evidence.advisoryReviewDirectory(stamp: stamp)
        let handoffDirectory = evidence.advisoryHandoffDirectory(stamp: stamp)
        do {
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            let file = directory.appendingPathComponent("studio-input.\(fileExtension(for: converterLanguage))")
            try sourceText.write(to: file, atomically: true, encoding: .utf8)
            output = "Building, reviewing, and packaging provider-neutral advisory handoff evidence..."
            let bundleResult = ConverterAdvisoryBundleRunner(
                location: bundleLocation,
                language: converterLanguage,
                sourceURL: file,
                outputDirectoryURL: bundleDirectory
            ).run()
            if bundleResult.status != .success {
                apply(result: bundleResult)
                advisoryBundlePath = bundleLocation.scriptURL.path
                return
            }
            let reviewResult = ConverterAdvisoryReviewRunner(
                location: reviewLocation,
                bundleDirectoryURL: bundleDirectory,
                outputDirectoryURL: reviewDirectory
            ).run()
            if reviewResult.status != .success {
                apply(result: reviewResult)
                advisoryBundlePath = bundleLocation.scriptURL.path
                advisoryReviewPath = reviewLocation.scriptURL.path
                return
            }
            let handoffResult = ConverterAdvisoryHandoffRunner(
                location: handoffLocation,
                bundleDirectoryURL: bundleDirectory,
                reviewDirectoryURL: reviewDirectory,
                outputDirectoryURL: handoffDirectory
            ).run()
            apply(result: handoffResult)
            if handoffResult.status == .success {
                output += "\nBundle output: \(bundleDirectory.path)"
                output += "\nReview output: \(reviewDirectory.path)"
                output += "\nHandoff output: \(handoffDirectory.path)"
            }
            advisoryBundlePath = bundleLocation.scriptURL.path
            advisoryReviewPath = reviewLocation.scriptURL.path
            advisoryHandoffPath = handoffLocation.scriptURL.path
        } catch {
            output = "Failed to prepare converter advisory handoff input: \(error.localizedDescription)"
            lastStatus = .failure
        }
    }

    func runAgenticStressTests() {
        guard let location = matrixLocator.locate() else {
            output = "No agentic dogfood matrix found. Open Garnet Studio from a source checkout or set GARNET_REPO_ROOT to the repository root."
            lastStatus = .failure
            return
        }
        output = "Running the agentic dogfood matrix..."
        let result = AgenticDogfoodRunner(
            location: location,
            garnetBinaryPath: AgenticDogfoodRunner.checkoutGarnetBinary(for: location),
            appExecutablePath: AgenticDogfoodRunner.appBundleExecutable(for: location)
        ).run()
        apply(result: result)
        agenticMatrixPath = location.scriptURL.path
    }

    func runMitReadinessPulse() {
        guard let location = mitReadinessLocator.locate() else {
            output = "No MIT/productization readiness reporter found. Open Garnet Studio from a source checkout or use the packaged app resources."
            lastStatus = .failure
            return
        }
        output = "Loading Garnet MIT/productization objective pulse..."
        let result = MitReadinessRunner(location: location).run()
        apply(result: result)
        mitReadinessPath = location.scriptURL.path
    }

    func runMacContinuationPulse() {
        guard let location = macContinuationLocator.locate() else {
            output = "No Mac-side continuation reporter found. Open Garnet Studio from a source checkout or use the packaged app resources."
            lastStatus = .failure
            return
        }
        output = "Loading Garnet Mac-side continuation pulse..."
        let result = MacContinuationRunner(location: location).run()
        apply(result: result)
        macContinuationPath = location.scriptURL.path
    }

    private func run(arguments: [String]) {
        guard let cliPath else {
            output = "No Garnet CLI found. Bundle Garnet Studio with the CLI or install `garnet` on PATH."
            lastStatus = .failure
            return
        }
        let result = GarnetCLI(executablePath: cliPath).run(arguments: arguments)
        apply(result: result)
    }

    private func runSource(arguments: [String], filename: String) {
        guard let cliPath else {
            output = "No Garnet CLI found. Bundle Garnet Studio with the CLI or install `garnet` on PATH."
            lastStatus = .failure
            return
        }
        do {
            let directory = FileManager.default.temporaryDirectory
                .appendingPathComponent("GarnetStudio", isDirectory: true)
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            let file = directory.appendingPathComponent(filename)
            try sourceText.write(to: file, atomically: true, encoding: .utf8)
            let result = GarnetCLI(executablePath: cliPath).run(
                arguments: arguments + [file.path],
                workingDirectory: directory
            )
            apply(result: result)
        } catch {
            output = error.localizedDescription
            lastStatus = .failure
        }
    }

    private func fileExtension(for language: String) -> String {
        switch language {
        case "python":
            return "py"
        case "ruby":
            return "rb"
        case "rust":
            return "rs"
        case "javascript":
            return "js"
        case "typescript":
            return "ts"
        case "csharp":
            return "cs"
        case "perl":
            return "pl"
        default:
            return language
        }
    }

    private func apply(result: GarnetCommandResult) {
        lastStatus = result.status
        let status = result.status == .success ? "PASS" : "FAIL"
        output = "$ \(result.command)\n[\(status), exit \(result.exitCode)]\n\n\(result.output)"
    }
}

enum StudioSection: String, CaseIterable, Identifiable {
    case overview = "Overview"
    case examples = "Examples"
    case converter = "Converter"
    case agentic = "Agentic Tests"
    case release = "Release"

    var id: String { rawValue }
}

struct GarnetStudioRootView: View {
    @StateObject private var model = GarnetStudioViewModel()

    var body: some View {
        NavigationSplitView {
            List(StudioSection.allCases, selection: $model.selectedSection) { section in
                Text(section.rawValue).tag(section)
            }
            .navigationTitle("Garnet Studio")
        } detail: {
            VStack(spacing: 0) {
                header
                Divider()
                content
            }
            .background(Color(red: 0.05, green: 0.05, blue: 0.06))
        }
        .frame(minWidth: 1080, minHeight: 720)
    }

    private var header: some View {
        HStack(spacing: 16) {
            LogoView()
                .frame(width: 54, height: 54)
            VStack(alignment: .leading, spacing: 4) {
                Text("Garnet Studio")
                    .font(.system(size: 28, weight: .bold, design: .rounded))
                Text("Rust rigor. Ruby velocity. One coherent local workbench.")
                    .foregroundStyle(.secondary)
            }
            Spacer()
            StatusPill(status: model.lastStatus)
            Button("Health Check") { model.runHealthCheck() }
                .buttonStyle(.borderedProminent)
        }
        .padding(22)
    }

    @ViewBuilder
    private var content: some View {
        switch model.selectedSection {
        case .overview:
            overview
        case .examples:
            examples
        case .converter:
            converter
        case .agentic:
            agenticTests
        case .release:
            release
        }
    }

    private var overview: some View {
        WorkbenchLayout {
            VStack(alignment: .leading, spacing: 16) {
                Panel(title: "CLI Status") {
                    Label(model.cliPath ?? "No bundled or PATH Garnet CLI found", systemImage: model.cliPath == nil ? "exclamationmark.triangle" : "checkmark.seal")
                        .font(.system(size: 14, weight: .medium))
                    Text("Garnet Studio prefers a bundled CLI inside the app, then searches PATH, /usr/local/bin, /opt/homebrew/bin, and /usr/local/garnet/bin.")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
                Panel(title: "Core Workflows") {
                    WorkflowGrid(model: model)
                }
                Panel(title: "First-Run Onboarding") {
                    OnboardingChecklist()
                }
            }
        } trailing: {
            ConsoleView(output: model.output)
        }
    }

    private var examples: some View {
        WorkbenchLayout {
            VStack(alignment: .leading, spacing: 16) {
                Panel(title: "Runnable Samples") {
                    ForEach(GarnetSampleCatalog.samples) { sample in
                        SampleRow(sample: sample, selected: sample.id == model.selectedSample.id) {
                            model.select(sample: sample)
                        }
                    }
                }
                editor(actions: [
                    ("Parse", model.runParse),
                    ("Check", model.runCheck),
                    ("Run", model.runProgram),
                    ("Run Selected", model.runSelectedSample),
                ])
            }
        } trailing: {
            ConsoleView(output: model.output)
        }
    }

    private var converter: some View {
        WorkbenchLayout {
            VStack(alignment: .leading, spacing: 16) {
                Panel(title: "Code Converter") {
                    Picker("Language", selection: $model.converterLanguage) {
                        Text("Python").tag("python")
                        Text("Ruby").tag("ruby")
                        Text("Rust").tag("rust")
                        Text("Go").tag("go")
                        Text("TypeScript").tag("typescript")
                        Text("JavaScript").tag("javascript")
                        Text("Swift").tag("swift")
                        Text("Java").tag("java")
                        Text("C").tag("c")
                        Text("C++").tag("cpp")
                        Text("C#").tag("csharp")
                        Text("Perl").tag("perl")
                        Text("Kotlin").tag("kotlin")
                        Text("Shell").tag("shell")
                        Text("SQL").tag("sql")
                        Text("Other").tag("other")
                    }
                    .pickerStyle(.menu)
                    Text("The converter is a migration assistant. Active conversion is limited to Rust, Ruby, Python, and Go; advisory planning covers broader languages, while native-boundary code should stay behind FFI or native modules until explicit backend evidence lands. Advisory bundles omit source by default.")
                        .font(.callout)
                        .foregroundStyle(.secondary)
                }
                editor(actions: [
                    ("Convert", model.runConverter),
                    ("Assist Plan", model.runConverterAssistPlan),
                    ("Advisory Bundle", model.runConverterAdvisoryBundle),
                    ("Advisory Review", model.runConverterAdvisoryReview),
                    ("Advisory Handoff", model.runConverterAdvisoryHandoff),
                ])
            }
        } trailing: {
            ConsoleView(output: model.output)
        }
    }

    private var release: some View {
        WorkbenchLayout {
            VStack(alignment: .leading, spacing: 16) {
                Panel(title: "Release Evidence") {
                    ReleaseLine(label: "Tracked plan", value: "87/87 slices complete on current main")
                    ReleaseLine(label: "MIT objective", value: "Run Objective Pulse for the current repo-native percentage")
                    ReleaseLine(label: "Mac continuation", value: "Run Continuation Pulse for actionable Mac-side lanes and blocked/delegated gates")
                    ReleaseLine(label: "Org release", value: "v0.4.2 published with deb, rpm, macOS tarball, and SHA256SUMS")
                    ReleaseLine(label: "macOS app", value: "Local .app/.dmg packaging active in this slice")
                    ReleaseLine(label: "Deferred", value: "Developer ID signing, notarization, App Store, iOS, Android, Windows/Linux Studio, broad converter frontends, and provider-backed LLM conversion")
                    HStack {
                        Button("Objective Pulse", action: model.runMitReadinessPulse)
                            .buttonStyle(.borderedProminent)
                        Button("Continuation Pulse", action: model.runMacContinuationPulse)
                            .buttonStyle(.bordered)
                    }
                }
                Panel(title: "Install Philosophy") {
                    Text("Garnet should be approachable in the same spirit as modern agent workbench apps: download, open, see what is possible, and run a real tool immediately. This app keeps that promise grounded by launching the actual Garnet CLI.")
                        .font(.body)
                        .foregroundStyle(.secondary)
                }
            }
        } trailing: {
            ConsoleView(output: model.output)
        }
    }

    private var agenticTests: some View {
        WorkbenchLayout {
            VStack(alignment: .leading, spacing: 16) {
                Panel(title: "Agentic Stress Matrix") {
                    ReleaseLine(label: "Coverage", value: "Probe count is reported by the current matrix output")
                    ReleaseLine(label: "Matrix", value: model.agenticMatrixPath ?? "No source-tree matrix script found")
                    ReleaseLine(label: "Output", value: "Writes a timestamped bundle to ~/Desktop/dogfood and verifies its manifest")
                    Button("Run Agentic Stress Tests") {
                        model.runAgenticStressTests()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(model.agenticMatrixPath == nil)
                }
                Panel(title: "Readiness Boundary") {
                    Text("A passing matrix proves the audited agent-facing workflows in this checkout. Production ARC, native backend, mechanized proof, empirical validation, notarized distribution, real-browser web/PWA, mobile, and promo-video lanes still need their own gates.")
                        .font(.body)
                        .foregroundStyle(.secondary)
                }
            }
        } trailing: {
            ConsoleView(output: model.output)
        }
    }

    private func editor(actions: [(String, () -> Void)]) -> some View {
        Panel(title: "Source") {
            TextEditor(text: $model.sourceText)
                .font(.system(.body, design: .monospaced))
                .frame(minHeight: 260)
                .scrollContentBackground(.hidden)
                .background(Color.black.opacity(0.18))
                .clipShape(RoundedRectangle(cornerRadius: 8))
            HStack {
                ForEach(actions, id: \.0) { title, action in
                    Button(title, action: action)
                        .buttonStyle(.borderedProminent)
                }
                Spacer()
            }
        }
    }
}

struct WorkbenchLayout<Leading: View, Trailing: View>: View {
    let leading: Leading
    let trailing: Trailing

    init(@ViewBuilder leading: () -> Leading, @ViewBuilder trailing: () -> Trailing) {
        self.leading = leading()
        self.trailing = trailing()
    }

    var body: some View {
        HStack(alignment: .top, spacing: 18) {
            ScrollView { leading.padding(22) }
                .frame(minWidth: 500)
            trailing
                .frame(minWidth: 420)
        }
    }
}

struct Panel<Content: View>: View {
    let title: String
    let content: Content

    init(title: String, @ViewBuilder content: () -> Content) {
        self.title = title
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(title)
                .font(.headline)
            content
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.white.opacity(0.06))
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color.white.opacity(0.08))
        )
    }
}

struct LogoView: View {
    var body: some View {
        let mainURL = Bundle.main.resourceURL?.appendingPathComponent("garnet-logo.png")
        let moduleURL = Bundle.module.url(forResource: "garnet-logo", withExtension: "png")
        if let url = mainURL ?? moduleURL,
           let image = NSImage(contentsOf: url) {
            Image(nsImage: image)
                .resizable()
                .scaledToFit()
                .clipShape(RoundedRectangle(cornerRadius: 12))
        } else {
            RoundedRectangle(cornerRadius: 12)
                .fill(Color(red: 0.62, green: 0.17, blue: 0.18))
                .overlay(Text("G").font(.title.bold()))
        }
    }
}

struct StatusPill: View {
    let status: GarnetCommandStatus?

    var body: some View {
        let text = status == nil ? "Idle" : (status == .success ? "Passing" : "Needs attention")
        let color = status == .failure ? Color.orange : Color.green
        Text(text)
            .font(.caption.weight(.semibold))
            .padding(.horizontal, 12)
            .padding(.vertical, 7)
            .background(color.opacity(0.18))
            .foregroundStyle(color)
            .clipShape(Capsule())
    }
}

struct WorkflowGrid: View {
    @ObservedObject var model: GarnetStudioViewModel

    var body: some View {
        LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
            WorkflowButton(title: "Run examples", systemImage: "play.circle") {
                model.selectedSection = .examples
            }
            WorkflowButton(title: "Check safe mode", systemImage: "checkmark.shield") {
                if let sample = GarnetSampleCatalog.samples.first(where: { $0.mode == .check }) {
                    model.select(sample: sample)
                    model.runSelectedSample()
                }
            }
            WorkflowButton(title: "Convert code", systemImage: "arrow.triangle.2.circlepath") {
                if let sample = GarnetSampleCatalog.samples.first(where: { $0.mode == .convert }) {
                    model.select(sample: sample)
                }
            }
            WorkflowButton(title: "Agentic tests", systemImage: "checklist.checked") {
                model.selectedSection = .agentic
            }
            WorkflowButton(title: "Release status", systemImage: "shippingbox") {
                model.selectedSection = .release
            }
        }
    }
}

struct OnboardingChecklist: View {
    private let items = [
        ("1", "Verify the bundled CLI", "Run Health Check and confirm the console reports `garnet 0.4.2`."),
        ("2", "Run a real Garnet example", "Open Examples, run the scheduler MVP, and inspect the returned value."),
        ("3", "Try code conversion", "Open Converter, convert the Python route sample, and review the migration checklist."),
        ("4", "Run agentic stress tests", "Open Agentic Tests and generate a Desktop dogfood bundle from the matrix."),
        ("5", "Check release boundaries", "Open Release and confirm signed/native distribution caveats are still explicit."),
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            ForEach(items, id: \.0) { number, title, detail in
                HStack(alignment: .top, spacing: 10) {
                    Text(number)
                        .font(.caption.weight(.bold))
                        .frame(width: 24, height: 24)
                        .background(Color(red: 0.62, green: 0.17, blue: 0.18).opacity(0.35))
                        .clipShape(Circle())
                    VStack(alignment: .leading, spacing: 2) {
                        Text(title)
                            .font(.system(size: 13, weight: .semibold))
                        Text(detail)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
    }
}

struct WorkflowButton: View {
    let title: String
    let systemImage: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Label(title, systemImage: systemImage)
                .frame(maxWidth: .infinity, minHeight: 44)
        }
        .buttonStyle(.bordered)
    }
}

struct SampleRow: View {
    let sample: GarnetSample
    let selected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack {
                VStack(alignment: .leading, spacing: 3) {
                    Text(sample.title).font(.system(size: 14, weight: .semibold))
                    Text(sample.subtitle).font(.caption).foregroundStyle(.secondary)
                }
                Spacer()
                Text(sample.mode.rawValue)
                    .font(.caption.weight(.bold))
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.white.opacity(0.08))
                    .clipShape(Capsule())
            }
            .padding(10)
            .background(selected ? Color(red: 0.62, green: 0.17, blue: 0.18).opacity(0.24) : Color.clear)
            .clipShape(RoundedRectangle(cornerRadius: 8))
        }
        .buttonStyle(.plain)
    }
}

struct ConsoleView: View {
    let output: String

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Console")
                .font(.headline)
            ScrollView {
                Text(output)
                    .font(.system(.callout, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
                    .padding(14)
            }
            .background(Color.black.opacity(0.45))
            .clipShape(RoundedRectangle(cornerRadius: 8))
        }
        .padding(22)
        .background(Color.black.opacity(0.18))
    }
}

struct ReleaseLine: View {
    let label: String
    let value: String

    var body: some View {
        HStack(alignment: .top) {
            Text(label)
                .font(.system(size: 13, weight: .semibold))
                .frame(width: 120, alignment: .leading)
            Text(value)
                .font(.callout)
                .foregroundStyle(.secondary)
            Spacer()
        }
    }
}

enum GarnetStudioSelfTest {
    static func run() -> Int32 {
        var failures: [String] = []

        let bundled = URL(fileURLWithPath: "/Applications/Garnet Studio.app/Contents/Resources/garnet")
        let bundledLocator = GarnetCLILocator(
            bundleResourceURL: bundled.deletingLastPathComponent(),
            environmentPath: "/usr/local/bin:/opt/homebrew/bin"
        )
        if bundledLocator.candidatePaths().first != bundled.path {
            failures.append("bundled CLI path was not preferred")
        }

        let fallbackLocator = GarnetCLILocator(bundleResourceURL: nil, environmentPath: "/custom/bin:/usr/bin")
        let candidates = fallbackLocator.candidatePaths()
        for expected in ["/custom/bin/garnet", "/usr/local/bin/garnet", "/opt/homebrew/bin/garnet"] {
            if !candidates.contains(expected) {
                failures.append("missing fallback candidate: \(expected)")
            }
        }

        let modes = Set(GarnetSampleCatalog.samples.map(\.mode))
        for expected in GarnetSampleMode.allCases where !modes.contains(expected) {
            failures.append("sample catalog missing mode: \(expected.rawValue)")
        }

        if !StudioSection.allCases.contains(.agentic) {
            failures.append("studio sections missing Agentic Tests")
        }

        let matrixLocator = AgenticDogfoodScriptLocator(
            bundleResourceURL: nil,
            environmentRepoRoot: "/repo",
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )
        if matrixLocator.candidateLocations().first?.scriptURL.path != "/repo/scripts/run_agentic_dogfood_matrix.py" {
            failures.append("agentic matrix locator did not prefer GARNET_REPO_ROOT")
        }

        let assistLocator = ConverterAssistPlanScriptLocator(
            bundleResourceURL: nil,
            environmentRepoRoot: "/repo",
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )
        if assistLocator.candidateLocations().first?.scriptURL.path != "/repo/scripts/garnet_converter_assist_plan.py" {
            failures.append("converter assist-plan locator did not prefer GARNET_REPO_ROOT")
        }

        let continuationLocator = MacContinuationScriptLocator(
            bundleResourceURL: nil,
            environmentRepoRoot: "/repo",
            currentDirectoryURL: URL(fileURLWithPath: "/repo/apps/garnet-studio-macos", isDirectory: true)
        )
        if continuationLocator.candidateLocations().first?.scriptURL.path != "/repo/scripts/garnet_mac_side_continuation_status.py" {
            failures.append("mac continuation locator did not prefer GARNET_REPO_ROOT")
        }

        let success = GarnetCommandResult(command: "garnet version", exitCode: 0, output: "garnet 0.4.2")
        let failure = GarnetCommandResult(command: "garnet check broken.garnet", exitCode: 1, output: "diagnostic")
        if success.status != .success {
            failures.append("zero exit was not classified as success")
        }
        if failure.status != .failure {
            failures.append("nonzero exit was not classified as failure")
        }

        if failures.isEmpty {
            print("GarnetStudio self-test passed")
            return 0
        }
        for failure in failures {
            fputs("GarnetStudio self-test failed: \(failure)\n", stderr)
        }
        return 1
    }

    static func runSmokeTest() -> Int32 {
        let locator = GarnetCLILocator()
        guard let cliPath = locator.locate() else {
            fputs("GarnetStudio smoke failed: no Garnet CLI found\n", stderr)
            return 2
        }

        let cli = GarnetCLI(executablePath: cliPath)
        let version = cli.run(arguments: ["version"])
        guard version.status == .success, version.output.contains("garnet 0.4.2") else {
            fputs("GarnetStudio smoke failed during version:\n\(version.output)\n", stderr)
            return 3
        }

        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("GarnetStudioSmoke-\(UUID().uuidString)", isDirectory: true)
        do {
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            defer { try? FileManager.default.removeItem(at: directory) }

            for sample in GarnetSampleCatalog.samples {
                let file = directory.appendingPathComponent(sample.filename)
                try sample.source.write(to: file, atomically: true, encoding: .utf8)

                let arguments: [String]
                switch sample.mode {
                case .parse:
                    arguments = ["parse", file.path]
                case .check:
                    arguments = ["check", file.path]
                case .run:
                    arguments = ["run", file.path]
                case .convert:
                    arguments = ["convert", sample.language ?? "python", file.path]
                }

                let result = cli.run(arguments: arguments, workingDirectory: directory)
                guard result.status == .success else {
                    fputs("GarnetStudio smoke failed for \(sample.title):\n\(result.output)\n", stderr)
                    return 4
                }
            }
        } catch {
            fputs("GarnetStudio smoke failed while preparing samples: \(error.localizedDescription)\n", stderr)
            return 5
        }

        print("GarnetStudio smoke passed with \(cliPath)")
        return 0
    }

    static func runAgenticMatrixTest() -> Int32 {
        let matrixLocator = AgenticDogfoodScriptLocator()
        guard let location = matrixLocator.locate() else {
            fputs("GarnetStudio agentic matrix failed: no matrix script found\n", stderr)
            return 6
        }

        let result = AgenticDogfoodRunner(
            location: location,
            garnetBinaryPath: AgenticDogfoodRunner.checkoutGarnetBinary(for: location),
            appExecutablePath: AgenticDogfoodRunner.appBundleExecutable(for: location)
        ).run()
        guard result.status == .success,
              AgenticDogfoodRunner.outputProvesCompleteReadiness(result.output)
        else {
            fputs("GarnetStudio agentic matrix failed:\n\(result.output)\n", stderr)
            return 7
        }

        print("GarnetStudio agentic matrix passed")
        return 0
    }
}

@main
struct GarnetStudioApp: App {
    init() {
        if CommandLine.arguments.contains("--self-test") {
            Foundation.exit(GarnetStudioSelfTest.run())
        }
        if CommandLine.arguments.contains("--smoke-test") {
            Foundation.exit(GarnetStudioSelfTest.runSmokeTest())
        }
        if CommandLine.arguments.contains("--agentic-matrix-test") {
            Foundation.exit(GarnetStudioSelfTest.runAgenticMatrixTest())
        }
    }

    var body: some Scene {
        WindowGroup {
            GarnetStudioRootView()
                .preferredColorScheme(.dark)
        }
        .windowStyle(.titleBar)
    }
}
