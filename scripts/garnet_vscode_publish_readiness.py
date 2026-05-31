#!/usr/bin/env python3
"""VS Code / OpenVSX / Marketplace publish-readiness gate (S54).

Makes the Garnet VS Code extension marketplace-READY (every field a publish
requires is present and cannot silently regress) and documents the publish path —
while honestly deferring the credentialed publish itself.

The publish path has three steps:
1. **Build the VSIX** — `vscode-extension.yml` builds it on every push.
2. **GitHub release asset** — published on tag (`Publish VSIX release assets`).
3. **OpenVSX + VS Code Marketplace** — `ovsx publish` / `vsce publish`. This needs
   `OVSX_TOKEN` / `VSCE_PAT` secrets and a publisher account → **DEFERRED**
   (credential/account territory; a release-truth decision for Jon, not made
   here). This gate ensures step 3 would not fail on missing manifest metadata.

## Honest scope (do not soften)
This does not publish anything and does not bundle marketplace credentials. It
checks the extension manifest + assets are publish-ready and reports the path;
the actual OpenVSX/Marketplace publish is deferred to a human with the tokens.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXT = ROOT / "editors" / "vscode"
MANIFEST = EXT / "package.json"

# Fields a vsce/ovsx publish requires or strongly recommends.
REQUIRED_FIELDS = ["name", "version", "publisher", "engines", "repository", "license"]
RECOMMENDED_FIELDS = ["displayName", "description", "categories", "keywords"]
REQUIRED_FILES = ["README.md", "LICENSE.md"]


@dataclass
class PublishReadiness:
    schema: str
    manifest_present: bool
    missing_required: list[str] = field(default_factory=list)
    missing_recommended: list[str] = field(default_factory=list)
    missing_files: list[str] = field(default_factory=list)
    publish_path: list[str] = field(default_factory=list)
    marketplace_ready: bool = False


def read_readiness() -> PublishReadiness:
    if not MANIFEST.is_file():
        return PublishReadiness(
            schema="garnet.vscode_publish_readiness/v1",
            manifest_present=False,
            missing_required=list(REQUIRED_FIELDS),
            marketplace_ready=False,
        )
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    missing_required = [f for f in REQUIRED_FIELDS if f not in manifest]
    # engines must specifically name vscode.
    if "engines" in manifest and "vscode" not in manifest.get("engines", {}):
        missing_required.append("engines.vscode")
    missing_recommended = [f for f in RECOMMENDED_FIELDS if f not in manifest]
    missing_files = [f for f in REQUIRED_FILES if not (EXT / f).is_file()]

    publish_path = [
        "build VSIX (vscode-extension.yml: build-vsix) — wired",
        "GitHub release asset on tag (release-vsix) — wired",
        "OpenVSX publish (`ovsx publish`) — DEFERRED (needs OVSX_TOKEN)",
        "VS Code Marketplace publish (`vsce publish`) — DEFERRED (needs VSCE_PAT)",
    ]
    return PublishReadiness(
        schema="garnet.vscode_publish_readiness/v1",
        manifest_present=True,
        missing_required=missing_required,
        missing_recommended=missing_recommended,
        missing_files=missing_files,
        publish_path=publish_path,
        marketplace_ready=not missing_required and not missing_files,
    )


def render_markdown(r: PublishReadiness) -> str:
    lines = [
        "# Garnet VS Code extension — publish readiness",
        "",
        f"_Schema {r.schema}._",
        "",
        f"- manifest present: {r.manifest_present}",
        f"- missing required: {r.missing_required or 'none'}",
        f"- missing recommended: {r.missing_recommended or 'none'}",
        f"- missing files: {r.missing_files or 'none'}",
        "",
        f"**Marketplace-ready (publish would not fail on metadata): "
        f"{'yes' if r.marketplace_ready else 'NO'}.**",
        "",
        "## Publish path",
    ]
    for step in r.publish_path:
        lines.append(f"- {step}")
    lines += [
        "",
        "Honest scope: this gate makes the extension marketplace-READY and "
        "documents the path; it does not publish and does not bundle credentials. "
        "The OpenVSX/Marketplace publish needs OVSX_TOKEN/VSCE_PAT — deferred to a "
        "human with the tokens.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if a marketplace-required field or file is missing",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    r = read_readiness()
    if args.format == "md":
        print(render_markdown(r))
    else:
        print(json.dumps(asdict(r), indent=2))

    if args.gate and not r.marketplace_ready:
        print(
            "vscode-publish-readiness gate FAILED: missing "
            f"required={r.missing_required}, files={r.missing_files}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
