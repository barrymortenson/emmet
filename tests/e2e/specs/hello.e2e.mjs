// SPDX-License-Identifier: AGPL-3.0-or-later
// The E2E hello-world (ADR 0002), on 1B's window: the real app starts with no project, says so, and shows its version.
// The start screen also sets up the agents (1F); with both fake CLIs there, each says it's ready.
import { $, expect } from "@wdio/globals";
import { screenshots } from "../lib/app.mjs";

describe("the app window", () => {
  it("starts with no project and shows its version", async () => {
    await expect($("h1")).toHaveText("Emmet");
    await expect($("#status")).toHaveText("No project open");
    await expect($(".versions")).toHaveText(expect.stringContaining("Emmet 0.1.0"));
    await expect($(".empty")).toBeDisplayed();
  });

  it("offers agent setup on the start screen", async () => {
    await expect($("#agents")).toBeDisplayed();
    await expect($("#ready-claude")).toHaveText(expect.stringContaining("Ready"));
    await expect($("#ready-codex")).toHaveText(expect.stringContaining("Ready"));
    await screenshots("start");
  });
});
