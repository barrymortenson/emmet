// SPDX-License-Identifier: AGPL-3.0-or-later
// The E2E hello-world (ADR 0002), on 4A's window: with the parts and Python set up and no project, the real app starts
// at first run's Start (the Cottage or a new model from a prompt), and it's interactive within section 14's budget
// (4A What good looks like 2; this spec logs the time on every run, "startup: interactive N ms").
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

  // The viewer turns the build animation off under the system's Reduce motion (plan section 13), and then the animate
  // and perf specs wait for a build that never plays. CI's mac-e2e job turns it off first ("Motion on"), and its
  // windows-e2e job turns window animations on, which Windows Server leaves off ("Window animations on").
  it("runs on a machine with Reduce motion off", async () => {
    const reduced = await browser.execute(() => matchMedia("(prefers-reduced-motion: reduce)").matches);
    console.log(`reduce motion: ${reduced ? "on" : "off"}`);
    const where = process.platform === "darwin" ? "System Settings → Accessibility → Display" : "Settings → Accessibility → Visual effects → Animation effects";
    // Windows only says so until a hosted run has shown the job's step turns it off (5B).
    if (reduced && process.platform === "win32") return console.log("reduce motion is on: the animation specs will fail");
    if (reduced) throw new Error(`this machine has Reduce motion on (${where}), so the build animation never plays: turn it off, then run again`);
  });

  // WebKit sometimes draws the window at 30 fps on this Mac (a throttle a restart clears, 2026-11-02), and then the
  // chat and keyboard specs time out one by one. hello runs first, alone (just e2e), so say it here, in a second. A
  // hosted runner's shared VM draws when it can; only this Mac is held to it.
  it("draws at the display's rate", async () => {
    const ms = await browser.execute(
      () =>
        new Promise((resolve) => {
          let n = 0;
          let t0 = 0;
          const tick = (t) => {
            if (n === 0) t0 = t;
            if (++n < 31) requestAnimationFrame(tick);
            else resolve((t - t0) / 30);
          };
          requestAnimationFrame(tick);
        }),
    );
    console.log(`frame interval: ${ms.toFixed(1)} ms with nothing drawn`);
    if (process.env.RUNNER_ENVIRONMENT === "github-hosted") return;
    if (ms >= 25) {
      throw new Error(`WebKit draws a frame every ${ms.toFixed(1)} ms, a 30 fps throttle or worse: restart this Mac, then run again`);
    }
  });
});
