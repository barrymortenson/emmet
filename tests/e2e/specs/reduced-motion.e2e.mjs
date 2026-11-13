// SPDX-License-Identifier: AGPL-3.0-or-later
// With the system's animations off (Windows: Settings → Accessibility → Visual effects → Animation effects; macOS:
// Reduce motion), the app's own durations are 0 ms (tokens.css) and the build animation is off. Menus, popovers, and
// dialogs must still open and close: a layer that waits for an animation's end to go away would wait forever
// (session 5B: on Windows Server, which ships with animations off, an open popover stayed over the window for the
// rest of the run). CI's windows-e2e job runs this before it turns animations on; on a machine with motion on, the
// test skips itself, since no page can ask for reduced motion.
import { $, expect } from "@wdio/globals";
import { layerState, open, stats } from "../lib/app.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = projectDir("reduced-motion");
const mod = process.platform === "darwin" ? "Meta" : "Control";

describe("with the system's animations off", () => {
  it("menus, popovers, and dialogs open and close, and the build doesn't animate", async function () {
    const reduced = await browser.execute(() => matchMedia("(prefers-reduced-motion: reduce)").matches);
    if (!reduced) {
      console.log("this machine has motion on: nothing to check");
      this.skip();
    }
    await open(dir);
    for (const [key, sel] of [
      ["s", "#pop-save"],
      ["e", "[data-menu='csv']"],
      [",", "#settings"],
      ["/", "#shortcuts"],
      ["e", "[data-menu='csv']"],
    ]) {
      await browser.execute(() => document.getElementById("viewer").focus());
      await browser.keys([mod, key]);
      await (await $(sel)).waitForDisplayed({ timeoutMsg: `${key} didn't open ${sel}` });
      await browser.keys("Escape");
      const gone = await (await $(sel)).waitForExist({ reverse: true, timeout: 5_000 }).then(() => true, () => false);
      if (!gone) throw new Error(`Esc didn't close ${sel}: ${JSON.stringify(await layerState(sel))}`);
    }
    // Replay build puts the model up at once.
    const animating = await browser.execute(() => {
      const v = window.__emmet.viewer();
      v.replayBuild();
      return v.stats().animating;
    });
    expect(animating).toBe(0);
    expect((await stats()).instances).toBeGreaterThan(100);
  });
});
