#!/usr/bin/env node
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";

const DEFAULT_INPUT_DIR = "/Users/idc2.0/Desktop/dogfood/garnet-promo-video";
const DEFAULT_QA_DIR = "/Users/idc2.0/Desktop/dogfood/garnet-promo-video-visual-qa";
const DEFAULT_OUTPUT_DIR = "/Users/idc2.0/Desktop/dogfood/garnet-promo-video-website-export";

function usage() {
  return `Usage: export_garnet_promo_video_site.mjs [options]

Create a website export package for the local Garnet promo render.

Options:
  --input-dir <dir>    Render artifact directory (default: Desktop promo render bundle)
  --qa-dir <dir>       Visual QA evidence directory (default: Desktop visual QA bundle)
  --output-dir <dir>   Website export evidence directory (default: Desktop website export bundle)
  -h, --help           Show this help

Outputs:
  garnet-promo.mp4
  garnet-promo.webm
  garnet-promo-poster.png
  embed-snippet.html
  promo-website-export-data.json
  promo-website-export-report.md
  MANIFEST.sha256`;
}

function parseArgs(argv) {
  const args = {
    inputDir: process.env.GARNET_PROMO_VIDEO_INPUT_DIR || DEFAULT_INPUT_DIR,
    qaDir: process.env.GARNET_PROMO_VIDEO_QA_DIR || DEFAULT_QA_DIR,
    outputDir: process.env.GARNET_PROMO_VIDEO_EXPORT_DIR || DEFAULT_OUTPUT_DIR,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--input-dir") args.inputDir = argv[++index];
    else if (arg === "--qa-dir") args.qaDir = argv[++index];
    else if (arg === "--output-dir") args.outputDir = argv[++index];
    else if (arg === "-h" || arg === "--help") {
      console.log(usage());
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  args.inputDir = resolve(args.inputDir);
  args.qaDir = resolve(args.qaDir);
  args.outputDir = resolve(args.outputDir);
  return args;
}

function sha256(pathname) {
  return createHash("sha256").update(readFileSync(pathname)).digest("hex");
}

function readJson(pathname) {
  return JSON.parse(readFileSync(pathname, "utf-8"));
}

function assertVisualQaPassed(qaPath) {
  if (!existsSync(qaPath)) throw new Error(`missing visual QA data: ${qaPath}`);
  const data = readJson(qaPath);
  const checks = Array.isArray(data.checks) ? data.checks : [];
  const passed =
    data.status === "visual-qa-ready" &&
    data.verdict === "pass" &&
    checks.length > 0 &&
    checks.every((check) => check && check.passed === true);
  if (!passed) throw new Error(`visual QA evidence did not pass: ${qaPath}`);
  return data;
}

function copyRequired(inputDir, outputDir, filename) {
  const source = join(inputDir, filename);
  const target = join(outputDir, filename);
  if (!existsSync(source)) throw new Error(`missing render artifact: ${source}`);
  copyFileSync(source, target);
  return { file: filename, sha256: sha256(target), size_bytes: readFileSync(target).length };
}

function writeManifest(outputDir, files) {
  const rows = files.map((file) => `${sha256(join(outputDir, file))}  ${file}`);
  writeFileSync(join(outputDir, "MANIFEST.sha256"), `${rows.join("\n")}\n`, "utf-8");
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const qaDataPath = join(args.qaDir, "promo-visual-qa-data.json");
  const qaData = assertVisualQaPassed(qaDataPath);
  mkdirSync(args.outputDir, { recursive: true });
  for (const file of [
    "garnet-promo.mp4",
    "garnet-promo.webm",
    "garnet-promo-poster.png",
    "embed-snippet.html",
    "promo-website-export-data.json",
    "promo-website-export-report.md",
    "MANIFEST.sha256",
  ]) {
    rmSync(join(args.outputDir, file), { force: true });
  }

  const assets = [
    copyRequired(args.inputDir, args.outputDir, "garnet-promo.mp4"),
    copyRequired(args.inputDir, args.outputDir, "garnet-promo.webm"),
    copyRequired(args.inputDir, args.outputDir, "garnet-promo-poster.png"),
  ];
  const snippet = `<video class="garnet-promo-video" controls preload="metadata" poster="/assets/garnet-promo-poster.png" width="1920" height="1080">
  <source src="/assets/garnet-promo.webm" type="video/webm">
  <source src="/assets/garnet-promo.mp4" type="video/mp4">
  <p>Download the Garnet promo video: <a href="/assets/garnet-promo.mp4">MP4</a>.</p>
</video>
`;
  writeFileSync(join(args.outputDir, "embed-snippet.html"), snippet, "utf-8");

  const checks = [
    { id: "visual-qa-evidence", passed: true, source: qaDataPath },
    ...assets.map((asset) => ({ id: `${asset.file}-copied`, passed: asset.size_bytes > 0, ...asset })),
    { id: "embed-snippet", passed: snippet.includes("garnet-promo.webm") && snippet.includes("garnet-promo.mp4") },
  ];
  const verdict = checks.every((check) => check.passed) ? "pass" : "fail";
  const data = {
    status: verdict === "pass" ? "website-export-ready" : "website-export-failed",
    verdict,
    source_render_dir: args.inputDir,
    source_visual_qa_dir: args.qaDir,
    output_dir: args.outputDir,
    assets,
    checks,
    caveats: [
      "This is a website export package, not embedded on the public site.",
      "Public-site copy and deployment review remain separate gates.",
      "Human aesthetic review may still reject the creative result.",
    ],
    inherited_visual_qa_status: qaData.status,
  };
  writeFileSync(join(args.outputDir, "promo-website-export-data.json"), `${JSON.stringify(data, null, 2)}\n`, "utf-8");
  writeFileSync(
    join(args.outputDir, "promo-website-export-report.md"),
    [
      "# Garnet Promo Website Export",
      "",
      `- Verdict: ${verdict}`,
      `- MP4: ${basename(assets[0].file)}`,
      `- WebM: ${basename(assets[1].file)}`,
      `- Poster: ${basename(assets[2].file)}`,
      "- Embed snippet: `embed-snippet.html`",
      "",
      "## Boundary",
      "",
      "- This package is not embedded on the public site.",
      "- Public-site copy/deployment and human review remain separate gates.",
      "",
    ].join("\n"),
    "utf-8",
  );
  writeManifest(args.outputDir, [
    "garnet-promo.mp4",
    "garnet-promo.webm",
    "garnet-promo-poster.png",
    "embed-snippet.html",
    "promo-website-export-data.json",
    "promo-website-export-report.md",
  ]);
  console.log(`Garnet promo website export: ${args.outputDir}`);
  if (verdict !== "pass") process.exit(1);
}

main();
