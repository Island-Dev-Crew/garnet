import { defineConfig, devices } from "@playwright/test";

// WV-4 — Studio UI harness. Drives the BUILT Vite dist in a real browser and
// asserts the overhaul's UI structure + pure-frontend behaviour (splash holds
// then dismisses, all panels present, simple-mode hides the power-only panels,
// panel switching, safety-contract copy, tooltips, status bar). Outside the
// Tauri shell the `invoke()` calls reject by design and main.ts boot() degrades
// to a "browser preview" — so this is structure/behaviour proof, NOT a CLI
// round-trip. Driving the real desktop shell (Run -> CommandResult, evidence
// bundle, persisted mode toggle) needs tauri-driver/WebDriver and is a flagged
// follow-up recorded in the WV-4 fleet section.
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? "github" : "list",
  use: {
    baseURL: "http://localhost:4317",
    trace: "on-first-retry",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: "npm run build && npm run preview -- --port 4317 --strictPort",
    url: "http://localhost:4317",
    reuseExistingServer: !process.env.CI,
    timeout: 180000,
  },
});
