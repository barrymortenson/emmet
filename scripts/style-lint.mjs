#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// No one-off styles (plan session 4A, What good looks like): every color and pixel size in the app's stylesheets and
// its components' inline styles comes from apps/desktop/src/styles/tokens.css. tokens.css defines them, and
// approved.css keeps OpenAI's own colors and sizes for its Continue with ChatGPT button (components.md, "Kept as the
// provider requires"). A value computed from data (a part's own color, a dragged width) isn't a literal, so it passes.
// Usage: node scripts/style-lint.mjs [src dir]  (default: apps/desktop/src)
import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

const EXEMPT = new Set(["styles/tokens.css", "styles/approved.css"]);
const COLOR_FN = /\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch)\(/i;
const HEX = /#[0-9a-f]{3,8}\b/i;
const PX = /(?<![\w$}-])\d*\.?\d+px\b/;
const NAMED = /:\s*(?:[^;{}]*\s)?(?:white|black|red|green|blue|gray|grey|orange|yellow|purple|pink|brown|silver|navy)\b/i;

function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walk(join(dir, e.name)) : [join(dir, e.name)]));
}

/** The problems in one file's text: [line, what]. */
export function lint(name, text) {
  const out = [];
  const lines = text.split("\n");
  if (name.endsWith(".css")) {
    lines.forEach((l, i) => {
      // Comments say what a rule does; they aren't styles.
      const code = l.replace(/\/\*.*?\*\//g, "");
      if (HEX.test(code) || COLOR_FN.test(code) || NAMED.test(code)) out.push([i + 1, "a raw color"]);
      if (PX.test(code)) out.push([i + 1, "a pixel size"]);
    });
  } else {
    // Inline styles: style={{ … }} and style="…" with a literal color or size. React makes a bare number a pixel size.
    for (const m of text.matchAll(/style=(\{\{[^}]*\}\}|"[^"]*")/g)) {
      const body = m[1];
      const line = text.slice(0, m.index).split("\n").length;
      if (HEX.test(body) || COLOR_FN.test(body) || /["'`]\s*(?:white|black|red)\b/i.test(body)) out.push([line, "a raw color in an inline style"]);
      if (PX.test(body) || /:\s*-?\d+(?:\.\d+)?\s*[,}]/.test(body)) out.push([line, "a pixel size in an inline style"]);
    }
  }
  return out;
}

const isMain = process.argv[1] && import.meta.url.endsWith(process.argv[1].replaceAll("\\", "/").split("/").pop());
if (isMain) {
  const src = process.argv[2] ?? join(import.meta.dirname, "..", "apps", "desktop", "src");
  const bad = [];
  let checked = 0;
  for (const file of walk(src).filter((f) => /\.(css|tsx|ts)$/.test(f))) {
    const rel = relative(src, file).replaceAll("\\", "/");
    if (EXEMPT.has(rel) || rel === "bindings.ts") continue;
    checked++;
    for (const [line, what] of lint(rel, readFileSync(file, "utf8"))) bad.push(`${rel}:${line}: ${what}; use a token from styles/tokens.css`);
  }
  for (const b of bad) console.error(`error: ${b}`);
  if (bad.length) {
    console.error(`style-lint: ${bad.length} one-off styles in ${checked} files`);
    process.exit(1);
  }
  console.log(`style-lint: ok (${checked} files)`);
}
