#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// WCAG 2.2 AA contrast for the design mockups (plan session 2A). Each stylesheet keeps its colors in two token blocks,
// [data-theme="light"] and [data-theme="dark"], named by role:
//   --bg-*  surfaces (must be opaque)       --fg-*  text: 4.5:1 against every --bg-*
//   --ui-*  icons and control edges: 3:1    --fill-X  a filled control: 3:1 against every --bg-*
//   --on-X  text on --fill-X: 4.5:1         anything else (--line, --shadow) is decorative and not checked
// A prefix keeps sets apart (--p-bg-page pairs only with --p-fg-*). Outside the token blocks a stylesheet may not
// write a color, and the mockups' scripts and pages may not either, so every color on screen is a checked token.
// Usage: node scripts/contrast-check.mjs [stylesheet ...]  (default: docs/design's)
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

const ROOT = join(import.meta.dirname, "..");
const DESIGN = join(ROOT, "docs/design");

/** #rgb, #rgba, #rrggbb, #rrggbbaa, or rgb()/rgba() → [r, g, b, a]. */
export function parseColor(s) {
  s = s.trim().toLowerCase();
  let m = s.match(/^#([0-9a-f]{3,8})$/);
  if (m) {
    let h = m[1];
    if (h.length === 3 || h.length === 4) h = [...h].map((c) => c + c).join("");
    if (h.length !== 6 && h.length !== 8) return null;
    const n = (i) => parseInt(h.slice(i, i + 2), 16);
    return [n(0), n(2), n(4), h.length === 8 ? n(6) / 255 : 1];
  }
  m = s.match(/^rgba?\(([^)]+)\)$/);
  if (m) {
    const p = m[1].split(/[\s,/]+/).filter(Boolean).map((x) => (x.endsWith("%") ? parseFloat(x) / 100 : Number(x)));
    if (p.length < 3 || p.some(Number.isNaN)) return null;
    return [p[0], p[1], p[2], p[3] ?? 1];
  }
  return null;
}

function luminance([r, g, b]) {
  const lin = (v) => ((v /= 255) <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

/** Contrast ratio; a translucent foreground is composited over the background first. */
export function contrast(fg, bg) {
  const a = fg[3];
  const top = a < 1 ? [0, 1, 2].map((i) => fg[i] * a + bg[i] * (1 - a)) : fg;
  const [l1, l2] = [luminance(top), luminance(bg)].sort((x, y) => y - x);
  return (l1 + 0.05) / (l2 + 0.05);
}

const BLOCK = /\[data-theme="(light|dark)"\]\s*\{([^}]*)\}/g;
const ROLE = /^--(?:([a-z]+)-)??(bg|fg|ui|fill|on)-([a-z0-9-]+)$/;
const COLOR_LITERAL = /#[0-9a-fA-F]{3,8}\b|\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch|color-mix)\(/;

/** Checks one stylesheet's text; returns { themes: {light: n, dark: n}, worst, errors }. */
export function checkCss(css, name = "stylesheet") {
  const errors = [];
  const themes = {};
  for (const [, theme, body] of css.matchAll(BLOCK)) {
    const tokens = themes[theme] ||= {};
    for (const [, k, v] of body.matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) tokens[k] = v.trim();
  }
  if (!themes.light || !themes.dark) errors.push(`${name}: needs both a [data-theme="light"] and a [data-theme="dark"] block`);
  const counts = {};
  let worst = null;
  for (const theme of Object.keys(themes)) {
    const sets = {};
    for (const [k, v] of Object.entries(themes[theme])) {
      const m = k.match(ROLE);
      if (!m) continue;
      const c = parseColor(v);
      if (!c) { errors.push(`${name} ${theme}: ${k} isn't a color I can read (${v})`); continue; }
      const set = (sets[m[1] || ""] ||= { bg: {}, fg: {}, ui: {}, fill: {}, on: {} });
      set[m[2]][m[3]] = c;
    }
    let n = 0;
    const need = (fgName, fg, bgName, bg, min) => {
      n++;
      const r = contrast(fg, bg);
      if (!worst || r - min < worst.ratio - worst.min) worst = { theme, fg: fgName, bg: bgName, ratio: r, min };
      if (r < min) errors.push(`${name} ${theme}: ${fgName} on ${bgName} is ${r.toFixed(2)}:1, needs ${min}:1`);
    };
    for (const [prefix, set] of Object.entries(sets)) {
      const t = (role, id) => `--${prefix ? prefix + "-" : ""}${role}-${id}`;
      for (const [id, c] of Object.entries(set.bg)) if (c[3] < 1) errors.push(`${name} ${theme}: ${t("bg", id)} must be opaque`);
      for (const [bgId, bg] of Object.entries(set.bg)) {
        for (const [id, c] of Object.entries(set.fg)) need(t("fg", id), c, t("bg", bgId), bg, 4.5);
        for (const [id, c] of Object.entries(set.ui)) need(t("ui", id), c, t("bg", bgId), bg, 3);
        for (const [id, c] of Object.entries(set.fill)) need(t("fill", id), c, t("bg", bgId), bg, 3);
      }
      for (const [id, c] of Object.entries(set.on)) {
        if (!set.fill[id]) errors.push(`${name} ${theme}: ${t("on", id)} has no ${t("fill", id)}`);
        else need(t("on", id), c, t("fill", id), set.fill[id], 4.5);
      }
    }
    counts[theme] = n;
  }
  if (themes.light && themes.dark) {
    const a = Object.keys(themes.light), b = Object.keys(themes.dark);
    for (const k of a.filter((k) => !b.includes(k))) errors.push(`${name}: ${k} is in light but not dark`);
    for (const k of b.filter((k) => !a.includes(k))) errors.push(`${name}: ${k} is in dark but not light`);
  }
  // Outside the token blocks: no color literals, and no fractional opacity (it would dim text past the check).
  const rest = css.replace(BLOCK, "").replace(/\/\*[\s\S]*?\*\//g, "");
  rest.split("\n").forEach((line) => {
    if (COLOR_LITERAL.test(line)) errors.push(`${name}: a color outside the tokens: ${line.trim().slice(0, 90)}`);
    const o = line.match(/(?:^|[;{\s])opacity\s*:\s*([\d.]+)/);
    if (o && Number(o[1]) > 0 && Number(o[1]) < 1) errors.push(`${name}: opacity ${o[1]} dims text past the check: ${line.trim().slice(0, 90)}`);
  });
  return { themes: counts, worst, errors };
}

/** A script or page may only take colors from tokens: no literals (data.js's part colors are content, not UI). */
export function checkSource(text, name) {
  const errors = [];
  // Prose that names a color in <code> (the brand page lists its hex values) isn't a color on screen.
  text.replace(/\/\*[\s\S]*?\*\//g, "").replace(/<code>[^<]*<\/code>/g, "").split("\n").forEach((line) => {
    if (/^\s*\/\//.test(line)) return;
    if (COLOR_LITERAL.test(line)) errors.push(`${name}: a color outside the tokens: ${line.trim().slice(0, 90)}`);
  });
  return errors;
}

function designFiles() {
  const css = [join(DESIGN, "shared/proto.css")];
  const src = [join(DESIGN, "shared/ui.js"), join(DESIGN, "shared/proto.js"), join(DESIGN, "review.html"), join(DESIGN, "review-data.js"), join(DESIGN, "brand/index.html")];
  const dirs = join(DESIGN, "directions");
  for (const d of existsSync(dirs) ? readdirSync(dirs).sort() : []) {
    css.push(join(dirs, d, "style.css"));
    src.push(join(dirs, d, "app.js"), join(dirs, d, "index.html"));
  }
  for (const f of ["brand/palette.css", "review.css"]) css.push(join(DESIGN, f));
  return { css: css.filter(existsSync), src: src.filter(existsSync) };
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const args = process.argv.slice(2);
  const { css, src } = args.length ? { css: args, src: [] } : designFiles();
  let failed = 0;
  for (const file of css) {
    const name = relative(ROOT, file);
    const r = checkCss(readFileSync(file, "utf8"), name);
    for (const e of r.errors) console.error(`error: ${e}`);
    failed += r.errors.length;
    const w = r.worst;
    console.log(`${r.errors.length ? "FAIL" : "ok  "} ${name}: ${r.themes.light ?? 0} light + ${r.themes.dark ?? 0} dark pairs` +
      (w ? `; tightest ${w.fg} on ${w.bg} (${w.theme}) ${w.ratio.toFixed(2)}:1, needs ${w.min}:1` : ""));
  }
  for (const file of src) {
    const errors = checkSource(readFileSync(file, "utf8"), relative(ROOT, file));
    for (const e of errors) console.error(`error: ${e}`);
    failed += errors.length;
  }
  if (src.length) console.log(`${src.length} scripts and pages: ${failed ? "see above" : "no colors outside the tokens"}`);
  if (failed) {
    console.error(`contrast-check: ${failed} problem${failed > 1 ? "s" : ""}`);
    process.exit(1);
  }
  console.log("contrast-check: ok");
}
