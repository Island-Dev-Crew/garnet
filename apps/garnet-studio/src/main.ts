import { invoke } from "@tauri-apps/api/core";
import { diffCapsCardHtml, type DiffCapsReport } from "./diff-caps";
import { velocityDiagnosticsHtml, latestOnly, type VelocityCheckReport } from "./velocity";
import { enforcementLegendHtml, type EnforcementLegend } from "./enforcement-legend";
import { agentLoopConsoleHtml, type AgentLoopDossier } from "./agent-loop";

interface CommandResult {
  success: boolean;
  stdout: string;
  stderr: string;
  exit_code: number;
  command: string[];
  evidence_path: string | null;
  timed_out: boolean;
  duration_ms: number;
  truncated: boolean;
}

interface HealthStatus {
  cli_found: boolean;
  cli_path: string;
  cli_version: string;
  repo_found: boolean;
  repo_path: string;
  python_found: boolean;
  python_version: string;
  evidence_dir: string;
  platform: string;
  arch: string;
}

interface BootstrapRequirement {
  id: string;
  label: string;
  found: boolean;
  detected: string;
  action: string;
  command: string;
  evidence_note: string;
}

interface BootstrapPlan {
  ready: boolean;
  ready_count: number;
  total_count: number;
  evidence_dir: string;
  summary: string;
  requirements: BootstrapRequirement[];
  safety_notes: string[];
}

interface EvidenceBundle {
  path: string;
  timestamp: string;
  manifest_path: string;
}

interface AppInfo {
  app_version: string;
  tauri_version: string;
  platform: string;
  arch: string;
  settings_path: string;
}

interface StudioSettings {
  mode: string;
  theme: string;
  command_timeout_secs: number;
  matrix_timeout_secs: number;
}

interface TruthSummary {
  found: boolean;
  path: string;
  version: string | null;
  latest_tag: string | null;
  generated_at_commit: string | null;
  readiness_pct: number | null;
  tracked_slices: string | null;
  primitive_count: number | null;
  workspace_tests_passed: number | null;
  workspace_tests_failed: number | null;
  workspace_tests_measured_at_commit: string | null;
  error: string | null;
}

interface EvidenceListing {
  root: string;
  files: { relative_path: string; size: number }[];
  truncated: boolean;
}

interface EvidenceText {
  path: string;
  content: string;
  size: number;
  truncated: boolean;
}

const DEFAULT_SETTINGS: StudioSettings = {
  mode: "simple",
  theme: "dark",
  command_timeout_secs: 900,
  matrix_timeout_secs: 5400,
};

let currentSettings: StudioSettings = { ...DEFAULT_SETTINGS };
let truthLoaded = false;
let legendLoaded = false;

function getInput(id: string): string {
  const el = document.getElementById(id) as HTMLInputElement | HTMLSelectElement | null;
  return el?.value.trim() ?? "";
}

function setInput(id: string, value: string): void {
  const el = document.getElementById(id) as HTMLInputElement | null;
  if (el) {
    el.value = value;
  }
}

function setText(id: string, value: string): void {
  const el = document.getElementById(id);
  if (el) {
    el.textContent = value;
  }
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

function formatDuration(ms: number): string {
  if (!ms || ms < 0) return "";
  if (ms < 1000) return `${ms}ms`;
  const seconds = ms / 1000;
  if (seconds < 90) return `${seconds.toFixed(1)}s`;
  const total = Math.round(seconds);
  return `${Math.floor(total / 60)}m ${total % 60}s`;
}

async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    try {
      const area = document.createElement("textarea");
      area.value = text;
      area.style.position = "fixed";
      area.style.opacity = "0";
      document.body.appendChild(area);
      area.select();
      const ok = document.execCommand("copy");
      area.remove();
      return ok;
    } catch {
      return false;
    }
  }
}

function wireCopyButtons(scope: HTMLElement): void {
  scope.querySelectorAll<HTMLButtonElement>("button[data-copy]").forEach((button) => {
    button.addEventListener("click", async () => {
      const payload = button.dataset.copy ?? "";
      const ok = await copyText(payload);
      const original = button.textContent ?? "Copy";
      button.textContent = ok ? "Copied" : "Copy failed";
      setTimeout(() => {
        button.textContent = original;
      }, 1500);
    });
  });
}

function renderOutput(targetId: string, result: CommandResult): void {
  const target = document.getElementById(targetId);
  if (!target) return;

  const statusClass = result.success ? "ok" : "fail";
  const stdout = result.stdout.trim();
  const stderr = result.stderr.trim();
  const sections = [
    `command: ${result.command.join(" ")}`,
    result.evidence_path ? `evidence: ${result.evidence_path}` : "",
    stdout ? `stdout:\n${stdout}` : "",
    stderr ? `stderr:\n${stderr}` : "",
  ].filter(Boolean);
  const body = sections.join("\n\n") || "(no output)";
  const headline = result.success
    ? "Passed"
    : result.timed_out
      ? "Timed out"
      : `Failed (${result.exit_code})`;
  const truncatedTitle = result.evidence_path
    ? "Display output was capped; the evidence bundle holds the full streams."
    : "Display output was capped; no evidence bundle was created for this run.";
  const badges = [
    result.duration_ms ? `<span class="badge">${formatDuration(result.duration_ms)}</span>` : "",
    result.truncated
      ? `<span class="badge warn" title="${truncatedTitle}">truncated</span>`
      : "",
  ].join("");
  const collapse = body.split("\n").length > 60;

  target.innerHTML = `
    <article class="result ${statusClass}">
      <header>
        <span>${headline}${badges}</span>
        <span class="result-tools">
          <button class="mini" data-copy="${escapeHtml(body)}">Copy</button>
          <time>${new Date().toLocaleTimeString()}</time>
        </span>
      </header>
      ${
        collapse
          ? `<details open><summary>output (${body.split("\n").length} lines)</summary><pre>${escapeHtml(body)}</pre></details>`
          : `<pre>${escapeHtml(body)}</pre>`
      }
    </article>
  `;
  wireCopyButtons(target as HTMLElement);
}

function renderError(targetId: string, error: unknown): void {
  const target = document.getElementById(targetId);
  if (!target) return;
  target.innerHTML = `
    <article class="result fail">
      <header>
        <span>Error</span>
        <time>${new Date().toLocaleTimeString()}</time>
      </header>
      <pre>${escapeHtml(String(error))}</pre>
    </article>
  `;
}

function renderDiffCaps(targetId: string, report: DiffCapsReport): void {
  const target = document.getElementById(targetId);
  if (!target) return;
  // All rendering lives in the pure, unit-tested diffCapsCardHtml — the band and
  // verdict are the CLI's, rendered verbatim and never recomputed here.
  target.innerHTML = diffCapsCardHtml(report, new Date().toLocaleTimeString());
}

function renderEnforcementLegend(targetId: string, legend: EnforcementLegend): void {
  const target = document.getElementById(targetId);
  if (!target) return;
  // All rendering lives in the pure, unit-tested enforcementLegendHtml — the
  // enforced/declared/deferred status and the live-probe confirmation come from
  // the backend payload, never recomputed or hand-written here.
  target.innerHTML = enforcementLegendHtml(legend, new Date().toLocaleTimeString());
}

async function refreshEnforcementLegend(): Promise<void> {
  const legend = await invoke<EnforcementLegend>("studio_enforcement_legend");
  renderEnforcementLegend("legend-result", legend);
}

function renderAgentLoop(targetId: string, dossier: AgentLoopDossier): void {
  const target = document.getElementById(targetId);
  if (!target) return;
  // All rendering lives in the pure, unit-tested agentLoopConsoleHtml — the
  // four-gate verdict comes from the CLI's record-dir (decision.md + artifacts),
  // never recomputed here.
  target.innerHTML = agentLoopConsoleHtml(dossier, new Date().toLocaleTimeString());
}

// Lazily populate the legend the first time its panel is opened — the probe
// spawns two `garnet check` subprocesses, so it must not run at boot (the panel
// is power-only and hidden in simple mode). On failure (e.g. browser preview
// where invoke rejects) the guard is released so a later activation retries and
// the static fallback copy stays visible.
async function loadEnforcementLegend(): Promise<void> {
  if (legendLoaded) return;
  legendLoaded = true;
  try {
    await refreshEnforcementLegend();
  } catch {
    legendLoaded = false;
  }
}

function setupVelocityEditor(): void {
  const buffer = document.getElementById("velocity-buffer") as HTMLTextAreaElement | null;
  const out = document.getElementById("velocity-diagnostics");
  if (!buffer || !out) return;

  // latestOnly routes EVERY update (including the empty-buffer hint) through one
  // sequence guard, so a slow earlier check can never overwrite a newer result.
  const update = latestOnly<string, string>(
    async (source) => {
      if (source.trim().length === 0) {
        return `<p class="diagnostics-ok">Type Garnet source to check it live.</p>`;
      }
      try {
        const report = await invoke<VelocityCheckReport>("studio_velocity_check", { source });
        return velocityDiagnosticsHtml(report, source);
      } catch (error) {
        // Render the rejection through the same honest "did not run" path.
        return velocityDiagnosticsHtml(
          {
            ran: false,
            diagnostics: [],
            errors: 0,
            warnings: 0,
            infos: 0,
            ok: false,
            exit_code: -1,
            stderr: String(error),
          },
          source,
        );
      }
    },
    (html) => {
      out.innerHTML = html;
    },
  );

  // Debounce: live-check 200ms after typing stops. The backend writes an
  // ephemeral temp file only — no evidence bundle is sealed per keystroke.
  let timer: ReturnType<typeof setTimeout> | undefined;
  buffer.addEventListener("input", () => {
    clearTimeout(timer);
    timer = setTimeout(() => void update(buffer.value), 200);
  });
}

function renderHealth(targetId: string, health: HealthStatus): void {
  const target = document.getElementById(targetId);
  if (!target) return;

  target.innerHTML = `
    <div class="status-grid">
      ${healthTile("Garnet CLI", health.cli_found, health.cli_found ? health.cli_version.split("\n")[0] : "Not found")}
      ${healthTile("Repository", health.repo_found, health.repo_found ? health.repo_path : "Not found")}
      ${healthTile("Python", health.python_found, health.python_found ? health.python_version : "Not found")}
      ${healthTile("Host", true, `${health.platform} / ${health.arch}`)}
    </div>
    <article class="result ok">
      <pre>${escapeHtml(
        [
          `cli: ${health.cli_path || "(not found)"}`,
          `repo: ${health.repo_path || "(not found)"}`,
          `dogfood: ${health.evidence_dir}`,
        ].join("\n"),
      )}</pre>
    </article>
  `;

  const cliLabel = health.cli_found
    ? `CLI: ${firstVersionLine(health.cli_version)}`
    : "CLI: not found — set GARNET_CLI";
  setText("sb-cli", cliLabel);
}

function renderBootstrapPlan(plan: BootstrapPlan): void {
  const target = document.getElementById("bootstrap-plan");
  if (!target) return;

  const requirements = plan.requirements
    .map(
      (requirement) => `
        <article class="setup-step ${requirement.found ? "ok" : "fail"}">
          <header>
            <span class="dot ${requirement.found ? "ok" : "fail"}"></span>
            <strong>${escapeHtml(requirement.label)}</strong>
          </header>
          <p>${escapeHtml(requirement.action)}</p>
          <code>${escapeHtml(requirement.command)}</code>
          <small>${escapeHtml(requirement.evidence_note)}</small>
        </article>
      `,
    )
    .join("");
  const notes = plan.safety_notes
    .map((note) => `<li>${escapeHtml(note)}</li>`)
    .join("");

  target.innerHTML = `
    <article class="setup-summary ${plan.ready ? "ok" : "fail"}">
      <strong>${escapeHtml(plan.summary)}</strong>
      <span>${plan.ready_count}/${plan.total_count} ready</span>
    </article>
    <div class="setup-grid">${requirements}</div>
    <ul class="setup-copy">${notes}</ul>
  `;
}

async function refreshBootstrapPlan(): Promise<void> {
  try {
    const plan = await invoke<BootstrapPlan>("studio_bootstrap_plan");
    renderBootstrapPlan(plan);
  } catch (error) {
    renderError("bootstrap-plan", error);
  }
}

function firstVersionLine(banner: string): string {
  const line = banner
    .split("\n")
    .map((entry) => entry.trim())
    .find((entry) => entry.toLowerCase().startsWith("garnet "));
  return line ?? banner.split("\n")[0] ?? "unknown";
}

function healthTile(label: string, ok: boolean, value: string): string {
  return `
    <div class="status-tile">
      <span class="dot ${ok ? "ok" : "fail"}"></span>
      <div>
        <strong>${escapeHtml(label)}</strong>
        <span>${escapeHtml(value)}</span>
      </div>
    </div>
  `;
}

function renderBundle(targetId: string, bundle: EvidenceBundle): void {
  const target = document.getElementById(targetId);
  if (!target) return;
  target.innerHTML = `
    <article class="result ok">
      <header>
        <span>Evidence bundle created</span>
        <time>${escapeHtml(bundle.timestamp)}</time>
      </header>
      <pre>${escapeHtml(`bundle: ${bundle.path}\nmanifest: ${bundle.manifest_path}\ninclude_source: false`)}</pre>
    </article>
  `;
}

function setBusy(button: HTMLButtonElement, busy: boolean): void {
  if (busy) {
    button.disabled = true;
    button.dataset.originalText = button.textContent ?? "";
    button.textContent = "Running";
    button.classList.add("busy");
  } else {
    button.disabled = false;
    button.textContent = button.dataset.originalText ?? button.textContent ?? "";
    button.classList.remove("busy");
  }
}

function wireButton(id: string, action: () => Promise<void>): void {
  const button = document.getElementById(id) as HTMLButtonElement | null;
  if (!button) return;

  button.addEventListener("click", async () => {
    setBusy(button, true);
    try {
      await action();
    } finally {
      setBusy(button, false);
    }
  });
}

function requireValue(id: string, label: string): string {
  const value = getInput(id);
  if (!value) {
    throw new Error(`${label} is required.`);
  }
  return value;
}

async function runCommand(
  targetId: string,
  command: string,
  args: Record<string, unknown>,
): Promise<CommandResult | null> {
  try {
    const result = await invoke<CommandResult>(command, args);
    renderOutput(targetId, result);
    return result;
  } catch (error) {
    renderError(targetId, error);
    return null;
  }
}

// ---------------------------------------------------------------------------
// Panels, modes, themes
// ---------------------------------------------------------------------------

function visiblePanelButtons(): HTMLButtonElement[] {
  return Array.from(document.querySelectorAll<HTMLButtonElement>("[data-panel]")).filter(
    (button) => button.offsetParent !== null,
  );
}

function activatePanel(name: string): void {
  const nav = document.querySelectorAll<HTMLButtonElement>("[data-panel]");
  const panels = document.querySelectorAll<HTMLElement>(".panel");
  nav.forEach((item) => item.classList.toggle("active", item.dataset.panel === name));
  panels.forEach((item) => item.classList.toggle("active", item.id === `panel-${name}`));
  if (name === "release") {
    void loadTruthTiles();
  }
  if (name === "legend") {
    void loadEnforcementLegend();
  }
}

function setupTabs(): void {
  document.querySelectorAll<HTMLButtonElement>("[data-panel]").forEach((button) => {
    button.addEventListener("click", () => {
      const panel = button.dataset.panel;
      if (panel) activatePanel(panel);
    });
  });
}

function applyMode(mode: string): void {
  document.body.dataset.mode = mode;
  setText("sb-mode", `mode: ${mode}`);
  setText("brand-subtitle", mode === "power" ? "Power mode" : "Simple mode");
  const active = document.querySelector<HTMLButtonElement>("[data-panel].active");
  if (mode === "simple" && active?.hasAttribute("data-power-only")) {
    activatePanel("health");
  }
}

function applyTheme(theme: string): void {
  if (theme === "system") {
    const prefersLight = window.matchMedia?.("(prefers-color-scheme: light)").matches;
    document.body.dataset.theme = prefersLight ? "light" : "dark";
  } else {
    document.body.dataset.theme = theme;
  }
}

function applySettings(settings: StudioSettings): void {
  currentSettings = settings;
  applyMode(settings.mode);
  applyTheme(settings.theme);
  const modeRadio = document.querySelector<HTMLInputElement>(
    `input[name="set-mode"][value="${settings.mode}"]`,
  );
  if (modeRadio) modeRadio.checked = true;
  const themeRadio = document.querySelector<HTMLInputElement>(
    `input[name="set-theme"][value="${settings.theme}"]`,
  );
  if (themeRadio) themeRadio.checked = true;
  setInput("set-timeout", String(settings.command_timeout_secs));
  setInput("set-matrix-timeout", String(settings.matrix_timeout_secs));
}

function readSettingsForm(): StudioSettings {
  const mode =
    document.querySelector<HTMLInputElement>('input[name="set-mode"]:checked')?.value ??
    currentSettings.mode;
  const theme =
    document.querySelector<HTMLInputElement>('input[name="set-theme"]:checked')?.value ??
    currentSettings.theme;
  const commandTimeout = Number.parseInt(getInput("set-timeout"), 10);
  const matrixTimeout = Number.parseInt(getInput("set-matrix-timeout"), 10);
  return {
    mode,
    theme,
    command_timeout_secs: Number.isFinite(commandTimeout)
      ? commandTimeout
      : currentSettings.command_timeout_secs,
    matrix_timeout_secs: Number.isFinite(matrixTimeout)
      ? matrixTimeout
      : currentSettings.matrix_timeout_secs,
  };
}

// ---------------------------------------------------------------------------
// Truth tiles (Release panel)
// ---------------------------------------------------------------------------

async function loadTruthTiles(): Promise<void> {
  if (truthLoaded) return;
  const target = document.getElementById("truth-tiles");
  if (!target) return;
  truthLoaded = true;
  try {
    const truth = await invoke<TruthSummary>("get_truth_summary");
    if (!truth.found) {
      // Not latched: retry on the next panel activation once the cause
      // (missing repo / regenerating truth.json) is fixed.
      truthLoaded = false;
      target.innerHTML = `
        <div class="status-tile">
          <span class="dot warn"></span>
          <div>
            <strong>Truth surface unavailable</strong>
            <span>${escapeHtml(truth.error ?? "docs/truth.json was not found; live stats are hidden rather than guessed.")}</span>
          </div>
        </div>
      `;
      return;
    }
    const tests =
      truth.workspace_tests_passed != null
        ? `${truth.workspace_tests_passed} passed / ${truth.workspace_tests_failed ?? 0} failed`
        : "unavailable";
    const testsCommit =
      truth.workspace_tests_measured_at_commit ?? truth.generated_at_commit ?? "unknown commit";
    target.innerHTML = `
      ${truthTile("Version", truth.version ?? "?", `latest tag ${truth.latest_tag ?? "?"}`)}
      ${truthTile("Tracked slices", truth.tracked_slices ?? "?", `readiness ${truth.readiness_pct ?? "?"}%`)}
      ${truthTile("Primitives", String(truth.primitive_count ?? "?"), "core + std layers")}
      ${truthTile("Workspace tests", tests, `measured at ${testsCommit}`)}
    `;
  } catch (error) {
    truthLoaded = false;
    target.innerHTML = `
      <div class="status-tile">
        <span class="dot warn"></span>
        <div>
          <strong>Truth surface unavailable</strong>
          <span>${escapeHtml(String(error))}</span>
        </div>
      </div>
    `;
  }
}

function truthTile(label: string, value: string, detail: string): string {
  return `
    <div class="status-tile">
      <span class="dot ok"></span>
      <div>
        <strong>${escapeHtml(label)}</strong>
        <span>${escapeHtml(value)} — ${escapeHtml(detail)}</span>
      </div>
    </div>
  `;
}

// ---------------------------------------------------------------------------
// Converter preview
// ---------------------------------------------------------------------------

async function renderConvertPreview(result: CommandResult): Promise<void> {
  const target = document.getElementById("convert-preview");
  if (!target) return;
  target.innerHTML = "";
  if (!result.success || !result.evidence_path) return;

  try {
    const listing = await invoke<EvidenceListing>("list_evidence_files", {
      dir: result.evidence_path,
    });
    const garnetFiles = listing.files.filter((file) => file.relative_path.endsWith(".garnet"));
    if (garnetFiles.length === 0) {
      target.innerHTML = `
        <article class="result ok">
          <header><span>Converted bundle</span></header>
          <pre>${escapeHtml(
            `No .garnet output found in the bundle root.\nBundle files:\n${listing.files
              .map((file) => `  ${file.relative_path} (${file.size} bytes)`)
              .join("\n")}`,
          )}</pre>
        </article>
      `;
      return;
    }
    const first = garnetFiles[0];
    const text = await invoke<EvidenceText>("read_evidence_text", {
      path: `${result.evidence_path}/${first.relative_path}`,
    });
    const others = garnetFiles.slice(1);
    target.innerHTML = `
      <article class="result ok">
        <header>
          <span>Converted output — ${escapeHtml(first.relative_path)}${
            text.truncated ? ' <span class="badge warn">preview truncated</span>' : ""
          }</span>
          <span class="result-tools">
            <button class="mini" data-copy="${escapeHtml(text.content)}">Copy</button>
          </span>
        </header>
        <pre>${escapeHtml(text.content)}</pre>
        ${
          others.length
            ? `<footer class="result-footer">Also produced: ${others
                .map((file) => escapeHtml(file.relative_path))
                .join(", ")}</footer>`
            : ""
        }
      </article>
    `;
    wireCopyButtons(target as HTMLElement);
  } catch (error) {
    target.innerHTML = `
      <article class="result fail">
        <header><span>Preview unavailable</span></header>
        <pre>${escapeHtml(String(error))}</pre>
      </article>
    `;
  }
}

// ---------------------------------------------------------------------------
// Keyboard shortcuts
// ---------------------------------------------------------------------------

const PRIMARY_ACTION_BY_PANEL: Record<string, string> = {
  health: "btn-health",
  garnet: "btn-check",
  convert: "btn-convert",
  advisory: "btn-assist",
  evidence: "btn-evidence",
  release: "btn-windows-status",
  "agent-loop": "btn-agent-loop",
  settings: "btn-save-settings",
};

function setupShortcuts(): void {
  document.addEventListener("keydown", (event) => {
    if (event.ctrlKey && !event.altKey && !event.shiftKey) {
      const digit = Number.parseInt(event.key, 10);
      if (Number.isInteger(digit) && digit >= 0 && digit <= 9) {
        const buttons = visiblePanelButtons();
        // Ctrl+1..9 select the first nine visible panels; Ctrl+0 selects the
        // tenth, so every visible power-mode panel has a digit.
        const index = digit === 0 ? 9 : digit - 1;
        const button = buttons[index];
        if (button?.dataset.panel) {
          event.preventDefault();
          activatePanel(button.dataset.panel);
          button.focus();
        }
        return;
      }
      if (event.key === "Enter") {
        const active = document.querySelector<HTMLButtonElement>("[data-panel].active");
        const panel = active?.dataset.panel;
        const actionId = panel ? PRIMARY_ACTION_BY_PANEL[panel] : undefined;
        if (actionId) {
          event.preventDefault();
          (document.getElementById(actionId) as HTMLButtonElement | null)?.click();
        }
      }
    }
    if (event.key === "Escape") {
      const active = document.querySelector<HTMLElement>(".panel.active");
      active?.focus();
    }
  });
}

// ---------------------------------------------------------------------------
// Splash + boot
// ---------------------------------------------------------------------------

const SPLASH_MINIMUM_MS = 700;
// Upper bound on how long the splash may wait for the health check.
const SPLASH_HEALTH_BUDGET_MS = 25_000;

function setSplashStatus(message: string): void {
  setText("splash-status", message);
}

function dismissSplash(): void {
  const splash = document.getElementById("splash");
  if (!splash) return;
  const reduceMotion = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
  if (reduceMotion) {
    splash.remove();
    return;
  }
  splash.classList.add("splash-leaving");
  splash.addEventListener("transitionend", () => splash.remove(), { once: true });
  // Safety net: never let a missed transition event strand the splash.
  setTimeout(() => splash.remove(), 1200);
}

async function boot(): Promise<void> {
  const bootStarted = performance.now();

  setSplashStatus("Loading preferences…");
  try {
    const [info, settings] = await Promise.all([
      invoke<AppInfo>("get_app_info"),
      invoke<StudioSettings>("studio_get_settings"),
    ]);
    setText("splash-version", `v${info.app_version}`);
    setText("sb-app", `Studio v${info.app_version}`);
    setText("settings-path", info.settings_path);
    applySettings(settings);
  } catch {
    // Outside the Tauri shell (plain vite preview) the invokes reject; the UI
    // still renders with defaults so the layout can be inspected.
    applySettings({ ...DEFAULT_SETTINGS });
    setText("sb-app", "Studio (browser preview — Tauri shell required for actions)");
  }

  setSplashStatus("Checking CLI health…");
  try {
    // The backend bounds the health probes (10s each), so this settles; the
    // race is defense-in-depth — the splash must lift no matter what.
    const health = await Promise.race([
      invoke<HealthStatus>("cli_health"),
      new Promise<null>((resolve) => setTimeout(() => resolve(null), SPLASH_HEALTH_BUDGET_MS)),
    ]);
    if (health) {
      renderHealth("health-result", health);
      await refreshBootstrapPlan();
      setSplashStatus(health.cli_found ? "Garnet CLI found" : "Garnet CLI not found");
    } else {
      setSplashStatus("Health check still running — opening the shell");
    }
  } catch (error) {
    renderError("health-result", error);
    setSplashStatus("Health check unavailable");
  }

  const elapsed = performance.now() - bootStarted;
  const remaining = Math.max(0, SPLASH_MINIMUM_MS - elapsed);
  setTimeout(dismissSplash, remaining);
}

window.addEventListener("DOMContentLoaded", () => {
  setupTabs();
  setupShortcuts();
  setupVelocityEditor();
  void boot();

  setInput("garnet-file", "examples/mvp_01_os_simulator.garnet");
  setInput("convert-source", "");
  setInput("assist-source", "");
  setInput("bundle-source", "");

  invoke<string>("get_evidence_dir")
    .then((dir) => {
      const target = document.getElementById("evidence-root");
      if (target) target.textContent = dir;
    })
    .catch(() => {});

  wireButton("btn-health", async () => {
    try {
      const health = await invoke<HealthStatus>("cli_health");
      renderHealth("health-result", health);
      await refreshBootstrapPlan();
    } catch (error) {
      renderError("health-result", error);
    }
  });

  wireButton("btn-bootstrap-plan", refreshBootstrapPlan);
  wireButton("btn-bootstrap-scripts", async () => {
    await runCommand("bootstrap-result", "studio_bootstrap_write_scripts", {});
    await refreshBootstrapPlan();
  });

  const bootstrapSteps: ReadonlyArray<readonly [string, string]> = [
    ["btn-bootstrap-run-preflight", "preflight"],
    ["btn-bootstrap-run-install-python", "install-python"],
    ["btn-bootstrap-run-build-cli", "build-cli"],
    ["btn-bootstrap-run-configure-env", "configure-env"],
  ];
  for (const [id, step] of bootstrapSteps) {
    wireButton(id, async () => {
      await runCommand("bootstrap-result", "studio_bootstrap_run_step", { step });
      // Re-check health + the setup plan so they reflect what THIS running
      // process can already see. winget / User-scope env changes only land
      // after a Studio restart, so an install step may still read "missing"
      // here until then — the step's own output says to restart.
      try {
        const health = await invoke<HealthStatus>("cli_health");
        renderHealth("health-result", health);
      } catch (error) {
        renderError("health-result", error);
      }
      await refreshBootstrapPlan();
    });
  }

  wireButton("btn-parse", async () => {
    await runCommand("garnet-result", "cli_parse", {
      filePath: requireValue("garnet-file", "Garnet file"),
    });
  });
  wireButton("btn-check", async () => {
    await runCommand("garnet-result", "cli_check", {
      filePath: requireValue("garnet-file", "Garnet file"),
    });
  });
  wireButton("btn-run", async () => {
    await runCommand("garnet-result", "cli_run", {
      filePath: requireValue("garnet-file", "Garnet file"),
    });
  });

  wireButton("btn-convert", async () => {
    const result = await runCommand("convert-result", "cli_convert", {
      sourceFile: requireValue("convert-source", "Source file"),
      sourceLang: requireValue("convert-lang", "Source language"),
    });
    if (result) {
      await renderConvertPreview(result);
    }
  });

  wireButton("btn-assist", async () => {
    await runCommand("assist-result", "advisory_assist_plan", {
      sourceFile: requireValue("assist-source", "Source file"),
      language: requireValue("assist-lang", "Language"),
    });
  });

  wireButton("btn-bundle", async () => {
    await runCommand("bundle-result", "advisory_bundle", {
      sourceFile: requireValue("bundle-source", "Source file"),
      language: requireValue("bundle-lang", "Language"),
    });
  });

  wireButton("btn-review", async () => {
    await runCommand("review-result", "advisory_review", {
      bundleDir: requireValue("review-bundle", "Bundle directory"),
    });
  });

  wireButton("btn-handoff", async () => {
    await runCommand("handoff-result", "advisory_handoff", {
      bundleDir: requireValue("handoff-bundle", "Bundle directory"),
      reviewDir: requireValue("handoff-review", "Review directory"),
    });
  });

  wireButton("btn-pulse", async () => {
    await runCommand("pulse-result", "objective_pulse", {});
  });
  wireButton("btn-dogfood", async () => {
    await runCommand("dogfood-result", "agentic_dogfood_matrix", {});
  });
  wireButton("btn-windows-status", async () => {
    await runCommand("release-result", "windows_linux_studio_status", {});
  });
  wireButton("btn-domain-proof", async () => {
    await runCommand("release-result", "domain_proof_matrix", {});
  });
  wireButton("btn-mac-domain-proofs", async () => {
    await runCommand("release-result", "mac_domain_proofs", {});
  });
  wireButton("btn-converter-status", async () => {
    await runCommand("release-result", "converter_status", {});
  });
  wireButton("btn-provider-options", async () => {
    await runCommand("release-result", "provider_options", {});
  });
  wireButton("btn-mit-demo", async () => {
    await runCommand("release-result", "mit_demo_route", {});
  });
  wireButton("btn-deck-outline", async () => {
    await runCommand("release-result", "mit_deck_outline", {});
  });
  wireButton("btn-deck-preview", async () => {
    await runCommand("release-result", "mit_deck_preview", {});
  });
  wireButton("btn-mac-continuation", async () => {
    await runCommand("release-result", "mac_continuation_pulse", {});
  });
  wireButton("btn-proof-status", async () => {
    await runCommand("release-result", "proof_benchmark_status", {});
  });
  wireButton("btn-benchmark-no-run", async () => {
    await runCommand("release-result", "benchmark_no_run", {});
  });
  wireButton("btn-notarization", async () => {
    await runCommand("release-result", "notarization_status", {});
  });
  wireButton("btn-windows-vm-installer", async () => {
    await runCommand("release-result", "windows_vm_installer_status", {});
  });

  wireButton("btn-legend", async () => {
    try {
      legendLoaded = true; // an explicit refresh also satisfies the lazy guard
      await refreshEnforcementLegend();
    } catch (error) {
      renderError("legend-result", error);
    }
  });

  wireButton("btn-agent-loop", async () => {
    try {
      const dossier = await invoke<AgentLoopDossier>("studio_agent_loop_dossier", {
        recordDir: requireValue("agent-loop-dir", "Record directory"),
      });
      renderAgentLoop("agent-loop-result", dossier);
    } catch (error) {
      renderError("agent-loop-result", error);
    }
  });

  wireButton("btn-diff-caps", async () => {
    try {
      const report = await invoke<DiffCapsReport>("studio_diff_caps", {
        oldPath: requireValue("diff-caps-old", "Old revision path"),
        newPath: requireValue("diff-caps-new", "New revision path"),
      });
      renderDiffCaps("diff-caps-result", report);
    } catch (error) {
      renderError("diff-caps-result", error);
    }
  });

  wireButton("btn-evidence", async () => {
    try {
      const bundle = await invoke<EvidenceBundle>("create_evidence_bundle");
      renderBundle("evidence-result", bundle);
    } catch (error) {
      renderError("evidence-result", error);
    }
  });

  wireButton("btn-save-settings", async () => {
    try {
      const saved = await invoke<StudioSettings>("studio_set_settings", {
        settings: readSettingsForm(),
      });
      applySettings(saved);
      const target = document.getElementById("settings-result");
      if (target) {
        target.innerHTML = `
          <article class="result ok">
            <header><span>Settings saved</span><time>${new Date().toLocaleTimeString()}</time></header>
            <pre>${escapeHtml(JSON.stringify(saved, null, 2))}</pre>
          </article>
        `;
      }
    } catch (error) {
      renderError("settings-result", error);
    }
  });

  window
    .matchMedia?.("(prefers-color-scheme: light)")
    .addEventListener?.("change", () => {
      if (currentSettings.theme === "system") applyTheme("system");
    });
});
