// M5 — Agent-Loop Console reader (nonisolated; only disk I/O lives here).
//
// Reads the six artifacts of an existing `garnet agent-loop --record-dir` dossier
// and hands their raw text to the pure parser (`AgentLoopDossier.parse`). It never
// runs the agent loop and never writes — read-only over a directory the user picks.

import Foundation

public enum AgentLoopCommand {
    /// Read the record-dir artifacts and parse them. A missing directory / missing
    /// `decision.md` yields a non-`ran` dossier with an honest error, never a
    /// fabricated verdict.
    public static func load(recordDir: String) -> AgentLoopDossier {
        let dir = URL(fileURLWithPath: recordDir, isDirectory: true)
        var isDir: ObjCBool = false
        let exists = FileManager.default.fileExists(atPath: dir.path, isDirectory: &isDir)
        guard exists, isDir.boolValue else {
            return AgentLoopDossier.error(
                recordDir, "record directory not found, or it is not a directory.")
        }
        func read(_ name: String) -> String? {
            try? String(contentsOf: dir.appendingPathComponent(name), encoding: .utf8)
        }
        let files = RecordDirFiles(
            decisionMd: read("decision.md"),
            diffCapsTxt: read("diff_caps.txt"),
            capabilityManifestJson: read("capability_manifest.json"),
            sealJson: read("seal.json"),
            transparencyLogJsonl: read("transparency_log.jsonl"),
            runTrapTxt: read("run_trap.txt"))
        return AgentLoopDossier.parse(recordDir: recordDir, files: files)
    }
}
