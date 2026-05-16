#!/usr/bin/env node
import { createServer } from "node:http";
import { createHash } from "node:crypto";
import { createReadStream, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { access, rm, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, extname, join, resolve, sep } from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const ROOT = resolve(fileURLToPath(new URL("..", import.meta.url)));
const DEFAULT_CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const DEFAULT_OUTPUT_DIR = "/Users/idc2.0/Desktop/dogfood/garnet-promo-video";

function usage() {
  return `Usage: render_garnet_promo_video.mjs [options]

Render the repo-owned Garnet promo composition to MP4 and WebM with manifest evidence.

Options:
  --docs-dir <dir>       Docs root to serve (default: docs)
  --composition <path>   Composition path under docs root (default: promo/composition.html)
  --output-dir <dir>     Evidence/artifact output directory (default: Desktop dogfood canonical path)
  --chrome <path>        Chrome executable (default: Google Chrome.app)
  --ffmpeg <path>        ffmpeg executable (default: ffmpeg)
  --fps <number>         Capture frame rate (default: 12)
  --width <number>       Viewport width (default: 1920)
  --height <number>      Viewport height (default: 1080)
  --duration <seconds>   Output duration in seconds (default: 30)
  --keep-frames          Preserve captured PNG frame directory
  -h, --help             Show this help

Outputs:
  garnet-promo.mp4
  garnet-promo.webm
  garnet-promo-poster.png
  promo-render-data.json
  promo-render-report.md
  MANIFEST.sha256`;
}

function parseArgs(argv) {
  const args = {
    docsDir: join(ROOT, "docs"),
    composition: "promo/composition.html",
    outputDir: process.env.GARNET_PROMO_VIDEO_OUTPUT_DIR || DEFAULT_OUTPUT_DIR,
    chrome: process.env.CHROME_BIN || DEFAULT_CHROME,
    ffmpeg: process.env.FFMPEG_BIN || "ffmpeg",
    fps: 12,
    width: 1920,
    height: 1080,
    duration: 30,
    keepFrames: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--docs-dir") args.docsDir = argv[++index];
    else if (arg === "--composition") args.composition = argv[++index];
    else if (arg === "--output-dir") args.outputDir = argv[++index];
    else if (arg === "--chrome") args.chrome = argv[++index];
    else if (arg === "--ffmpeg") args.ffmpeg = argv[++index];
    else if (arg === "--fps") args.fps = Number(argv[++index]);
    else if (arg === "--width") args.width = Number(argv[++index]);
    else if (arg === "--height") args.height = Number(argv[++index]);
    else if (arg === "--duration") args.duration = Number(argv[++index]);
    else if (arg === "--keep-frames") args.keepFrames = true;
    else if (arg === "-h" || arg === "--help") {
      console.log(usage());
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  args.docsDir = resolve(args.docsDir);
  args.outputDir = resolve(args.outputDir);
  args.chrome = resolve(args.chrome);
  for (const [label, value] of [
    ["--fps", args.fps],
    ["--width", args.width],
    ["--height", args.height],
    ["--duration", args.duration],
  ]) {
    if (!Number.isFinite(value) || value <= 0) throw new Error(`${label} must be positive`);
  }
  return args;
}

function contentType(pathname) {
  const ext = extname(pathname);
  if (ext === ".html") return "text/html; charset=utf-8";
  if (ext === ".js" || ext === ".mjs") return "application/javascript; charset=utf-8";
  if (ext === ".png") return "image/png";
  if (ext === ".css") return "text/css; charset=utf-8";
  return "application/octet-stream";
}

async function fileExists(pathname) {
  try {
    await access(pathname);
    return true;
  } catch {
    return false;
  }
}

async function startServer(docsDir) {
  const rootPrefix = docsDir.endsWith(sep) ? docsDir : `${docsDir}${sep}`;
  const server = createServer(async (request, response) => {
    const url = new URL(request.url || "/", "http://127.0.0.1");
    const relative = decodeURIComponent(url.pathname === "/" ? "/index.html" : url.pathname);
    const candidate = resolve(docsDir, `.${relative}`);
    const insideRoot = candidate === docsDir || candidate.startsWith(rootPrefix);
    const path = insideRoot && (await fileExists(candidate)) ? candidate : join(docsDir, "index.html");
    try {
      const info = await stat(path);
      response.writeHead(200, {
        "content-length": info.size,
        "content-type": contentType(path),
        "cache-control": "no-store",
      });
      createReadStream(path).pipe(response);
    } catch (error) {
      response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
      response.end(String(error));
    }
  });
  await new Promise((resolveStart) => server.listen(0, "127.0.0.1", resolveStart));
  const address = server.address();
  return { server, baseUrl: `http://127.0.0.1:${address.port}` };
}

function wait(ms) {
  return new Promise((resolveWait) => setTimeout(resolveWait, ms));
}

async function waitForJson(url, timeoutMs = 10_000, init = {}) {
  const deadline = Date.now() + timeoutMs;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url, init);
      if (response.ok) return await response.json();
      lastError = new Error(`HTTP ${response.status} from ${url}`);
    } catch (error) {
      lastError = error;
    }
    await wait(100);
  }
  throw lastError || new Error(`timed out waiting for ${url}`);
}

function launchChrome(chrome, port, userDataDir) {
  return spawn(
    chrome,
    [
      "--headless=new",
      "--disable-gpu",
      "--no-first-run",
      "--no-default-browser-check",
      `--remote-debugging-port=${port}`,
      `--user-data-dir=${userDataDir}`,
      "about:blank",
    ],
    { stdio: ["ignore", "pipe", "pipe"] },
  );
}

class CdpClient {
  constructor(wsUrl) {
    this.nextId = 1;
    this.pending = new Map();
    this.events = new Map();
    this.socket = new WebSocket(wsUrl);
  }

  async open() {
    await new Promise((resolveOpen, rejectOpen) => {
      this.socket.addEventListener("open", resolveOpen, { once: true });
      this.socket.addEventListener("error", rejectOpen, { once: true });
    });
    this.socket.addEventListener("message", (message) => {
      const payload = JSON.parse(message.data);
      if (payload.id && this.pending.has(payload.id)) {
        const { resolveCommand, rejectCommand } = this.pending.get(payload.id);
        this.pending.delete(payload.id);
        if (payload.error) rejectCommand(new Error(payload.error.message));
        else resolveCommand(payload.result || {});
        return;
      }
      if (payload.method && this.events.has(payload.method)) {
        for (const listener of this.events.get(payload.method)) listener(payload.params || {});
      }
    });
  }

  send(method, params = {}) {
    const id = this.nextId++;
    this.socket.send(JSON.stringify({ id, method, params }));
    return new Promise((resolveCommand, rejectCommand) => {
      this.pending.set(id, { resolveCommand, rejectCommand });
    });
  }

  once(method, timeoutMs = 10_000) {
    return new Promise((resolveEvent, rejectEvent) => {
      const timer = setTimeout(() => rejectEvent(new Error(`timed out waiting for ${method}`)), timeoutMs);
      const listener = (params) => {
        clearTimeout(timer);
        const listeners = this.events.get(method) || [];
        this.events.set(method, listeners.filter((item) => item !== listener));
        resolveEvent(params);
      };
      const listeners = this.events.get(method) || [];
      listeners.push(listener);
      this.events.set(method, listeners);
    });
  }

  close() {
    this.socket.close();
  }
}

async function navigate(client, url) {
  const loaded = client.once("Page.loadEventFired", 20_000);
  await client.send("Page.navigate", { url });
  await loaded;
}

async function evaluate(client, expression, awaitPromise = true) {
  const result = await client.send("Runtime.evaluate", {
    expression,
    awaitPromise,
    returnByValue: true,
    userGesture: true,
  });
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.text || "runtime evaluation failed");
  }
  return result.result?.value;
}

function runCommand(command, args, cwd = ROOT) {
  const result = spawnSync(command, args, { cwd, text: true, encoding: "utf-8" });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed\n${result.stdout || ""}${result.stderr || ""}`);
  }
  return result;
}

function sha256(pathname) {
  return createHash("sha256").update(readFileSync(pathname)).digest("hex");
}

function writeManifest(outputDir, files) {
  const rows = files.map((file) => `${sha256(join(outputDir, file))}  ${file}`);
  writeFileSync(join(outputDir, "MANIFEST.sha256"), `${rows.join("\n")}\n`, "utf-8");
}

async function waitForProcessExit(child, timeoutMs = 5_000) {
  if (child.exitCode !== null || child.signalCode !== null) return;
  await new Promise((resolveExit) => {
    const timer = setTimeout(resolveExit, timeoutMs);
    child.once("exit", () => {
      clearTimeout(timer);
      resolveExit();
    });
  });
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const compositionPath = join(args.docsDir, args.composition);
  if (!(await fileExists(compositionPath))) throw new Error(`missing composition: ${compositionPath}`);
  if (!existsSync(args.chrome)) throw new Error(`Chrome executable not found: ${args.chrome}`);
  runCommand(args.ffmpeg, ["-version"]);
  if (typeof WebSocket !== "function") {
    throw new Error("Node runtime does not expose WebSocket; use Node 22+ or provide another CDP client");
  }

  mkdirSync(args.outputDir, { recursive: true });
  const framesDir = join(args.outputDir, "frames");
  rmSync(framesDir, { recursive: true, force: true });
  mkdirSync(framesDir, { recursive: true });

  const { server, baseUrl } = await startServer(args.docsDir);
  const userDataDir = mkdtempSync(join(tmpdir(), "garnet-promo-render-"));
  const remotePort = 9400 + Math.floor(Math.random() * 1000);
  const chrome = launchChrome(args.chrome, remotePort, userDataDir);
  const stderr = [];
  chrome.stderr.on("data", (chunk) => stderr.push(String(chunk)));
  let client;

  try {
    const target = await waitForJson(
      `http://127.0.0.1:${remotePort}/json/new?${encodeURIComponent(`${baseUrl}/${args.composition}`)}`,
      10_000,
      { method: "PUT" },
    );
    const version = await waitForJson(`http://127.0.0.1:${remotePort}/json/version`);
    client = new CdpClient(target.webSocketDebuggerUrl);
    await client.open();
    await client.send("Page.enable");
    await client.send("Runtime.enable");
    await client.send("Emulation.setDeviceMetricsOverride", {
      width: args.width,
      height: args.height,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await navigate(client, `${baseUrl}/${args.composition}`);
    const timelineReady = await evaluate(
      client,
      `new Promise((resolve) => {
        let attempts = 0;
        const tick = () => {
          const timeline = window.__timelines && window.__timelines["garnet-promo-main"];
          if (timeline && typeof timeline.seek === "function") resolve(true);
          else if (attempts++ > 200) resolve(false);
          else setTimeout(tick, 100);
        };
        tick();
      })`,
    );
    if (!timelineReady) throw new Error("garnet-promo-main timeline did not register");

    const frameCount = Math.ceil(args.duration * args.fps);
    for (let frame = 0; frame < frameCount; frame += 1) {
      const seconds = frame / args.fps;
      await evaluate(
        client,
        `(() => {
          const timeline = window.__timelines["garnet-promo-main"];
          timeline.pause();
          timeline.seek(${seconds.toFixed(4)}, false);
          document.body.offsetHeight;
          return true;
        })()`,
      );
      await wait(8);
      const screenshot = await client.send("Page.captureScreenshot", {
        format: "png",
        captureBeyondViewport: false,
      });
      const framePath = join(framesDir, `frame-${String(frame).padStart(5, "0")}.png`);
      writeFileSync(framePath, Buffer.from(screenshot.data, "base64"));
    }

    const posterFrame = join(framesDir, `frame-${String(Math.floor(frameCount / 2)).padStart(5, "0")}.png`);
    writeFileSync(join(args.outputDir, "garnet-promo-poster.png"), readFileSync(posterFrame));

    const inputPattern = join(framesDir, "frame-%05d.png");
    runCommand(args.ffmpeg, [
      "-y",
      "-framerate",
      String(args.fps),
      "-i",
      inputPattern,
      "-t",
      String(args.duration),
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p",
      "-movflags",
      "+faststart",
      join(args.outputDir, "garnet-promo.mp4"),
    ]);
    runCommand(args.ffmpeg, [
      "-y",
      "-framerate",
      String(args.fps),
      "-i",
      inputPattern,
      "-t",
      String(args.duration),
      "-c:v",
      "libvpx-vp9",
      "-b:v",
      "0",
      "-crf",
      "34",
      join(args.outputDir, "garnet-promo.webm"),
    ]);

    const evidence = {
      status: "rendered-artifact-ready",
      source: ROOT,
      composition: args.composition,
      output_dir: args.outputDir,
      fps: args.fps,
      duration_seconds: args.duration,
      width: args.width,
      height: args.height,
      frame_count: frameCount,
      chrome: {
        executable: args.chrome,
        browser: version.Browser,
        protocolVersion: version["Protocol-Version"],
      },
      artifacts: {
        mp4: "garnet-promo.mp4",
        webm: "garnet-promo.webm",
        poster: "garnet-promo-poster.png",
      },
      caveats: [
        "Rendered media exists as local/Desktop evidence only.",
        "Visual QA verdict is still required before website use.",
        "Website-ready export is still required before public embedding.",
      ],
    };
    writeFileSync(join(args.outputDir, "promo-render-data.json"), `${JSON.stringify(evidence, null, 2)}\n`, "utf-8");
    writeFileSync(
      join(args.outputDir, "promo-render-report.md"),
      [
        "# Garnet Promo Render Evidence",
        "",
        `- Status: ${evidence.status}`,
        `- Composition: \`${args.composition}\``,
        `- Duration: ${args.duration} seconds`,
        `- FPS: ${args.fps}`,
        `- Frame size: ${args.width}x${args.height}`,
        "- MP4: `garnet-promo.mp4`",
        "- WebM: `garnet-promo.webm`",
        "- Poster: `garnet-promo-poster.png`",
        "",
        "## Open Gates",
        "",
        "- Visual QA verdict",
        "- Website-ready export",
        "- Repo/site overclaim check",
        "",
      ].join("\n"),
      "utf-8",
    );
    writeManifest(args.outputDir, [
      "garnet-promo.mp4",
      "garnet-promo.webm",
      "garnet-promo-poster.png",
      "promo-render-data.json",
      "promo-render-report.md",
    ]);
    if (!args.keepFrames) rmSync(framesDir, { recursive: true, force: true });
    console.log(`Garnet promo render: ${args.outputDir}`);
  } finally {
    if (client) client.close();
    await new Promise((resolveClose) => server.close(resolveClose));
    chrome.kill("SIGTERM");
    await waitForProcessExit(chrome);
    await rm(userDataDir, { recursive: true, force: true, maxRetries: 4, retryDelay: 100 });
    if (stderr.length && process.env.GARNET_PROMO_RENDER_DEBUG) {
      console.error(stderr.join(""));
    }
  }
}

main().catch((error) => {
  console.error(error.stack || error.message || String(error));
  process.exit(1);
});
