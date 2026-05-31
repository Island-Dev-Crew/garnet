#!/usr/bin/env python3
"""One-line install / readme consistency gate (S52).

The curl|sh ethos (Kelley): the one-line install must just work — and the docs
must stay accurate. The #1 adoption footgun is a README that documents an install
command the actual installer no longer matches. This gate ties the two together:

- the README's one-line `curl … install.sh | sh` command, and
- the bootstrap command `installer/sh.garnet-lang.org/install.sh` self-documents
  in its header,

must be byte-identical (modulo the comment prefix), and both must reference the
canonical install URL. `--gate` fails on any drift.

## Honest scope (do not soften)
This is a **doc-consistency** check, not a live end-to-end install test (the
installer pulls GitHub Releases over the network). `install.sh` is separately
shellcheck-gated (the `shellcheck-installer` CI job); this gate does not duplicate
that. It asserts the documented install command and URL are consistent, nothing
about a real network install succeeding.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
README = ROOT / "README.md"
INSTALLER = ROOT / "installer" / "sh.garnet-lang.org" / "install.sh"
INSTALL_URL = "https://garnet-lang.org/install.sh"


@dataclass
class InstallReadmeCheck:
    schema: str
    readme_command: str | None
    installer_command: str | None
    commands_match: bool
    url_in_readme: bool
    url_in_installer: bool
    consistent: bool


def _extract_curl(text: str) -> str | None:
    """The first `curl … install.sh | sh` line, normalized (leading comment
    marker + surrounding whitespace stripped, internal whitespace collapsed)."""
    for raw in text.splitlines():
        line = raw.strip()
        if line.startswith("#"):
            line = line.lstrip("#").strip()
        if "curl" in line and "install.sh" in line and "| sh" in line:
            return re.sub(r"\s+", " ", line).strip()
    return None


def read_check() -> InstallReadmeCheck:
    readme_text = README.read_text(encoding="utf-8") if README.is_file() else ""
    installer_text = INSTALLER.read_text(encoding="utf-8") if INSTALLER.is_file() else ""

    readme_cmd = _extract_curl(readme_text)
    installer_cmd = _extract_curl(installer_text)
    commands_match = readme_cmd is not None and readme_cmd == installer_cmd
    url_in_readme = INSTALL_URL in readme_text
    url_in_installer = INSTALL_URL in installer_text

    return InstallReadmeCheck(
        schema="garnet.install_readme_check/v1",
        readme_command=readme_cmd,
        installer_command=installer_cmd,
        commands_match=commands_match,
        url_in_readme=url_in_readme,
        url_in_installer=url_in_installer,
        consistent=commands_match and url_in_readme and url_in_installer,
    )


def render_markdown(c: InstallReadmeCheck) -> str:
    return "\n".join(
        [
            "# Garnet one-line install / readme check",
            "",
            f"_Schema {c.schema}._",
            "",
            f"- README command:    `{c.readme_command}`",
            f"- installer command: `{c.installer_command}`",
            f"- commands match: {'✅' if c.commands_match else '❌'}",
            f"- install URL in README: {'✅' if c.url_in_readme else '❌'}",
            f"- install URL in installer: {'✅' if c.url_in_installer else '❌'}",
            "",
            f"**Install docs consistent: {'yes' if c.consistent else 'NO'}.**",
            "",
            "Honest scope: a doc-consistency check, not a live network install "
            "test; install.sh is separately shellcheck-gated.",
            "",
        ]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if the README install command/URL drift from install.sh",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    check = read_check()
    if args.format == "md":
        print(render_markdown(check))
    else:
        print(json.dumps(asdict(check), indent=2))

    if args.gate and not check.consistent:
        print(
            "install-readme gate FAILED: the README one-line install command/URL "
            "drifted from installer/sh.garnet-lang.org/install.sh",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
