// SPDX-License-Identifier: AGPL-3.0-or-later
// Photo to model in the app (session 4D) with each fake agent: a geotagged photo dropped on the composer and another
// pasted into it become chips, the first one says once where photos go, and Send takes them to the agent as image
// content (Claude Code) or localImage items (Codex), with no metadata left, saved in the project's refs/. A sent photo
// opens the compare view beside the model, and neither opening nor closing it moves the camera.
import { $, expect } from "@wdio/globals";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { camera, instances, invoke, open, screenshots } from "../lib/app.mjs";
import { fakeDir, scenario } from "../lib/agents.mjs";
import { repo } from "../lib/paths.mjs";
import { projectDir } from "../lib/projects.mjs";

const projects = { claude: projectDir("photo"), codex: projectDir("photo-codex") };
const NOTICE = { claude: "Your photo goes to Claude Code, under Anthropic's terms.", codex: "Your photo goes to Codex, under OpenAI's terms." };
const photo = (name) => readFileSync(join(repo, "tests", "fixtures", "photos", name)).toString("base64");

const chat = () => browser.execute(() => window.__emmet.chat());
const log = () => browser.execute(() => document.getElementById("dock")?.innerText ?? "");
async function waitFor(what, test, timeout = 30_000) {
  await browser.waitUntil(test, { timeout, interval: 50, timeoutMsg: `${what}; the dock says:\n${await log()}` });
}
const idle = () => waitFor("the turn never ended", async () => !(await chat()).running);
const chips = () => browser.execute(() => [...document.querySelectorAll("#composer-photos .photo-chip[data-path]")].map((c) => c.dataset.path));

/** A file in the page, dropped on the composer as Finder drops one (the window's own drop handler is off, 4D). */
async function drop(name, b64, type = "image/jpeg") {
  await browser.execute(
    (n, data, t) => {
      const bytes = Uint8Array.from(atob(data), (c) => c.charCodeAt(0));
      const dt = new DataTransfer();
      dt.items.add(new File([bytes], n, { type: t }));
      const target = document.querySelector("#dock .composer");
      target.dispatchEvent(new DragEvent("dragover", { bubbles: true, cancelable: true, dataTransfer: dt }));
      target.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: dt }));
    },
    name,
    b64,
    type,
  );
}

/** A file pasted into the message box, as a screenshot copied to the clipboard is. */
async function paste(name, b64) {
  await browser.execute(
    (n, data) => {
      const bytes = Uint8Array.from(atob(data), (c) => c.charCodeAt(0));
      const dt = new DataTransfer();
      dt.items.add(new File([bytes], n, { type: "image/jpeg" }));
      const e = new Event("paste", { bubbles: true, cancelable: true });
      Object.defineProperty(e, "clipboardData", { value: dt });
      document.getElementById("chat-input").dispatchEvent(e);
    },
    name,
    b64,
  );
}

/** What the fake read from the adapter since `from` lines, as it arrived (the transcript keeps placeholders). */
const received = (from = 0) =>
  readFileSync(join(fakeDir, "stdin.jsonl"), "utf8")
    .split("\n")
    .filter(Boolean)
    .slice(from)
    .map((l) => JSON.parse(l));
const lines = () => {
  try {
    return readFileSync(join(fakeDir, "stdin.jsonl"), "utf8").split("\n").filter(Boolean).length;
  } catch {
    return 0;
  }
};
/** Takes the photo notice off the user's seen tips (the dock reads them at a message's first photo). */
function forgetPhotoNotice() {
  const file = join(process.env.EMMET_DATA_DIR, "settings.json");
  let s = {};
  try {
    s = JSON.parse(readFileSync(file, "utf8"));
  } catch {}
  s.tips_seen = (s.tips_seen ?? []).filter((t) => t !== "photo");
  writeFileSync(file, JSON.stringify(s, null, 2));
}
const metadata = (buf) => ["Exif\0\0", "GPS", "xmpmeta", "ICC_PROFILE", "Broadway"].find((m) => buf.includes(Buffer.from(m, "latin1")));

for (const kind of ["claude", "codex"]) {
  describe(`photo to model with the fake ${kind === "claude" ? "Claude Code" : "Codex"}`, () => {
    let before = 0;

    it("attaches a dropped and a pasted photo, says once where they go, and sends them cleaned", async () => {
      // Claude's run is the user's first photo: the notice is due again (the data dir outlives a run and a retry).
      if (kind === "claude") forgetPhotoNotice();
      await open(projects[kind]);
      await instances(8);
      scenario(kind, "plain-turn");
      if ((await chat()).agent !== kind) {
        await browser.execute((e) => e.focus(), await $("#agent-picker"));
        await browser.keys("Enter");
        await (await $(`[data-menu="agent-${kind}"]`)).click();
      } else await (await $("#start-fresh")).click();
      await waitFor("no fresh conversation", async () => (await chat()).session == null && (await chat()).agent === kind);
      if (kind === "claude") await invoke("tip_seen", { tip: "knob" }); // keep the panel's tip out of the screenshots
      // The notice shows once per user: Codex's run comes after Claude's has shown it.
      const noticeDue = kind === "claude";

      await drop("IMG_2424.jpg", photo("geotagged.jpg"));
      await waitFor("the dropped photo never became a chip", async () => (await chips()).length === 1);
      await paste("side.jpg", photo("angle-a.jpg"));
      await waitFor("the pasted photo never became a chip", async () => (await chips()).length === 2);
      if (noticeDue) {
        await expect($("#photo-notice")).toHaveText(expect.stringContaining(NOTICE[kind]));
        await screenshots("photo-attached");
        await (await $("#photo-notice-ok")).click();
      }
      await expect($("#photo-notice")).not.toBeExisting();
      // A file that isn't a photo is turned away in words.
      await drop("notes.txt", Buffer.from("not a photo").toString("base64"), "text/plain");
      await waitFor("no word about the text file", async () => (await log()).includes("Emmet takes photos as JPEG, PNG, or HEIC."));
      const paths = await chips();
      for (const p of paths) {
        expect(p).toMatch(/^refs\/[a-z0-9-]+-[0-9a-f]{12}\.jpg$/);
        expect(metadata(readFileSync(join(projects[kind], p)))).toBeUndefined();
      }

      before = lines();
      await (await $("#chat-send")).click();
      await idle();
      expect(await chips()).toEqual([]);
      const turn = received(before).find((m) => m.type === "user" || m.method === "turn/start");
      if (kind === "claude") {
        const content = turn.message.content;
        expect(content.map((b) => b.type)).toEqual(["image", "image", "text"]);
        // The words, then what the user sees, as every message carries it (4B).
        expect(content[2].text).toMatch(/^Build this\.(\n\n\[Emmet context\]|$)/);
        for (const [i, b] of content.slice(0, 2).entries()) {
          const sent = Buffer.from(b.source.data, "base64");
          expect(metadata(sent)).toBeUndefined();
          expect(sent.equals(readFileSync(join(projects[kind], paths[i])))).toBe(true);
        }
      } else {
        expect(turn.params.input.slice(0, 2)).toEqual([
          { type: "localImage", path: join(projects[kind], paths[0]) },
          { type: "localImage", path: join(projects[kind], paths[1]) },
        ]);
        expect(turn.params.input[2].type).toBe("text");
        expect(turn.params.input[2].text).toMatch(/^Build this\.(\n\n\[Emmet context\]|$)/);
      }
      const saved = readdirSync(join(projects[kind], "refs"));
      expect(saved.length).toBe(2);
    });

    it("opens a sent photo beside the model without moving the camera", async () => {
      if (!(await (await $("#chat-log")).isExisting())) await (await $("#history-toggle")).click();
      const photoButton = await $("#chat-log .message-photo");
      await photoButton.waitForExist({ timeout: 10_000 });
      const was = await camera();
      const wide = await browser.execute(() => document.querySelector("#viewer > canvas").clientWidth);
      await photoButton.click();
      await (await $("#photo-compare img")).waitForDisplayed({ timeout: 10_000 });
      await browser.execute(() => window.__emmet.viewer().nextFrame());
      expect(await camera()).toEqual(was);
      const narrow = await browser.execute(() => document.querySelector("#viewer > canvas").clientWidth);
      expect(Math.abs(narrow - wide / 2)).toBeLessThanOrEqual(2);
      if (kind === "claude") await screenshots("photo-compare");
      await (await $("#photo-compare-close")).click();
      await expect($("#photo-compare")).not.toBeExisting();
      await browser.execute(() => window.__emmet.viewer().nextFrame());
      expect(await camera()).toEqual(was);
      if (await (await $("#chat-log")).isExisting()) await (await $("#history-toggle")).click();
    });
  });
}

after(() => {
  // The next spec's agent starts clean.
  writeFileSync(join(fakeDir, "stdin.jsonl"), "");
});
