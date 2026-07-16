import init, {
  check_source,
  diff_caps_source,
  run_source,
} from "./pkg/garnet_wasm.js";

const RUN_SCHEMA = "garnet.wasm.run/1";
const CHECK_SCHEMA = "garnet.wasm.check/1";
const DIFF_SCHEMA = "garnet.wasm.diff-caps/1";
const MACHINE_SCHEMA = "garnet.playground.diff-caps-verdict/1";

const element = (id) => {
  const found = document.getElementById(id);
  if (!found) throw new Error(`missing playground element: ${id}`);
  return found;
};

const ui = {
  runtime: element("runtime-status"),
  source: element("source-editor"),
  baseline: element("baseline-editor"),
  example: element("example-picker"),
  runButton: element("run-source"),
  checkButton: element("check-source"),
  diffButton: element("diff-caps"),
  runState: element("run-state"),
  runResult: element("run-result"),
  checkState: element("check-state"),
  checkResult: element("check-result"),
  diffVerdict: element("diff-verdict"),
  machineVerdict: element("machine-verdict"),
};

const publicState = {
  ready: false,
  lastRun: null,
  lastCheck: null,
  lastDiff: null,
  lastMachineVerdict: null,
};

function parseAdapterJson(raw, schema) {
  const value = JSON.parse(raw);
  if (!value || typeof value !== "object" || value.schema !== schema) {
    throw new Error(`adapter schema mismatch: expected ${schema}`);
  }
  return value;
}

function renderJson(target, value) {
  target.textContent = JSON.stringify(value, null, 2);
}

function setVerdict(target, text, state = "") {
  target.textContent = text;
  target.dataset.state = state;
}

export function machineVerdictFromDiff(result) {
  const verdict = result.ok
    ? result.authority_expanded
      ? "expanded"
      : "not_expanded"
    : "indeterminate";
  return {
    schema: MACHINE_SCHEMA,
    verdict,
    authority_expanded: result.authority_expanded,
    aggregate_added: result.aggregate_added,
    aggregate_removed: result.aggregate_removed,
    wildcard_introduced: result.wildcard_introduced,
    scope: result.scope,
  };
}

function runCurrentSource() {
  const result = parseAdapterJson(run_source(ui.source.value), RUN_SCHEMA);
  publicState.lastRun = result;
  ui.runResult.dataset.exitClass = result.exit_class;
  const lines = [];
  if (result.stdout) lines.push(result.stdout.replace(/\n$/, ""));
  if (result.diagnostic) lines.push(result.diagnostic);
  ui.runResult.textContent = lines.join("\n") || "(no output)";
  if (result.exit_class === "ok") {
    setVerdict(ui.runState, "Completed", "ok");
  } else {
    setVerdict(ui.runState, "Denied", "denied");
  }
  return result;
}

function checkCurrentSource() {
  const result = parseAdapterJson(check_source(ui.source.value), CHECK_SCHEMA);
  publicState.lastCheck = result;
  setVerdict(
    ui.checkState,
    result.ok ? "Check passed" : "Check failed",
    result.ok ? "ok" : "denied",
  );
  renderJson(ui.checkResult, result);
  return result;
}

function diffCurrentSource() {
  const result = parseAdapterJson(
    diff_caps_source(ui.baseline.value, ui.source.value),
    DIFF_SCHEMA,
  );
  const machine = machineVerdictFromDiff(result);
  publicState.lastDiff = result;
  publicState.lastMachineVerdict = machine;
  const human = machine.verdict === "expanded"
    ? "Authority expanded"
    : machine.verdict === "not_expanded"
      ? "No authority expansion"
      : "Diff unavailable";
  setVerdict(
    ui.diffVerdict,
    human,
    machine.verdict === "not_expanded" ? "ok" : "denied",
  );
  renderJson(ui.machineVerdict, machine);
  return { adapterResult: result, humanVerdict: human, machineVerdict: machine };
}

async function loadExamples() {
  const response = await fetch("./playground/examples.json", { cache: "no-store" });
  if (!response.ok) throw new Error(`examples request failed: ${response.status}`);
  const payload = await response.json();
  for (const example of payload.examples || []) {
    const option = document.createElement("option");
    option.value = example.name;
    option.textContent = example.title || example.name;
    option.dataset.source = example.source;
    ui.example.appendChild(option);
  }
}

ui.example.addEventListener("change", () => {
  const option = ui.example.selectedOptions[0];
  if (option?.dataset.source) ui.source.value = option.dataset.source;
});
ui.runButton.addEventListener("click", runCurrentSource);
ui.checkButton.addEventListener("click", checkCurrentSource);
ui.diffButton.addEventListener("click", diffCurrentSource);

window.__garnetPlayground = {
  state: publicState,
  run: runCurrentSource,
  check: checkCurrentSource,
  diff: diffCurrentSource,
};

try {
  await init();
  publicState.ready = true;
  ui.runtime.textContent = "Runtime ready";
  ui.runtime.dataset.state = "ready";
  for (const button of document.querySelectorAll("[data-action]")) {
    button.disabled = false;
  }
  await loadExamples();
} catch (error) {
  publicState.ready = false;
  ui.runtime.textContent = "Runtime failed";
  ui.runtime.dataset.state = "error";
  setVerdict(ui.runState, "Runtime unavailable", "denied");
  ui.runResult.textContent = error instanceof Error ? error.message : String(error);
}
