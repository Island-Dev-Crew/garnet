// StudioShell.swift — the macOS port of the PR #391 Studio shell standard.
//
// This file ports the *standard*, not the Tauri code (per
// F_Project_Management/GARNET_STUDIO_SUITE_HANDOFF_2026_06_12.md §1):
//
//   Row 1  Version truth        — one stamp, gated against the workspace release
//                                 by scripts/test_garnet_macos_studio_shell.py.
//   Row 4  Validated settings   — JSON settings clamped in normalized();
//                                 a corrupt or missing file never blocks boot.
//   Row 5  Process discipline   — per-category timeout, thread-drained pipes
//                                 (no pipe deadlocks), best-effort process-tree
//                                 SIGKILL, timed_out + duration surfaced, UI
//                                 payload caps with honest markers.
//   Row 6  Truth surface        — live stats from docs/truth.json with an
//                                 explicit "unavailable" state; zero
//                                 hand-written release numbers.
//   Row 8  Evidence readers     — read-only, evidence-root-constrained
//                                 list/read (canonicalized, symlink-skipping,
//                                 size-capped).
//
// Rows 2/3/7/9 (splash, simple/power modes, hover help, keyboard/status
// bar/themes/a11y) live in GarnetStudioApp.swift on top of this layer.
//
// Boundaries (same as the Windows/Linux shell contract): no provider API call
// path, no network handoff, no credential storage. This file deliberately
// contains no network-client usage of any kind.

import Foundation

// MARK: - Row 1 · Version truth

/// The single version stamp for the macOS Studio app.
///
/// `scripts/test_garnet_macos_studio_shell.py` gates this constant against
/// `[workspace.package].version` in the root `Cargo.toml` — the Swift package
/// cannot inherit the workspace version, so the contract test *is* the sync
/// gate (the same mechanism the Windows shell uses for its excluded crate).
/// A workspace version bump must bump this constant in the same PR.
/// Never reintroduce a second hand-stamped version anywhere in the app.
public enum StudioVersion {
    public static let release = "0.8.1"
}

// MARK: - Row 4 · Validated settings

public enum StudioInterfaceMode: String, Codable, CaseIterable, Sendable {
    case simple
    case power
}

public enum StudioTheme: String, Codable, CaseIterable, Sendable {
    case system
    case dark
    case light
}

/// Persisted Studio settings, mirroring the Windows shell's `settings.rs`
/// contract: every write is validated/clamped in `normalized()`, and a corrupt
/// or missing settings file must never block startup — defaults win.
public struct StudioSettings: Codable, Equatable, Sendable {
    public var mode: StudioInterfaceMode
    public var theme: StudioTheme
    public var commandTimeoutSecs: Int
    public var matrixTimeoutSecs: Int

    public static let commandTimeoutRange = 5...600
    public static let matrixTimeoutRange = 30...1800

    public static let defaults = StudioSettings(
        mode: .simple,
        theme: .system,
        commandTimeoutSecs: 120,
        matrixTimeoutSecs: 600
    )

    public init(mode: StudioInterfaceMode, theme: StudioTheme, commandTimeoutSecs: Int, matrixTimeoutSecs: Int) {
        self.mode = mode
        self.theme = theme
        self.commandTimeoutSecs = commandTimeoutSecs
        self.matrixTimeoutSecs = matrixTimeoutSecs
    }

    /// Clamp every field into its valid range. All loads and saves go through
    /// this; out-of-range timeouts clamp rather than error.
    public func normalized() -> StudioSettings {
        StudioSettings(
            mode: mode,
            theme: theme,
            commandTimeoutSecs: min(max(commandTimeoutSecs, Self.commandTimeoutRange.lowerBound), Self.commandTimeoutRange.upperBound),
            matrixTimeoutSecs: min(max(matrixTimeoutSecs, Self.matrixTimeoutRange.lowerBound), Self.matrixTimeoutRange.upperBound)
        )
    }

    /// Tolerant decode: unknown/invalid enum strings fall back to defaults so a
    /// hand-edited settings file degrades instead of failing the boot path.
    private struct Raw: Codable {
        var mode: String?
        var theme: String?
        var commandTimeoutSecs: Int?
        var matrixTimeoutSecs: Int?
    }

    public static func decodeTolerantly(from data: Data) -> StudioSettings {
        guard let raw = try? JSONDecoder().decode(Raw.self, from: data) else {
            return .defaults
        }
        let settings = StudioSettings(
            mode: raw.mode.flatMap(StudioInterfaceMode.init(rawValue:)) ?? Self.defaults.mode,
            theme: raw.theme.flatMap(StudioTheme.init(rawValue:)) ?? Self.defaults.theme,
            commandTimeoutSecs: raw.commandTimeoutSecs ?? Self.defaults.commandTimeoutSecs,
            matrixTimeoutSecs: raw.matrixTimeoutSecs ?? Self.defaults.matrixTimeoutSecs
        )
        return settings.normalized()
    }
}

/// JSON-on-disk store under the per-user Application Support directory.
/// Corrupt file, missing directory, or unwritable disk never throws into the
/// boot path — every failure degrades to `StudioSettings.defaults`.
public struct StudioSettingsStore {
    public let fileURL: URL

    public init(fileURL: URL? = nil) {
        if let fileURL {
            self.fileURL = fileURL
        } else {
            let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
                ?? URL(fileURLWithPath: NSTemporaryDirectory())
            self.fileURL = support
                .appendingPathComponent("GarnetStudio", isDirectory: true)
                .appendingPathComponent("settings.json")
        }
    }

    public func load() -> StudioSettings {
        guard let data = try? Data(contentsOf: fileURL) else { return .defaults }
        return StudioSettings.decodeTolerantly(from: data)
    }

    @discardableResult
    public func save(_ settings: StudioSettings) -> Bool {
        let normalized = settings.normalized()
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        guard let data = try? encoder.encode(normalized) else { return false }
        do {
            try FileManager.default.createDirectory(
                at: fileURL.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            try data.write(to: fileURL, options: .atomic)
            return true
        } catch {
            return false
        }
    }
}

// MARK: - Row 5 · Process discipline

/// Result of a disciplined process run. `output` is capped for UI display with
/// an honest marker; `fullOutput` carries the complete streams for evidence
/// bundles. `timedOut` and `durationSeconds` are always surfaced.
public struct StudioProcessResult: Sendable {
    public let command: String
    public let exitCode: Int32
    public let output: String
    public let fullOutput: String
    public let timedOut: Bool
    public let durationSeconds: Double
    public let outputTruncatedForUI: Bool
}

public enum StudioProcessRunner {
    /// UI payload cap per run, mirroring the Windows shell's
    /// PAYLOAD_STREAM_CAP idea: the rendered console stays responsive while
    /// the full output still lands in the evidence bundle when one exists.
    public static let uiPayloadByteCap = 64 * 1024

    public static let truncationMarker =
        "\n…[output capped for UI display — full output is written to the evidence bundle when one exists]"

    /// Run a process with a hard timeout, thread-drained output (no pipe
    /// deadlocks), and best-effort process-tree SIGKILL on expiry.
    ///
    /// Every Studio spawn must go through this path, not raw
    /// `Process()` + `waitUntilExit()` (the same rule as the Windows shell's
    /// `run_process_with_timeout`).
    public static func run(
        executableURL: URL,
        arguments: [String],
        workingDirectory: URL? = nil,
        timeoutSeconds: Int
    ) -> StudioProcessResult {
        let command = ([executableURL.path] + arguments).joined(separator: " ")
        let process = Process()
        process.executableURL = executableURL
        process.arguments = arguments
        process.currentDirectoryURL = workingDirectory

        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe

        let start = Date()
        do {
            try process.run()
        } catch {
            return StudioProcessResult(
                command: command,
                exitCode: 127,
                output: error.localizedDescription,
                fullOutput: error.localizedDescription,
                timedOut: false,
                durationSeconds: 0,
                outputTruncatedForUI: false
            )
        }

        // Drain the pipe on a background thread so a chatty child can never
        // deadlock against a full pipe buffer while we wait for exit.
        final class DrainBox: @unchecked Sendable {
            var data = Data()
        }
        let drainBox = DrainBox()
        let drainDone = DispatchSemaphore(value: 0)
        let reader = pipe.fileHandleForReading
        DispatchQueue.global(qos: .userInitiated).async {
            drainBox.data = reader.readDataToEndOfFile()
            drainDone.signal()
        }

        // Wait for exit with a hard ceiling; SIGKILL the tree on expiry.
        let exitDone = DispatchSemaphore(value: 0)
        DispatchQueue.global(qos: .userInitiated).async {
            process.waitUntilExit()
            exitDone.signal()
        }

        var timedOut = false
        if exitDone.wait(timeout: .now() + .seconds(max(1, timeoutSeconds))) == .timedOut {
            timedOut = true
            killProcessTree(rootPid: process.processIdentifier)
            // The tree kill closes the write ends; both semaphores resolve.
            exitDone.wait()
        }
        drainDone.wait()

        let duration = Date().timeIntervalSince(start)
        let full = String(data: drainBox.data, encoding: .utf8) ?? ""
        let (display, truncated) = capForUI(full)
        let suffix = timedOut
            ? "\n[timed_out: killed after \(timeoutSeconds)s — partial output above]"
            : ""

        return StudioProcessResult(
            command: command,
            exitCode: process.terminationStatus,
            output: display + suffix,
            fullOutput: full,
            timedOut: timedOut,
            durationSeconds: duration,
            outputTruncatedForUI: truncated
        )
    }

    static func capForUI(_ full: String) -> (String, Bool) {
        let bytes = Array(full.utf8)
        guard bytes.count > uiPayloadByteCap else { return (full, false) }
        let head = bytes.prefix(uiPayloadByteCap)
        let capped = String(decoding: head, as: UTF8.self)
        return (capped + truncationMarker, true)
    }

    /// Best-effort process-tree kill: walk descendants via `pgrep -P`, then
    /// SIGKILL children-first, root last. "Best-effort" is the honest claim —
    /// a child that re-parents between the walk and the kill can escape; the
    /// timeout result is still reported either way.
    public static func killProcessTree(rootPid: Int32) {
        var order: [Int32] = []
        collectDescendants(of: rootPid, into: &order)
        for pid in order.reversed() {
            kill(pid, SIGKILL)
        }
        kill(rootPid, SIGKILL)
    }

    private static func collectDescendants(of pid: Int32, into order: inout [Int32]) {
        let pgrep = Process()
        pgrep.executableURL = URL(fileURLWithPath: "/usr/bin/pgrep")
        pgrep.arguments = ["-P", String(pid)]
        let out = Pipe()
        pgrep.standardOutput = out
        pgrep.standardError = Pipe()
        guard (try? pgrep.run()) != nil else { return }
        let data = out.fileHandleForReading.readDataToEndOfFile()
        pgrep.waitUntilExit()
        let children = String(decoding: data, as: UTF8.self)
            .split(whereSeparator: \.isNewline)
            .compactMap { Int32($0.trimmingCharacters(in: .whitespaces)) }
        for child in children {
            order.append(child)
            collectDescendants(of: child, into: &order)
        }
    }
}

// MARK: - Row 6 · Truth surface

/// Live release statistics read from `docs/truth.json` (the RB-0a truth
/// surface). The UI must render `unavailable` explicitly rather than invent
/// values, and must never reintroduce hand-written release numbers.
public enum StudioTruthSummary: Equatable, Sendable {
    case loaded(TruthFields)
    case unavailable(reason: String)

    public struct WorkspaceTests: Codable, Equatable, Sendable {
        public let passed: Int?
        public let failed: Int?
        public let measuredAtCommit: String?

        enum CodingKeys: String, CodingKey {
            case passed
            case failed
            case measuredAtCommit = "measured_at_commit"
        }
    }

    /// Mirrors the real shape emitted by `cargo xtask truth`. Note:
    /// `security_test_count` is deliberately absent — truth.json's omissions
    /// block refuses to stamp an unverifiable figure, and rendering one here
    /// would reintroduce exactly the drift the truth surface exists to kill.
    public struct TruthFields: Codable, Equatable, Sendable {
        public let version: String?
        public let primitiveCount: Int?
        public let workspaceTests: WorkspaceTests?
        public let latestTag: String?
        public let generatedAtCommit: String?

        enum CodingKeys: String, CodingKey {
            case version
            case primitiveCount = "primitive_count"
            case workspaceTests = "workspace_tests"
            case latestTag = "latest_tag"
            case generatedAtCommit = "generated_at_commit"
        }
    }

    public static func load(repoRoot: URL?) -> StudioTruthSummary {
        guard let repoRoot else {
            return .unavailable(reason: "repo root not located — truth surface unavailable")
        }
        let url = repoRoot.appendingPathComponent("docs/truth.json")
        guard let data = try? Data(contentsOf: url) else {
            return .unavailable(reason: "docs/truth.json not found — truth surface unavailable")
        }
        guard let fields = try? JSONDecoder().decode(TruthFields.self, from: data) else {
            return .unavailable(reason: "docs/truth.json unreadable — truth surface unavailable")
        }
        return .loaded(fields)
    }

    /// Walk ancestors of a starting directory looking for the repo root
    /// (identified by docs/truth.json beside a workspace Cargo.toml).
    public static func locateRepoRoot(startingAt start: URL = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)) -> URL? {
        var current = start.standardizedFileURL
        let fm = FileManager.default
        for _ in 0..<12 {
            let truth = current.appendingPathComponent("docs/truth.json")
            let cargo = current.appendingPathComponent("Cargo.toml")
            if fm.fileExists(atPath: truth.path), fm.fileExists(atPath: cargo.path) {
                return current
            }
            let parent = current.deletingLastPathComponent()
            if parent.path == current.path { break }
            current = parent
        }
        return nil
    }
}

// MARK: - Row 8 · Evidence readers

/// Read-only, evidence-root-constrained file access for previewing converter
/// and reporter output in-app. Mirrors the Windows shell's
/// `resolve_within_evidence_roots`: canonicalize both sides, reject anything
/// outside the Studio evidence roots, skip symlinks, cap sizes and entries.
/// Do not widen this into a general filesystem read primitive.
public struct StudioEvidenceReader {
    public let roots: [URL]

    public static let maxEntries = 200
    public static let maxReadBytes = 256 * 1024

    public init(roots: [URL]? = nil) {
        if let roots {
            self.roots = roots.map { $0.standardizedFileURL }
        } else {
            let home = FileManager.default.homeDirectoryForCurrentUser
            self.roots = [
                home.appendingPathComponent("Desktop/dogfood", isDirectory: true),
                URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
                    .appendingPathComponent("target/mac-studio-domain-proofs", isDirectory: true),
            ].map { $0.standardizedFileURL }
        }
    }

    public enum EvidenceError: Error, Equatable {
        case outsideRoots
        case isSymlink
        case tooLarge
        case unreadable
    }

    /// Canonicalize and confirm `path` is inside one of the evidence roots.
    public func resolveWithinEvidenceRoots(_ path: URL) -> Result<URL, EvidenceError> {
        let fm = FileManager.default
        // Reject symlinks before resolution so a link cannot smuggle a target
        // from outside the roots.
        if let values = try? path.resourceValues(forKeys: [.isSymbolicLinkKey]),
           values.isSymbolicLink == true {
            return .failure(.isSymlink)
        }
        let canonical = path.standardizedFileURL.resolvingSymlinksInPath()
        for root in roots {
            let canonicalRoot = root.resolvingSymlinksInPath()
            if canonical.path == canonicalRoot.path || canonical.path.hasPrefix(canonicalRoot.path + "/") {
                guard fm.fileExists(atPath: canonical.path) else {
                    return .failure(.unreadable)
                }
                return .success(canonical)
            }
        }
        // Honest, greppable refusal — same message family as the Windows shell.
        return .failure(.outsideRoots) // outside the Studio evidence roots
    }

    public func listEvidenceFiles(under directory: URL) -> Result<[String], EvidenceError> {
        switch resolveWithinEvidenceRoots(directory) {
        case .failure(let error):
            return .failure(error)
        case .success(let canonical):
            let fm = FileManager.default
            guard let names = try? fm.contentsOfDirectory(atPath: canonical.path) else {
                return .failure(.unreadable)
            }
            return .success(Array(names.sorted().prefix(Self.maxEntries)))
        }
    }

    /// Newest entries under a root by modification date (constrained, capped).
    /// Fixes the lexical-sort miscalibration: past `maxEntries` items a
    /// name-sorted prefix silently drops the newest bundles.
    public func newestEntries(under directory: URL, limit: Int = 20) -> Result<[String], EvidenceError> {
        switch resolveWithinEvidenceRoots(directory) {
        case .failure(let error):
            return .failure(error)
        case .success(let canonical):
            let fm = FileManager.default
            guard let urls = try? fm.contentsOfDirectory(
                at: canonical,
                includingPropertiesForKeys: [.contentModificationDateKey],
                options: [.skipsHiddenFiles]
            ) else {
                return .failure(.unreadable)
            }
            let dated = urls.map { url -> (String, Date) in
                let date = (try? url.resourceValues(forKeys: [.contentModificationDateKey]))?.contentModificationDate ?? .distantPast
                return (url.lastPathComponent, date)
            }
            let newest = dated.sorted { $0.1 > $1.1 }.prefix(min(limit, Self.maxEntries)).map(\.0)
            return .success(Array(newest))
        }
    }

    public func readEvidenceText(at path: URL) -> Result<String, EvidenceError> {
        switch resolveWithinEvidenceRoots(path) {
        case .failure(let error):
            return .failure(error)
        case .success(let canonical):
            guard let handle = try? FileHandle(forReadingFrom: canonical) else {
                return .failure(.unreadable)
            }
            defer { try? handle.close() }
            guard let data = try? handle.read(upToCount: Self.maxReadBytes + 1) else {
                return .failure(.unreadable)
            }
            if data.count > Self.maxReadBytes {
                let capped = data.prefix(Self.maxReadBytes)
                return .success(String(decoding: capped, as: UTF8.self) + StudioProcessRunner.truncationMarker)
            }
            return .success(String(decoding: data, as: UTF8.self))
        }
    }
}

// MARK: - Row 2 · Launch sequencing support

/// Boot/splash timing constants: the splash holds at least
/// `minimumMilliseconds` (no flash-through) and is force-dismissed at the
/// `ceilingMilliseconds` hard ceiling even if boot checks hang — the same
/// 700 ms / 25 s bounds as the Windows shell.
public enum StudioBootSequence {
    public static let minimumMilliseconds = 700
    public static let ceilingMilliseconds = 25_000

    public struct Status: Equatable, Sendable {
        public let message: String
        public let isComplete: Bool

        public init(message: String, isComplete: Bool) {
            self.message = message
            self.isComplete = isComplete
        }
    }
}

// MARK: - Legacy bridge

/// Process category → which settings timeout applies. The agentic stress
/// matrix gets the larger budget, same as the Windows shell.
public enum StudioProcessCategory: Sendable {
    case command
    case matrix
}

public extension StudioProcessRunner {
    /// Bridge for the app's existing call sites: same disciplined path
    /// (timeout, tree-kill, drained pipes, capped UI payload), returning the
    /// legacy command/exitCode/output triple with honest `timed_out` and
    /// truncation markers folded into the output text.
    static func runBridged(
        executable: String,
        arguments: [String],
        workingDirectory: URL?,
        category: StudioProcessCategory,
        settings: StudioSettings = StudioSettingsStore().load()
    ) -> (command: String, exitCode: Int32, output: String) {
        let normalized = settings.normalized()
        let timeout = category == .matrix ? normalized.matrixTimeoutSecs : normalized.commandTimeoutSecs
        let result = run(
            executableURL: URL(fileURLWithPath: executable),
            arguments: arguments,
            workingDirectory: workingDirectory,
            timeoutSeconds: timeout
        )
        return (result.command, result.exitCode, result.output)
    }
}
