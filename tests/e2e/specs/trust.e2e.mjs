// SPDX-License-Identifier: AGPL-3.0-or-later
// Workspace trust (D-27, session 4A Done when 3): a project this machine didn't make opens read-only, with the banner,
// "Read-only" in the panel, its knobs locked, nothing run, and the host refusing every change the panel makes;
// Trust this project writes the marker, builds it, and unlocks it.
import { $, expect } from "@wdio/globals";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { invoke, screenshots } from "../lib/app.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = projectDir("trust");
const files = () => ["emmet.toml", "params.json", "model.py"].map((f) => readFileSync(join(dir, f), "utf8"));
/** A command's error code, or null when it worked. */
const refusal = (cmd, args) =>
  browser.execute((c, a) => window.__TAURI_INTERNALS__.invoke(c, a).then(() => null, (e) => e?.code ?? String(e)), cmd, args);

describe("workspace trust", () => {
  it("opens an untrusted project read-only, and runs nothing", async () => {
    const { root } = await invoke("open_recent", { path: dir });
    await browser.waitUntil(() => browser.execute((r) => window.__emmet.project() === r, root));
    await expect($("#banner-untrusted")).toHaveText(expect.stringContaining("Read-only"));
    await expect($("#model-state .panel-head-text")).toHaveText("Read-only");
    await expect($("#tweaks-locked")).toHaveText("Trust the project to use these");
    await expect($("#settings-change")).toBeDisabled();
    // The host refuses what the locked controls would send, and nothing changes.
    const before = files();
    const any = { base: null, scale: null, age: null, detail: null, pieces: null, style: null };
    for (const [cmd, args] of [
      ["set_build_settings", { settings: { ...any, age: "9+" } }],
      ["allow_finding", { rule: "floating", parts: ["roof/1"] }],
      ["unallow_finding", { rule: "floating", parts: ["roof/1"] }],
      ["keep_params", { names: ["weathering"] }],
      ["reset_params", { names: ["weathering"] }],
    ])
      expect([cmd, await refusal(cmd, args)]).toEqual([cmd, "PROJECT_UNTRUSTED"]);
    expect(files()).toEqual(before);
    await browser.pause(1000);
    expect(existsSync(join(dir, "build"))).toBe(false);
    await screenshots("untrusted", { baseline: true });
  });

  it("trusts it, builds it, and unlocks it", async () => {
    await (await $("#trust")).click();
    await browser.waitUntil(() => browser.execute(() => window.__emmet.shown() >= 1), { timeout: 30_000, timeoutMsg: "no build after Trust" });
    await expect($("#banner-untrusted")).not.toBeExisting();
    expect(existsSync(join(dir, ".emmet", "trust"))).toBe(true);
    await expect($("#param-weathering")).toBeDisplayed();
    await expect($("#model-state .panel-head-text")).toHaveText("No problems");
  });
});
