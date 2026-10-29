// SPDX-License-Identifier: AGPL-3.0-or-later
// The frozen design (plan session 2B): every section 14 state and key flow has a screen; every screen, in both themes
// and on both platforms, is built from inventory components with no one-off styles; components.css takes every size
// and duration from tokens.css (contrast-check, in the plan job, refuses its colors); tokens.css keeps to its color
// budget and honors reduced motion; the generated icons and token lists are current; and the committed icons are the
// icon script's output. The browser audit (screen.html#audit: both sizes, contrast against what's really behind each
// text, and nothing outside the window) runs in Chrome and its result goes in the PR, as 2A's did.
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import vm from "node:vm";
import { makeIcons, SOURCES } from "../../scripts/app-icon.mjs";
import { appCss, componentTokens, generate } from "../../scripts/design-frozen.mjs";
import { ROOT, tempDir } from "./helpers.mjs";

const read = (p) => readFileSync(join(ROOT, p), "utf8");
const FROZEN = "docs/design/frozen";
const plan = read("docs/plan.html");
const s14 = plan.slice(plan.indexOf('<section id="s14">'), plan.indexOf('<section id="s15">'));

/** The frozen pages' scripts, run as the browser runs them, with a window and nothing else. */
function frozen() {
  const ctx = { window: { EMMET_ROOT: "../" } };
  vm.createContext(ctx);
  for (const f of ["docs/design/shared/data.js", "docs/design/shared/states.js", `${FROZEN}/generated.js`, `${FROZEN}/components.js`, `${FROZEN}/screens.js`]) vm.runInContext(read(f), ctx, { filename: f });
  return ctx.window;
}

test("every section 14 state and key flow has a frozen screen", () => {
  const { EMMET_FROZEN: F } = frozen();
  const para = s14.match(/States every screen designs for<\/h3>\s*<p[^>]*>([\s\S]*?)<\/p>/)[1];
  // A state's name without its note in parentheses ("build animating (with Skip)").
  const name = (s) => s.replace(/\s*\(.*$/s, "").replace(/\.$/, "").trim().toLowerCase();
  const states = para.replace(/<[^>]+>/g, "").split(" · ").map(name);
  assert.ok(states.length >= 17, `read ${states.length} states from section 14`);
  const covered = new Set(F.screens.flatMap((s) => s.covers.map(name)));
  for (const st of states) assert.ok(covered.has(st), `no frozen screen covers section 14's "${st}"`);
  const keyFlows = s14.slice(s14.indexOf("Key flows"), s14.indexOf("</ol>", s14.indexOf("Key flows")));
  const flows = [...keyFlows.matchAll(/<li><b>([^<]+)<\/b>/g)].map((m) => m[1]);
  assert.equal(flows.length, 9, "section 14's nine key flows");
  for (const name of flows) {
    const flow = F.FLOWS.find((f) => f.name.toLowerCase().startsWith(name.toLowerCase()));
    assert.ok(flow?.path.length, `no frozen flow for section 14's "${name}"`);
    for (const id of flow.path) assert.ok(F.byId[id], `${flow.name} goes through ${id}, which isn't a screen`);
  }
  // A click that moves to a screen that doesn't exist would land on "built" without a word.
  for (const s of F.screens) for (const [act, id] of Object.entries(s.on || {})) assert.ok(F.byId[id], `${s.id}: ${act} goes to ${id}, which isn't a screen`);
  const harness = read(`${FROZEN}/harness.js`);
  for (const [, act, id] of harness.match(/const GLOBAL = \{([^}]*)\}/)[1].matchAll(/(\w+): "([^"]+)"/g)) assert.ok(F.byId[id], `${act} goes to ${id}, which isn't a screen`);
});

test("every screen, in both themes and on both platforms, uses only inventory components and no one-off styles", () => {
  const w = frozen();
  const { EMMET_FROZEN: F, EMMET_GEN: GEN } = w;
  // Placement in the render's terms, a fill's share (a percent), where a floating piece sits (in tokens, as Radix will
  // place it), and a swatch's part color: nothing else, and never a pixel size or a UI color.
  const allowed = /^(--u|--v|--k|--ty|width|left|top|right|bottom)$/;
  let n = 0, styles = 0;
  for (const p of ["macos", "windows"]) for (const t of ["light", "dark"]) for (const s of F.screens) {
    w.EMMET_THEME = t;
    const html = F.compose(s, { platform: p });
    const where = `${s.id} (${t}, ${p})`;
    for (const [, c] of html.matchAll(/data-c="([^"]+)"/g)) assert.ok(GEN.tokens[c], `${where}: ${c} isn't in components.css`);
    assert.doesNotMatch(html, /data-missing="/, `${where}: an icon isn't in generated.js`);
    for (const [tag] of html.matchAll(/<\w+\s[^>]*\bstyle="[^"]*"[^>]*>/g)) {
      const cls = tag.match(/\bclass="([^"]*)"/)?.[1];
      const style = tag.match(/\bstyle="([^"]*)"/)[1];
      styles++;
      for (const decl of style.split(";").filter((d) => d.trim())) {
        const [prop, value] = decl.split(/:(.*)/s).map((x) => x.trim());
        if (prop === "background" && /\bswatch\b/.test(cls || "")) continue;
        assert.match(prop, allowed, `${where}: a one-off style: ${decl.trim()}`);
        assert.doesNotMatch(value, /\d(\.\d+)?px\b|#[0-9a-f]{3,8}\b|rgba?\(/i, `${where}: a one-off value: ${decl.trim()}`);
      }
    }
    n++;
  }
  assert.equal(n, F.screens.length * 4);
  assert.ok(styles > n, `read ${styles} inline styles`);
});

test("components.css takes every size and duration from tokens.css", () => {
  const tokens = read("apps/desktop/src/styles/tokens.css");
  const css = read(`${FROZEN}/components.css`).replace(/\/\*[\s\S]*?\*\//g, "");
  const defined = new Set([...tokens.matchAll(/(--[a-z0-9-]+)\s*:/g)].map((m) => m[1]));
  // Custom properties a component sets for itself: the viewer's fit and a point in the render.
  const own = new Set([...css.matchAll(/(--[a-z0-9-]+)\s*:/g)].map((m) => m[1]).concat(["--u", "--v", "--k", "--ty"]));
  for (const [, v] of css.matchAll(/var\((--[a-z0-9-]+)/g)) assert.ok(defined.has(v) || own.has(v), `components.css uses ${v}, which tokens.css doesn't define`);
  assert.doesNotMatch(css, /\d(\.\d+)?px\b/, "a pixel size outside tokens.css");
  // A duration of its own would move under reduced motion; every one is a --duration-* token.
  assert.doesNotMatch(css, /\d(\.\d+)?m?s\b/, "a duration outside tokens.css");
});

test("tokens.css keeps to 15 colors, defines both themes alike, and turns motion off when asked", () => {
  const tokens = read("apps/desktop/src/styles/tokens.css");
  const block = (t) => Object.fromEntries([...tokens.match(new RegExp(`\\[data-theme="${t}"\\]\\s*\\{([^}]*)\\}`))[1].matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g)].map((m) => [m[1], m[2].trim()]));
  const light = block("light");
  // A color token holds a color of its own; the hairline, scrim, and shadows are mixed from those.
  const colors = Object.entries(light).filter(([, v]) => /^#[0-9a-f]{6}$/i.test(v)).map(([k]) => k);
  assert.ok(colors.length <= 15, `${colors.length} color tokens: ${colors.join(" ")}`);
  for (const [k, v] of Object.entries(light)) if (!colors.includes(k)) assert.match(v, /var\(--/, `${k} should be mixed from the color tokens`);
  const declared = [...tokens.replace(/@media[\s\S]*$/, "").matchAll(/(--duration-[a-z0-9-]+)\s*:\s*([^;]+);/g)];
  assert.ok(declared.length >= 3, "tokens.css has its durations");
  for (const [, name, v] of declared) assert.match(v, /^\d+ms$/, `${name} is ${v}: durations are in ms`);
  const durations = declared.map(([, name, v]) => [null, name, parseInt(v, 10)]);
  const reduced = tokens.match(/@media \(prefers-reduced-motion: reduce\)\s*\{\s*:root\s*\{([^}]*)\}/)[1];
  for (const [, name, ms] of durations) if (Number(ms) > 0) assert.match(reduced, new RegExp(`${name}:\\s*0ms`), `reduced motion leaves ${name} on`);
  assert.match(reduced, /--rise:\s*0px/);
  // Subtle: nothing that moves takes longer than 300 ms (the spinner's turn aside, which reduced motion stops).
  for (const [, name, ms] of durations) if (name !== "--duration-spin") assert.ok(Number(ms) <= 300, `${name} is ${ms} ms`);
});

test("every component has an inventory entry and a specimen, and the plan's list is all there", () => {
  const ids = Object.keys(componentTokens(read(`${FROZEN}/components.css`)));
  const md = read("docs/design/components.md");
  for (const id of ids) assert.match(md, new RegExp(`^### \`${id}\``, "m"), `components.md has no entry for ${id}`);
  const specimens = read(`${FROZEN}/specimens.js`);
  // A specimen is a row in SPECS (["button", …]); the window's regions are listed in REGIONS.
  for (const id of ids) assert.ok(specimens.includes(`["${id}"`) || new RegExp(`REGIONS = \\[[^\\]]*"${id}"`).test(specimens), `components.html has no specimen of ${id}`);
  // Session 2B's list, in the inventory's ids.
  const plan2B = { button: "button", "icon button": "icon-button", slider: "slider", select: "select", "color picker": "color-picker", toggle: "switch", tabs: "tabs", tooltip: "tooltip", dialog: "dialog", menu: "menu", toast: "toast", "resizable panes": "resizable", tree: "tree", "data table": "data-table", "chat message": "chat-message", "tool card": "tool-card", "approval card": "approval-card", "finding row": "finding-row" };
  const listed = plan.match(/component inventory[^]*?Radix primitives \(D-29\): ([^.]+)\./)[1].replace(/\s*\(BOM\)|\s*\(limited to produced colors\)/g, "").split(", ").map((s) => s.trim());
  for (const name of listed) assert.ok(ids.includes(plan2B[name]), `the plan's "${name}" has no component`);
});

test("generated.js and the app's components.css are current", async () => {
  assert.equal(await generate(), read(`${FROZEN}/generated.js`), "run node scripts/design-frozen.mjs");
  assert.equal(appCss(), read("apps/desktop/src/styles/components.css"), "run node scripts/design-frozen.mjs");
});

/** What an icon file holds: a PNG's size, an .ico's sizes, an .icns's entry types. */
function iconInfo(buf, name) {
  if (name.endsWith(".png")) return `${buf.readUInt32BE(16)}x${buf.readUInt32BE(20)}`;
  if (name.endsWith(".ico")) return Array.from({ length: buf.readUInt16LE(4) }, (_, i) => buf[6 + 16 * i] || 256).sort((a, b) => a - b).join(",");
  const types = [];
  for (let o = 8; o < buf.length; o += buf.readUInt32BE(o + 4)) types.push(buf.toString("ascii", o, o + 4));
  return types.sort().join(",");
}

/** An .icns's entries by type. */
function icnsEntries(buf) {
  const out = {};
  for (let o = 8; o < buf.length; o += buf.readUInt32BE(o + 4)) out[buf.toString("ascii", o, o + 4)] = buf.subarray(o, o + buf.readUInt32BE(o + 4)).toString("base64");
  return out;
}

test("the icon script makes every platform size, and the committed icons are its output", () => {
  const out = tempDir();
  const files = makeIcons(out);
  assert.deepEqual(files.sort(), Object.values(SOURCES).flat().sort());
  const made = Object.fromEntries(files.map((f) => [f, iconInfo(readFileSync(join(out, f)), f)]));
  assert.deepEqual(made, {
    "32x32.png": "32x32", "128x128.png": "128x128", "128x128@2x.png": "256x256", "icon.png": "512x512",
    "icon.ico": "16,24,32,48,64,256",
    // 16, 32, 64, 128, 256, 512, and 1024 px, with the @2x pairs macOS asks for.
    "icon.icns": "ic07,ic08,ic09,ic10,ic11,ic12,ic13,ic14,il32,is32,l8mk,s8mk",
  });
  // The committed icons are this output, so an SVG changed without rerunning the script fails here. The .icns writes
  // its entries in any order, so it's compared entry by entry; the rest byte for byte.
  const dir = join(ROOT, "apps/desktop/src-tauri/icons");
  assert.deepEqual(readdirSync(dir).sort(), files.sort());
  for (const f of files) {
    const [mine, theirs] = [readFileSync(join(dir, f)), readFileSync(join(out, f))];
    if (f.endsWith(".icns")) assert.deepEqual(icnsEntries(mine), icnsEntries(theirs), `${f}: run node scripts/app-icon.mjs`);
    else assert.ok(mine.equals(theirs), `${f} isn't the script's output: run node scripts/app-icon.mjs`);
  }
  const conf = JSON.parse(read("apps/desktop/src-tauri/tauri.conf.json"));
  for (const f of conf.bundle.icon) assert.ok(files.includes(f.replace("icons/", "")), `tauri.conf.json bundles ${f}, which the script doesn't make`);
});
