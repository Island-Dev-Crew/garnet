#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import vm from "node:vm";

const args = parseArgs(process.argv.slice(2));
const docsDir = path.resolve(args.docsDir ?? "docs");
const outputPath = args.output ? path.resolve(args.output) : null;
const serviceWorkerPath = path.join(docsDir, "service-worker.js");
const indexPath = path.join(docsDir, "index.html");
const baseUrl = "https://garnet.local/";

const report = {
  docsDir,
  serviceWorkerPath,
  checks: [],
  networkCalls: [],
};

function pass(label, evidence = {}) {
  report.checks.push({ status: "pass", label, evidence });
}

function fail(label, evidence = {}) {
  report.checks.push({ status: "fail", label, evidence });
}

function assertCheck(condition, label, evidence = {}) {
  if (condition) {
    pass(label, evidence);
  } else {
    fail(label, evidence);
  }
}

class MockResponse {
  constructor(body, init = {}) {
    this.body = body;
    this.status = init.status ?? 200;
    this.ok = this.status >= 200 && this.status < 300;
    this.headers = init.headers ?? {};
  }

  clone() {
    return new MockResponse(this.body, {
      status: this.status,
      headers: { ...this.headers },
    });
  }

  async text() {
    return String(this.body);
  }
}

class MockRequest {
  constructor(url, init = {}) {
    this.url = new URL(url, baseUrl).href;
    this.method = init.method ?? "GET";
    this.mode = init.mode ?? "cors";
  }

  clone() {
    return new MockRequest(this.url, {
      method: this.method,
      mode: this.mode,
    });
  }
}

class MockCache {
  constructor(name) {
    this.name = name;
    this.entries = new Map();
    this.addedAssets = [];
  }

  async addAll(assets) {
    this.addedAssets.push(...assets);
    for (const asset of assets) {
      const filePath = pathForAsset(asset);
      const body = fs.readFileSync(filePath);
      this.entries.set(cacheKey(asset), new MockResponse(body));
    }
  }

  async match(request) {
    return this.entries.get(cacheKey(request))?.clone();
  }

  async put(request, response) {
    this.entries.set(cacheKey(request), response.clone());
  }
}

class MockCacheStorage {
  constructor() {
    this.caches = new Map();
  }

  async open(name) {
    if (!this.caches.has(name)) {
      this.caches.set(name, new MockCache(name));
    }
    return this.caches.get(name);
  }

  async keys() {
    return [...this.caches.keys()];
  }

  async delete(name) {
    return this.caches.delete(name);
  }

  async match(request) {
    for (const cache of this.caches.values()) {
      const hit = await cache.match(request);
      if (hit) {
        return hit;
      }
    }
    return undefined;
  }
}

const listeners = new Map();
const caches = new MockCacheStorage();
let skipWaitingCalled = false;
let clientsClaimCalled = false;

const self = {
  addEventListener(type, handler) {
    listeners.set(type, handler);
  },
  skipWaiting() {
    skipWaitingCalled = true;
  },
  clients: {
    claim() {
      clientsClaimCalled = true;
    },
  },
};

let failNavigationFetch = false;
async function fetchMock(request) {
  const key = cacheKey(request);
  report.networkCalls.push({ key, mode: request.mode, method: request.method });
  if (failNavigationFetch && request.mode === "navigate") {
    throw new Error("simulated navigation network failure");
  }
  return new MockResponse(`network:${key}`);
}

try {
  assertCheck(fs.existsSync(serviceWorkerPath), "service worker file exists", {
    path: serviceWorkerPath,
  });
  assertCheck(fs.existsSync(indexPath), "offline index exists", {
    path: indexPath,
  });

  const source = fs.readFileSync(serviceWorkerPath, "utf8");
  vm.runInNewContext(source, {
    caches,
    console,
    fetch: fetchMock,
    Promise,
    Request: MockRequest,
    Response: MockResponse,
    self,
    URL,
  }, {
    filename: serviceWorkerPath,
  });

  for (const type of ["install", "activate", "fetch"]) {
    assertCheck(typeof listeners.get(type) === "function", `${type} handler is registered`);
  }

  await caches.open("garnet-web-old");
  await dispatchLifecycle("install");
  const cacheNamesAfterInstall = await caches.keys();
  const activeCacheName = cacheNamesAfterInstall.find((name) => name !== "garnet-web-old");
  const activeCache = caches.caches.get(activeCacheName);
  assertCheck(skipWaitingCalled, "install handler calls skipWaiting");
  assertCheck(Boolean(activeCache), "install handler opens an active cache", {
    cacheNames: cacheNamesAfterInstall,
  });
  assertCheck(
    activeCache?.addedAssets.includes("index.html"),
    "install handler caches the offline shell",
    { addedAssets: activeCache?.addedAssets ?? [] },
  );

  await dispatchLifecycle("activate");
  const cacheNamesAfterActivate = await caches.keys();
  assertCheck(clientsClaimCalled, "activate handler claims clients");
  assertCheck(
    !cacheNamesAfterActivate.includes("garnet-web-old"),
    "activate handler removes obsolete caches",
    { cacheNames: cacheNamesAfterActivate },
  );

  failNavigationFetch = true;
  const offlineNavigation = await dispatchFetch(new MockRequest("/agent-workbench", {
    mode: "navigate",
  }));
  const expectedIndex = fs.readFileSync(indexPath, "utf8");
  assertCheck(
    (await offlineNavigation.text()) === expectedIndex,
    "offline navigation falls back to cached index.html",
    { request: "/agent-workbench" },
  );

  const callsBeforeManifest = report.networkCalls.length;
  const cachedManifest = await dispatchFetch(new MockRequest("/manifest.webmanifest", {
    mode: "same-origin",
  }));
  assertCheck(
    (await cachedManifest.text()) === fs.readFileSync(path.join(docsDir, "manifest.webmanifest"), "utf8"),
    "cached static asset is served without network",
    { request: "/manifest.webmanifest" },
  );
  assertCheck(
    report.networkCalls.length === callsBeforeManifest,
    "cached static asset does not call fetch",
    { before: callsBeforeManifest, after: report.networkCalls.length },
  );

  failNavigationFetch = false;
  const networkedAsset = await dispatchFetch(new MockRequest("/uncached-agent-probe.json", {
    mode: "same-origin",
  }));
  assertCheck(
    (await networkedAsset.text()) === "network:uncached-agent-probe.json",
    "uncached GET falls through to network",
    { request: "/uncached-agent-probe.json" },
  );
  const cachedNetworkedAsset = await caches.match(new MockRequest("/uncached-agent-probe.json"));
  assertCheck(
    (await cachedNetworkedAsset?.text()) === "network:uncached-agent-probe.json",
    "successful uncached GET response is stored for later offline use",
    { request: "/uncached-agent-probe.json" },
  );
} catch (error) {
  fail("offline service worker harness crashed", {
    message: error instanceof Error ? error.message : String(error),
  });
}

const failures = report.checks.filter((check) => check.status !== "pass");
report.summary = {
  checks: report.checks.length,
  failures: failures.length,
  passed: failures.length === 0,
};

const serialized = `${JSON.stringify(report, null, 2)}\n`;
if (outputPath) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, serialized);
} else {
  process.stdout.write(serialized);
}

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(`FAIL: ${failure.label}`);
  }
  process.exit(1);
}

console.log(`Garnet service worker offline behavior: passed ${report.summary.checks}/${report.summary.checks}`);

async function dispatchLifecycle(type) {
  const handler = listeners.get(type);
  const pending = [];
  handler({
    waitUntil(promise) {
      pending.push(Promise.resolve(promise));
    },
  });
  await Promise.all(pending);
}

async function dispatchFetch(request) {
  const handler = listeners.get("fetch");
  let responsePromise;
  handler({
    request,
    respondWith(promise) {
      responsePromise = Promise.resolve(promise);
    },
  });
  if (!responsePromise) {
    throw new Error(`fetch handler did not respond for ${request.url}`);
  }
  const response = await responsePromise;
  if (!response) {
    throw new Error(`fetch handler returned no response for ${request.url}`);
  }
  return response;
}

function cacheKey(input) {
  const raw = typeof input === "string" ? input : input.url;
  const pathname = new URL(raw, baseUrl).pathname.replace(/^\/+/, "");
  if (raw === "./" || pathname === "") {
    return "index.html";
  }
  return pathname;
}

function pathForAsset(asset) {
  const key = cacheKey(asset);
  return path.join(docsDir, key);
}

function parseArgs(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--docs-dir") {
      parsed.docsDir = argv[++index];
    } else if (arg === "--output") {
      parsed.output = argv[++index];
    } else if (arg === "-h" || arg === "--help") {
      console.log("Usage: smoke_garnet_web_pwa_offline.mjs [--docs-dir PATH] [--output PATH]");
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  return parsed;
}
