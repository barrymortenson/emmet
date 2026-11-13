// SPDX-License-Identifier: AGPL-3.0-or-later
// Keyboard reach (docs/design/keyboard.md, session 4A Done when 5): each shortcut the map lists that works without an
// agent's turn does its action, Esc closes what's open, F6 moves between the regions, and Tab shows a visible focus
// ring on everything it reaches. The chat's own keys (⌘L, Enter, Esc, ⌥⌘Z) are in chat.e2e.mjs (4B); the viewer's orbit
// and zoom keys are 4C's.
import { $, expect } from "@wdio/globals";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { camera, escape, nextScene, open, shown } from "../lib/app.mjs";
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

  it("a double click on the title row maximizes the window, and another restores it", async () => {
    // Tauri's drag region turns a double click into its own maximize command, which the capability has to allow (4D).
    // macOS zooms on the second click's mouseup, as its own title bars do; Windows on its mousedown.
    const maximized = () => browser.execute(() => window.__TAURI_INTERNALS__.invoke("plugin:window|is_maximized", { label: "main" }));
    const doubleClick = () =>
      browser.execute(() => {
        const bar = document.querySelector(".titlebar");
        const r = bar.getBoundingClientRect();
        const at = { bubbles: true, cancelable: true, button: 0, detail: 2, clientX: r.left + 8, clientY: r.top + r.height / 2 };
        bar.dispatchEvent(new MouseEvent("mousedown", at));
        bar.dispatchEvent(new MouseEvent("mouseup", at));
      });
    const was = await maximized();
    await doubleClick();
    await browser.waitUntil(async () => (await maximized()) !== was, { timeout: 5_000, timeoutMsg: "the double click didn't maximize the window" });
    await doubleClick();
    await browser.waitUntil(async () => (await maximized()) === was, { timeout: 5_000, timeoutMsg: "the second double click didn't restore it" });
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
    // Windows' redo is Ctrl+Y (keyboard.md).
    await browser.keys(process.platform === "win32" ? [mod, "y"] : [mod, "Shift", "z"]);
    await browser.waitUntil(() => params().weathering === 3, { timeout: 10_000, timeoutMsg: "⇧⌘Z (Ctrl+Y on Windows) didn't redo it" });
    await (await $("#knobs-reset")).click();
  });

  it("⌘S, ⌘E, ⌘,, ⌘/, and ⌘N open their surfaces, and Esc closes each", async () => {
    for (const [key, sel] of [
      ["s", "#pop-save"],
      ["e", "[data-menu='csv']"],
      [",", "#settings"],
      ["/", "#shortcuts"],
      ["n", "#new-model"],
    ]) {
      await press(mod, key);
      await (await $(sel)).waitForDisplayed({ timeoutMsg: `⌘${key} didn't open ${sel}` });
      await escape(sel);
      try {
        await (await $(sel)).waitForDisplayed({ reverse: true, timeoutMsg: `Esc didn't close ${sel}` });
      } catch (e) {
        // What the page thinks is open, for a runner where it stays up.
        const state = await browser.execute((q) => {
          const el = document.querySelector(q);
          const layer = el?.closest("[data-state]");
          const style = layer && getComputedStyle(layer);
          return {
            state: layer?.dataset.state,
            animation: style && `${style.animationName} ${style.animationDuration} ${style.animationPlayState}`,
            running: document.getAnimations().map((a) => `${a.animationName ?? a.constructor.name}:${a.playState}`),
            reduced: matchMedia("(prefers-reduced-motion: reduce)").matches,
            visibility: document.visibilityState,
            focus: document.hasFocus(),
            active: document.activeElement?.outerHTML.slice(0, 120),
            layers: [...document.querySelectorAll("[data-radix-popper-content-wrapper] > *")].map((n) => `${n.id || n.className}:${n.dataset.state}`),
          };
        }, sel);
        throw new Error(`${e.message}: ${JSON.stringify(state)}`, { cause: e });
      }
    }
  });

  it("⌘O opens what the file picker returns", async () => {
    pickNext(join(dir, "emmet.toml"));
    const before = await shown();
    await press(mod, "o");
    await browser.waitUntil(() => browser.execute(() => window.__emmet.shown()).then((s) => s !== before), { timeout: 30_000 });
    expect(await browser.execute(() => window.__emmet.project())).toMatch(/keyboard$/);
  });

  it("Windows: the title row's menus open, with the same actions", async function () {
    if (process.platform !== "win32") this.skip();
    // Radix opens a menu on pointerdown, which the embedded driver doesn't send: focus, then Enter.
    await browser.execute(() => [...document.querySelectorAll(".menubar-trigger")].find((t) => t.textContent === "File").focus());
    await browser.keys("Enter");
    await (await $("[data-menu='new']")).waitForDisplayed({ timeoutMsg: "File didn't open" });
    // File lists this project under Open recent, and ends with Settings and Exit (keyboard.md's Windows column).
    await expect($("[data-menu='recent:0']")).toHaveText("keyboard");
    await expect($("[data-menu='exit'] .menu-key")).toHaveText("Alt+F4");
    await browser.keys("Escape");
  });

  it("Windows: F11 and View's Full screen fill the screen, and again go back", async function () {
    if (process.platform !== "win32") this.skip();
    const full = () => browser.execute(() => window.__TAURI_INTERNALS__.invoke("plugin:window|is_fullscreen", { label: "main" }));
    expect(await full()).toBe(false);
    await press("F11");
    await browser.waitUntil(full, { timeout: 5_000, timeoutMsg: "F11 didn't go full screen" });
    await browser.execute(() => [...document.querySelectorAll(".menubar-trigger")].find((t) => t.textContent === "View").focus());
    await browser.keys("Enter");
    const item = await $("[data-menu='fullscreen']");
    await item.waitForDisplayed({ timeoutMsg: "View didn't open" });
    await expect(item.$(".menu-key")).toHaveText("F11");
    await browser.execute((el) => el.focus(), item);
    await browser.keys("Enter");
    await browser.waitUntil(async () => !(await full()), { timeout: 5_000, timeoutMsg: "Full screen didn't go back" });
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
    // ⌘O just opened the project again: its knobs show once the model's been read.
    await (await $("#param-weathering-thumb")).waitForDisplayed({ timeout: 30_000, timeoutMsg: "the knobs never showed" });
    const rings = await browser.execute(() => {
      const focus = /:focus(-visible)?(?![-\w])/g;
      const within = /:focus-within/g;
      const copy = (owner) => {
        const rules = owner.cssRules;
        for (let i = rules.length - 1; i >= 0; i--) {
          const r = rules[i];
          if (r instanceof CSSStyleRule && focus.test(r.selectorText)) owner.insertRule(`${r.selectorText.replace(focus, "[data-e2e-focus]")} { ${r.style.cssText} }`, i + 1);
          else if (r instanceof CSSStyleRule && within.test(r.selectorText)) owner.insertRule(`${r.selectorText.replace(within, "[data-e2e-within]")} { ${r.style.cssText} }`, i + 1);
          else if (!(r instanceof CSSStyleRule) && r.cssRules) copy(r);
          focus.lastIndex = 0;
          within.lastIndex = 0;
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
        let ring = s.outlineStyle !== "none" && parseFloat(s.outlineWidth) >= 2;
        at.removeAttribute("data-e2e-focus");
        // A field that shows focus on its box, as the composer does (its border turns the accent color, :focus-within).
        // The box's own border is read with the field blurred: in an active window :focus-within already matches, and
        // the marked copy would change nothing.
        at.blur();
        for (let box = at.parentElement; !ring && box; box = box.parentElement) {
          const before = getComputedStyle(box).borderColor;
          box.setAttribute("data-e2e-within", "");
          ring = getComputedStyle(box).borderColor !== before;
          box.removeAttribute("data-e2e-within");
        }
        return [{ el: at.id || at.getAttribute("aria-label") || at.textContent?.trim().slice(0, 30), ring }];
      });
    });
    expect(rings.filter((r) => !r.ring).map((r) => r.el)).toEqual([]);
    const reached = rings.map((r) => r.el);
    for (const id of ["project-chip", "agent-picker", "start-fresh", "chat-input", "model-chip", "param-weathering-thumb", "param-roof-switch", "export-button"]) expect(reached).toContain(id);
    // The tab list is one stop in the Tab order: its open tab (arrows move between tabs).
    expect(reached.some((id) => String(id).startsWith("tab-"))).toBe(true);
    // The window's controls: the title row, the dock (the picker, the plan badge, Start fresh, the composer), the panel.
    expect(rings.length).toBeGreaterThanOrEqual(18);
  });
});
