// SPDX-License-Identifier: AGPL-3.0-or-later
// Opening an existing LDraw file (plan section 14's key flow, session 4A Done when 3): Open… (one file picker,
// docs/decisions.md 2026-10-28) takes an .ldr, which becomes an imported project in Documents/Emmet with the banner and
// Make it parametric, its steps in the Steps tab, and no knobs yet; it takes a project's emmet.toml too, which opens its
// folder.
import { $, $$, expect } from "@wdio/globals";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { open, screenshots } from "../lib/app.mjs";
import { agentHome, scenario } from "../lib/agents.mjs";
import { repo } from "../lib/paths.mjs";
import { pickNext, projectDir } from "../lib/projects.mjs";

const mod = process.platform === "darwin" ? "Meta" : "Control";

describe("open an LDraw file", () => {
  it("imports an .ldr as a project with its steps and no knobs", async () => {
    await open(projectDir("import"));
    pickNext(join(repo, "tests", "fixtures", "projects", "dsl-v1", "wagon.ldr"));
    await browser.keys([mod, "o"]);
    const root = join(agentHome, "Documents", "Emmet", "wagon (imported)");
    await browser.waitUntil(() => browser.execute((r) => window.__emmet.project() === r && window.__emmet.shown() >= 1, root), { timeout: 30_000, timeoutMsg: "the import never showed" });
    expect(existsSync(join(root, "wagon.ldr"))).toBe(true);
    await expect($("#banner-imported .banner-text")).toHaveText(expect.stringMatching(/^Opened wagon\.ldr: \d+ parts? in \d+ steps?\.$/));
    await expect($("#make-parametric")).toBeDisplayed();
    await expect($("#tweaks-title")).not.toBeExisting();
    await (await $("#tab-steps")).click();
    expect((await $$("#steps-tree [data-n]")).length).toBeGreaterThan(0);
    await screenshots("import", { baseline: true });
    // Make it parametric asks the agent, which starts on it: the chat shows the plain words, sent from the imported file.
    scenario("claude", "plain-turn");
    await (await $("#make-parametric")).click();
    await browser.waitUntil(() => browser.execute(() => !window.__emmet.chat().running && window.__emmet.chat().items > 0), { timeout: 30_000, timeoutMsg: "the agent never took the request" });
    if (!(await (await $("#chat-log")).isExisting())) await (await $("#history-toggle")).click();
    await expect($("#chat-log")).toHaveText(expect.stringContaining("Sent from the imported file"), { wait: 10_000 });
    await (await $("#history-toggle")).click();
  });

  it("opens a project from its emmet.toml", async () => {
    pickNext(join(projectDir("import"), "emmet.toml"));
    await browser.keys([mod, "o"]);
    await browser.waitUntil(() => browser.execute(() => window.__emmet.project()?.endsWith("import")), { timeout: 30_000 });
    await expect($("#banner-imported")).not.toBeExisting();
  });
});
