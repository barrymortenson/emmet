// SPDX-License-Identifier: AGPL-3.0-or-later
// Killing a run mid-way never breaks the UI (1B What good looks like 2): the error says what happened, the last good
// model stays, and the next change rebuilds. On the scene-grid fixture, whose delay parameter makes a run slow.
import { execFileSync } from "node:child_process";
import { $, expect } from "@wdio/globals";
import { instances, invoke, nextScene, open, shown, stats } from "../lib/app.mjs";
import { projectDir } from "../lib/projects.mjs";

const project = projectDir("runs");

/** Kills the emmet run for `dir`, as a user might from Activity Monitor or Task Manager. */
function killRun(dir) {
  if (process.platform === "win32") {
    const filter = `$_.Name -eq 'emmet.exe' -and $_.CommandLine -like '*${dir.replaceAll("'", "''")}*'`;
    execFileSync("powershell", ["-NoProfile", "-Command", `Get-CimInstance Win32_Process | Where-Object { ${filter} } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }`]);
  } else {
    execFileSync("pkill", ["-KILL", "-f", `emmet .*${dir}`]);
  }
}

describe("a run killed mid-way", () => {
  it("shows what happened and keeps the last good model", async () => {
    const root = await open(project);
    await instances(500);
    await invoke("set_params", { values: { delay: 20 } });
    await $("#status").waitUntil(async function () {
      return (await this.getText()).includes("rebuilding");
    }, { timeout: 20_000, timeoutMsg: "the slow run never started" });
    await browser.pause(500);
    killRun(root);
    await $("#error-panel").waitForDisplayed({ timeout: 20_000 });
    await expect($("#error-panel .message")).toHaveText(expect.stringContaining("emmet stopped before it finished"));
    expect((await stats()).instances).toBe(500);
  });

  it("the next change rebuilds", async () => {
    const gen = await shown();
    await invoke("set_params", { values: { delay: 0, count: 120 } });
    await nextScene(gen);
    await expect($("#error-panel")).not.toBeExisting();
    await instances(120);
  });

  it("a change while a run is going cancels it quietly and the newer one shows", async () => {
    await invoke("set_params", { values: { delay: 5, count: 100 } });
    await $("#status").waitUntil(async function () {
      return (await this.getText()).includes("rebuilding");
    }, { timeout: 20_000, timeoutMsg: "the slow run never started" });
    await invoke("set_params", { values: { delay: 0, count: 60 } });
    // Sampled until the newer scene is drawn: the canceled run never shows as an error.
    let sawError = false;
    await browser.waitUntil(
      async () => {
        sawError ||= await $("#error-panel").isExisting();
        return (await stats()).instances === 60;
      },
      { timeout: 30_000, interval: 100, timeoutMsg: "the newer run never showed" },
    );
    expect(sawError).toBe(false);
  });
});
