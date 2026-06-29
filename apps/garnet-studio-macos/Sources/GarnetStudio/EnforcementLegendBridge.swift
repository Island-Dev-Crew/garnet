// M4 — Enforced / Declared Legend (pure catalog + projection).
//
// The single source of truth for WHICH fences the runtime actually *enforces*,
// which are only *declared*, and which are platform-*deferred*. Ported verbatim
// from the Windows shell's catalog (apps/garnet-studio/src-tauri/src/commands.rs
// `enforcement_catalog`) so the two Studios make the SAME claim. The
// enforced-vs-declared boundary is a load-bearing honesty surface: it is never
// widened here, and an "enforced" row reads as *confirmed live* ONLY when the
// live static-gate probe reproduced this run (see `EnforcementLegendCard`).
//
// Honesty anchors (CLAUDE.md): enforced = @caps + @max_depth (both backends;
// seccomp Linux-only); @bounded / @mailbox / memory / time are declared-not-
// enforced; OS sandbox off Linux is deferred. Garnet is research-grade (v0.x).

import Foundation

public enum FenceStatus: String, Equatable, Sendable {
    case enforced, declared, deferred

    public var label: String {
        switch self {
        case .enforced: return "Enforced"
        case .declared: return "Declared"
        case .deferred: return "Deferred"
        }
    }
}

/// One fence in the catalog. `runtimeAttestedBy` is the evidence trail for an
/// enforced runtime trap (shown but NOT re-run by the legend's probe);
/// `probeCode` is the static-gate diagnostic the live probe must reproduce.
public struct EnforcementFence: Equatable, Sendable {
    public let name: String
    public let status: FenceStatus
    public let backends: String
    public let basis: String
    public let runtimeAttestedBy: String
    public let probeCode: String

    public init(
        name: String, status: FenceStatus, backends: String, basis: String,
        runtimeAttestedBy: String, probeCode: String
    ) {
        self.name = name
        self.status = status
        self.backends = backends
        self.basis = basis
        self.runtimeAttestedBy = runtimeAttestedBy
        self.probeCode = probeCode
    }
}

/// The outcome of one live static-gate probe. A probe `confirmed` ONLY when it
/// ran AND observed the expected diagnostic; a missing/stale CLI (`ran == false`)
/// is inconclusive, never a confirmation.
public struct EnforcementProbe: Equatable, Sendable {
    public let fence: String
    public let expectedCode: String
    public let confirmed: Bool
    public let ran: Bool
    public let exitCode: Int
    public let observedCodes: [String]

    public init(
        fence: String, expectedCode: String, confirmed: Bool, ran: Bool, exitCode: Int,
        observedCodes: [String]
    ) {
        self.fence = fence
        self.expectedCode = expectedCode
        self.confirmed = confirmed
        self.ran = ran
        self.exitCode = exitCode
        self.observedCodes = observedCodes
    }

    /// Derive a probe verdict from a velocity check report. Pure (no CLI) so the
    /// confirm/inconclusive logic is unit-testable.
    public static func from(
        fence: String, expectedCode: String, report: VelocityReport
    ) -> EnforcementProbe {
        let observed = report.output?.diagnostics.map(\.code) ?? []
        let confirmed = report.ran && observed.contains(expectedCode)
        return EnforcementProbe(
            fence: fence, expectedCode: expectedCode, confirmed: confirmed, ran: report.ran,
            exitCode: report.exitCode, observedCodes: observed)
    }
}

/// The full legend payload: the catalog + live probes + whether a CLI was found.
public struct EnforcementLegend: Equatable, Sendable {
    public let fences: [EnforcementFence]
    public let probes: [EnforcementProbe]
    public let cliAvailable: Bool

    public init(fences: [EnforcementFence], probes: [EnforcementProbe], cliAvailable: Bool) {
        self.fences = fences
        self.probes = probes
        self.cliAvailable = cliAvailable
    }
}

public enum EnforcementCatalog {
    /// Probe fixture: a `@caps()` function that transitively calls `read_file`
    /// (requires `fs`). `garnet check` must flag `check.caps_coverage`.
    public static let capsProbeSource =
        "@caps()\ndef caller() -> String {\n  read_file(\"/tmp/garnet-studio-legend-probe\")\n}\n"

    /// Probe fixture: `@max_depth(100)` is outside the enforced `1..=64` range, so
    /// `garnet check` must flag `check.annotation_error`.
    public static let depthProbeSource = "@max_depth(100)\n@caps()\ndef f() -> Bool { true }\n"

    /// The fence catalog — ported verbatim from the Windows shell so both Studios
    /// claim the same enforced-vs-declared boundary. Never widen this here.
    public static func fences() -> [EnforcementFence] {
        [
            EnforcementFence(
                name: "@caps",
                status: .enforced,
                backends: "VM + interpreter",
                basis:
                    "Deny-by-default host authority: an undeclared fs/net/env/proc primitive "
                    + "traps at the boundary. The static caps-coverage gate flags a function "
                    + "that transitively requires a capability it does not declare.",
                runtimeAttestedBy:
                    "S100 require_capability trap (VM + interp); S114-FIX-2 deny-by-default at "
                    + "active_frames == 0; red-team",
                probeCode: "check.caps_coverage"),
            EnforcementFence(
                name: "@max_depth",
                status: .enforced,
                backends: "VM + interpreter",
                basis:
                    "Per-function recursion ceiling, trapped at depth N+1. The static gate "
                    + "enforces the 1..=64 range at check time.",
                runtimeAttestedBy: "S99 recursion-depth trap (VM + interp); red-team",
                probeCode: "check.annotation_error"),
            EnforcementFence(
                name: "@bounded",
                status: .declared,
                backends: "Wasmtime fuel only",
                basis:
                    "Lowers to Wasmtime fuel metering on the VM path (S39); not enforced on the "
                    + "interpreter. Declared, not a cross-backend trap.",
                runtimeAttestedBy: "", probeCode: ""),
            EnforcementFence(
                name: "@mailbox",
                status: .declared,
                backends: "actor runtime",
                basis:
                    "Overrides the default 1024-message inbox cap for an actor; not enforced at "
                    + "the host-authority boundary.",
                runtimeAttestedBy: "", probeCode: ""),
            EnforcementFence(
                name: "memory",
                status: .declared,
                backends: "—",
                basis: "Named-deferred resource ceiling: declared in source, no runtime trap.",
                runtimeAttestedBy: "", probeCode: ""),
            EnforcementFence(
                name: "time",
                status: .declared,
                backends: "—",
                basis:
                    "Named-deferred resource ceiling: `check` flags top-level under-declaration, "
                    + "but there is no runtime trap.",
                runtimeAttestedBy: "", probeCode: ""),
            EnforcementFence(
                name: "OS sandbox (macOS / Windows)",
                status: .deferred,
                backends: "Linux seccomp only",
                basis:
                    "Platform OS-sandbox application is deferred off Linux. Linux applies a "
                    + "seccomp policy; macOS and Windows do not apply an OS sandbox.",
                runtimeAttestedBy: "", probeCode: ""),
        ]
    }

    /// The enforced fences paired with the fixture whose `garnet check` run must
    /// reproduce that fence's catalog `probeCode`.
    public static func probeFixtures() -> [(fence: String, expected: String, source: String)] {
        [
            (fence: "@caps", expected: "check.caps_coverage", source: capsProbeSource),
            (fence: "@max_depth", expected: "check.annotation_error", source: depthProbeSource),
        ]
    }
}

/// The view-ready projection of an `EnforcementLegend`. `render` is the pure
/// function M4's SwiftUI view lays out; the honesty rules (confirmed-only-when-
/// reproduced; rows ordered enforced→declared→deferred) live here and are tested.
public struct EnforcementLegendCard: Equatable, Sendable {
    public enum GateState: String, Sendable { case confirmed, unconfirmed, notProbed, notApplicable }

    public struct Row: Equatable, Sendable {
        public let name: String
        public let status: FenceStatus
        public let statusLabel: String
        public let backends: String
        public let basis: String
        public let runtimeAttestedBy: String
        public let gateState: GateState
        public let gateLine: String
    }

    public let cliAvailable: Bool
    public let rows: [Row]

    public static func render(_ legend: EnforcementLegend) -> EnforcementLegendCard {
        let byFence = Dictionary(
            legend.probes.map { ($0.fence, $0) }, uniquingKeysWith: { first, _ in first })
        let order: [FenceStatus] = [.enforced, .declared, .deferred]
        let rows = order.flatMap { status in
            legend.fences.filter { $0.status == status }
        }.map { fence -> Row in
            let (state, line) = gate(for: fence, probe: byFence[fence.name])
            return Row(
                name: fence.name, status: fence.status, statusLabel: fence.status.label,
                backends: fence.backends, basis: fence.basis,
                runtimeAttestedBy: fence.runtimeAttestedBy, gateState: state, gateLine: line)
        }
        return EnforcementLegendCard(cliAvailable: legend.cliAvailable, rows: rows)
    }

    /// The static-gate line for a fence. CONFIRMED only when the live probe
    /// reproduced the expected diagnostic this run; otherwise an honest
    /// not-confirmed / not-probed — never a faked green. Non-enforced fences
    /// carry no gate line.
    private static func gate(
        for fence: EnforcementFence, probe: EnforcementProbe?
    ) -> (GateState, String) {
        guard fence.status == .enforced, let probe else { return (.notApplicable, "") }
        if probe.confirmed {
            return (
                .confirmed,
                "✓ Static gate confirmed live this run — \(probe.expectedCode) reproduced "
                    + "(exit \(probe.exitCode))."
            )
        }
        if probe.ran {
            let saw =
                probe.observedCodes.isEmpty
                ? "no diagnostics" : probe.observedCodes.joined(separator: ", ")
            return (
                .unconfirmed,
                "⚠ Static gate NOT confirmed this run — expected \(probe.expectedCode), saw \(saw)."
            )
        }
        return (
            .notProbed,
            "Static gate not probed — no Garnet CLI to run \(probe.expectedCode)."
        )
    }
}
