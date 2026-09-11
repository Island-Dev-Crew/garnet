#!/usr/bin/env node
// Regression journeys for the playground's failure and preset behaviour.
//
// The W-PLAY browser proof (smoke_garnet_playground_browser.mjs) covers the
// working runtime. This script covers what happens around it: a module that
// never loads must say so, presets must survive a runtime failure, and the
// opening Hello preset must not overwrite a visitor's edit, including an edit
// made before the adapter module has run and a changed source whose input
// event has not been dispatched yet.
//
// It drives the committed docs/ tree in headless Chrome through the Studio
// npm ci tree's @playwright/test, the same trust path as the browser proof.
// Faults are injected with page.route. A journey that depends on timing holds
// the intercepted request open until its edits are done, checks that the
// request really was intercepted, and only then releases it, so no journey
// can pass because a response happened to arrive early. Every journey runs to
// completion, releases what it holds and closes its page even when it throws,
// and records its own failures; an unexpected error or a timed-out wait is a
// failure of that journey, not a crash of the script.
//
// Usage: node scripts/smoke_garnet_playground_failure_modes.mjs [--chrome path]
import { readFileSync, existsSync } from "node:fs";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { platform } from "node:os";
import { extname, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = resolve(fileURLToPath(new URL("..", import.meta.url)));
const DOCS = resolve(ROOT, "docs");
const STUDIO_REQUIRE = createRequire(pathToFileURL(resolve(ROOT, "apps/garnet-studio/package.json")));
const WAIT_MS = 15_000;
const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json",
  ".wasm": "application/wasm",
  ".png": "image/png",
  ".css": "text/css",
};

function defaultChrome() {
  if (process.env.CHROME_BIN) return process.env.CHROME_BIN;
  if (platform() === "win32") return "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
  if (platform() === "linux") return "/usr/bin/google-chrome";
  return "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
}

function parseArgs(argv) {
  const args = { chrome: defaultChrome() };
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] === "--chrome") args.chrome = resolve(argv[++index]);
    else throw new Error(`unknown argument: ${argv[index]}`);
  }
  return args;
}

function serveDocs() {
  const server = createServer((request, response) => {
    const pathname = decodeURIComponent(new URL(request.url, "http://127.0.0.1").pathname);
    const file = resolve(DOCS, `.${pathname === "/" ? "/playground.html" : pathname}`);
    if (!file.startsWith(DOCS + sep) || !existsSync(file)) {
      response.writeHead(404);
      response.end("not found");
      return;
    }
    response.writeHead(200, { "content-type": TYPES[extname(file)] || "application/octet-stream" });
    response.end(readFileSync(file));
  });
  return new Promise((resolveServer) => {
    server.listen(0, "127.0.0.1", () => resolveServer(server));
  });
}

const failures = [];
const failedJourneys = new Set();
function check(journey, condition, detail) {
  if (!condition) {
    failures.push(`${journey}: ${detail}`);
    failedJourneys.add(journey);
  }
  return Boolean(condition);
}

// Run one journey; an exception is that journey's failure, not the script's.
async function runJourney(journey, body) {
  try {
    await body(journey);
  } catch (error) {
    check(journey, false, `unexpected error: ${String(error?.message ?? error).split("\n")[0]}`);
  }
}

// Wait for a page condition; a timeout is recorded as this journey's failure.
async function waitUntil(page, journey, detail, predicate) {
  try {
    await page.waitForFunction(predicate, null, { timeout: WAIT_MS });
    return true;
  } catch {
    return check(journey, false, detail);
  }
}

// Wait for a condition on this side of the browser.
async function waitHere(journey, detail, condition) {
  const deadline = Date.now() + WAIT_MS;
  while (Date.now() < deadline) {
    if (condition()) return true;
    await new Promise((resolveTick) => setTimeout(resolveTick, 25));
  }
  return check(journey, false, detail);
}

// Intercept one request and hold it open until release() is called.
async function hold(page, pattern) {
  let release;
  const released = new Promise((resolveRelease) => { release = resolveRelease; });
  let intercepted = false;
  await page.route(pattern, async (route) => {
    intercepted = true;
    await released;
    await route.continue();
  });
  return { release: () => release(), intercepted: () => intercepted };
}

const runtimeSettled = () => document.getElementById("runtime-status").dataset.state !== "loading";
const anyPresetEntry = () => document.getElementById("example-picker").options.length > 1;
const helloListed = () => [...document.getElementById("example-picker").options].some((option) => option.value === "hello");

async function pageState(page) {
  return page.evaluate(() => ({
    runtime: document.getElementById("runtime-status").textContent,
    runtimeState: document.getElementById("runtime-status").dataset.state,
    runState: document.getElementById("run-state").textContent,
    runResult: document.getElementById("run-result").textContent,
    picker: document.getElementById("example-picker").value,
    options: [...document.querySelectorAll("#example-picker option")].map((option) => ({
      value: option.value,
      text: option.textContent,
      disabled: option.disabled,
    })),
    source: document.getElementById("source-editor").value,
    defaultSource: document.getElementById("source-editor").defaultValue,
    ready: window.__garnetPlayground?.state?.ready === true,
  }));
}

// Open a page, run the journey body, and always release and close.
async function withPage(browser, setup, body) {
  const page = await browser.newPage();
  const errors = [];
  page.on("pageerror", (error) => errors.push(`pageerror: ${error.message}`));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  const held = [];
  try {
    await setup(page, held);
    await body(page, errors);
  } finally {
    for (const request of held) request.release();
    await page.close();
  }
}

async function open(page, baseUrl) {
  await page.goto(`${baseUrl}/playground.html`, { waitUntil: "domcontentloaded" });
}

async function holdPresets(page, held) {
  const presets = await hold(page, "**/playground/examples.json");
  held.push(presets);
  return presets;
}

const args = parseArgs(process.argv.slice(2));
if (!existsSync(args.chrome)) throw new Error(`Chrome executable not found: ${args.chrome}`);
const { chromium } = STUDIO_REQUIRE("@playwright/test");
const examples = JSON.parse(readFileSync(resolve(DOCS, "playground/examples.json"), "utf-8")).examples;
const hello = examples.find((example) => example.name === "hello");
if (!hello) throw new Error("examples.json has no hello preset");
const JOURNEYS = 10;

const server = await serveDocs();
const baseUrl = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch({ executablePath: args.chrome, headless: true });
try {
  await runJourney("normal load", (journey) => withPage(browser, async () => {}, async (page, errors) => {
    await open(page, baseUrl);
    if (!(await waitUntil(page, journey, "the runtime never settled", runtimeSettled))) return;
    if (!(await waitUntil(page, journey, "the Hello preset never appeared", helloListed))) return;
    const state = await pageState(page);
    check(journey, state.ready && state.runtimeState === "ready", `runtime not ready (${state.runtime})`);
    check(journey, state.picker === "hello", `picker is "${state.picker}", expected "hello"`);
    check(journey, state.source === hello.source, "editor does not hold the Hello preset source");
    check(journey, errors.length === 0, `console or page errors: ${JSON.stringify(errors)}`);
  }));

  await runJourney("editing a preset", (journey) => withPage(browser, async () => {}, async (page) => {
    await open(page, baseUrl);
    if (!(await waitUntil(page, journey, "the Hello preset never appeared", helloListed))) return;
    await page.locator("#example-picker").selectOption("hello");
    await page.locator("#source-editor").fill(`${hello.source}\n# edited\n`);
    const edited = await pageState(page);
    check(journey, edited.picker === "", `picker is "${edited.picker}" after an edit, expected Custom source`);
  }));

  for (const [journey, fulfil] of [
    ["module served as text/plain", (route) => route.fulfill({
      status: 200, contentType: "text/plain", body: readFileSync(resolve(DOCS, "playground/live.js")),
    })],
    ["module 404", (route) => route.fulfill({ status: 404, contentType: "text/plain", body: "not found" })],
  ]) {
    await runJourney(journey, () => withPage(browser, (page) => page.route("**/playground/live.js", fulfil), async (page) => {
      await open(page, baseUrl);
      if (!(await waitUntil(page, journey, 'the status stayed "Loading runtime"', runtimeSettled))) return;
      const state = await pageState(page);
      check(journey, state.runtime === "Runtime failed" && state.runtimeState === "error", `status is "${state.runtime}"/${state.runtimeState}`);
      check(journey, state.runState === "Runtime unavailable", `run state is "${state.runState}"`);
      check(journey, state.runResult.includes("did not load") && state.runResult.includes("text/javascript"), "run result does not name the cause");
    }));
  }

  await runJourney("presets 404", (journey) => withPage(browser,
    (page) => page.route("**/playground/examples.json", (route) => route.fulfill({ status: 404, body: "not found" })),
    async (page) => {
      await open(page, baseUrl);
      if (!(await waitUntil(page, journey, "the runtime never settled", runtimeSettled))) return;
      if (!(await waitUntil(page, journey, "the picker never listed an unavailable entry", anyPresetEntry))) return;
      const state = await pageState(page);
      check(journey, state.ready, `runtime not ready (${state.runtime})`);
      check(journey, state.options.some((option) => option.text === "Examples unavailable" && option.disabled), "no disabled Examples unavailable entry");
      check(journey, state.picker === "", `picker is "${state.picker}"`);
      check(journey, state.source === state.defaultSource, "editor source changed");
    }));

  await runJourney("runtime fails, presets still load", (journey) => withPage(browser,
    (page) => page.route("**/playground/pkg/garnet_wasm_bg.wasm", (route) => route.fulfill({ status: 404, body: "not found" })),
    async (page) => {
      await open(page, baseUrl);
      if (!(await waitUntil(page, journey, "the runtime never settled", runtimeSettled))) return;
      if (!(await waitUntil(page, journey, "the Hello preset never appeared", helloListed))) return;
      const state = await pageState(page);
      check(journey, !state.ready && state.runtimeState === "error", `status is "${state.runtime}"/${state.runtimeState}`);
      check(journey, !state.options.some((option) => option.text === "Examples unavailable"), "presets reported unavailable");
      check(journey, state.picker === "hello" && state.source === hello.source, "the Hello preset did not open");
    }));

  await runJourney("edit before presets arrive", (journey) => {
    let presets;
    return withPage(browser, async (page, held) => { presets = await holdPresets(page, held); }, async (page) => {
      await open(page, baseUrl);
      if (!(await waitHere(journey, "the presets request was never made", presets.intercepted))) return;
      const typed = "@caps()\ndef main() { 7 }\n";
      await page.locator("#source-editor").fill(typed);
      presets.release();
      if (!(await waitUntil(page, journey, "the Hello preset never arrived", helloListed))) return;
      const state = await pageState(page);
      check(journey, state.source === typed, "the visitor's edit was replaced");
      check(journey, state.picker === "", `picker is "${state.picker}"`);
    });
  });

  await runJourney("edit then restore before presets arrive", (journey) => {
    let presets;
    return withPage(browser, async (page, held) => { presets = await holdPresets(page, held); }, async (page) => {
      await open(page, baseUrl);
      if (!(await waitHere(journey, "the presets request was never made", presets.intercepted))) return;
      const original = await page.locator("#source-editor").inputValue();
      await page.locator("#source-editor").fill(`${original}# scratch\n`);
      await page.locator("#source-editor").fill(original);
      presets.release();
      if (!(await waitUntil(page, journey, "the Hello preset never arrived", helloListed))) return;
      const state = await pageState(page);
      check(journey, state.source === original, "the restored source was replaced by the Hello preset");
      check(journey, state.picker === "", `picker is "${state.picker}"`);
    });
  });

  await runJourney("source changed with its input event still pending", (journey) => {
    let presets;
    return withPage(browser, async (page, held) => { presets = await holdPresets(page, held); }, async (page) => {
      await open(page, baseUrl);
      if (!(await waitHere(journey, "the presets request was never made", presets.intercepted))) return;
      // Assign the value without dispatching input or beforeinput. This models
      // the window the HTML standard allows, in which a browser delays a
      // textarea's input event until the visitor pauses.
      const typed = "@caps()\ndef main() { 13 }\n";
      await page.locator("#source-editor").evaluate((editor, value) => { editor.value = value; }, typed);
      presets.release();
      if (!(await waitUntil(page, journey, "the Hello preset never arrived", helloListed))) return;
      const state = await pageState(page);
      check(journey, state.source === typed, "a changed source whose input event had not fired was replaced");
      check(journey, state.picker === "", `picker is "${state.picker}"`);
    });
  });

  await runJourney("edit before the adapter module runs", (journey) => {
    let adapter;
    return withPage(browser, async (page, held) => {
      adapter = await hold(page, "**/playground/live.js");
      held.push(adapter);
    }, async (page) => {
      // A held module script also holds DOMContentLoaded (module scripts are
      // deferred), so this journey waits only for the navigation to commit
      // and the editor to exist.
      await page.goto(`${baseUrl}/playground.html`, { waitUntil: "commit" });
      await page.waitForSelector("#source-editor");
      if (!(await waitHere(journey, "the adapter request was never made", adapter.intercepted))) return;
      const typed = "@caps()\ndef main() { 11 }\n";
      await page.locator("#source-editor").fill(typed);
      adapter.release();
      if (!(await waitUntil(page, journey, "the Hello preset never arrived", helloListed))) return;
      const state = await pageState(page);
      check(journey, state.source === typed, "an edit made before the adapter ran was replaced");
      check(journey, state.picker === "", `picker is "${state.picker}"`);
    });
  });
} finally {
  await browser.close();
  server.close();
}

if (failures.length) {
  console.error(`Garnet playground failure-mode smoke: FAIL (${failedJourneys.size} of ${JOURNEYS} journeys)`);
  for (const failure of failures) console.error(`  - ${failure}`);
  process.exit(1);
}
console.log(`Garnet playground failure-mode smoke: PASS (${JOURNEYS} journeys)`);
