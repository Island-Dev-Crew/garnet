import { defineConfig, devices } from "@playwright/test";

// WV-4 — Studio UI harness. Drives the BUILT Vite dist in a real browser and
// asserts the overhaul's UI structure + pure-frontend behaviour (splash holds
// then dismisses, all panels present, simple-mode hides the power-only panels,
// panel switching, safety-contract copy, tooltips, status bar). Outside the
// Tauri shell the `invoke()` calls reject by design and main.ts boot() degrades
// to a "browser preview" — so this is structure/behaviour proof, NOT a CLI
// round-trip. Driving the real desktop shell (Run -> CommandResult, evidence
// bundle, persisted mode toggle) needs tauri-driver/WebDriver and is out of
// scope here — see this PR's "Deferred" section for the follow-up.
//
// Run: `npm run test:e2e` (the pretest hook fetches the pinned Chromium first).
// The preview server uses a strict, env-overridable port; it must be free.
const PORT = process.env.STUDIO_E2E_PORT ?? "4317";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? "github" : "list",
  use: {
    baseURL: `http://localhost:${PORT}`,
    trace: "on-first-retry",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: `npm run build && npm run preview -- --port ${PORT} --strictPort`,
    url: `http://localhost:${PORT}`,
    // Always build + serve fresh: never let a stale server squatting the port
    // silently serve the suite without a rebuild.
    reuseExistingServer: false,
    timeout: 180000,
  },
});
