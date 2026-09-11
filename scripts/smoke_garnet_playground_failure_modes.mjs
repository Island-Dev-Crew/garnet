#!/usr/bin/env node
// Regression journeys for the playground's failure and preset behaviour.
//
// The W-PLAY browser proof (smoke_garnet_playground_browser.mjs) covers the
// working runtime. This script covers what happens around it: a module that
// never loads must say so, presets must survive a runtime failure, and the
// opening Hello preset must never overwrite a visitor's edit.
//
// It drives the committed docs/ tree in headless Chrome through the Studio
// npm ci tree's @playwright/test, the same trust path as the browser proof.
// Faults are injected with page.route, so no fixture files are needed. Every
// journey runs to completion and records its own failures; a wait that times
// out is a failure of that journey, not a crash of the script.
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

// Wait for a page condition; a timeout is recorded as this journey's failure.
async function waitUntil(page, journey, detail, predicate) {
  try {
    await page.waitForFunction(predicate, null, { timeout: WAIT_MS });
    return true;
  } catch {
    return check(journey, false, detail);
  }
}

const runtimeSettled = () => document.getElementById("runtime-status").dataset.state !== "loading";
const presetsListed = () => document.getElementById("example-picker").options.length > 1;

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

async function openPage(browser, baseUrl, routes = async () => {}) {
  const page = await browser.newPage();
  const errors = [];
  page.on("pageerror", (error) => errors.push(`pageerror: ${error.message}`));
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  await routes(page);
  await page.goto(`${baseUrl}/playground.html`, { waitUntil: "domcontentloaded" });
  return { page, errors };
}

async function normalLoad(browser, baseUrl, hello) {
  const journey = "normal load";
  const { page, errors } = await openPage(browser, baseUrl);
  if (await waitUntil(page, journey, "the runtime never settled", runtimeSettled) &&
      await waitUntil(page, journey, "the presets never appeared", presetsListed)) {
    const state = await pageState(page);
    check(journey, state.ready && state.runtimeState === "ready", `runtime not ready (${state.runtime})`);
    check(journey, state.picker === "hello", `picker is "${state.picker}", expected "hello"`);
    check(journey, state.source === hello.source, "editor does not hold the Hello preset source");
    check(journey, errors.length === 0, `console or page errors: ${JSON.stringify(errors)}`);

    const editJourney = "editing a preset";
    await page.locator("#example-picker").selectOption("hello");
    await page.locator("#source-editor").fill(`${hello.source}\n# edited\n`);
    const edited = await pageState(page);
    check(editJourney, edited.picker === "", `picker is "${edited.picker}" after an edit, expected Custom source`);
  }
  await page.close();
}

async function moduleFailure(browser, baseUrl, journey, fulfil) {
  const { page } = await openPage(browser, baseUrl, (target) => target.route("**/playground/live.js", fulfil));
  if (await waitUntil(page, journey, 'the status stayed "Loading runtime"', runtimeSettled)) {
    const state = await pageState(page);
    check(journey, state.runtime === "Runtime failed" && state.runtimeState === "error", `status is "${state.runtime}"/${state.runtimeState}`);
    check(journey, state.runState === "Runtime unavailable", `run state is "${state.runState}"`);
    check(journey, state.runResult.includes("did not load") && state.runResult.includes("text/javascript"), "run result does not name the cause");
  }
  await page.close();
}

async function presetsMissing(browser, baseUrl) {
  const journey = "presets 404";
  const { page } = await openPage(browser, baseUrl, (target) =>
    target.route("**/playground/examples.json", (route) => route.fulfill({ status: 404, body: "not found" })));
  if (await waitUntil(page, journey, "the runtime never settled", runtimeSettled) &&
      await waitUntil(page, journey, "the picker never listed an unavailable entry", presetsListed)) {
    const state = await pageState(page);
    check(journey, state.ready, `runtime not ready (${state.runtime})`);
    check(journey, state.options.some((option) => option.text === "Examples unavailable" && option.disabled), "no disabled Examples unavailable entry");
    check(journey, state.picker === "", `picker is "${state.picker}"`);
    check(journey, state.source === state.defaultSource, "editor source changed");
  }
  await page.close();
}

const delayedPresets = (target) => target.route("**/playground/examples.json", async (route) => {
  await new Promise((resolveDelay) => setTimeout(resolveDelay, 1_500));
  await route.continue();
});

async function editBeforePresets(browser, baseUrl) {
  const journey = "edit before presets arrive";
  const { page } = await openPage(browser, baseUrl, delayedPresets);
  const typed = "@caps()\ndef main() { 7 }\n";
  await page.locator("#source-editor").fill(typed);
  if (await waitUntil(page, journey, "the presets never appeared", presetsListed)) {
    const state = await pageState(page);
    check(journey, state.source === typed, "the visitor's edit was replaced");
    check(journey, state.picker === "", `picker is "${state.picker}"`);
  }
  await page.close();
}

async function editThenRestore(browser, baseUrl) {
  const journey = "edit then restore before presets arrive";
  const { page } = await openPage(browser, baseUrl, delayedPresets);
  const original = await page.locator("#source-editor").inputValue();
  await page.locator("#source-editor").fill(`${original}# scratch\n`);
  await page.locator("#source-editor").fill(original);
  if (await waitUntil(page, journey, "the presets never appeared", presetsListed)) {
    const state = await pageState(page);
    check(journey, state.source === original, "the restored source was replaced by the Hello preset");
    check(journey, state.picker === "", `picker is "${state.picker}"`);
  }
  await page.close();
}

const args = parseArgs(process.argv.slice(2));
if (!existsSync(args.chrome)) throw new Error(`Chrome executable not found: ${args.chrome}`);
const { chromium } = STUDIO_REQUIRE("@playwright/test");
const examples = JSON.parse(readFileSync(resolve(DOCS, "playground/examples.json"), "utf-8")).examples;
const hello = examples.find((example) => example.name === "hello");
if (!hello) throw new Error("examples.json has no hello preset");
const JOURNEYS = 7;

const server = await serveDocs();
const baseUrl = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch({ executablePath: args.chrome, headless: true });
try {
  await normalLoad(browser, baseUrl, hello);
  await moduleFailure(browser, baseUrl, "module served as text/plain", (route) => route.fulfill({
    status: 200, contentType: "text/plain", body: readFileSync(resolve(DOCS, "playground/live.js")),
  }));
  await moduleFailure(browser, baseUrl, "module 404", (route) => route.fulfill({
    status: 404, contentType: "text/plain", body: "not found",
  }));
  await presetsMissing(browser, baseUrl);
  await editBeforePresets(browser, baseUrl);
  await editThenRestore(browser, baseUrl);
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
