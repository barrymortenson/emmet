// SPDX-License-Identifier: AGPL-3.0-or-later
// Keyboard reach (docs/design/keyboard.md, session 4A Done when 5): each shortcut the map lists that works without an
// agent's turn does its action, Esc closes what's open, F6 moves between the regions, and Tab shows a visible focus
// ring on everything it reaches. The chat's own keys (Enter, ⇧↩, ⌘↩, a card's 1–5) are 4B's; the viewer's orbit and
// zoom keys are 4C's; ⌥⌘Z needs 4B's versions before each turn.
import { $, expect } from "@wdio/globals";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { camera, nextScene, open, shown } from "../lib/app.mjs";
import { pickNext, projectDir } from "../lib/projects.mjs";

const dir = projectDir("keyboard");
const mod = process.platform === "darwin" ? "Meta" : "Control";
const active = () => browser.execute(() => document.activeElement?.id || document.activeElement?.className || "");
const params = () => JSON.parse(readFileSync(join(dir, "params.json"), "utf8"));

/** Presses `keys` with focus on the viewer, outside any text field. */
async function press(...keys) {
  await browser.execute(() => document.getElementById("viewer").focus());
  await browser.keys(keys);
}

describe("the keyboard map", () => {
  before(async () => {
    await open(dir);
  });

  it("⌘1–⌘4 pick the panel's tabs", async () => {
    for (const [i, tab] of ["options", "checks", "parts", "steps"].entries()) {
      await press(mod, String(i + 1));
      await expect($(`#tab-${tab}`)).toHaveAttribute("aria-selected", "true");
    }
  });

  it("1–6 pick the views, and F frames what's selected", async () => {
    const seen = [];
    for (const k of ["1", "2", "3", "4", "5", "6"]) {
      await press(k);
      seen.push(JSON.stringify(await camera()));
    }
    expect(new Set(seen).size).toBe(6);
    await press("1");
    await expect($("#view-iso")).toHaveAttribute("data-state", "on");
    await press(mod, "3");
    await (await $("#parts-list tbody tr")).click();
    const before = JSON.stringify(await camera());
    await press("f");
    await browser.waitUntil(async () => JSON.stringify(await camera()) !== before, { timeout: 5000, timeoutMsg: "F didn't frame the selection" });
  });

  it("[ and ] step through the steps, opening Steps", async () => {
    await press(mod, "1");
    await press("[");
    await expect($("#tab-steps")).toHaveAttribute("aria-selected", "true");
    const head = await (await $("#steps-head")).getText();
    const [, n, total] = head.match(/Step (\d+) of (\d+)/).map(Number);
    expect(n).toBe(total - 1);
    await press("]");
    await expect($("#steps-head")).toHaveText(`Step ${total} of ${total}`);
  });

  it("a key that types a character doesn't fire in a text field", async () => {
    await press(mod, "1");
    await (await $("#settings-change")).click();
    const field = await $("[aria-label='Your own style']");
    await field.click();
    await browser.keys(["2", "f"]);
    await expect(field).toHaveValue("2f");
    await expect($("#view-iso")).toHaveAttribute("data-state", "on");
  });

  it("⌘Z and ⇧⌘Z undo and redo a knob's draft", async () => {
    const before = await shown();
    await (await $("#param-weathering-thumb")).click();
    await browser.keys(["ArrowRight", "ArrowRight", "ArrowRight"]);
    await nextScene(before);
    expect(params().weathering).toBe(3);
    await browser.keys([mod, "z"]);
    await browser.waitUntil(() => params().weathering === 2, { timeout: 10_000, timeoutMsg: "⌘Z didn't undo the last step" });
    await browser.keys([mod, "Shift", "z"]);
    await browser.waitUntil(() => params().weathering === 3, { timeout: 10_000, timeoutMsg: "⇧⌘Z didn't redo it" });
    await (await $("#knobs-reset")).click();
  });

  it("⌘S, ⌘E, ⌘,, ⌘/, and ⌘N open their surfaces, and Esc closes each", async () => {
    for (const [key, sel] of [
      ["s", "#pop-save"],
      ["e", "[data-menu='csv']"],
      [",", "#settings"],
      ["/", "#shortcuts"],
      ["n", "#first-run"],
    ]) {
      await press(mod, key);
      await (await $(sel)).waitForDisplayed({ timeoutMsg: `⌘${key} didn't open ${sel}` });
      await browser.keys("Escape");
      await (await $(sel)).waitForDisplayed({ reverse: true, timeoutMsg: `Esc didn't close ${sel}` });
    }
  });

  it("⌘O opens what the file picker returns", async () => {
    pickNext(join(dir, "emmet.toml"));
    const before = await shown();
    await press(mod, "o");
    await browser.waitUntil(() => browser.execute(() => window.__emmet.shown()).then((s) => s !== before), { timeout: 30_000 });
    expect(await browser.execute(() => window.__emmet.project())).toMatch(/keyboard$/);
  });

  it("F6 moves between the viewer, the chat, and the panel", async () => {
    await browser.execute(() => document.getElementById("viewer").focus());
    await browser.keys("F6");
    expect(await active()).toBe("dock");
    await browser.keys("F6");
    expect(await active()).toBe("panel");
    await browser.keys("F6");
    expect(await active()).toBe("viewer");
  });

  it("every control Tab reaches shows a visible focus ring", async () => {
    // A synthetic Tab doesn't move a webview's focus, so each control in the Tab order is focused the way Tab would.
    // WebKit matches :focus only while the window is the active one, which an E2E run beside other work (or a CI
    // desktop) can't count on, so the rings are read from the window's own CSS: each :focus and :focus-visible rule
    // gets a copy right after it that matches [data-e2e-focus], and the control focus lands on is marked in turn.
    await press(mod, "1");
    const rings = await browser.execute(() => {
      const focus = /:focus(-visible)?(?![-\w])/g;
      const copy = (owner) => {
        const rules = owner.cssRules;
        for (let i = rules.length - 1; i >= 0; i--) {
          const r = rules[i];
          if (r instanceof CSSStyleRule && focus.test(r.selectorText)) owner.insertRule(`${r.selectorText.replace(focus, "[data-e2e-focus]")} { ${r.style.cssText} }`, i + 1);
          else if (!(r instanceof CSSStyleRule) && r.cssRules) copy(r);
          focus.lastIndex = 0;
        }
      };
      for (const sheet of document.styleSheets) copy(sheet);
      const tabbable = [...document.querySelectorAll('button, input, textarea, select, [tabindex]:not([tabindex="-1"]), a[href]')].filter(
        (el) => el.tabIndex >= 0 && !el.disabled && el.getClientRects().length > 0 && getComputedStyle(el).visibility !== "hidden",
      );
      return tabbable.flatMap((el) => {
        el.focus();
        // Radix's tab list hands focus to its open tab, where Tab lands: that's the control whose ring shows.
        const at = document.activeElement;
        if (!at || (at !== el && !el.contains(at))) return [];
        at.setAttribute("data-e2e-focus", "");
        const s = getComputedStyle(at);
        const ring = s.outlineStyle !== "none" && parseFloat(s.outlineWidth) >= 2;
        at.removeAttribute("data-e2e-focus");
        return [{ el: at.id || at.getAttribute("aria-label") || at.textContent?.trim().slice(0, 30), ring }];
      });
    });
    expect(rings.filter((r) => !r.ring).map((r) => r.el)).toEqual([]);
    const reached = rings.map((r) => r.el);
    for (const id of ["project-chip", "param-weathering-thumb", "param-chimney-switch", "export-button"]) expect(reached).toContain(id);
    // The tab list is one stop in the Tab order: its open tab (arrows move between tabs).
    expect(reached.some((id) => String(id).startsWith("tab-"))).toBe(true);
    // The window's controls: the title row, the chat's agent cards (more when an earlier spec chose Codex), the panel.
    expect(rings.length).toBeGreaterThanOrEqual(18);
  });
});
