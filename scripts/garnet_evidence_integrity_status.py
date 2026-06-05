#!/usr/bin/env python3
"""Cross-OS evidence-integrity gate (S112 consolidation + S113 integrity gate).

The surge lanes (Stage X + R) recorded cross-OS proof bundles under `proofs/`, each
sealed with a `MANIFEST.sha256`. This gate verifies **every** bundle's manifest
against the committed bytes and reports the honest pass/fail — it is the integrity
floor Stage P (positioning, readiness) rests on, so no cross-OS claim is presented
as verified unless its bundle's hashes actually check out.

Background (the defect this gate caught + closed): five Windows/WSL bundles failed
their own manifests because git **EOL-normalized** the CRLF-recorded text proof
files to LF *after* sealing (the CRLF-restored hash matched the seal — the content
was byte-identical modulo line endings, not tampered). Fixed by re-sealing those
manifests against the committed bytes and adding `proofs/** -text` to
`.gitattributes` so proof bundles are never normalized again.

## Honest scope (do not soften)
This verifies **hash integrity** (the manifest matches the committed bytes) for
every `proofs/**/MANIFEST.sha256`. It does not, by itself, attest that a bundle is
*complete* or that its claims are true — that is the per-slice dogfood evidence's
job. A green gate means the recorded cross-OS evidence is tamper-evident and
reproducible bit-for-bit from the repo.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROOFS = ROOT / "proofs"


def _sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def _verify_manifest(manifest: Path) -> tuple[bool, int, list[str]]:
    """Return (ok, entry_count, problems) for one MANIFEST.sha256."""
    base = manifest.parent
    problems: list[str] = []
    entries = 0
    for line in manifest.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        # shasum -a 256 format: "<64-hex>  <relative/path>"
        parts = line.split(None, 1)
        if len(parts) != 2:
            problems.append(f"malformed line: {line[:60]}")
            continue
        want, rel = parts[0], parts[1].lstrip("*").strip()
        entries += 1
        target = base / rel
        if not target.is_file():
            problems.append(f"missing: {rel}")
            continue
        if _sha256(target) != want:
            problems.append(f"hash mismatch: {rel}")
    return (not problems, entries, problems)


@dataclass
class EvidenceIntegrityStatus:
    schema: str
    bundles_total: int
    bundles_ok: int
    bundles_failed: int
    failed: list[str] = field(default_factory=list)
    ok: bool = False


def read_status() -> EvidenceIntegrityStatus:
    manifests = sorted(PROOFS.rglob("MANIFEST.sha256")) if PROOFS.is_dir() else []
    ok_n = 0
    failed: list[str] = []
    for m in manifests:
        good, _entries, problems = _verify_manifest(m)
        if good:
            ok_n += 1
        else:
            rel = m.parent.relative_to(ROOT).as_posix()
            failed.append(f"{rel} ({len(problems)} problem(s); e.g. {problems[0]})")
    total = len(manifests)
    # Require a non-trivial corpus AND every bundle verifying.
    ok = total >= 20 and not failed
    return EvidenceIntegrityStatus(
        schema="garnet.evidence_integrity/v1",
        bundles_total=total,
        bundles_ok=ok_n,
        bundles_failed=len(failed),
        failed=failed,
        ok=ok,
    )


def render_markdown(r: EvidenceIntegrityStatus) -> str:
    lines = [
        "# Garnet cross-OS evidence-integrity status (S112/S113)",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- proof bundles verified: **{r.bundles_ok}/{r.bundles_total}** "
        f"(`proofs/**/MANIFEST.sha256`)",
        f"- bundles failing integrity: {r.bundles_failed}",
    ]
    for f in r.failed:
        lines.append(f"  - FAIL: {f}")
    lines += [
        "",
        "Every cross-OS proof bundle hash-verifies against the committed bytes "
        "(tamper-evident, bit-for-bit reproducible). `proofs/** -text` keeps them "
        "free of EOL normalization. This is hash integrity only — completeness + "
        "truth of each claim is the per-slice dogfood evidence's job.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero unless every proofs/**/MANIFEST.sha256 verifies against "
        "the committed bytes (>= 20 bundles present).",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_status()
    print(render_markdown(r) if args.format == "md" else json.dumps(asdict(r), indent=2))

    if args.gate and not r.ok:
        print(f"evidence-integrity gate FAILED: {r.bundles_failed} bundle(s)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
