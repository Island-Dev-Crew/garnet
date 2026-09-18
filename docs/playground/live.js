import init, { check_source, diff_caps_source, run_source } from "./pkg/garnet_wasm.js";

const RUN_SCHEMA = "garnet.wasm.run/1";
const CHECK_SCHEMA = "garnet.wasm.check/1";
const DIFF_SCHEMA = "garnet.wasm.diff-caps/1";
const MACHINE_SCHEMA = "garnet.playground.diff-caps-verdict/1";
const SHARE_LIMIT = 16000;
const element = (id) => {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing playground element: ${id}`);
  return found;
};
const ui = Object.fromEntries([
  "runtime-status", "source-editor", "baseline-editor", "example-picker", "run-source", "check-source", "diff-caps",
  "run-state", "run-result", "run-json", "check-state", "check-result", "check-json", "diff-verdict", "diff-result", "diff-json", "machine-verdict",
  "lane-card", "authority-rows", "authority-total", "share-source", "share-link", "share-panel", "share-status", "preset-description", "preset-alternate",
  "toggle-baseline", "baseline-panel", "evidence-badge",
].map(id => [id, element(id)]));
const source = ui["source-editor"];
const baseline = ui["baseline-editor"];
const publicState = {ready:false, lastRun:null, lastCheck:null, lastDiff:null, lastMachineVerdict:null};
const examples = new Map();
function parseAdapterJson(raw, schema) {
  const value = JSON.parse(raw);
  if (!value || typeof value !== "object" || value.schema !== schema) throw new Error(`adapter schema mismatch: expected ${schema}`);
  return value;
}
const check = text => parseAdapterJson(check_source(text), CHECK_SCHEMA);
const diff = (oldText, newText) => parseAdapterJson(diff_caps_source(oldText, newText), DIFF_SCHEMA);
const renderJson = (target, value) => { target.textContent = JSON.stringify(value, null, 2); };
function setVerdict(target, text, state = "") { target.textContent = text; target.dataset.state = state; }
export function machineVerdictFromDiff(result) {
  return {schema:MACHINE_SCHEMA, verdict:result.ok ? result.authority_expanded ? "expanded" : "not_expanded" : "indeterminate", authority_expanded:result.authority_expanded, aggregate_added:result.aggregate_added, aggregate_removed:result.aggregate_removed, wildcard_introduced:result.wildcard_introduced, scope:result.scope};
}
function renderAuthority(result) {
  const surface = result.new_surface;
  ui["authority-rows"].replaceChildren();
  if (!surface) { ui["authority-total"].textContent = "Declarations unavailable: source did not parse."; return; }
  for (const fn of surface.per_function) {
    const row = document.createElement("tr");
    for (const text of [fn.name, fn.caps.join(", ") || "none declared"]) {
      const cell = document.createElement("td"); cell.textContent = text; row.appendChild(cell);
    }
    ui["authority-rows"].appendChild(row);
  }
  ui["authority-total"].textContent = `Declared total: ${surface.aggregate.join(", ") || "none"}. ${surface.per_function.length} annotated functions; unannotated functions are omitted.`;
}
function runCurrentSource() {
  const result = parseAdapterJson(run_source(source.value), RUN_SCHEMA);
  publicState.lastRun = result;
  ui["run-result"].dataset.exitClass = result.exit_class;
  ui["run-result"].textContent = [result.stdout?.replace(/\n$/, ""), result.diagnostic, `Exit: ${result.exit_class}`].filter(Boolean).join("\n");
  setVerdict(ui["run-state"], result.exit_class === "ok" ? "Completed" : "Denied", result.exit_class === "ok" ? "ok" : "denied");
  renderJson(ui["run-json"], result);
  return result;
}
function checkCurrentSource() {
  const result = check(source.value);
  publicState.lastCheck = result;
  setVerdict(ui["check-state"], result.ok ? "Check passed" : "Check failed", result.ok ? "ok" : "denied");
  ui["check-result"].textContent = result.diagnostics.map(d => `${d.severity} · ${d.code}\n${d.message}`).join("\n\n") || "No checker diagnostics.";
  renderJson(ui["check-json"], result);
  renderAuthority(diff(source.value, source.value));
  return result;
}
function diffCurrentSource() {
  const result = diff(baseline.value, source.value);
  const machine = machineVerdictFromDiff(result);
  publicState.lastDiff = result; publicState.lastMachineVerdict = machine;
  const human = machine.verdict === "expanded" ? "Authority expanded" : machine.verdict === "not_expanded" ? "No authority expansion" : "Diff unavailable";
  setVerdict(ui["diff-verdict"], human, machine.verdict === "not_expanded" ? "ok" : "denied");
  renderJson(ui["machine-verdict"], machine); renderJson(ui["diff-json"], result);
  ui["diff-result"].textContent = result.ok ? `Added: ${result.aggregate_added.join(", ") || "none"}\nRemoved: ${result.aggregate_removed.join(", ") || "none"}\n${result.functions_caps_expanded.map(fn => `${fn.name} gained ${fn.gained.join(", ")}`).join("\n")}` : `Cannot compare: ${result.parse_error?.diagnostic?.message || "invalid source"}`;
  renderAuthority(result);
  // Diff only parses; independently check BOTH revisions before illustrating
  // any lane. The adapter has no complete-coverage assertion: unchanged is
  // deliberately unknown, never permission or a green policy verdict.
  const checks = {baseline:check(baseline.value), current:check(source.value)};
  if (!result.ok || !checks.baseline.ok || !checks.current.ok) {
    setVerdict(ui["lane-card"], "Blocked: a source failed parsing or checking. Resolve its diagnostics before policy review.", "blocked");
  } else if (result.authority_expanded) {
    const gained = result.functions_caps_expanded.map(fn => `${fn.name} gained ${fn.gained.join(", ")}`).join("; ");
    setVerdict(ui["lane-card"], `${gained || "Declared capabilities expanded"}. A human reviews this change.`, "review");
  } else {
    setVerdict(ui["lane-card"], "No new declared capabilities detected. Eligible for further policy checks; this is not merge approval. Complete checker coverage is unknown; tests, path policy and bound base/head evidence are still required.", "unknown");
  }
  return {adapterResult:result, humanVerdict:human, machineVerdict:machine};
}
function refreshLines(editor) {
  const gutter = element(editor === source ? "source-lines" : "baseline-lines");
  gutter.textContent = Array.from({length:editor.value.split("\n").length}, (_, i) => i + 1).join("\n");
  gutter.scrollTop = editor.scrollTop;
}
function invalidate() {
  publicState.lastRun = null; publicState.lastCheck = null; publicState.lastDiff = null; publicState.lastMachineVerdict = null;
  setVerdict(ui["run-state"], "Not run"); setVerdict(ui["check-state"], "Not checked"); setVerdict(ui["diff-verdict"], "Not compared");
  for (const id of ["run-result","run-json","check-result","check-json","diff-result","diff-json","machine-verdict"]) ui[id].textContent = "";
  ui["authority-rows"].replaceChildren(); ui["authority-total"].textContent = "Check or compare the edited source to inspect its declarations.";
  setVerdict(ui["lane-card"], "Source changed. Compare again before interpreting declarations.", "unknown");
  refreshLines(source); refreshLines(baseline);
}
function selectExample(example, alternate = false) {
  source.value = alternate ? example.alternate_source : example.source;
  if (example.baseline) baseline.value = example.baseline;
  ui["preset-description"].textContent = example.description || "";
  ui["preset-alternate"].hidden = !example.alternate_source || alternate;
  invalidate();
}
async function loadExamples() {
  const response = await fetch("./playground/examples.json", {cache:"no-store"});
  if (!response.ok) throw new Error(`examples request failed: ${response.status}`);
  const payload = await response.json();
  for (const example of payload.examples || []) {
    examples.set(example.name, example);
    const option = document.createElement("option"); option.value = example.name; option.textContent = example.title || example.name; option.dataset.source = example.source; ui["example-picker"].appendChild(option);
  }
  if (!location.hash && source.dataset.edited !== "true" && source.value === source.defaultValue && examples.has("hello")) {
    ui["example-picker"].value = "hello"; selectExample(examples.get("hello"));
  }
}
function restoreShare() {
  if (!location.hash.startsWith("#garnet=")) return;
  try {
    const encoded = location.hash.slice(8);
    if (encoded.length > SHARE_LIMIT || !/^[A-Za-z0-9_-]+$/.test(encoded)) throw new Error("invalid fragment");
    const binary = atob(encoded.replace(/-/g,"+").replace(/_/g,"/"));
    const value = JSON.parse(new TextDecoder("utf-8", {fatal:true}).decode(Uint8Array.from(binary, ch => ch.charCodeAt(0))));
    if (!value || value.v !== 1 || typeof value.source !== "string" || typeof value.baseline !== "string" || Object.keys(value).sort().join(",") !== "baseline,source,v") throw new Error("unsupported share format");
    source.value = value.source; baseline.value = value.baseline; source.dataset.edited = "true";
    setVerdict(ui["share-status"], "Shared source restored. Nothing has been run.");
  } catch {
    setVerdict(ui["share-status"], "Share link rejected: invalid, unsupported or larger than 16,000 fragment characters. Editors were not restored.", "denied");
  }
}
async function copyShare() {
  try {
    if (source.value.length + baseline.value.length > SHARE_LIMIT) throw new Error("too large");
    const bytes = new TextEncoder().encode(JSON.stringify({v:1,source:source.value,baseline:baseline.value}));
    const encoded = btoa(Array.from(bytes, byte => String.fromCharCode(byte)).join("")).replace(/\+/g,"-").replace(/\//g,"_").replace(/=+$/,"");
    if (encoded.length > SHARE_LIMIT) throw new Error("too large");
    const url = new URL(location.href); url.search = ""; url.hash = `garnet=${encoded}`;
    ui["share-link"].value = url.href; ui["share-panel"].hidden = false;
    try { await navigator.clipboard.writeText(url.href); setVerdict(ui["share-status"], "Link copied. Shared source never runs automatically."); }
    catch { ui["share-link"].focus(); ui["share-link"].select(); setVerdict(ui["share-status"], "Copy this selected link. Clipboard access was unavailable."); }
  } catch { ui["share-panel"].hidden = true; ui["share-link"].value = ""; setVerdict(ui["share-status"], "Source is too large for a share link (16,000 fragment characters maximum).", "denied"); }
}
async function loadEvidence() {
  const response = await fetch("./playground/pkg/provenance.json");
  if (!response.ok) throw new Error("provenance unavailable");
  const p = await response.json();
  const wasm = p.artifacts?.["garnet_wasm_bg.wasm"]?.sha256;
  const tree = p.source?.source_tree_sha256;
  if (p.schema !== "garnet.playground.wasm-package/1" || !/^[a-f0-9]{64}$/.test(wasm) || !/^[a-f0-9]{64}$/.test(tree)) throw new Error("invalid provenance");
  setVerdict(ui["evidence-badge"], `Recorded WASM SHA-256: ${wasm}\nSource tree SHA-256: ${tree}\nRebuild and compare: python3 scripts/build_playground_wasm.py --verify-reproducible\nThe provenance records content digests, not a build commit.`, "ready");
}
ui["example-picker"].addEventListener("change", () => { const example = examples.get(ui["example-picker"].value); if (example) selectExample(example); });
ui["preset-alternate"].addEventListener("click", () => { const example = examples.get(ui["example-picker"].value); if (example?.alternate_source) selectExample(example, true); });
for (const editor of [source, baseline]) {
  let escapeTab = false;
  editor.addEventListener("input", () => { if (editor === source) { ui["example-picker"].value = ""; ui["preset-alternate"].hidden = true; } invalidate(); });
  editor.addEventListener("scroll", () => refreshLines(editor));
  editor.addEventListener("keydown", event => {
    if (event.key === "Escape") { escapeTab = true; return; }
    if (event.key === "Tab" && !event.shiftKey && !escapeTab) { event.preventDefault(); editor.setRangeText("  ", editor.selectionStart, editor.selectionEnd, "end"); editor.dispatchEvent(new Event("input", {bubbles:true})); }
    escapeTab = false;
    if (event.key === "Enter" && (event.metaKey || event.ctrlKey) && publicState.ready) { event.preventDefault(); event.shiftKey ? diffCurrentSource() : checkCurrentSource(); }
  });
}
ui["run-source"].addEventListener("click", runCurrentSource);
ui["check-source"].addEventListener("click", checkCurrentSource);
ui["diff-caps"].addEventListener("click", diffCurrentSource);
ui["share-source"].addEventListener("click", copyShare);
function showBaseline(show) {
  ui["baseline-panel"].hidden = !show; ui["toggle-baseline"].setAttribute("aria-expanded", String(show));
  ui["toggle-baseline"].textContent = show ? "Hide baseline" : "Show baseline";
  document.querySelector(".editors").classList.toggle("baseline-hidden", !show);
}
ui["toggle-baseline"].addEventListener("click", () => showBaseline(ui["baseline-panel"].hidden));
if (window.self !== window.top || new URLSearchParams(location.search).get("embed") === "1") { document.body.classList.add("embedded"); showBaseline(false); }
restoreShare(); refreshLines(source); refreshLines(baseline);
window.__garnetPlayground = {state:publicState, run:runCurrentSource, check:checkCurrentSource, diff:diffCurrentSource};
loadExamples().catch(() => { const option = document.createElement("option"); option.disabled = true; option.textContent = "Examples unavailable"; ui["example-picker"].appendChild(option); });
loadEvidence().catch(() => setVerdict(ui["evidence-badge"], "Package provenance unavailable. No digest is claimed.", "denied"));
try {
  await init(); publicState.ready = true; setVerdict(ui["runtime-status"], "Runtime ready", "ready");
  for (const button of document.querySelectorAll("[data-action]")) button.disabled = false;
} catch (error) {
  publicState.ready = false; setVerdict(ui["runtime-status"], "Runtime failed", "error");
  setVerdict(ui["run-state"], "Runtime unavailable", "denied"); ui["run-result"].textContent = error instanceof Error ? error.message : String(error);
}
