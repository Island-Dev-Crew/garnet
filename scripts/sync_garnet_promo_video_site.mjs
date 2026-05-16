#!/usr/bin/env node
import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(SCRIPT_DIR, "..");
const DEFAULT_EXPORT_DIR = "/Users/idc2.0/Desktop/dogfood/garnet-promo-video-website-export";
const DEFAULT_DOCS_DIR = join(ROOT, "docs");
const DEFAULT_OUTPUT_DIR = "/Users/idc2.0/Desktop/dogfood/garnet-promo-video-site-sync";
const SITE_ASSET_DIR_LABEL = "docs/assets";
const REQUIRED_ASSETS = [
  "garnet-promo.mp4",
  "garnet-promo.webm",
  "garnet-promo-poster.png",
];

function usage() {
  return `Usage: sync_garnet_promo_video_site.mjs [options]

Sync the verified Garnet promo website export into the public site.

Options:
  --export-dir <dir>   Website export evidence directory (default: Desktop website export bundle)
  --docs-dir <dir>     Public site docs directory (default: repo docs)
  --output-dir <dir>   Public site sync evidence directory (default: Desktop site-sync bundle)
  -h, --help           Show this help

Copies:
  docs/assets/garnet-promo.mp4
  docs/assets/garnet-promo.webm
  docs/assets/garnet-promo-poster.png

Outputs:
  promo-site-sync-data.json
  promo-site-sync-report.md
  MANIFEST.sha256`;
}

function parseArgs(argv) {
  const args = {
    exportDir: process.env.GARNET_PROMO_VIDEO_EXPORT_DIR || DEFAULT_EXPORT_DIR,
    docsDir: process.env.GARNET_PROMO_VIDEO_DOCS_DIR || DEFAULT_DOCS_DIR,
    outputDir: process.env.GARNET_PROMO_VIDEO_SITE_SYNC_DIR || DEFAULT_OUTPUT_DIR,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--export-dir") args.exportDir = argv[++index];
    else if (arg === "--docs-dir") args.docsDir = argv[++index];
    else if (arg === "--output-dir") args.outputDir = argv[++index];
    else if (arg === "-h" || arg === "--help") {
      console.log(usage());
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  args.exportDir = resolve(args.exportDir);
  args.docsDir = resolve(args.docsDir);
  args.outputDir = resolve(args.outputDir);
  return args;
}

function sha256(pathname) {
  return createHash("sha256").update(readFileSync(pathname)).digest("hex");
}

function readJson(pathname) {
  return JSON.parse(readFileSync(pathname, "utf-8"));
}

function assertWebsiteExportPassed(exportDir) {
  const dataPath = join(exportDir, "promo-website-export-data.json");
  if (!existsSync(dataPath)) throw new Error(`missing website export data: ${dataPath}`);
  const data = readJson(dataPath);
  const checks = Array.isArray(data.checks) ? data.checks : [];
  const passed =
    data.status === "website-export-ready" &&
    data.verdict === "pass" &&
    checks.length > 0 &&
    checks.every((check) => check && check.passed === true);
  if (!passed) throw new Error(`website export evidence did not pass: ${dataPath}`);
  return data;
}

function copyRequired(exportDir, docsDir, filename) {
  const source = join(exportDir, filename);
  const target = join(docsDir, "assets", filename);
  if (!existsSync(source)) throw new Error(`missing exported promo asset: ${source}`);
  copyFileSync(source, target);
  return {
    file: filename,
    source,
    target,
    site_path: `${SITE_ASSET_DIR_LABEL}/${filename}`,
    sha256: sha256(target),
    size_bytes: readFileSync(target).length,
  };
}

function assertPublicSiteReferences(docsDir) {
  const sitePath = join(docsDir, "index.html");
  const workerPath = join(docsDir, "service-worker.js");
  if (!existsSync(sitePath)) throw new Error(`missing public site file: ${sitePath}`);
  if (!existsSync(workerPath)) throw new Error(`missing service worker file: ${workerPath}`);
  const site = readFileSync(sitePath, "utf-8");
  const worker = readFileSync(workerPath, "utf-8");
  const requiredSiteTokens = [
    'id="promo"',
    'class="promo-video"',
    'poster="assets/garnet-promo-poster.png"',
    'src="assets/garnet-promo.webm"',
    'src="assets/garnet-promo.mp4"',
    "Public-site embedded",
    "human/aesthetic acceptance",
    "not full MIT/productization completion",
  ];
  const requiredWorkerTokens = [
    "assets/garnet-promo.mp4",
    "assets/garnet-promo.webm",
    "assets/garnet-promo-poster.png",
  ];
  return [
    ...requiredSiteTokens.map((token) => ({
      id: `site-token:${token}`,
      passed: site.includes(token),
      source: relative(ROOT, sitePath),
    })),
    ...requiredWorkerTokens.map((token) => ({
      id: `service-worker-token:${token}`,
      passed: worker.includes(token),
      source: relative(ROOT, workerPath),
    })),
  ];
}

function writeManifest(outputDir, files) {
  const rows = files.map((file) => `${sha256(join(outputDir, file))}  ${file}`);
  writeFileSync(join(outputDir, "MANIFEST.sha256"), `${rows.join("\n")}\n`, "utf-8");
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const exportData = assertWebsiteExportPassed(args.exportDir);
  mkdirSync(join(args.docsDir, "assets"), { recursive: true });
  mkdirSync(args.outputDir, { recursive: true });
  for (const file of ["promo-site-sync-data.json", "promo-site-sync-report.md", "MANIFEST.sha256"]) {
    rmSync(join(args.outputDir, file), { force: true });
  }

  const assets = REQUIRED_ASSETS.map((asset) => copyRequired(args.exportDir, args.docsDir, asset));
  const checks = [
    { id: "website-export-evidence", passed: true, source: join(args.exportDir, "promo-website-export-data.json") },
    ...assets.map((asset) => ({ id: `${asset.file}-synced-to-public-site`, passed: asset.size_bytes > 0, ...asset })),
    ...assertPublicSiteReferences(args.docsDir),
  ];
  const verdict = checks.every((check) => check.passed) ? "pass" : "fail";
  const data = {
    status: verdict === "pass" ? "public-site-embedded" : "public-site-sync-failed",
    verdict,
    source_export_dir: args.exportDir,
    docs_dir: args.docsDir,
    output_dir: args.outputDir,
    assets,
    checks,
    caveats: [
      "This proves the verified website export is copied into the repo public site.",
      "It does not prove final human/aesthetic acceptance.",
      "It does not claim full MIT/productization completion.",
    ],
    inherited_website_export_status: exportData.status,
  };
  writeFileSync(join(args.outputDir, "promo-site-sync-data.json"), `${JSON.stringify(data, null, 2)}\n`, "utf-8");
  writeFileSync(
    join(args.outputDir, "promo-site-sync-report.md"),
    [
      "# Garnet Promo Public Site Sync",
      "",
      `- Verdict: ${verdict}`,
      `- Docs directory: \`${args.docsDir}\``,
      `- MP4: \`${basename(assets[0].target)}\``,
      `- WebM: \`${basename(assets[1].target)}\``,
      `- Poster: \`${basename(assets[2].target)}\``,
      "",
      "## Boundary",
      "",
      "- Public-site embedded evidence is now present.",
      "- Human/aesthetic acceptance remains open.",
      "- This is not full MIT/productization completion.",
      "",
    ].join("\n"),
    "utf-8",
  );
  writeManifest(args.outputDir, ["promo-site-sync-data.json", "promo-site-sync-report.md"]);
  console.log(`Garnet promo public site sync: ${args.outputDir}`);
  if (verdict !== "pass") process.exit(1);
}

main();
