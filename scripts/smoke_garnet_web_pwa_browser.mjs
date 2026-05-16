#!/usr/bin/env node
import { createServer } from "node:http";
import { createReadStream, mkdtempSync, writeFileSync } from "node:fs";
import { access, readFile, rm, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, extname, join, resolve, sep } from "node:path";
import { spawn } from "node:child_process";

const ROOT = resolve(new URL("..", import.meta.url).pathname);
const DEFAULT_CHROME = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

function parseArgs(argv) {
  const args = {
    docsDir: join(ROOT, "docs"),
    output: join(ROOT, "target", "web-pwa-browser-offline-check.json"),
    chrome: process.env.CHROME_BIN || DEFAULT_CHROME,
    keepBrowser: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--docs-dir") args.docsDir = argv[++index];
    else if (arg === "--output") args.output = argv[++index];
    else if (arg === "--chrome") args.chrome = argv[++index];
    else if (arg === "--keep-browser") args.keepBrowser = true;
    else if (arg === "-h" || arg === "--help") {
      console.log("Usage: smoke_garnet_web_pwa_browser.mjs [--docs-dir docs] [--output file] [--chrome path]");
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  args.docsDir = resolve(args.docsDir);
  args.output = resolve(args.output);
  args.chrome = resolve(args.chrome);
  return args;
}

function contentType(pathname) {
  const ext = extname(pathname);
  if (ext === ".html") return "text/html; charset=utf-8";
  if (ext === ".js" || ext === ".mjs") return "application/javascript; charset=utf-8";
  if (ext === ".json" || ext === ".webmanifest") return "application/manifest+json; charset=utf-8";
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

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const required = ["index.html", "manifest.webmanifest", "service-worker.js"];
  for (const file of required) {
    if (!(await fileExists(join(args.docsDir, file)))) {
      throw new Error(`missing required docs PWA file: ${file}`);
    }
  }
  if (!(await fileExists(args.chrome))) {
    throw new Error(`Chrome executable not found: ${args.chrome}`);
  }
  if (typeof WebSocket !== "function") {
    throw new Error("Node runtime does not expose WebSocket; use Node 22+ or provide another CDP client");
  }

  const { server, baseUrl } = await startServer(args.docsDir);
  const userDataDir = mkdtempSync(join(tmpdir(), "garnet-pwa-browser-"));
  const remotePort = 9222 + Math.floor(Math.random() * 1000);
  const chrome = launchChrome(args.chrome, remotePort, userDataDir);
  const stderr = [];
  chrome.stderr.on("data", (chunk) => stderr.push(String(chunk)));

  let client;
  try {
    const target = await waitForJson(
      `http://127.0.0.1:${remotePort}/json/new?${encodeURIComponent(`${baseUrl}/`)}`,
      10_000,
      { method: "PUT" },
    );
    const version = await waitForJson(`http://127.0.0.1:${remotePort}/json/version`);
    client = new CdpClient(target.webSocketDebuggerUrl);
    await client.open();
    await client.send("Page.enable");
    await client.send("Runtime.enable");
    await client.send("Network.enable");

    await navigate(client, `${baseUrl}/`);
    const serviceWorker = await evaluate(
      client,
      `navigator.serviceWorker.ready.then(async () => {
        await new Promise((resolve) => setTimeout(resolve, 250));
        return {
          controller: Boolean(navigator.serviceWorker.controller),
          cacheKeys: await caches.keys(),
        };
      })`,
    );
    await navigate(client, `${baseUrl}/`);
    const controlled = await evaluate(client, "Boolean(navigator.serviceWorker.controller)");

    await client.send("Network.emulateNetworkConditions", {
      offline: true,
      latency: 0,
      downloadThroughput: 0,
      uploadThroughput: 0,
    });
    await navigate(client, `${baseUrl}/offline-agent-route`);
    const offlineNavigation = await evaluate(client, `({
      title: document.title,
      hasGarnet: document.body.innerText.includes("Garnet"),
      url: location.href,
    })`);
    const manifestFetch = await evaluate(
      client,
      `fetch("/manifest.webmanifest").then(async (response) => ({
        ok: response.ok,
        status: response.status,
        textHasName: (await response.text()).includes("Garnet")
      }))`,
    );
    await client.send("Network.emulateNetworkConditions", {
      offline: false,
      latency: 0,
      downloadThroughput: -1,
      uploadThroughput: -1,
    });

    const evidence = {
      baseUrl,
      docsDir: args.docsDir,
      chrome: {
        executable: args.chrome,
        browser: version.Browser,
        protocolVersion: version["Protocol-Version"],
      },
      serviceWorker,
      controlled,
      offlineNavigation,
      manifestFetch,
      passed:
        controlled &&
        offlineNavigation?.hasGarnet === true &&
        manifestFetch?.ok === true &&
        manifestFetch?.textHasName === true,
    };
    writeFileSync(args.output, `${JSON.stringify(evidence, null, 2)}\n`, "utf-8");
    if (!evidence.passed) {
      throw new Error(`browser PWA offline smoke failed; evidence written to ${args.output}`);
    }
    console.log(`Garnet browser PWA offline smoke: passed (${basename(args.output)})`);
  } finally {
    if (client) client.close();
    await new Promise((resolveClose) => server.close(resolveClose));
    chrome.kill("SIGTERM");
    await waitForProcessExit(chrome);
    if (!args.keepBrowser) await removeBrowserProfile(userDataDir);
    if (stderr.length && process.env.GARNET_PWA_BROWSER_DEBUG) {
      console.error(stderr.join(""));
    }
  }
}

main().catch((error) => {
  console.error(error.stack || error.message || String(error));
  process.exit(1);
});
