// SPDX-License-Identifier: AGPL-3.0-or-later
// States of plan section 14 that no other core or area spec reaches (session 5A's test matrix): offline, where
// everything but the agent and downloads still works; an old parts library, whose update picks up where a dropped
// download stopped; the model's LDraw export from the Export menu; and a name a screen reader can
// say on every control. The by-hand screens spec keeps these states' pictures.
import { $, expect } from "@wdio/globals";
import { copyFileSync, existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { instances, invoke, nextScene, open, shown } from "../lib/app.mjs";
import { agentHome } from "../lib/agents.mjs";
import { dropNext } from "../lib/library-server.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = projectDir("states");
const mod = process.platform === "darwin" ? "Meta" : "Control";

/** Every control on screen with no name: no label, no text, no title. */
const unnamed = () =>
  browser.execute(() => {
    const controls = "button, a[href], input, select, textarea, [role='button'], [role='tab'], [role='slider'], [role='switch'], [role='menuitem'], [role='treeitem'], [role='checkbox'], [role='radio'], [tabindex='0']";
    const text = (ids) =>
      (ids ?? "")
        .split(/\s+/)
        .map((id) => document.getElementById(id)?.textContent ?? "")
        .join(" ");
    return [...document.querySelectorAll(controls)]
      .filter((el) => el.getClientRects().length > 0 && !el.closest("[aria-hidden='true']"))
      .filter((el) => {
        const label = el.id ? document.querySelector(`label[for="${CSS.escape(el.id)}"]`)?.textContent : "";
        const name = [el.getAttribute("aria-label"), text(el.getAttribute("aria-labelledby")), el.closest("label")?.textContent, label, el.getAttribute("title"), el.getAttribute("alt"), el.getAttribute("placeholder"), el.textContent]
          .map((s) => (s ?? "").trim())
          .find((s) => s.length > 0);
        return !name;
      })
      .map((el) => el.outerHTML.slice(0, 160));
  });

describe("states the other specs don't reach", () => {
  before(async () => {
    await open(dir);
  });

  it("offline says so, and a knob still rebuilds the model", async () => {
    await browser.execute(() => window.__emmet.offline(true));
    try {
      await expect($("#banner-offline .banner-text")).toHaveText("You're offline.");
      const before = await shown();
      const parts = (await browser.execute(() => window.__emmet.viewer().stats())).instances;
      // Width isn't one of the Cottage's knobs, so it's set as the CLI and the agent set it (14 to 20).
      await invoke("set_params", { values: { width: 18 } });
      await nextScene(before);
      expect((await browser.execute(() => window.__emmet.viewer().stats())).instances).not.toBe(parts);
    } finally {
      await browser.execute(() => window.__emmet.offline(null));
    }
    await expect($("#banner-offline")).not.toBeExisting();
  });

  // Later isn't clicked here: it holds for as long as the app runs, and every spec file shares this app.
  it("an old library says which release it is, and Update the library picks up where a dropped download stopped", async () => {
    // The shared data dir's library, labeled an older release for this test, then put back as it was.
    const json = join(process.env.EMMET_DATA_DIR, "library.json");
    copyFileSync(json, `${json}.bak`);
    try {
      const lib = JSON.parse(readFileSync(json, "utf8"));
      writeFileSync(json, JSON.stringify({ ...lib, release: "2026-06" }));
      await open(dir);
      await (await $("#banner-old-library")).waitForDisplayed({ timeout: 15_000, timeoutMsg: "the old library's banner never showed" });
      await expect($("#banner-old-library")).toHaveText(expect.stringContaining("Your parts library is 2026-06."));
      await expect($("#banner-old-library")).toHaveText(expect.stringContaining("Later"));
      await dropNext(100_000);
      await (await $("#update-library")).click();
      await expect($("#banner-old-library")).toHaveText(expect.stringContaining("The connection dropped at"));
      await expect($("#update-library")).toHaveText("Try again");
      await (await $("#update-library")).click();
      await (await $("#banner-old-library")).waitForExist({ reverse: true, timeout: 60_000, timeoutMsg: "Update the library didn't finish" });
      // The model is still the model: the update changed the library under it, not the build.
      await instances((await browser.execute(() => window.__emmet.viewer().stats())).instances);
    } finally {
      copyFileSync(`${json}.bak`, json);
    }
  });

  it("exports the model as an LDraw file into Downloads, and says so", async () => {
    await open(dir);
    await browser.keys([mod, "e"]);
    await (await $("[data-menu='mpd']")).waitForDisplayed();
    await (await $("[data-menu='mpd']")).click();
    await expect($("#toast")).toHaveText(expect.stringContaining("to Downloads"));
    const name = (await (await $("#toast")).getText()).match(/“([^”]+\.mpd)”/)?.[1];
    expect(name).toBeTruthy();
    const file = join(agentHome, "Downloads", name);
    expect(existsSync(file)).toBe(true);
    // An LDraw file another program opens: its first line names a file, and it places parts.
    const text = readFileSync(file, "utf8");
    expect(text.startsWith("0 FILE ")).toBe(true);
    expect(text.split("\n").filter((l) => l.startsWith("1 ")).length).toBeGreaterThan(10);
    await (await $("#toast [aria-label='Dismiss']")).click();
  });

  it("every control has a name a screen reader can say: the window, each panel tab, Export, and Settings", async () => {
    const found = {};
    const look = async (where) => {
      const bad = await unnamed();
      if (bad.length) found[where] = bad;
    };
    await look("the window");
    for (const tab of ["options", "checks", "parts", "steps"]) {
      await (await $(`#tab-${tab}`)).click();
      await look(`the ${tab} tab`);
    }
    await browser.keys([mod, "e"]);
    await (await $("[data-menu='csv']")).waitForDisplayed();
    await look("Export");
    await browser.keys("Escape");
    await browser.keys([mod, ","]);
    await (await $("#settings")).waitForDisplayed();
    await look("Settings");
    await browser.keys("Escape");
    expect(found).toEqual({});
  });
});
