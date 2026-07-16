#!/usr/bin/env node
import { createHash } from "node:crypto";
import { createReadStream, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { access, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { arch, platform } from "node:os";
import { dirname, extname, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { chromium } from "../apps/garnet-studio/node_modules/playwright/index.mjs";

const ROOT = resolve(fileURLToPath(new URL("..", import.meta.url)));
const DOCS = resolve(ROOT, "docs");
const DEFAULT_PROOF = resolve(ROOT, "F_Project_Management/LAUNCH/W_PLAY_BROWSER_PROOF.json");
const DEFAULT_SCREENSHOT = resolve(ROOT, "ops/lane2a/evidence/30-playground-browser.png");
const RUNTIME_INPUTS = [
  "docs/playground.html",
  "docs/playground/live.js",
  "docs/playground/examples.json",
  "docs/playground/pkg/garnet_wasm.js",
  "docs/playground/pkg/garnet_wasm_bg.wasm",
  "docs/playground/pkg/provenance.json",
  "docs/icons/garnet-192.png",
];
const DIFF_SCOPE = "declared-surface-only; does not prove absence of undeclared authority; bound annotations are not part of this surface";

function defaultChrome() {
  if (process.env.CHROME_BIN) return process.env.CHROME_BIN;
  if (platform() === "win32") {
    return "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
  }
  if (platform() === "linux") return "/usr/bin/google-chrome";
  return "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
}

function parseArgs(argv) {
  const args = {
    proof: DEFAULT_PROOF,
    screenshot: DEFAULT_SCREENSHOT,
    chrome: defaultChrome(),
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--proof") args.proof = resolve(argv[++index]);
    else if (arg === "--screenshot") args.screenshot = resolve(argv[++index]);
    else if (arg === "--chrome") args.chrome = resolve(argv[++index]);
    else if (arg === "-h" || arg === "--help") {
      console.log("Usage: smoke_garnet_playground_browser.mjs [--proof path] [--screenshot path] [--chrome path]");
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  return args;
}

function runGit(args) {
  const result = spawnSync("git", args, {
    cwd: ROOT,
    encoding: "utf-8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`git ${args.join(" ")} failed\n${result.stdout}\n${result.stderr}`);
  }
  return result.stdout.trim();
}

function sha256(raw) {
  return createHash("sha256").update(raw).digest("hex");
}

function assertEqual(actual, expected, label) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function verifyCommittedRuntimeInputs() {
  runGit(["ls-files", "--error-unmatch", "--", ...RUNTIME_INPUTS]);
  const dirty = runGit(["status", "--porcelain", "--", ...RUNTIME_INPUTS]);
  if (dirty) throw new Error(`browser runtime inputs are not clean:\n${dirty}`);
  const provenance = JSON.parse(
    readFileSync(resolve(ROOT, "docs/playground/pkg/provenance.json"), "utf-8"),
  );
  assertEqual(provenance.schema, "garnet.playground.wasm-package/1", "package schema");
  for (const name of ["garnet_wasm.js", "garnet_wasm_bg.wasm"]) {
    const raw = readFileSync(resolve(ROOT, "docs/playground/pkg", name));
    assertEqual(raw.length, provenance.artifacts[name].bytes, `${name} bytes`);
    assertEqual(sha256(raw), provenance.artifacts[name].sha256, `${name} sha256`);
  }
  return provenance;
}

function contentType(pathname) {
  const extension = extname(pathname);
  if (extension === ".html") return "text/html; charset=utf-8";
  if (extension === ".js" || extension === ".mjs") return "text/javascript; charset=utf-8";
  if (extension === ".json") return "application/json; charset=utf-8";
  if (extension === ".wasm") return "application/wasm";
  if (extension === ".png") return "image/png";
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

async function startTrackedServer() {
  const tracked = new Set(runGit(["ls-files", "--", "docs"]).split("\n").filter(Boolean));
  const requested = new Set();
  const untrackedRequests = new Set();
  const docsPrefix = DOCS.endsWith(sep) ? DOCS : `${DOCS}${sep}`;
  const server = createServer(async (request, response) => {
    try {
      const url = new URL(request.url || "/", "http://127.0.0.1");
      const pathname = decodeURIComponent(
        url.pathname === "/" ? "/playground.html" : url.pathname,
      );
      const candidate = resolve(DOCS, `.${pathname}`);
      const insideDocs = candidate === DOCS || candidate.startsWith(docsPrefix);
      const trackedPath = insideDocs ? relative(ROOT, candidate).split(sep).join("/") : "";
      requested.add(trackedPath || pathname);
      if (!insideDocs || !tracked.has(trackedPath) || !(await fileExists(candidate))) {
        untrackedRequests.add(trackedPath || pathname);
        response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
        response.end("not a committed docs input");
        return;
      }
      const info = await stat(candidate);
      if (!info.isFile()) throw new Error("requested path is not a regular file");
      response.writeHead(200, {
        "cache-control": "no-store",
        "content-length": info.size,
        "content-type": contentType(candidate),
        "x-content-type-options": "nosniff",
      });
      createReadStream(candidate).pipe(response);
    } catch (error) {
      response.writeHead(400, { "content-type": "text/plain; charset=utf-8" });
      response.end(String(error));
    }
  });
  await new Promise((resolveStart) => server.listen(0, "127.0.0.1", resolveStart));
  const address = server.address();
  return {
    baseUrl: `http://127.0.0.1:${address.port}`,
    requested,
    server,
    untrackedRequests,
  };
}

async function closeServer(server) {
  if (typeof server.closeAllConnections === "function") server.closeAllConnections();
  await new Promise((resolveClose) => server.close(resolveClose));
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const started = performance.now();
  const packageProvenance = verifyCommittedRuntimeInputs();
  if (!existsSync(args.chrome)) throw new Error(`Chrome executable not found: ${args.chrome}`);
  const trackedServer = await startTrackedServer();
  const externalRequests = new Set();
  const consoleErrors = [];
  const pageErrors = [];
  const browser = await chromium.launch({ executablePath: args.chrome, headless: true });
  const context = await browser.newContext({
    serviceWorkers: "block",
    viewport: { width: 1440, height: 1000 },
  });
  await context.route("**/*", async (route) => {
    const url = new URL(route.request().url());
    if (url.hostname === "127.0.0.1" && url.origin === trackedServer.baseUrl) {
      await route.continue();
    } else {
      externalRequests.add(url.href);
      await route.abort("blockedbyclient");
    }
  });
  const page = await context.newPage();
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  page.on("pageerror", (error) => pageErrors.push(error.message));

  try {
    await page.goto(`${trackedServer.baseUrl}/playground.html`, {
      waitUntil: "networkidle",
      timeout: 15_000,
    });
    await page.waitForFunction(
      () => window.__garnetPlayground?.state.ready === true,
      null,
      { timeout: 10_000 },
    );

    const browserBoundary = await page.evaluate(() => ({
      node_global_present: typeof globalThis.process !== "undefined",
      runtime_ready: window.__garnetPlayground?.state.ready === true,
      user_agent: navigator.userAgent,
    }));
    assertEqual(browserBoundary.node_global_present, false, "browser Node global");
    assertEqual(browserBoundary.runtime_ready, true, "browser runtime ready");

    const hello = '@caps()\ndef main() {\n  println("Hello from Garnet!")\n  0\n}\n';
    await page.locator("#source-editor").fill(hello);
    await page.locator("#run-source").click();
    const run = await page.evaluate(() => window.__garnetPlayground.state.lastRun);
    assertEqual(run.schema, "garnet.wasm.run/1", "run schema");
    assertEqual(run.exit_class, "ok", "run exit class");
    assertEqual(run.stdout, "Hello from Garnet!\n", "run stdout");

    await page.locator("#check-source").click();
    const check = await page.evaluate(() => window.__garnetPlayground.state.lastCheck);
    assertEqual(check.schema, "garnet.wasm.check/1", "check schema");
    assertEqual(check.ok, true, "check verdict");

    const baseline = "@caps()\ndef main() { 0 }\n";
    const expanded = "@caps(fs)\ndef main() { 0 }\n";
    await page.locator("#baseline-editor").fill(baseline);
    await page.locator("#source-editor").fill(expanded);
    await page.locator("#diff-caps").click();
    const diff = await page.evaluate(() => ({
      adapter_result: window.__garnetPlayground.state.lastDiff,
      machine_verdict: window.__garnetPlayground.state.lastMachineVerdict,
      human_verdict: document.getElementById("diff-verdict").textContent,
    }));
    assertEqual(diff.adapter_result.schema, "garnet.wasm.diff-caps/1", "diff schema");
    assertEqual(diff.adapter_result.ok, true, "diff adapter");
    assertEqual(diff.adapter_result.authority_expanded, true, "authority expansion");
    assertEqual(diff.adapter_result.aggregate_added, ["fs"], "aggregate added");
    assertEqual(diff.adapter_result.scope, DIFF_SCOPE, "diff scope");
    assertEqual(diff.human_verdict, "Authority expanded", "human diff verdict");
    assertEqual(diff.machine_verdict.schema, "garnet.playground.diff-caps-verdict/1", "machine schema");
    assertEqual(diff.machine_verdict.verdict, "expanded", "machine verdict");
    assertEqual(diff.machine_verdict.authority_expanded, true, "machine expansion");

    const denied = '@caps(proc)\ndef main() {\n  proc::run("echo hi")\n  0\n}\n';
    await page.locator("#source-editor").fill(denied);
    await page.locator("#run-source").click();
    const denial = await page.evaluate(() => ({
      run: window.__garnetPlayground.state.lastRun,
      ui_state: document.getElementById("run-state").textContent,
    }));
    assertEqual(denial.run.schema, "garnet.wasm.run/1", "denial schema");
    assertEqual(denial.run.exit_class, "runtime_error", "denial exit class");
    assertEqual(denial.run.stdout, "", "denial stdout");
    assertEqual(denial.ui_state, "Denied", "denial UI state");
    if (!denial.run.diagnostic?.toLowerCase().includes("proc")) {
      throw new Error(`denial diagnostic does not name proc: ${denial.run.diagnostic}`);
    }

    const desktop = await page.evaluate(() => ({
      horizontal_overflow: document.documentElement.scrollWidth > window.innerWidth,
      runtime_state: document.getElementById("runtime-status").dataset.state,
    }));
    mkdirSync(dirname(args.screenshot), { recursive: true });
    await page.screenshot({ path: args.screenshot, fullPage: true });
    await page.setViewportSize({ width: 390, height: 844 });
    const mobile = await page.evaluate(() => ({
      horizontal_overflow: document.documentElement.scrollWidth > window.innerWidth,
    }));
    const durationMs = Math.round(performance.now() - started);
    const screenshotRaw = readFileSync(args.screenshot);
    const requiredRequests = [
      "docs/playground.html",
      "docs/playground/live.js",
      "docs/playground/pkg/garnet_wasm.js",
      "docs/playground/pkg/garnet_wasm_bg.wasm",
    ];
    for (const required of requiredRequests) {
      if (!trackedServer.requested.has(required)) {
        throw new Error(`browser did not request committed runtime input: ${required}`);
      }
    }

    const passed =
      durationMs < 30_000 &&
      externalRequests.size === 0 &&
      trackedServer.untrackedRequests.size === 0 &&
      consoleErrors.length === 0 &&
      pageErrors.length === 0 &&
      desktop.horizontal_overflow === false &&
      mobile.horizontal_overflow === false;
    const proof = {
      schema: "garnet.w-play.browser-proof/1",
      captured_at: new Date().toISOString(),
      verdict: passed ? "pass" : "fail",
      duration_ms: durationMs,
      execution: {
        engine: "playwright-browser-page",
        browser: await browser.version(),
        node_global_present: browserBoundary.node_global_present,
        runtime_ready: browserBoundary.runtime_ready,
        service_workers: "blocked",
      },
      git: {
        tested_commit: runGit(["rev-parse", "HEAD"]),
        tested_tree: runGit(["rev-parse", "HEAD^{tree}"]),
        runtime_inputs_clean: true,
      },
      host: {
        os: platform(),
        arch: arch(),
        node: process.version,
      },
      package: {
        schema: packageProvenance.schema,
        source_tree_sha256: packageProvenance.source.source_tree_sha256,
        artifacts: packageProvenance.artifacts,
      },
      network: {
        external_requests: [...externalRequests].sort(),
        requested_committed_files: [...trackedServer.requested].sort(),
        untracked_requests: [...trackedServer.untrackedRequests].sort(),
      },
      journeys: {
        run,
        check,
        diff,
        denial,
      },
      visual: {
        screenshot: relative(ROOT, args.screenshot).split(sep).join("/"),
        screenshot_sha256: sha256(screenshotRaw),
        desktop,
        mobile,
      },
      diagnostics: {
        console_errors: consoleErrors,
        page_errors: pageErrors,
      },
      claim_boundary: "Real clean-browser execution from committed package bytes; no OS host authority; capability diff is declared-surface-only.",
    };
    mkdirSync(dirname(args.proof), { recursive: true });
    writeFileSync(args.proof, `${JSON.stringify(proof, null, 2)}\n`, "utf-8");
    if (!passed) throw new Error(`browser proof failed: ${args.proof}`);
    console.log(
      `Garnet playground browser proof: PASS (${durationMs} ms, ${trackedServer.requested.size} committed requests)`,
    );
  } finally {
    await context.close();
    await browser.close();
    await closeServer(trackedServer.server);
  }
}

main().catch((error) => {
  console.error(error.stack || error.message || String(error));
  process.exit(1);
});
