import { invoke } from "@tauri-apps/api/core";

interface CommandResult {
  success: boolean;
  stdout: string;
  stderr: string;
  exit_code: number;
  command: string[];
  evidence_path: string | null;
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

interface EvidenceBundle {
  path: string;
  timestamp: string;
  manifest_path: string;
}

interface LanguageGroup {
  name: string;
  languages: string[];
}

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

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
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

  target.innerHTML = `
    <article class="result ${statusClass}">
      <header>
        <span>${result.success ? "Passed" : `Failed (${result.exit_code})`}</span>
        <time>${new Date().toLocaleTimeString()}</time>
      </header>
      <pre>${escapeHtml(sections.join("\n\n") || "(no output)")}</pre>
    </article>
  `;
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

function renderHealth(targetId: string, health: HealthStatus): void {
  const target = document.getElementById(targetId);
  if (!target) return;

  target.innerHTML = `
    <div class="status-grid">
      ${healthTile("Garnet CLI", health.cli_found, health.cli_found ? health.cli_version : "Not found")}
      ${healthTile("Repository", health.repo_found, health.repo_found ? health.repo_path : "Not found")}
      ${healthTile("Python", health.python_found, health.python_found ? health.python_version : "Not found")}
      ${healthTile("Host", true, `${health.platform} / ${health.arch}`)}
    </div>
    <article class="result ok">
      <pre>${escapeHtml([
        `cli: ${health.cli_path || "(not found)"}`,
        `repo: ${health.repo_path || "(not found)"}`,
        `dogfood: ${health.evidence_dir}`,
      ].join("\n"))}</pre>
    </article>
  `;
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
  } else {
    button.disabled = false;
    button.textContent = button.dataset.originalText ?? button.textContent ?? "";
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

async function runCommand(targetId: string, command: string, args: Record<string, unknown>): Promise<void> {
  try {
    const result = await invoke<CommandResult>(command, args);
    renderOutput(targetId, result);
  } catch (error) {
    renderError(targetId, error);
  }
}

function setupTabs(): void {
  const nav = document.querySelectorAll<HTMLButtonElement>("[data-panel]");
  const panels = document.querySelectorAll<HTMLElement>(".panel");

  nav.forEach((button) => {
    button.addEventListener("click", () => {
      const panel = button.dataset.panel;
      nav.forEach((item) => item.classList.toggle("active", item === button));
      panels.forEach((item) => item.classList.toggle("active", item.id === `panel-${panel}`));
    });
  });
}

async function loadTaxonomy(): Promise<void> {
  const target = document.getElementById("taxonomy");
  if (!target) return;

  try {
    const groups = await invoke<LanguageGroup[]>("get_language_taxonomy");
    target.innerHTML = groups
      .map(
        (group) => `
          <section>
            <h3>${escapeHtml(group.name)}</h3>
            <div class="pills">
              ${group.languages.map((language) => `<span>${escapeHtml(language)}</span>`).join("")}
            </div>
          </section>
        `,
      )
      .join("");
  } catch {
    target.textContent = "Taxonomy is available inside the Tauri shell.";
  }
}

window.addEventListener("DOMContentLoaded", () => {
  setupTabs();
  loadTaxonomy();

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
    } catch (error) {
      renderError("health-result", error);
    }
  });

  wireButton("btn-parse", () =>
    runCommand("garnet-result", "cli_parse", { filePath: requireValue("garnet-file", "Garnet file") }),
  );
  wireButton("btn-check", () =>
    runCommand("garnet-result", "cli_check", { filePath: requireValue("garnet-file", "Garnet file") }),
  );
  wireButton("btn-run", () =>
    runCommand("garnet-result", "cli_run", { filePath: requireValue("garnet-file", "Garnet file") }),
  );

  wireButton("btn-convert", () =>
    runCommand("convert-result", "cli_convert", {
      sourceFile: requireValue("convert-source", "Source file"),
      sourceLang: requireValue("convert-lang", "Source language"),
    }),
  );

  wireButton("btn-assist", () =>
    runCommand("assist-result", "advisory_assist_plan", {
      sourceFile: requireValue("assist-source", "Source file"),
      language: requireValue("assist-lang", "Language"),
    }),
  );

  wireButton("btn-bundle", () =>
    runCommand("bundle-result", "advisory_bundle", {
      sourceFile: requireValue("bundle-source", "Source file"),
      language: requireValue("bundle-lang", "Language"),
    }),
  );

  wireButton("btn-review", () =>
    runCommand("review-result", "advisory_review", {
      bundleDir: requireValue("review-bundle", "Bundle directory"),
    }),
  );

  wireButton("btn-handoff", () =>
    runCommand("handoff-result", "advisory_handoff", {
      bundleDir: requireValue("handoff-bundle", "Bundle directory"),
      reviewDir: requireValue("handoff-review", "Review directory"),
    }),
  );

  wireButton("btn-pulse", () => runCommand("pulse-result", "objective_pulse", {}));
  wireButton("btn-dogfood", () => runCommand("dogfood-result", "agentic_dogfood_matrix", {}));
  wireButton("btn-windows-status", () =>
    runCommand("release-result", "windows_linux_studio_status", {}),
  );
  wireButton("btn-domain-proof", () => runCommand("release-result", "domain_proof_matrix", {}));
  wireButton("btn-mac-domain-proofs", () =>
    runCommand("release-result", "mac_domain_proofs", {}),
  );
  wireButton("btn-converter-status", () => runCommand("release-result", "converter_status", {}));
  wireButton("btn-provider-options", () => runCommand("release-result", "provider_options", {}));
  wireButton("btn-mit-demo", () => runCommand("release-result", "mit_demo_route", {}));
  wireButton("btn-deck-outline", () => runCommand("release-result", "mit_deck_outline", {}));
  wireButton("btn-deck-preview", () => runCommand("release-result", "mit_deck_preview", {}));
  wireButton("btn-mac-continuation", () =>
    runCommand("release-result", "mac_continuation_pulse", {}),
  );
  wireButton("btn-proof-status", () => runCommand("release-result", "proof_benchmark_status", {}));
  wireButton("btn-benchmark-no-run", () => runCommand("release-result", "benchmark_no_run", {}));
  wireButton("btn-notarization", () => runCommand("release-result", "notarization_status", {}));
  wireButton("btn-windows-vm-installer", () =>
    runCommand("release-result", "windows_vm_installer_status", {}),
  );

  wireButton("btn-evidence", async () => {
    try {
      const bundle = await invoke<EvidenceBundle>("create_evidence_bundle");
      renderBundle("evidence-result", bundle);
    } catch (error) {
      renderError("evidence-result", error);
    }
  });

  const healthButton = document.getElementById("btn-health") as HTMLButtonElement | null;
  healthButton?.click();
});
