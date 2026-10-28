#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Makes the app's icons from the final SVGs (plan session 2B): `tauri icon` on docs/design/icon/app-icon-macos.svg
// gives icon.icns and the PNGs, and on app-icon-windows.svg gives icon.ico, since macOS icons sit on a plate and
// Windows icons don't. Only the files tauri.conf.json bundles are kept; the Android, iOS, and Store logos it also
// writes are dropped. Usage: node scripts/app-icon.mjs [out-dir]  (default: apps/desktop/src-tauri/icons)
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";
import { pathToFileURL } from "node:url";

const ROOT = join(import.meta.dirname, "..");
const TAURI = join(ROOT, "apps/desktop/node_modules/.bin/tauri");

/** Each source SVG and the files taken from what `tauri icon` makes of it. */
export const SOURCES = {
  "docs/design/icon/app-icon-macos.svg": ["icon.icns", "icon.png", "32x32.png", "128x128.png", "128x128@2x.png"],
  "docs/design/icon/app-icon-windows.svg": ["icon.ico"],
};

/** Writes every icon into `out`; returns the file names written. */
export function makeIcons(out) {
  const work = mkdtempSync(join(tmpdir(), "emmet-icons-"));
  const written = [];
  try {
    mkdirSync(out, { recursive: true });
    for (const [svg, files] of Object.entries(SOURCES)) {
      const dir = join(work, String(written.length));
      // tauri lists every file it writes on stderr; that's shown only if it fails (execFileSync's error carries it).
      execFileSync(TAURI, ["icon", join(ROOT, svg), "--output", dir], { stdio: "pipe" });
      for (const f of files) {
        copyFileSync(join(dir, f), join(out, f));
        written.push(f);
      }
    }
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
  return written;
}

if (process.argv[1] && import.meta.url === pathToFileURL(realpathSync(process.argv[1])).href) {
  const out = process.argv[2] ?? join(ROOT, "apps/desktop/src-tauri/icons");
  const files = makeIcons(out);
  console.log(`app-icon: wrote ${files.join(", ")} to ${relative(ROOT, out) || "."}`);
}
