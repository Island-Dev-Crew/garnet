// M4 — Enforced / Declared Legend builder (nonisolated, swift-testable).
//
// Builds the legend payload = the static catalog + a live static-gate probe per
// enforced fence. Each probe reuses the M3 velocity check plumbing
// (`VelocityCheckCommand`, which runs `garnet check --format json` in a throwaway
// cwd, never executing the fixture), so the legend's "confirmed live" badges come
// from a real `garnet check` run — not hand-written markup.

import Foundation

public enum EnforcementLegendCommand {
    /// Build the legend. With no CLI the catalog still renders, but every probe is
    /// inconclusive (`ran == false`) — never a faked confirmation.
    public static func build(cli: String?, timeoutSeconds: Int) -> EnforcementLegend {
        let fences = EnforcementCatalog.fences()
        guard let cli else {
            // No CLI: still emit a probe per enforced fence, but inconclusive
            // (`ran == false`) so the row reads "not probed" — never a faked green.
            let probes = EnforcementCatalog.probeFixtures().map { fixture in
                EnforcementProbe(
                    fence: fixture.fence, expectedCode: fixture.expected, confirmed: false,
                    ran: false, exitCode: -1, observedCodes: [])
            }
            return EnforcementLegend(fences: fences, probes: probes, cliAvailable: false)
        }
        let probes = EnforcementCatalog.probeFixtures().map { fixture in
            let report = VelocityCheckCommand.run(
                cli: cli, source: fixture.source, timeoutSeconds: timeoutSeconds)
            return EnforcementProbe.from(
                fence: fixture.fence, expectedCode: fixture.expected, report: report)
        }
        return EnforcementLegend(fences: fences, probes: probes, cliAvailable: true)
    }
}
