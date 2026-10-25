// SPDX-License-Identifier: AGPL-3.0-or-later
// Memory for the leak test (plan section 13). WebView2 reports the JS heap (performance.memory, made precise by the
// browser arguments in wdio.conf.mjs) after a forced GC. WKWebView has no heap API, so on macOS this reads the
// physical footprint of the app's WebContent processes with the system's `footprint` tool.
import { execFileSync } from "node:child_process";

const WEBCONTENT = "com.apple.WebKit.WebContent";

export function webContentPids() {
  try {
    return execFileSync("pgrep", ["-f", WEBCONTENT], { encoding: "utf8" }).split("\n").filter(Boolean).map(Number);
  } catch {
    return []; // pgrep exits 1 when nothing matches
  }
}

function footprint(pid) {
  const text = execFileSync("footprint", ["--noCategories", "-f", "bytes", "-p", String(pid)], { encoding: "utf8" });
  const m = /Footprint: (\d+) B/.exec(text);
  if (!m) throw new Error(`footprint gave no number for ${pid}: ${text.slice(0, 200)}`);
  return Number(m[1]);
}

/** { bytes, what }: the number the leak test compares. */
export async function measure() {
  if (process.platform === "darwin") {
    const before = new Set((process.env.EMMET_E2E_WEBCONTENT_BEFORE ?? "").split(",").filter(Boolean).map(Number));
    const ours = webContentPids().filter((p) => !before.has(p));
    if (!ours.length) throw new Error("found no WebContent process for the app");
    // The median of three samples, so one allocator burst doesn't decide the test.
    const samples = [];
    for (let i = 0; i < 3; i++) {
      samples.push(ours.reduce((sum, p) => sum + footprint(p), 0));
      await new Promise((r) => setTimeout(r, 500));
    }
    samples.sort((a, b) => a - b);
    return { bytes: samples[1], what: `WebContent footprint (pids ${ours.join(", ")})` };
  }
  const heap = await browser.execute(() => {
    window.gc?.();
    return window.__emmet.viewer().stats().heap;
  });
  if (heap == null) throw new Error("this webview reports no JS heap");
  return { bytes: heap, what: "JS heap" };
}
