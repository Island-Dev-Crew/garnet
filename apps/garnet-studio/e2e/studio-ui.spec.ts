import { test, expect } from "@playwright/test";

// Garnet Studio is a Tauri shell. Outside Tauri the `invoke()` calls reject and
// main.ts boot() renders with defaults (the status bar reads "browser preview").
// These specs assert the UI STRUCTURE and pure-frontend behaviour that needs no
// Tauri runtime — the surface the ~20 Python shell-contract + xvfb-window smokes
// never actually render in a browser.

const POWER_ONLY = ["Advisory Pipeline", "Evidence", "Release / Readiness"];
const SIMPLE = ["CLI Health", "Parse / Check / Run", "Active Conversion", "Settings"];

test.describe("Garnet Studio UI (built dist in a browser)", () => {
  test("the launch splash holds, then dismisses, and the shell renders", async ({ page }) => {
    await page.goto("/");
    // The splash overlay exists in the DOM on first paint, then main.ts removes
    // it after boot (>= the 700ms minimum hold, well under the 25s ceiling).
    await expect(page.locator("#splash")).toHaveCount(0, { timeout: 30000 });
    await expect(page.locator(".sidebar .brand h1")).toHaveText("Garnet Studio");
    await expect(page.locator("footer.statusbar")).toBeVisible();
  });

  test("all seven panels are present in the nav", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("nav.nav button[data-panel]")).toHaveCount(7);
    for (const label of [...SIMPLE, ...POWER_ONLY]) {
      await expect(page.locator("nav.nav button[data-panel]", { hasText: label })).toHaveCount(1);
    }
  });

  test("Phase 1 honesty cleanup: dead-weight surfaces are gone", async ({ page }) => {
    await page.goto("/");
    // Taxonomy panel removed from the UI (its backend command still feeds --studio-smoke).
    await expect(page.locator('nav.nav button[data-panel="taxonomy"]')).toHaveCount(0);
    await expect(page.locator("#panel-taxonomy")).toHaveCount(0);
    // The redundant status-bar evidence button is gone (the path is shown in the
    // Evidence panel and in CLI Health output).
    await expect(page.locator("#sb-evidence")).toHaveCount(0);
    // Advisory is labelled local-evidence-only — no implied backend or delivery.
    await expect(page.locator("#panel-advisory")).toContainText("local evidence only");
  });

  test("simple mode (default) hides the power-only panels and shows the simple ones", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("body")).toHaveAttribute("data-mode", "simple");
    for (const label of SIMPLE) {
      await expect(page.locator("nav.nav button[data-panel]", { hasText: label })).toBeVisible();
    }
    for (const label of POWER_ONLY) {
      await expect(page.locator("nav.nav button[data-panel]", { hasText: label })).toBeHidden();
    }
  });

  test("panel switching works (pure-frontend tab toggle)", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#panel-health")).toHaveClass(/active/);
    await page.locator("nav.nav button[data-panel]", { hasText: "Parse / Check / Run" }).click();
    await expect(page.locator("#panel-garnet")).toHaveClass(/active/);
    await expect(page.locator("#panel-health")).not.toHaveClass(/active/);
  });

  test("the safety-contract copy renders", async ({ page }) => {
    await page.goto("/");
    const c = page.locator(".contract");
    await expect(c).toContainText("No provider APIs");
    await expect(c).toContainText("Source omitted by default");
    await expect(c).toContainText("Advisory output is never marked safe");
  });

  test("CLI Health exposes the setup assistant controls", async ({ page }) => {
    await page.goto("/");
    const assistant = page.locator(".setup-assistant");
    await expect(assistant).toContainText("Setup Assistant");
    await expect(assistant).toContainText("Install Garnet CLI");
    await expect(assistant).toContainText("Install Python");
    await expect(assistant).toContainText("Set GARNET_REPO");
    await expect(page.locator("#btn-bootstrap-scripts")).toHaveText("Generate Setup Scripts");
  });

  test("CLI Health exposes the typed bootstrap run-step controls", async ({ page }) => {
    await page.goto("/");
    const assistant = page.locator(".setup-assistant");
    await expect(assistant.locator("#btn-bootstrap-run-preflight")).toHaveText("Run Preflight");
    await expect(assistant.locator("#btn-bootstrap-run-install-python")).toHaveText(
      "Install Python",
    );
    await expect(assistant.locator("#btn-bootstrap-run-build-cli")).toHaveText("Build CLI");
    await expect(assistant.locator("#btn-bootstrap-run-configure-env")).toHaveText(
      "Configure Env",
    );
    // The run controls state plainly that they execute locally and record
    // every run to an evidence bundle — no overselling install success.
    await expect(assistant.locator(".setup-run-copy")).toContainText("bootstrap-run");
  });

  test("hover help is present across the surface", async ({ page }) => {
    await page.goto("/");
    const tips = await page.locator("[data-tip]").count();
    expect(tips).toBeGreaterThanOrEqual(30);
  });

  test("the status bar reports a build version and mode", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#sb-mode")).toContainText("mode:");
    // Outside Tauri get_app_info rejects, so the version line degrades to the
    // browser-preview notice — assert one of the two honest states renders.
    await expect(page.locator("#sb-app")).toContainText(/Studio/);
  });
});
