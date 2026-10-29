// SPDX-License-Identifier: AGPL-3.0-or-later
// WebdriverIO config for the app E2E suite (plan section 16, ADR 0002). `just e2e` builds the debug app with the
// embedded WebDriver server (the emmet-desktop `webdriver` feature) and the E2E hooks (EMMET_E2E=1), and runs this.
//
// The service starts the app once for the whole run, and every spec file shares it. Before each spec file, a fresh
// copy of its project (lib/projects.mjs) goes on the recents list, and the spec opens it with open_recent. The data
// dir is the one the integration tests share (ldraw-mini and Python, from target/debug/prepare-data). Screenshots,
// renders, and bench results go to tests/e2e/out/, which CI uploads.
import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, rmSync } from "node:fs";
import { basename, join } from "node:path";
import { agentHome, prepareAgents, startOpenAi, stopOpenAi } from "./lib/agents.mjs";
import { webContentPids } from "./lib/memory.mjs";
import { startLibrary, stopLibrary } from "./lib/library-server.mjs";
import { appBinary, bin, out } from "./lib/paths.mjs";
import { prepareProject } from "./lib/projects.mjs";

const bench = process.env.EMMET_BENCH === "1";
// First run (EMMET_E2E_FIRST_RUN=1, session 4A) starts the app on its own fresh data dir: no library and no Python
// environment, only the Python interpreter the shared one downloaded (uv would download the same), and the library
// from lib/library-server.mjs. `just e2e` runs it after the shared run.
const firstRun = process.env.EMMET_E2E_FIRST_RUN === "1";
const fresh = join(out, "first-run-data");

/** The specs this run takes. */
function specs() {
  if (bench) return { specs: ["./specs/bench.e2e.mjs"], exclude: [] };
  if (firstRun) return { specs: ["./specs/first-run.e2e.mjs"], exclude: [] };
  return { specs: ["./specs/hello.e2e.mjs", "./specs/**/*.e2e.mjs"], exclude: ["./specs/bench.e2e.mjs", "./specs/first-run.e2e.mjs"] };
}

export const config = {
  runner: "local",
  // hello checks the app's first window (no project open), so it runs first; the app is shared by every spec file.
  ...specs(),
  maxInstances: 1,
  capabilities: [{ browserName: "tauri", "tauri:options": { application: appBinary } }],
  services: [["@wdio/tauri-service", { driverProvider: "embedded" }]],
  framework: "mocha",
  reporters: ["spec"],
  logLevel: "warn",
  // One retry is allowed in E2E only (plan section 16).
  // First run can't be tried again on the data dir its first try set up.
  specFileRetries: bench || firstRun ? 0 : 1,
  mochaOpts: { timeout: bench ? 600_000 : 120_000 },
  async onPrepare() {
    const shared = execFileSync(bin("prepare-data"), { encoding: "utf8" }).trim();
    process.env.EMMET_DATA_DIR = shared;
    process.env.EMMET_E2E_OUT = out;
    // Open… reads its pick from here in a debug build (src-tauri/src/projects.rs), since WebDriver can't drive the
    // system's file dialog.
    process.env.EMMET_E2E_PICK_FILE = join(out, "pick.txt");
    // The app reopens its last project at launch; the run starts from none, as hello expects.
    rmSync(join(shared, "recent.json"), { force: true });
    // Old-library checks and Update the library download ldraw-mini from here too.
    await startLibrary(shared);
    if (firstRun) {
      rmSync(fresh, { recursive: true, force: true });
      rmSync(join(agentHome, "Documents", "Emmet"), { recursive: true, force: true });
      mkdirSync(join(fresh, "python"), { recursive: true });
      // EMMET_E2E_REAL=1 (by hand, never in CI): the real library from library.ldraw.org and uv's real Python
      // download, to time a first run on a real connection (4A Done when 2).
      if (process.env.EMMET_E2E_REAL === "1") {
        for (const k of ["EMMET_LIBRARY_URL", "EMMET_LIBRARY_SHA256", "EMMET_LIBRARY_RELEASE"]) delete process.env[k];
      } else {
        cpSync(join(shared, "python", "installs"), join(fresh, "python", "installs"), { recursive: true, verbatimSymlinks: true });
      }
      process.env.EMMET_DATA_DIR = fresh;
    }
    // The chat specs' fake agents, and the test home with sentinel agent configs (lib/agents.mjs).
    prepareAgents();
    await startOpenAi();
    // WKWebView has no JS heap API, so the leak test reads the app's WebContent processes: the ones that appear after
    // this hook, which runs before the service starts the app (docs/decisions.md, 2026-10-25).
    if (process.platform === "darwin") process.env.EMMET_E2E_WEBCONTENT_BEFORE = webContentPids().join(",");
    // WebView2: exact heap numbers and window.gc() for the leak test.
    if (process.platform === "win32") {
      process.env.WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--enable-precise-memory-info --js-flags=--expose-gc";
    }
  },
  onComplete() {
    stopOpenAi();
    stopLibrary();
  },
  // Before each spec file: its fresh project, on the recents list.
  onWorkerStart(_cid, _caps, specs) {
    if (firstRun) return;
    // New, import, and Export write into the test home's Documents/Emmet and Downloads: each spec starts with neither.
    for (const d of ["Documents", "Downloads"]) rmSync(join(agentHome, d), { recursive: true, force: true });
    prepareProject(basename(specs[0] ?? "", ".e2e.mjs"));
  },
  // A failed test's window, for the CI artifact.
  async afterTest(test, _context, { passed }) {
    if (passed) return;
    const dir = join(out, "failures");
    mkdirSync(dir, { recursive: true });
    await browser.saveScreenshot(join(dir, `${test.parent} ${test.title}`.replace(/[^\w.-]+/g, "-").slice(0, 120) + ".png")).catch(() => {});
  },
  // The app has one window. Choosing it explicitly turns off the service's per-command focus check, which needs the
  // service's companion plugin (tauri-plugin-wdio) and waits 5 s on every command without it (ADR 0002).
  beforeSuite: async () => {
    await browser.tauri.switchWindow("main");
  },
};
