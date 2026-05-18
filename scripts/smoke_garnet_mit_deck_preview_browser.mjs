#!/usr/bin/env node
import { createServer } from "node:http";
import { createServer as createNetServer } from "node:net";
import { createReadStream, existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { access, rm, stat } from "node:fs/promises";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { homedir, tmpdir } from "node:os";
import { basename, extname, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(fileURLToPath(new URL("..", import.meta.url)));
const DEFAULT_CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

function defaultPython() {
  const candidates = [
    process.env.PYTHON,
    process.env.PYTHON3,
    "/opt/homebrew/bin/python3",
    "/usr/local/bin/python3",
    "/usr/bin/python3",
    "python3",
  ].filter(Boolean);
  return candidates.find((candidate) => candidate === "python3" || existsSync(candidate)) || "python3";
}

function timestamp() {
  const now = new Date();
  const pad = (value) => String(value).padStart(2, "0");
  return `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
}

function parseArgs(argv) {
  const args = {
    evidenceDir: join(ROOT, "target", "mit-deck-preview-browser-smoke"),
    chrome: process.env.CHROME_BIN || DEFAULT_CHROME,
    python: defaultPython(),
    keepBrowser: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--evidence-dir") args.evidenceDir = argv[++index];
    else if (arg === "--chrome") args.chrome = argv[++index];
    else if (arg === "--python") args.python = argv[++index];
    else if (arg === "--copy-to-desktop") {
      args.evidenceDir = join(homedir(), "Desktop", "dogfood", `garnet-mit-deck-preview-browser-smoke-${timestamp()}`);
    } else if (arg === "--keep-browser") args.keepBrowser = true;
    else if (arg === "-h" || arg === "--help") {
      console.log("Usage: smoke_garnet_mit_deck_preview_browser.mjs [--evidence-dir dir] [--chrome path] [--python path] [--copy-to-desktop]");
      console.log("");
      console.log("Generates the MIT deck-preview bundle, serves it locally, opens it in headless Chrome,");
      console.log("checks desktop and mobile layout boundaries, captures a screenshot, and writes manifested evidence.");
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  args.evidenceDir = resolve(args.evidenceDir);
  args.chrome = resolve(args.chrome);
  if (args.python !== "python3") args.python = resolve(args.python);
  return args;
}

function contentType(pathname) {
  const ext = extname(pathname);
  if (ext === ".html") return "text/html; charset=utf-8";
  if (ext === ".json") return "application/json; charset=utf-8";
  if (ext === ".md" || ext === ".txt") return "text/plain; charset=utf-8";
  if (ext === ".png") return "image/png";
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

function runChecked(command, args, cwd) {
  const completed = spawnSync(command, args, {
    cwd,
    encoding: "utf-8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (completed.error) throw completed.error;
  if (completed.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed with ${completed.status}\n${completed.stdout}\n${completed.stderr}`);
  }
  return completed;
}

async function startServer(rootDir) {
  const rootPrefix = rootDir.endsWith(sep) ? rootDir : `${rootDir}${sep}`;
  const server = createServer(async (request, response) => {
    const url = new URL(request.url || "/", "http://127.0.0.1");
    const relative = decodeURIComponent(url.pathname === "/" ? "/garnet-mit-deck-preview.html" : url.pathname);
    const candidate = resolve(rootDir, `.${relative}`);
    const insideRoot = candidate === rootDir || candidate.startsWith(rootPrefix);
    const path = insideRoot && (await fileExists(candidate)) ? candidate : join(rootDir, "garnet-mit-deck-preview.html");
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

async function reservePort() {
  const server = createNetServer();
  await new Promise((resolveStart, rejectStart) => {
    server.once("error", rejectStart);
    server.listen(0, "127.0.0.1", resolveStart);
  });
  const address = server.address();
  const port = address.port;
  await new Promise((resolveClose) => server.close(resolveClose));
  return port;
}

function wait(ms) {
  return new Promise((resolveWait) => setTimeout(resolveWait, ms));
}

async function waitForProcessExit(child, timeoutMs = 5_000) {
  if (child.exitCode !== null || child.signalCode !== null) return true;
  let exited = false;
  await new Promise((resolveExit) => {
    const timer = setTimeout(resolveExit, timeoutMs);
    child.once("exit", () => {
      exited = true;
      clearTimeout(timer);
      resolveExit();
    });
  });
  return exited || child.exitCode !== null || child.signalCode !== null;
}

async function removeBrowserProfile(path, maxRetries = 8) {
  for (let attempt = 0; attempt < maxRetries; attempt += 1) {
    try {
      await rm(path, { recursive: true, force: true, maxRetries: 2, retryDelay: 100 });
      return;
    } catch (error) {
      if (attempt === maxRetries - 1) throw error;
      await wait(150 * (attempt + 1));
    }
  }
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
      "--disable-background-networking",
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
      const timer = setTimeout(() => {
        rejectEvent(new Error(`timed out waiting for ${method}`));
      }, timeoutMs);
      const listener = (params) => {
        clearTimeout(timer);
        const listeners = this.events.get(method) || [];
        this.events.set(
          method,
          listeners.filter((item) => item !== listener),
        );
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
  const loaded = client.once("Page.loadEventFired", 15_000);
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

async function inspectPage(client, width, height, mobile, expectedObjectiveMetric) {
  await client.send("Emulation.setDeviceMetricsOverride", {
    width,
    height,
    deviceScaleFactor: mobile ? 2 : 1,
    mobile,
  });
  return evaluate(
    client,
    `(() => {
      const text = document.body.textContent || "";
      const externalAssets = Array.from(document.querySelectorAll("script[src], link[rel='stylesheet'], img[src], video[src], audio[src]"))
        .map((item) => item.getAttribute("src") || item.getAttribute("href") || "");
      const metrics = Array.from(document.querySelectorAll(".metric strong")).map((item) => item.textContent.trim());
      const slideIds = Array.from(document.querySelectorAll(".slide")).map((item) => item.dataset.slideId || "");
      const horizontalOverflow = document.documentElement.scrollWidth > window.innerWidth + 1;
      return {
        title: document.title,
        width: window.innerWidth,
        height: window.innerHeight,
        scrollWidth: document.documentElement.scrollWidth,
        slideCount: slideIds.length,
        slideIds,
        metrics,
        externalAssets,
        horizontalOverflow,
        hasEvidence: text.includes("Evidence"),
        hasSpeakerNotes: text.includes("Speaker note:"),
        hasBlockedGates: text.includes("Blocked and deferred gates remain separate."),
        hasFinalAcceptanceBoundary: text.includes("final MIT/productization acceptance"),
        hasCompletionBoundary: text.includes("not full MIT/productization completion"),
        hasTrackedSlices: text.includes("87/87"),
        hasObjectiveMetric: metrics.includes(${JSON.stringify(expectedObjectiveMetric)}),
      };
    })()`,
  );
}

function writeManifest(files, evidenceDir) {
  const lines = files.map((file) => {
    const hash = createHash("sha256").update(readFileSync(file)).digest("hex");
    return `${hash}  ./${basename(file)}\n`;
  });
  writeFileSync(join(evidenceDir, "MANIFEST.sha256"), lines.join(""), "utf-8");
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (!(await fileExists(args.chrome))) {
    throw new Error(`Chrome executable not found: ${args.chrome}`);
  }
  if (typeof WebSocket !== "function") {
    throw new Error("Node runtime does not expose WebSocket; use Node 22+ or provide another CDP client");
  }

  const evidenceDir = args.evidenceDir;
  const bundleDir = join(evidenceDir, "deck-preview");
  const output = join(evidenceDir, "browser-smoke-data.json");
  const report = join(evidenceDir, "browser-smoke-report.md");
  const screenshot = join(evidenceDir, "browser-smoke.png");
  mkdirSync(bundleDir, { recursive: true });

  runChecked(args.python, [join(ROOT, "scripts", "garnet_mit_deck_preview.py"), "--output-dir", bundleDir], ROOT);
  const previewData = JSON.parse(readFileSync(join(bundleDir, "garnet-mit-deck-preview.json"), "utf-8"));
  const expectedObjectiveMetric = `${previewData.objective_completion_percent.toFixed(1)}%`;
  const manifestCheck = runChecked("shasum", ["-a", "256", "-c", "MANIFEST.sha256"], bundleDir);

  const { server, baseUrl } = await startServer(bundleDir);
  const userDataDir = mkdtempSync(join(tmpdir(), "garnet-deck-preview-browser-"));
  const remotePort = await reservePort();
  const chrome = launchChrome(args.chrome, remotePort, userDataDir);
  const stderr = [];
  chrome.stderr.on("data", (chunk) => stderr.push(String(chunk)));

  let client;
  try {
    const target = await waitForJson(
      `http://127.0.0.1:${remotePort}/json/new?${encodeURIComponent(`${baseUrl}/garnet-mit-deck-preview.html`)}`,
      10_000,
      { method: "PUT" },
    );
    const version = await waitForJson(`http://127.0.0.1:${remotePort}/json/version`);
    client = new CdpClient(target.webSocketDebuggerUrl);
    await client.open();
    await client.send("Page.enable");
    await client.send("Runtime.enable");

    await navigate(client, `${baseUrl}/garnet-mit-deck-preview.html`);
    const desktop = await inspectPage(client, 1440, 1000, false, expectedObjectiveMetric);
    const capture = await client.send("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: false,
    });
    writeFileSync(screenshot, Buffer.from(capture.data, "base64"));

    await navigate(client, `${baseUrl}/garnet-mit-deck-preview.html`);
    const mobile = await inspectPage(client, 390, 844, true, expectedObjectiveMetric);
    await client.send("Emulation.clearDeviceMetricsOverride");

    const passed =
      desktop.title === "Garnet MIT Deck Preview" &&
      desktop.slideCount >= 8 &&
      desktop.hasEvidence &&
      desktop.hasSpeakerNotes &&
      desktop.hasBlockedGates &&
      desktop.hasFinalAcceptanceBoundary &&
      desktop.hasCompletionBoundary &&
      desktop.hasTrackedSlices &&
      desktop.hasObjectiveMetric &&
      desktop.externalAssets.length === 0 &&
      desktop.horizontalOverflow === false &&
      mobile.horizontalOverflow === false;

    const evidence = {
      passed,
      baseUrl,
      evidenceDir,
      bundleDir,
      chrome: {
        executable: args.chrome,
        browser: version.Browser,
        protocolVersion: version["Protocol-Version"],
      },
      manifestCheck: manifestCheck.stdout.trim().split("\n"),
      desktop,
      mobile,
      screenshot: basename(screenshot),
      boundaries: [
        "browser smoke only",
        "not final MIT/productization acceptance",
        "not human/aesthetic deck approval",
        "not Apple Developer ID notarization",
        "not Windows/Linux runtime proof",
      ],
    };
    writeFileSync(output, `${JSON.stringify(evidence, null, 2)}\n`, "utf-8");
    writeFileSync(
      report,
      `# Garnet MIT Deck Preview Browser Smoke\n\n` +
        `Status: ${passed ? "pass" : "fail"}\n\n` +
        `- Evidence directory: \`${evidenceDir}\`\n` +
        `- Bundle directory: \`${bundleDir}\`\n` +
        `- Chrome: \`${version.Browser}\`\n` +
        `- Desktop slides: \`${desktop.slideCount}\`\n` +
        `- Desktop horizontal overflow: \`${desktop.horizontalOverflow}\`\n` +
        `- Mobile horizontal overflow: \`${mobile.horizontalOverflow}\`\n` +
        `- External assets: \`${desktop.externalAssets.length}\`\n` +
        `- Screenshot: \`${basename(screenshot)}\`\n\n` +
        `This is browser-layout evidence for a generated review artifact. It is not final MIT/productization acceptance or human/aesthetic deck approval.\n`,
      "utf-8",
    );
    writeManifest([output, report, screenshot], evidenceDir);
    if (!passed) {
      throw new Error(`MIT deck-preview browser smoke failed; evidence written to ${output}`);
    }
    console.log(`Garnet MIT deck-preview browser smoke: passed (${basename(output)})`);
  } finally {
    if (client) client.close();
    chrome.kill("SIGTERM");
    if (!(await waitForProcessExit(chrome, 2_000))) {
      chrome.kill("SIGKILL");
      await waitForProcessExit(chrome, 5_000);
    }
    if (typeof server.closeAllConnections === "function") server.closeAllConnections();
    await new Promise((resolveClose) => server.close(resolveClose));
    if (!args.keepBrowser) await removeBrowserProfile(userDataDir);
    if (stderr.length && process.env.GARNET_DECK_PREVIEW_BROWSER_DEBUG) {
      console.error(stderr.join(""));
    }
  }
}

main().catch((error) => {
  console.error(error.stack || error.message || String(error));
  process.exit(1);
});
