// SPDX-License-Identifier: AGPL-3.0-or-later
// scripts/contrast-check.mjs: the WCAG math against known ratios, the token grammar (text, UI, fills, prefixes), and
// what it refuses: a translucent surface, a theme missing a token, and a color or fractional opacity outside the tokens
// (an inactive control's rule may dim).
// One test runs it on docs/design as it stands, so a mockup that loses AA fails here; the others run it from a path
// with a space through a symlink, and on files no list names.
import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { checkCss, checkSource, contrast, parseColor } from "../../scripts/contrast-check.mjs";
import { ROOT, run, tempDir } from "./helpers.mjs";

const css = (light, dark = light, rest = "") => `[data-theme="light"] { ${light} }\n[data-theme="dark"] { ${dark} }\n${rest}`;

test("contrast matches WCAG's reference values", () => {
  assert.equal(contrast(parseColor("#000"), parseColor("#fff")).toFixed(2), "21.00");
  assert.equal(contrast(parseColor("#767676"), parseColor("#ffffff")).toFixed(2), "4.54");
  assert.equal(contrast(parseColor("#777777"), parseColor("#ffffff")).toFixed(2), "4.48");
  // Half-transparent black over white is the gray it composites to.
  assert.equal(contrast(parseColor("#00000080"), parseColor("#fff")).toFixed(2), contrast(parseColor("#7f7f7f"), parseColor("#fff")).toFixed(2));
  assert.deepEqual(parseColor("rgb(10 20 30 / 50%)"), [10, 20, 30, 0.5]);
  assert.equal(parseColor("blue"), null);
});

test("every text token is checked against every surface, and UI tokens at 3:1", () => {
  const ok = checkCss(css("--bg-a: #fff; --bg-b: #f0f0f0; --fg-text: #222; --ui-edge: #888; --line: #eee;"), "t");
  assert.deepEqual(ok.errors, []);
  assert.equal(ok.themes.light, 4); // 2 surfaces × (1 text + 1 UI); --line is decorative.
  const bad = checkCss(css("--bg-a: #fff; --bg-b: #ddd; --fg-muted: #777; --ui-edge: #aaa;"), "t");
  assert.ok(bad.errors.some((e) => /--fg-muted on --bg-b is \d\.\d\d:1, needs 4\.5:1/.test(e)), bad.errors.join("\n"));
  assert.ok(bad.errors.some((e) => /--ui-edge on --bg-a .* needs 3:1/.test(e)));
});

test("a fill pairs with its on-color, and prefixes keep sets apart", () => {
  const r = checkCss(css("--bg-a: #fff; --fill-accent: #c2410c; --on-accent: #fff; --p-bg-bar: #000; --p-fg-text: #fff;"), "t");
  assert.deepEqual(r.errors, []);
  const lost = checkCss(css("--bg-a: #fff; --on-accent: #fff;"), "t");
  assert.ok(lost.errors.some((e) => /--on-accent has no --fill-accent/.test(e)));
  const dark = checkCss(css("--bg-a: #fff; --fill-accent: #ffb48a; --on-accent: #fff;"), "t");
  assert.ok(dark.errors.some((e) => /--on-accent on --fill-accent/.test(e)));
});

test("it refuses a translucent surface, a token in one theme only, and colors outside the tokens", () => {
  const r = checkCss(css("--bg-a: #ffffff80; --fg-t: #000;", "--bg-a: #000; --fg-t: #fff; --fg-extra: #fff;",
    ".x { color: #123456; }\n.y { background: rgb(1, 2, 3); }\n.z { opacity: 0.6; }\n.w { opacity: 0; border-color: transparent; }\n.v:disabled { opacity: 0.6; }\n.u[data-disabled] .label { opacity: 0.5; }"), "t");
  const all = r.errors.join("\n");
  assert.match(all, /--bg-a must be opaque/);
  assert.match(all, /--fg-extra is in dark but not light/);
  assert.match(all, /a color outside the tokens: \.x/);
  assert.match(all, /a color outside the tokens: \.y/);
  assert.match(all, /opacity 0\.6 dims text/);
  assert.doesNotMatch(all, /\.w/);
  // An inactive control may dim (WCAG 1.4.3 exempts it).
  assert.doesNotMatch(all, /\.v|\.u/);
  assert.deepEqual(checkSource('const a = "#fff";\n// #000 in a comment\nel.style.width = "10%";\n<p>Orange is <code>#E8590C</code>.</p>', "s.js").length, 1);
});

test("a stylesheet that only uses another's tokens needs no blocks, but can't write a color or keep one theme", () => {
  assert.deepEqual(checkCss(".button { color: var(--fg-text); background: var(--fill-accent); }", "uses.css").errors, []);
  const color = checkCss(".button { color: var(--fg-text); border-color: #c2410c; }", "uses.css").errors.join("\n");
  assert.match(color, /a color outside the tokens: \.button/);
  const one = checkCss('[data-theme="light"] { --bg-a: #fff; --fg-t: #000; }', "one.css").errors.join("\n");
  assert.match(one, /needs both a \[data-theme="light"\] and a \[data-theme="dark"\] block/);
});

test("the command fails on a bad stylesheet and passes docs/design as it stands", () => {
  const file = join(tempDir(), "bad.css");
  writeFileSync(file, css("--bg-a: #fff; --fg-t: #ccc;"));
  const bad = run("node", [join(ROOT, "scripts/contrast-check.mjs"), file]);
  assert.equal(bad.code, 1, bad.out);
  assert.match(bad.out, /contrast-check: 2 problems/);
  const real = run("node", [join(ROOT, "scripts/contrast-check.mjs")]);
  assert.equal(real.code, 0, real.out);
  assert.match(real.out, /contrast-check: ok$/m);
});

test("the command still runs from a path with a space that goes through a symlink", () => {
  // A guard that compared raw paths ran nothing from such a path. A junction needs no admin rights on Windows.
  const top = tempDir();
  const dir = join(top, "my repo");
  mkdirSync(join(dir, "scripts"), { recursive: true });
  copyFileSync(join(ROOT, "scripts/contrast-check.mjs"), join(dir, "scripts/contrast-check.mjs"));
  symlinkSync(dir, join(top, "my link"), "junction");
  const file = join(dir, "bad.css");
  writeFileSync(file, css("--bg-a: #fff; --fg-t: #ccc;"));
  const bad = run("node", [join(top, "my link", "scripts/contrast-check.mjs"), file]);
  assert.equal(bad.code, 1, bad.out);
  assert.match(bad.out, /contrast-check: 2 problems/);
});

test("the command checks every stylesheet, script, and page under docs/design and the app's styles, and fails when there are none", () => {
  const dir = tempDir();
  mkdirSync(join(dir, "scripts"));
  copyFileSync(join(ROOT, "scripts/contrast-check.mjs"), join(dir, "scripts/contrast-check.mjs"));
  mkdirSync(join(dir, "docs/design"), { recursive: true });
  const none = run("node", [join(dir, "scripts/contrast-check.mjs")]);
  assert.equal(none.code, 1, none.out);
  assert.match(none.out, /no stylesheets to check/);
  // docs/design without the app's tokens.css: everything the design uses comes from it, so that fails too.
  writeFileSync(join(dir, "docs/design/ok.css"), css("--bg-a: #fff; --fg-t: #000;"));
  const lost = run("node", [join(dir, "scripts/contrast-check.mjs")]);
  assert.equal(lost.code, 1, lost.out);
  assert.match(lost.out, /apps\/desktop\/src\/styles\/tokens\.css is missing/);
  // Files a fixed list wouldn't name: a stylesheet in a new folder and a module in shared/.
  mkdirSync(join(dir, "docs/design/extra"), { recursive: true });
  mkdirSync(join(dir, "docs/design/shared"), { recursive: true });
  writeFileSync(join(dir, "docs/design/extra/theme.css"), css("--bg-a: #fff; --fg-t: #ccc;"));
  writeFileSync(join(dir, "docs/design/shared/notes.mjs"), 'const c = "#cccccc";\n');
  // And the app's own tokens (session 2B), which the frozen design and the app take every color from.
  mkdirSync(join(dir, "apps/desktop/src/styles"), { recursive: true });
  writeFileSync(join(dir, "apps/desktop/src/styles/tokens.css"), css("--bg-a: #fff; --fg-t: #ddd;"));
  const bad = run("node", [join(dir, "scripts/contrast-check.mjs")]);
  assert.equal(bad.code, 1, bad.out);
  assert.match(bad.out, /FAIL docs\/design\/extra\/theme\.css/);
  assert.match(bad.out, /FAIL apps\/desktop\/src\/styles\/tokens\.css/);
  assert.match(bad.out, /docs\/design\/shared\/notes\.mjs: a color outside the tokens/);
});
