// M7 — Distribution Reporter (pure catalog + projection).
//
// An honest macOS packaging/notarization status surface. The load-bearing truth:
// the Garnet Studio .app is NOT code-signed and NOT notarized — it is a
// research-grade prototype for local run, not a Gatekeeper-distributable build.
// Packaging artifacts that DO exist (the packager, the DMG smoke) are probed live
// on the filesystem; the signing/notarization items are catalog truths marked
// `deferred`. Nothing here claims a readiness the pipeline does not have.

import Foundation

public enum DistributionStatus: String, Equatable, Sendable {
    case ready  // an artifact exists / the step is done
    case deferred  // a named-deferred step (signing, notarization)
    case absent  // a probed artifact was expected but not found
    case unverified  // a probe could not run (no repo root provided)

    public var label: String {
        switch self {
        case .ready: return "Ready"
        case .deferred: return "Deferred"
        case .absent: return "Absent"
        case .unverified: return "Unverified"
        }
    }
}

public struct DistributionItem: Equatable, Sendable {
    public let name: String
    public let status: DistributionStatus
    public let detail: String
}

/// The distribution catalog. Items with a `probePath` are verified against the
/// filesystem (a repo root the user provides); items without one are catalog
/// truths (the signing/notarization posture).
enum DistributionCatalogEntry {
    static let entries:
        [(name: String, probePath: String?, catalog: DistributionStatus, detail: String)] = [
            (
                name: "Studio .app packager",
                probePath: "scripts/package_garnet_studio_macos.sh",
                catalog: .ready,
                detail:
                    "Builds the .app bundle, version-stamped from Cargo.toml. The bundle is UNSIGNED."
            ),
            (
                name: "DMG smoke harness",
                probePath: "scripts/smoke_garnet_studio_dmg.sh",
                catalog: .ready,
                detail:
                    "Verifies the packaged bundle structure + bundled source/logo; not a signed-DMG proof."
            ),
            (
                name: "Code signing (Developer ID)",
                probePath: nil,
                catalog: .deferred,
                detail:
                    "No Developer ID signing in the macOS pipeline — the .app ships unsigned."
            ),
            (
                name: "Notarization (Apple notary)",
                probePath: nil,
                catalog: .deferred,
                detail:
                    "Not submitted to Apple notarization; Gatekeeper will quarantine the app on first open."
            ),
            (
                name: "CLI signed artifacts",
                probePath: nil,
                catalog: .ready,
                detail:
                    "Separate from the .app: the v0.8.1 release ships a GPG-signed SHA256SUMS + SBOM for the garnet CLI tarballs."
            ),
            (
                name: "Gatekeeper acceptance",
                probePath: nil,
                catalog: .deferred,
                detail:
                    "Unsigned + un-notarized → users must clear quarantine manually. Not distribution-ready."
            ),
        ]
}

public struct DistributionReport: Equatable, Sendable {
    public let repoRootProvided: Bool
    public let items: [DistributionItem]

    public var readyCount: Int { items.filter { $0.status == .ready }.count }
    public var deferredCount: Int { items.filter { $0.status == .deferred }.count }
    public var absentCount: Int { items.filter { $0.status == .absent }.count }

    /// The honest one-line posture. The macOS app is never described as
    /// distribution-ready while it is unsigned + un-notarized.
    public var headline: String {
        "macOS Studio .app is unsigned and un-notarized — research-grade, local run only. "
            + "\(readyCount) ready, \(deferredCount) deferred"
            + (absentCount > 0 ? ", \(absentCount) absent" : "") + "."
    }

    /// Build the report. `exists` probes the filesystem for an item's `probePath`
    /// (relative to the repo root); injected so the projection is unit-testable
    /// without disk. With no repo root, probe items read `unverified` — never a
    /// fabricated "ready".
    public static func build(repoRootProvided: Bool, exists: (String) -> Bool) -> DistributionReport {
        let items = DistributionCatalogEntry.entries.map { entry -> DistributionItem in
            guard let probePath = entry.probePath else {
                return DistributionItem(name: entry.name, status: entry.catalog, detail: entry.detail)
            }
            let status: DistributionStatus
            if !repoRootProvided {
                status = .unverified
            } else {
                status = exists(probePath) ? .ready : .absent
            }
            return DistributionItem(name: entry.name, status: status, detail: entry.detail)
        }
        return DistributionReport(repoRootProvided: repoRootProvided, items: items)
    }
}
