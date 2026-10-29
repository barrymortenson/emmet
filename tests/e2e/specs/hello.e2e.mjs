// SPDX-License-Identifier: AGPL-3.0-or-later
// The E2E hello-world (ADR 0002), on 4A's window: with the parts and Python set up and no project, the real app starts
// at first run's Start (the Cottage or a new model from a prompt), and it's interactive within section 14's budget
// (4A What good looks like 2; the bench's table prints the time on every run).
import { $, expect } from "@wdio/globals";

describe("the app window", () => {
  it("starts at Start with no project", async () => {
    await (await $("#first-run")).waitForDisplayed({ timeout: 30_000 });
    await expect($("#first-run h1")).toHaveText("What do you want to build?");
    await expect($("#start-cottage")).toBeEnabled();
    await expect($("#start-prompt")).toBeEnabled();
    expect(await browser.execute(() => window.__emmet.project())).toBe(null);
  });

  it("was interactive soon after it started", async () => {
    let ms = null;
    await browser.waitUntil(async () => (ms = await browser.execute(() => window.__emmet.startup())) != null, { timeout: 10_000 });
    // The E2E app is a debug build and this is its first, cold start; the budget is for a warm release build, timed by
    // hand on this Mac for the PR. This catches a startup that has gone badly wrong.
    console.log(`startup: interactive ${ms} ms after the process started`);
    expect(ms).toBeLessThan(10_000);
  });
});
