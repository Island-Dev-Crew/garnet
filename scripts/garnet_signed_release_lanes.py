#!/usr/bin/env python3
"""Signed release lanes — inventory + active-lane gate (S51).

Garnet's "signed release" posture spans three distinct lanes. This reporter makes
each one's status explicit and falsifiable, and gates the one lane that is
actually ACTIVE so it cannot silently regress:

1. **Program-manifest signing** — `garnet build --sign <key>` produces an Ed25519
   signature over the deterministic build manifest, verified to "signature valid"
   in `linux-packages.yml`. This lane is **ACTIVE** and gated here.
2. **Release-artifact signing** — the tagged release job signs `SHA256SUMS` with
   `gpg --detach-sign` and uploads `SHA256SUMS.asc`; an unsigned tagged release
   fails closed unless it is a recorded, deliberate act. **ACTIVE** since v0.8.1
   and gated here. The key lives in CI; Garnet does not bundle GPG.
3. **Supply-chain attestation** — `garnet seal [--out]` emits an in-toto predicate
   over the build + capability manifests, meant for `cosign attest --predicate`.
   **PARTIAL**: Garnet produces (and now writes) the predicate; `cosign` is
   detected, never bundled — supply-chain *signing* is external.

## Scope (do not soften)
Garnet does **not** sign its own supply chain and does **not** bundle
cosign/GPG/minisign. Lane 3 is partial by design; its status is reported, not
faked. Lanes 1 and 2 are gated: lane 1 is in-language manifest signing, which
Garnet fully owns; lane 2 is the release pipeline's own GPG signature.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


@dataclass
class Lane:
    id: str
    name: str
    status: str  # active | broken | partial
    owned_by_garnet: bool
    evidence: str
    present: bool


@dataclass
class SignedReleaseLanes:
    schema: str
    lanes: list[Lane]
    active_lane_ok: bool


def _read(rel: str) -> str:
    p = ROOT / rel
    return p.read_text(encoding="utf-8") if p.is_file() else ""


# Every piece must be present in linux-packages.yml for release-artifact signing to
# count as wired: the detached signature, its upload, and the unsigned-release guard.
RELEASE_ARTIFACT_EVIDENCE = (
    "--detach-sign --armor SHA256SUMS",
    "files: release-dist/SHA256SUMS.asc",
    "name: Require signed SHA256SUMS (fail-closed)",
)


# The release job's signing steps. Checking their names alone let the refusal stop
# refusing (`exit 1` -> `exit 0`) while the lane still read "active", so the sign
# and refusal steps are pinned as exact text, and changing either is a reviewed
# change to these constants. The attach step is pinned by meaning, because its
# action pin is bumped routinely. Order matters too: sign, then refuse, then
# publish, then attach the signature.
SIGN_STEP_NAME = "Sign SHA256SUMS (activates when GPG_SIGNING_KEY secret is set)"
SIGN_STEP = (
    '      - name: Sign SHA256SUMS (activates when GPG_SIGNING_KEY secret is set)\n'
    "        if: env.HAS_GPG == 'true'\n"
    '        env:\n'
    '          GPG_SIGNING_KEY: ${{ secrets.GPG_SIGNING_KEY }}\n'
    '          GPG_PASSPHRASE: ${{ secrets.GPG_PASSPHRASE }}\n'
    '        run: |\n'
    '          cd release-dist\n'
    '          echo "$GPG_SIGNING_KEY" | gpg --batch --import\n'
    '          if [ -n "$GPG_PASSPHRASE" ]; then\n'
    '            gpg --batch --yes --pinentry-mode loopback --passphrase "$GPG_PASSPHRASE" \\\n'
    '              --detach-sign --armor SHA256SUMS\n'
    '          else\n'
    '            gpg --batch --yes --detach-sign --armor SHA256SUMS\n'
    '          fi\n'
    '          echo "signed → SHA256SUMS.asc"\n'
)
REFUSAL_STEP_NAME = "Require signed SHA256SUMS (fail-closed)"
REFUSAL_STEP = (
    '      - name: Require signed SHA256SUMS (fail-closed)\n'
    "        if: env.HAS_GPG != 'true'\n"
    '        run: |\n'
    '          if [ "${{ vars.ALLOW_UNSIGNED_RELEASE }}" = "true" ]; then\n'
    '            echo "::warning::GPG_SIGNING_KEY absent and ALLOW_UNSIGNED_RELEASE=true — publishing a DELIBERATELY UNSIGNED research-grade release."\n'
    '          else\n'
    '            echo "::error::Tagged release requires a signed SHA256SUMS, but GPG_SIGNING_KEY is not set. Provide GPG_SIGNING_KEY (+ optional GPG_PASSPHRASE), or set repository variable ALLOW_UNSIGNED_RELEASE=true to deliberately ship unsigned."\n'
    '            exit 1\n'
    '          fi\n'
)
ATTACH_STEP_NAME = "Attach SHA256SUMS signature (only when signed)"
ATTACH_STEP_LINES = (
    "        if: env.HAS_GPG == 'true'",
    "        with:",
    "          files: release-dist/SHA256SUMS.asc",
    "          fail_on_unmatched_files: true",
)
ATTACH_ACTION_PREFIX = "        uses: softprops/action-gh-release@"
PUBLISH_STEP_HEAD = "      - name: Publish release\n"
PUBLISH_STEP_NAME = "Publish release"
PUBLISH_STEP_LINES = (
    '        with:',
    '          files: |',
    '            release-dist/*.deb',
    '            release-dist/*.rpm',
    '            release-dist/*.tar.gz',
    '            release-dist/*.zip',
    '            release-dist/garnet-sbom-cyclonedx.tgz',
    '            release-dist/SHA256SUMS',
    '          fail_on_unmatched_files: true',
    '          body_path: release-body.md',
)
RELEASE_JOB_HEAD = "\n  release:\n"
HAS_GPG_LINE = "      HAS_GPG: ${{ secrets.GPG_SIGNING_KEY != '' }}\n"

# SHA-256 of the reviewed `release` job's text, as `_release_job` returns it. Any
# edit to the job (a step added, named or not, a command changed, a condition
# moved) breaks the lane until a reviewed change updates this pin with the job.
# A list of publishing commands is never complete; the reviewed job is.
RELEASE_JOB_SHA256 = "c8f598f0a4008d6072581c31e6033984a5484ad5531bf7e4f09a663bcc03ef26"

# Outside the release job, nothing in the workflow may hold a token that can
# write a release. The repository's default workflow token is read-only (the
# governance gate records that setting), so a writer needs a `permissions` key
# or a secret. The workflow is read as decoded YAML, so a quoted key, an escape
# in a double-quoted value or another casing of `secrets` counts the same.
SECRETS_RE = re.compile(r"\bsecrets\b", re.IGNORECASE)


def _step_block(text: str, name: str) -> tuple[int, str] | None:
    """The step named `name` (at the release job's step indent) and where it starts."""
    head = f"      - name: {name}\n"
    if text.count(head) != 1:
        return None
    start = text.index(head)
    lines = text[start:].split("\n")
    block = [lines[0]]
    for line in lines[1:]:
        if line.strip() and not line.startswith("        "):
            break
        block.append(line)
    while block and not block[-1].strip():
        block.pop()
    return start, "\n".join(block) + "\n"


def release_signing_steps_pinned(pkg: str) -> bool:
    sign = _step_block(pkg, SIGN_STEP_NAME)
    refusal = _step_block(pkg, REFUSAL_STEP_NAME)
    attach = _step_block(pkg, ATTACH_STEP_NAME)
    if sign is None or refusal is None or attach is None:
        return False
    if pkg.count(PUBLISH_STEP_HEAD) != 1 or pkg.count(HAS_GPG_LINE) != 1:
        return False
    if sign[1] != SIGN_STEP or refusal[1] != REFUSAL_STEP:
        return False
    attach_lines = attach[1].splitlines()[1:]
    uses = [line for line in attach_lines if line.startswith("        uses:")]
    rest = tuple(line for line in attach_lines if not line.startswith("        uses:"))
    if len(uses) != 1 or not uses[0].startswith(ATTACH_ACTION_PREFIX) or rest != ATTACH_STEP_LINES:
        return False
    if not sign[0] < refusal[0] < pkg.index(PUBLISH_STEP_HEAD) < attach[0]:
        return False
    return release_job_cannot_publish_past_the_refusal(pkg)


def _split_release_job(pkg: str) -> tuple[str, str] | None:
    """The text of the `release` job, up to the next top-level job, and the rest
    of the workflow without it."""
    if pkg.count(RELEASE_JOB_HEAD) != 1:
        return None
    start = pkg.index(RELEASE_JOB_HEAD) + 1
    lines = pkg[start:].split("\n")
    n = 1
    for line in lines[1:]:
        if line.startswith("  ") and not line.startswith("   ") and line.strip() and not line.lstrip().startswith("#"):
            break
        n += 1
    return "\n".join(lines[:n]) + "\n", pkg[:start] + "\n".join(lines[n:])


def _release_job(pkg: str) -> str | None:
    """The text of the `release` job, up to the next top-level job."""
    split = _split_release_job(pkg)
    return None if split is None else split[0]


def release_job_is_the_reviewed_one(pkg: str) -> bool:
    """The release job is byte for byte the reviewed job, and no other job in the
    workflow declares a write permission or reads a secret."""
    split = _split_release_job(pkg)
    if split is None:
        return False
    job, _rest = split
    if hashlib.sha256(job.encode("utf-8")).hexdigest() != RELEASE_JOB_SHA256:
        return False
    return no_other_writer(pkg)


def no_other_writer(pkg: str) -> bool:
    """No job but `release`, and not the workflow's own top level, has a
    `permissions` key or mentions `secrets`, in any key or value of the decoded
    YAML. Without PyYAML, or for YAML that does not compose to one document with
    exactly one `release` job, the lane cannot be read and is not active."""
    try:
        import yaml
    except ImportError:
        return False
    try:
        root = yaml.compose(pkg, Loader=yaml.BaseLoader)
    except yaml.YAMLError:
        return False
    if not isinstance(root, yaml.MappingNode):
        return False
    jobs = [value for key, value in root.value if getattr(key, "value", None) == "jobs"]
    if len(jobs) != 1 or not isinstance(jobs[0], yaml.MappingNode):
        return False
    job_ids = [getattr(key, "value", None) for key, _ in jobs[0].value]
    if job_ids.count("release") != 1:
        return False
    scanned = [pair for pair in root.value if getattr(pair[0], "value", None) != "jobs"]
    scanned += [pair for pair in jobs[0].value if getattr(pair[0], "value", None) != "release"]
    seen: set[int] = set()

    def writer(node: object) -> bool:
        if id(node) in seen:
            return False
        seen.add(id(node))
        if isinstance(node, yaml.ScalarNode):
            return bool(SECRETS_RE.search(node.value))
        if isinstance(node, yaml.SequenceNode):
            return any(writer(item) for item in node.value)
        if isinstance(node, yaml.MappingNode):
            return any(
                getattr(key, "value", None) == "permissions" or writer(key) or writer(value)
                for key, value in node.value
            )
        return True

    return not any(
        getattr(key, "value", None) == "permissions" or writer(key) or writer(value)
        for key, value in scanned
    )


def release_job_cannot_publish_past_the_refusal(pkg: str) -> bool:
    """Once the refusal fails, nothing in the release job may still publish.

    The job never continues on error; after the refusal come exactly Publish
    release (unconditional, so it is skipped when the refusal fails) and the
    attach step; and no other step publishes (`action-gh-release` or
    `gh release`).
    """
    job = _release_job(pkg)
    if job is None or "continue-on-error" in job:
        return False
    names = [line[len("      - name: "):] for line in job.split("\n") if line.startswith("      - name: ")]
    if REFUSAL_STEP_NAME not in names:
        return False
    if names[names.index(REFUSAL_STEP_NAME) + 1:] != [PUBLISH_STEP_NAME, ATTACH_STEP_NAME]:
        return False
    publish = _step_block(job, PUBLISH_STEP_NAME)
    if publish is None:
        return False
    publish_lines = publish[1].splitlines()[1:]
    uses = [line for line in publish_lines if line.startswith("        uses:")]
    rest = tuple(line for line in publish_lines if not line.startswith("        uses:"))
    if len(uses) != 1 or not uses[0].startswith(ATTACH_ACTION_PREFIX) or rest != PUBLISH_STEP_LINES:
        return False
    for name in names:
        if name in (PUBLISH_STEP_NAME, ATTACH_STEP_NAME):
            continue
        block = _step_block(job, name)
        if block is None or "action-gh-release" in block[1] or "gh release" in block[1]:
            return False
    return True


def release_artifact_lane(pkg: str) -> Lane:
    # The job pin is what binds; the step checks name what the reviewed job does.
    present = (
        all(piece in pkg for piece in RELEASE_ARTIFACT_EVIDENCE)
        and release_signing_steps_pinned(pkg)
        and release_job_is_the_reviewed_one(pkg)
    )
    return Lane(
        id="release-artifact",
        name="Release-artifact signing (SHA256SUMS detached signature)",
        status="active" if present else "broken",
        owned_by_garnet=False,
        evidence="linux-packages.yml: gpg --detach-sign SHA256SUMS → SHA256SUMS.asc uploaded; "
        "unsigned tagged release fails closed (key held in the release environment, GPG not bundled)",
        present=present,
    )


def read_lanes() -> SignedReleaseLanes:
    pkg = _read(".github/workflows/linux-packages.yml")
    seal = _read("garnet-cli/src/cmd/seal.rs")

    lane1_present = "--sign" in pkg and "signature valid" in pkg
    lane3_present = "garnet seal" in seal and "--out" in seal and "cosign" in seal

    lanes = [
        Lane(
            id="program-manifest",
            name="Program-manifest signing (garnet build --sign)",
            status="active" if lane1_present else "broken",
            owned_by_garnet=True,
            evidence="linux-packages.yml: --sign round-trip → 'signature valid'",
            present=lane1_present,
        ),
        release_artifact_lane(pkg),
        Lane(
            id="supply-chain-attestation",
            name="Supply-chain attestation (garnet seal → cosign attest)",
            status="partial",
            owned_by_garnet=False,
            evidence="cmd/seal.rs: in-toto predicate emitted (now --out writable); cosign detected, not bundled",
            present=lane3_present,
        ),
    ]
    active_lane_ok = all(l.present for l in lanes if l.id in ("program-manifest", "release-artifact"))
    return SignedReleaseLanes(
        schema="garnet.signed_release_lanes/v1",
        lanes=lanes,
        active_lane_ok=active_lane_ok,
    )


def render_markdown(s: SignedReleaseLanes) -> str:
    lines = [
        "# Garnet signed release lanes",
        "",
        f"_Schema {s.schema}._",
        "",
        "| lane | status | Garnet-owned | evidence |",
        "|---|---|---|---|",
    ]
    for l in s.lanes:
        mark = {"active": "✅ active", "broken": "✗ broken", "partial": "◐ partial"}.get(
            l.status, l.status
        )
        owned = "yes" if l.owned_by_garnet else "no (external tool)"
        lines.append(f"| {l.name} | {mark} | {owned} | {l.evidence} |")
    lines += [
        "",
        f"**Active lanes (program-manifest and release-artifact signing) wired: "
        f"{'yes' if s.active_lane_ok else 'NO'}.**",
        "",
        "Scope: Garnet does not sign its own supply chain or bundle "
        "cosign/GPG/minisign. Lane 3 (supply-chain) is partial by design — an "
        "external signing tool. Lanes 1 (program-manifest) and 2 (release-artifact, "
        "the release job's GPG signature) are gated.",
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=["json", "md"], default="json")
    parser.add_argument(
        "--gate",
        action="store_true",
        help="exit non-zero if an ACTIVE lane (program-manifest or release-artifact signing) is not wired",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    status = read_lanes()
    if args.format == "md":
        print(render_markdown(status))
    else:
        print(json.dumps(asdict(status), indent=2))

    if args.gate and not status.active_lane_ok:
        print(
            "signed-release-lanes gate FAILED: program-manifest signing "
            "(`garnet build --sign` → 'signature valid') or release-artifact signing "
            "(gpg --detach-sign SHA256SUMS → SHA256SUMS.asc, fail-closed) is no longer wired in CI",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
