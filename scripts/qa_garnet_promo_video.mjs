#!/usr/bin/env node
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const DEFAULT_INPUT_DIR = "/Users/idc2.0/Desktop/dogfood/garnet-promo-video";
const DEFAULT_OUTPUT_DIR = "/Users/idc2.0/Desktop/dogfood/garnet-promo-video-visual-qa";

function usage() {
  return `Usage: qa_garnet_promo_video.mjs [options]

Create automated visual QA evidence for the local Garnet promo render.

Options:
  --input-dir <dir>    Render artifact directory (default: Desktop promo render bundle)
  --output-dir <dir>   Visual QA evidence directory (default: Desktop visual QA bundle)
  --ffmpeg <path>      ffmpeg executable (default: ffmpeg)
  --ffprobe <path>     ffprobe executable (default: ffprobe)
  -h, --help           Show this help

Outputs:
  sample-00.png
  sample-15.png
  sample-29.png
  promo-visual-qa-data.json
  promo-visual-qa-report.md
  MANIFEST.sha256`;
}

function parseArgs(argv) {
  const args = {
    inputDir: process.env.GARNET_PROMO_VIDEO_INPUT_DIR || DEFAULT_INPUT_DIR,
    outputDir: process.env.GARNET_PROMO_VIDEO_QA_OUTPUT_DIR || DEFAULT_OUTPUT_DIR,
    ffmpeg: process.env.FFMPEG_BIN || "ffmpeg",
    ffprobe: process.env.FFPROBE_BIN || "ffprobe",
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--input-dir") args.inputDir = argv[++index];
    else if (arg === "--output-dir") args.outputDir = argv[++index];
    else if (arg === "--ffmpeg") args.ffmpeg = argv[++index];
    else if (arg === "--ffprobe") args.ffprobe = argv[++index];
    else if (arg === "-h" || arg === "--help") {
      console.log(usage());
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  args.inputDir = resolve(args.inputDir);
  args.outputDir = resolve(args.outputDir);
  return args;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { text: true, encoding: "utf-8", ...options });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed\n${result.stdout || ""}${result.stderr || ""}`);
  }
  return result;
}

function probe(ffprobe, file) {
  const result = run(ffprobe, [
    "-v",
    "error",
    "-show_entries",
    "stream=codec_name,width,height,r_frame_rate,duration:format=duration,size",
    "-of",
    "json",
    file,
  ]);
  return JSON.parse(result.stdout);
}

function streamMeta(data) {
  const stream = (data.streams || [])[0] || {};
  const format = data.format || {};
  return {
    codec: stream.codec_name || "",
    width: Number(stream.width || 0),
    height: Number(stream.height || 0),
    frame_rate: stream.r_frame_rate || "",
    duration_seconds: Number(format.duration || stream.duration || 0),
    size_bytes: Number(format.size || 0),
  };
}

function checkMetadata(id, meta, codec) {
  const passed =
    meta.codec === codec &&
    meta.width === 1920 &&
    meta.height === 1080 &&
    meta.frame_rate === "12/1" &&
    Math.abs(meta.duration_seconds - 30.0) < 0.05 &&
    meta.size_bytes > 100_000;
  return { id, passed, meta };
}

function extractSample(ffmpeg, input, second, output) {
  run(ffmpeg, ["-y", "-ss", String(second), "-i", input, "-frames:v", "1", output]);
  const bytes = readFileSync(output);
  const png = bytes.slice(0, 8).equals(Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]));
  return { output, passed: png && bytes.length > 25_000, size_bytes: bytes.length };
}

function sha256(pathname) {
  return createHash("sha256").update(readFileSync(pathname)).digest("hex");
}

function writeManifest(outputDir, files) {
  const rows = files.map((file) => `${sha256(join(outputDir, file))}  ${file}`);
  writeFileSync(join(outputDir, "MANIFEST.sha256"), `${rows.join("\n")}\n`, "utf-8");
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const mp4 = join(args.inputDir, "garnet-promo.mp4");
  const webm = join(args.inputDir, "garnet-promo.webm");
  for (const file of [mp4, webm]) {
    if (!existsSync(file)) throw new Error(`missing render artifact: ${file}`);
  }
  mkdirSync(args.outputDir, { recursive: true });
  for (const sample of ["sample-00.png", "sample-15.png", "sample-29.png"]) {
    rmSync(join(args.outputDir, sample), { force: true });
  }

  const mp4Check = checkMetadata("mp4-metadata", streamMeta(probe(args.ffprobe, mp4)), "h264");
  const webmCheck = checkMetadata("webm-metadata", streamMeta(probe(args.ffprobe, webm)), "vp9");
  const samples = [
    extractSample(args.ffmpeg, mp4, 0, join(args.outputDir, "sample-00.png")),
    extractSample(args.ffmpeg, mp4, 15, join(args.outputDir, "sample-15.png")),
    extractSample(args.ffmpeg, mp4, 29, join(args.outputDir, "sample-29.png")),
  ];
  const sampleCheck = {
    id: "sample-frames",
    passed: samples.every((sample) => sample.passed),
    samples: samples.map((sample) => ({ file: sample.output.split("/").pop(), size_bytes: sample.size_bytes })),
  };
  const checks = [mp4Check, webmCheck, sampleCheck];
  const verdict = checks.every((check) => check.passed) ? "pass" : "fail";
  const data = {
    status: verdict === "pass" ? "visual-qa-ready" : "visual-qa-failed",
    verdict,
    source_render_dir: args.inputDir,
    output_dir: args.outputDir,
    checks,
    caveats: [
      "Automated QA checks metadata and representative frame extraction only.",
      "Website-ready export remains a separate gate.",
      "Human review may still reject the creative/aesthetic result.",
    ],
  };
  writeFileSync(join(args.outputDir, "promo-visual-qa-data.json"), `${JSON.stringify(data, null, 2)}\n`, "utf-8");
  writeFileSync(
    join(args.outputDir, "promo-visual-qa-report.md"),
    [
      "# Garnet Promo Automated Visual QA",
      "",
      `- Verdict: ${verdict}`,
      `- MP4: ${mp4Check.passed ? "pass" : "fail"}`,
      `- WebM: ${webmCheck.passed ? "pass" : "fail"}`,
      `- Sample frames: ${sampleCheck.passed ? "pass" : "fail"}`,
      "",
      "## Open Gates",
      "",
      "- Website-ready export",
      "- Public-site embedding",
      "- Optional human aesthetic review",
      "",
    ].join("\n"),
    "utf-8",
  );
  writeManifest(args.outputDir, [
    "sample-00.png",
    "sample-15.png",
    "sample-29.png",
    "promo-visual-qa-data.json",
    "promo-visual-qa-report.md",
  ]);
  console.log(`Garnet promo visual QA: ${args.outputDir}`);
  if (verdict !== "pass") process.exit(1);
}

main();
