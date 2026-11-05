// SPDX-License-Identifier: AGPL-3.0-or-later
// Building instructions (session 4E, ADR 0014): Export → Instructions → PDF writes the book into Downloads, and each
// page matches its baseline, for rung 1 (the Cottage) and rung 2 (the station). A page is drawn here from the book
// the export laid out (the display list the PDF is written from, with the pictures the webview drew), at PAGE_WIDTH
// pixels wide, and compared with pixelmatch against tests/e2e/baselines/<os>/instructions/. The PDF's own bytes are
// read back in tests/it/tests/instructions.rs. A missing baseline is written and the test fails until it's committed;
// EMMET_UPDATE_BASELINES=1 rewrites them all.
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { $, expect } from "@wdio/globals";
import pixelmatch from "pixelmatch";
import { open } from "../lib/app.mjs";
import { agentHome } from "../lib/agents.mjs";
import { os, out, repo } from "../lib/paths.mjs";
import { decode, encode } from "../lib/png.mjs";
import { projectDir } from "../lib/projects.mjs";

const PAGE_WIDTH = 200;
/** As visual.e2e.mjs: at most this share of a page's pixels may differ past pixelmatch's threshold. */
const TOLERANCE = 0.002;
const baselines = join(repo, "tests", "e2e", "baselines", os, "instructions");
const update = process.env.EMMET_UPDATE_BASELINES === "1";
const click = async (sel) => (await $(sel)).click();

const BOOKS = [
  { rung: "rung1", dir: projectDir("instructions"), file: "Instructions instructions.pdf" },
  { rung: "rung2", dir: projectDir("instructions-rung2"), file: "New model instructions.pdf" },
];

/** Draws page `i` of the last book the export laid out as a PNG (base64). The book stays in the page between calls. */
function drawPage(i, width) {
  return browser.execute(
    async (i, width) => {
      window.__book ??= await window.__TAURI_INTERNALS__.invoke("last_book");
      const book = window.__book;
      window.__pics ??= new Map();
      const pic = async (n) => {
        if (!window.__pics.has(n)) {
          const img = new Image();
          img.src = `data:image/jpeg;base64,${book.images[n].jpeg}`;
          await img.decode();
          window.__pics.set(n, img);
        }
        return window.__pics.get(n);
      };
      const k = width / book.width;
      const c = document.createElement("canvas");
      c.width = width;
      c.height = Math.round(book.height * k);
      const g = c.getContext("2d");
      g.fillStyle = "#fff";
      g.fillRect(0, 0, c.width, c.height);
      g.scale(k, k);
      g.lineJoin = g.lineCap = "round";
      const color = (c) => `rgb(${c[0]}, ${c[1]}, ${c[2]})`;
      for (const it of book.pages[i].items) {
        if (it.kind === "rect" || it.kind === "path") {
          g.beginPath();
          if (it.kind === "rect") g.roundRect(it.x, it.y, it.w, it.h, Math.min(it.radius, it.w / 2, it.h / 2));
          else {
            it.points.forEach(([x, y], j) => (j ? g.lineTo(x, y) : g.moveTo(x, y)));
            if (it.closed) g.closePath();
          }
          if (it.fill) {
            g.fillStyle = color(it.fill);
            g.fill();
          }
          if (it.stroke) {
            g.strokeStyle = color(it.stroke);
            g.lineWidth = it.width;
            g.stroke();
          }
        } else if (it.kind === "text") {
          g.font = `${it.bold ? "bold " : ""}${it.size}px Helvetica, Arial, sans-serif`;
          g.fillStyle = color(it.color);
          g.textAlign = { start: "left", middle: "center", end: "right" }[it.align];
          g.textBaseline = "alphabetic";
          g.fillText(it.text, it.x, it.y);
        } else if (it.kind === "image") {
          g.drawImage(await pic(it.image), it.x, it.y, it.w, it.h);
        }
      }
      return c.toDataURL("image/png").split(",")[1];
    },
    i,
    width,
  );
}

// A book is hundreds of renders: about 20 s on a Mac's GPU, minutes on a hosted runner's software renderer (WebView2 on
// WARP), so the export gets EXPORT_WAIT and a test BOOK_TIMEOUT, past wdio.conf.mjs's two minutes.
const EXPORT_WAIT = 480_000;
const BOOK_TIMEOUT = 600_000;

describe("building instructions", function () {
  this.timeout(BOOK_TIMEOUT);
  for (const { rung, dir, file } of BOOKS) {
    it(`exports ${rung} as a PDF into Downloads, and every page matches its baseline`, async () => {
      // wdio.conf.mjs starts each spec with no Downloads in the test home. macOS's Downloads is HOME's, made on write;
      // Windows names it only when it exists (Tauri says "unknown path"), so it's made here.
      mkdirSync(join(agentHome, "Downloads"), { recursive: true });
      await open(dir);
      await browser.execute(() => {
        delete window.__book;
        delete window.__pics;
      });
      await browser.keys([process.platform === "darwin" ? "Meta" : "Control", "e"]);
      await (await $("[data-menu='pdf']")).waitForDisplayed();
      await click("[data-menu='pdf']");
      await expect($("#toast")).toHaveText(expect.stringContaining(`Saved “${file}” to Downloads`), { wait: EXPORT_WAIT });
      const pdf = join(agentHome, "Downloads", file);
      expect(readFileSync(pdf).subarray(0, 8).toString("latin1")).toBe("%PDF-1.4");
      const pages = await browser.execute(async () => {
        window.__book = await window.__TAURI_INTERNALS__.invoke("last_book");
        return window.__book.pages.length;
      });
      console.log(`${rung}: ${pages} pages, ${(statSync(pdf).size / 1e6).toFixed(1)} MB`);
      const failed = [];
      for (let i = 0; i < pages; i++) {
        const name = `${rung}-p${String(i + 1).padStart(3, "0")}`;
        const png = Buffer.from(await drawPage(i, PAGE_WIDTH), "base64");
        const baseline = join(baselines, `${name}.png`);
        if (update || !existsSync(baseline)) {
          mkdirSync(baselines, { recursive: true });
          writeFileSync(baseline, png);
          mkdirSync(join(out, "instructions", os, "new"), { recursive: true });
          writeFileSync(join(out, "instructions", os, "new", `${name}.png`), png);
          if (!update) failed.push(`${name}: a new baseline (check it and commit it)`);
          continue;
        }
        const actual = decode(png);
        const expected = decode(readFileSync(baseline));
        if (actual.width !== expected.width || actual.height !== expected.height) {
          failed.push(`${name}: ${actual.width} × ${actual.height}, its baseline ${expected.width} × ${expected.height}`);
          continue;
        }
        const { width, height } = actual;
        const diff = { width, height, data: Buffer.alloc(width * height * 4) };
        const bad = pixelmatch(expected.data, actual.data, diff.data, width, height, { threshold: 0.1 });
        if (bad / (width * height) > TOLERANCE) {
          const d = join(out, "instructions", os);
          mkdirSync(d, { recursive: true });
          writeFileSync(join(d, `${name}-actual.png`), png);
          writeFileSync(join(d, `${name}-diff.png`), encode(diff));
          failed.push(`${name}: ${((100 * bad) / (width * height)).toFixed(2)} % of its pixels differ`);
        }
      }
      // A page more or fewer than the baselines: the last baseline's next page shouldn't exist.
      const extra = join(baselines, `${rung}-p${String(pages + 1).padStart(3, "0")}.png`);
      if (!update && existsSync(extra)) failed.push(`${rung} has ${pages} pages, and there's a baseline for page ${pages + 1}`);
      expect(failed).toEqual([]);
    });
  }
});
