#!/usr/bin/env python3
"""Render a bounded browser-smokeable MIT deck preview from Garnet evidence."""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import asdict, dataclass
from html import escape
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import garnet_mit_deck_outline  # noqa: E402


@dataclass(frozen=True)
class DeckPreviewStatus:
    source: str
    overall_status: str
    objective_completion_percent: float
    tracked_slices: str
    target_slide_count: int
    current_truth: list[str]
    slides: list[garnet_mit_deck_outline.DeckSlide]
    blocked_gates: list[garnet_mit_deck_outline.garnet_mit_demo_route.BlockedGate]
    forbidden_claims: list[str]
    output_formats: list[str]
    claims_final_acceptance: bool


def read_preview() -> DeckPreviewStatus:
    outline = garnet_mit_deck_outline.read_outline()
    return DeckPreviewStatus(
        source=outline.source,
        overall_status=outline.overall_status,
        objective_completion_percent=outline.objective_completion_percent,
        tracked_slices=outline.tracked_slices,
        target_slide_count=outline.target_slide_count,
        current_truth=[
            "browser-smokeable HTML preview",
            "generated from the current MIT deck outline",
            "not full MIT/productization completion",
            "final acceptance and rendered presentation approval remain separate gates",
        ],
        slides=outline.slides,
        blocked_gates=outline.blocked_gates,
        forbidden_claims=outline.forbidden_claims,
        output_formats=[
            "garnet-mit-deck-preview.html",
            "garnet-mit-deck-preview.json",
            "garnet-mit-deck-outline.md",
            "MANIFEST.sha256",
        ],
        claims_final_acceptance=False,
    )


def _items(items: list[str], *, code: bool = False) -> str:
    lines = []
    for item in items:
        body = f"<code>{escape(item)}</code>" if code else escape(item)
        lines.append(f"<li>{body}</li>")
    return "\n".join(lines)


def render_html(preview: DeckPreviewStatus) -> str:
    slides = []
    for index, slide in enumerate(preview.slides, start=1):
        slides.append(
            f"""
      <section class="slide" data-slide-id="{escape(slide.id)}">
        <div class="slide-number">Slide {index:02d}</div>
        <p class="kicker">{escape(slide.title)}</p>
        <h2>{escape(slide.headline)}</h2>
        <div class="columns">
          <div>
            <h3>Story</h3>
            <ul>{_items(slide.body)}</ul>
          </div>
          <div>
            <h3>Evidence</h3>
            <ul>{_items(slide.evidence, code=True)}</ul>
          </div>
        </div>
        <p class="speaker-note"><strong>Speaker note:</strong> {escape(slide.speaker_note)}</p>
      </section>"""
        )

    blocked = "\n".join(
        f"""
          <tr>
            <td>{escape(gate.label)}</td>
            <td>{escape(gate.reason)}</td>
            <td>{escape(gate.next_unlock)}</td>
          </tr>"""
        for gate in preview.blocked_gates
    )
    return (
        """<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Garnet MIT Deck Preview</title>
  <style>
    :root {
      color-scheme: dark;
      --bg: #0a0a0f;
      --panel: #15151d;
      --ink: #f7f2ea;
      --muted: #c7bdb2;
      --garnet: #b44c43;
      --gold: #e5c07b;
      --line: rgba(247, 242, 234, 0.16);
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      background: var(--bg);
      color: var(--ink);
      font: 16px/1.55 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    header, main, footer { width: min(1180px, calc(100% - 40px)); margin: 0 auto; }
    header { padding: 48px 0 28px; }
    h1, h2, h3, p { margin-top: 0; }
    h1 { font-size: 44px; line-height: 1; margin-bottom: 16px; }
    h2 { font-size: 34px; line-height: 1.08; max-width: 920px; }
    h3 { color: var(--gold); font-size: 15px; text-transform: uppercase; }
    .summary {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: 12px;
      margin-top: 28px;
    }
    .metric {
      border: 1px solid var(--line);
      background: var(--panel);
      padding: 16px;
      min-height: 104px;
    }
    .metric span { display: block; color: var(--muted); font-size: 13px; }
    .metric strong { display: block; font-size: 26px; margin-top: 8px; }
    .slide {
      min-height: 620px;
      border-top: 1px solid var(--line);
      padding: 42px 0;
    }
    .slide-number, .kicker { color: var(--gold); letter-spacing: 0; }
    .kicker { text-transform: uppercase; font-weight: 700; }
    .columns {
      display: grid;
      grid-template-columns: minmax(0, 1.1fr) minmax(280px, 0.9fr);
      gap: 28px;
      margin-top: 28px;
    }
    ul { margin: 0; padding-left: 20px; }
    li { margin: 10px 0; }
    code {
      color: #ffd7c2;
      background: rgba(180, 76, 67, 0.18);
      padding: 2px 5px;
      border-radius: 4px;
    }
    .speaker-note {
      border-left: 4px solid var(--garnet);
      color: var(--muted);
      margin-top: 28px;
      padding-left: 16px;
    }
    table { width: 100%; border-collapse: collapse; margin-top: 18px; }
    th, td { border: 1px solid var(--line); padding: 12px; text-align: left; vertical-align: top; }
    th { color: var(--gold); }
    footer { color: var(--muted); padding: 32px 0 56px; }
    @media (max-width: 800px) {
      header, main, footer { width: min(100% - 24px, 1180px); }
      h1 { font-size: 34px; }
      h2 { font-size: 27px; }
      .summary, .columns { grid-template-columns: 1fr; }
      .slide { min-height: auto; }
    }
  </style>
</head>
<body>
  <header>
    <p class="kicker">Garnet MIT Deck Preview</p>
    <h1>Reviewer-safe deck preview generated from current evidence.</h1>
    <p>This browser-smokeable HTML preview packages the outline into a readable artifact while preserving the boundary that it is not full MIT/productization completion or final MIT/productization acceptance.</p>
    <div class="summary">
      <div class="metric"><span>Status</span><strong>"""
        + escape(preview.overall_status)
        + """</strong></div>
      <div class="metric"><span>Objective</span><strong>"""
        + f"{preview.objective_completion_percent:.1f}%"
        + """</strong></div>
      <div class="metric"><span>Tracked Slices</span><strong>"""
        + escape(preview.tracked_slices)
        + """</strong></div>
      <div class="metric"><span>Slides</span><strong>"""
        + str(preview.target_slide_count)
        + """</strong></div>
    </div>
  </header>
  <main>
    <section class="slide" data-slide-id="current-truth">
      <div class="slide-number">Truth Gate</div>
      <h2>Current truth before story.</h2>
      <ul>"""
        + _items(preview.current_truth)
        + """</ul>
    </section>
"""
        + "\n".join(slides)
        + """
    <section class="slide" data-slide-id="blocked-gates">
      <div class="slide-number">Boundary</div>
      <h2>Blocked and deferred gates remain separate.</h2>
      <table>
        <thead><tr><th>Gate</th><th>Reason</th><th>Next unlock</th></tr></thead>
        <tbody>"""
        + blocked
        + """</tbody>
      </table>
      <h3>Forbidden claims</h3>
      <ul>"""
        + _items(preview.forbidden_claims)
        + """</ul>
    </section>
  </main>
  <footer>
    Source: <code>"""
        + escape(preview.source)
        + """</code>. Generated from <code>scripts/garnet_mit_deck_outline.py</code>.
  </footer>
</body>
</html>
"""
    )


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_bundle(preview: DeckPreviewStatus, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    html_path = output_dir / "garnet-mit-deck-preview.html"
    data_path = output_dir / "garnet-mit-deck-preview.json"
    outline_path = output_dir / "garnet-mit-deck-outline.md"
    manifest_path = output_dir / "MANIFEST.sha256"

    html_path.write_text(render_html(preview), encoding="utf-8")
    data_path.write_text(json.dumps(asdict(preview), indent=2) + "\n", encoding="utf-8")
    outline = garnet_mit_deck_outline.read_outline()
    outline_path.write_text(garnet_mit_deck_outline.render_markdown(outline), encoding="utf-8")
    files = [html_path, data_path, outline_path]
    manifest_path.write_text(
        "".join(f"{_sha256(path)}  ./{path.name}\n" for path in files),
        encoding="utf-8",
    )


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("html", "json"), default="html")
    parser.add_argument("--output-dir", type=Path, help="write HTML, JSON, outline Markdown, and manifest evidence")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    preview = read_preview()
    if args.output_dir:
        write_bundle(preview, args.output_dir)
    if args.format == "json":
        print(json.dumps(asdict(preview), indent=2))
    else:
        print(render_html(preview), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
