// SPDX-License-Identifier: AGPL-3.0-or-later
// WebdriverIO config for the app E2E suite (plan section 16, ADR 0002). `just e2e` builds the debug app with the
// embedded WebDriver server (the emmet-desktop `webdriver` feature) and runs this.
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(fileURLToPath(new URL(".", import.meta.url)), "..", "..");
const binary = join(root, "target", "debug", process.platform === "win32" ? "emmet-desktop.exe" : "emmet-desktop");

export const config = {
  runner: "local",
  specs: ["./specs/**/*.e2e.mjs"],
  maxInstances: 1,
  capabilities: [{ browserName: "tauri", "tauri:options": { application: binary } }],
  services: [["@wdio/tauri-service", { driverProvider: "embedded" }]],
  framework: "mocha",
  reporters: ["spec"],
  logLevel: "warn",
  // One retry is allowed in E2E only (plan section 16).
  specFileRetries: 1,
  mochaOpts: { timeout: 60_000 },
  // The app has one window. Choosing it explicitly turns off the service's per-command focus check, which needs the
  // service's companion plugin (tauri-plugin-wdio) and waits 5 s on every command without it (ADR 0002).
  beforeSuite: async () => {
    await browser.tauri.switchWindow("main");
  },
};
