// SPDX-License-Identifier: AGPL-3.0-or-later
// WebdriverIO config for the app E2E suite (plan section 16, ADR 0002). `just e2e` builds the debug app with the
// embedded WebDriver server (the emmet-desktop `webdriver` feature) and the E2E hooks (EMMET_E2E=1), and runs this.
//
// The service starts the app once for the whole run, and every spec file shares it. Before each spec file, a fresh
// copy of its project (lib/projects.mjs) goes on the recents list, and the spec opens it with open_recent. The data
// dir is the one the integration tests share (ldraw-mini and Python, from target/debug/prepare-data). Screenshots,
// renders, and bench results go to tests/e2e/out/, which CI uploads.
import { execFileSync } from "node:child_process";
import { basename } from "node:path";
import { webContentPids } from "./lib/memory.mjs";
import { appBinary, bin, out } from "./lib/paths.mjs";
import { prepareProject } from "./lib/projects.mjs";

const bench = process.env.EMMET_BENCH === "1";

export const config = {
  runner: "local",
  specs: [bench ? "./specs/bench.e2e.mjs" : "./specs/**/*.e2e.mjs"],
  exclude: bench ? [] : ["./specs/bench.e2e.mjs"],
  maxInstances: 1,
  capabilities: [{ browserName: "tauri", "tauri:options": { application: appBinary } }],
  services: [["@wdio/tauri-service", { driverProvider: "embedded" }]],
  framework: "mocha",
  reporters: ["spec"],
  logLevel: "warn",
  // One retry is allowed in E2E only (plan section 16).
  specFileRetries: bench ? 0 : 1,
  mochaOpts: { timeout: bench ? 600_000 : 120_000 },
  onPrepare() {
    process.env.EMMET_DATA_DIR = execFileSync(bin("prepare-data"), { encoding: "utf8" }).trim();
    process.env.EMMET_E2E_OUT = out;
    // WKWebView has no JS heap API, so the leak test reads the app's WebContent processes: the ones that appear after
    // this hook, which runs before the service starts the app (docs/decisions.md, 2026-10-25).
    if (process.platform === "darwin") process.env.EMMET_E2E_WEBCONTENT_BEFORE = webContentPids().join(",");
    // WebView2: exact heap numbers and window.gc() for the leak test.
    if (process.platform === "win32") {
      process.env.WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--enable-precise-memory-info --js-flags=--expose-gc";
    }
  },
  // Before each spec file: its fresh project, on the recents list.
  onWorkerStart(_cid, _caps, specs) {
    prepareProject(basename(specs[0] ?? "", ".e2e.mjs"));
  },
  // The app has one window. Choosing it explicitly turns off the service's per-command focus check, which needs the
  // service's companion plugin (tauri-plugin-wdio) and waits 5 s on every command without it (ADR 0002).
  beforeSuite: async () => {
    await browser.tauri.switchWindow("main");
  },
};
