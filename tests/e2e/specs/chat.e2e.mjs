// SPDX-License-Identifier: AGPL-3.0-or-later
// The chat pane with each fake agent (session 1C, plan sections 10 and 16): a chat edits model.py and the viewer
// updates; an approval card appears and is answered; a question card appears and is answered in the same turn; a
// message sent mid-turn is delivered and the agent acts on it; a crashed agent recovers with one click. The UI adds
// under 50 ms between an agent event and its pixels, and the user's agent configs are untouched.
import { $, expect } from "@wdio/globals";
import { instances, open, screenshots } from "../lib/app.mjs";
import { rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { agentConfigHashes, fakeDir, scenario } from "../lib/agents.mjs";
import { projectDir } from "../lib/projects.mjs";

const projects = { claude: projectDir("chat"), codex: projectDir("chat-codex") };

/** Text into a field the way typing does: the native setter, then input (WebView2's driver can't type into React). */
async function type(selector, text) {
  const el = await $(selector);
  await el.waitForExist({ timeout: 10_000 });
  await browser.execute(
    (e, t) => {
      const proto = e instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
      Object.getOwnPropertyDescriptor(proto, "value").set.call(e, t);
      e.dispatchEvent(new Event("input", { bubbles: true }));
    },
    el,
    text,
  );
}

async function say(text) {
  await type("#chat-input", text);
  await (await $("#chat-send")).click();
}

/** The chat log's text. */
const log = () => browser.execute(() => document.getElementById("chat-log")?.innerText ?? "");

async function waitFor(what, test, timeout = 30_000) {
  await browser.waitUntil(test, { timeout, interval: 50, timeoutMsg: `${what}; the chat says:\n${await log()}` });
}

/** Waits until the turn is over: the header offers New chat again. */
const idle = () => waitFor("the turn never ended", async () => (await $("#chat-new")).isExisting());

/** Starts a chat with `kind` whose fake plays `name`. */
async function start(kind, name) {
  scenario(kind, name);
  if (await (await $("#chat-new")).isExisting()) await (await $("#chat-new")).click();
  const pick = await $(`#agent-${kind}`);
  await pick.waitForEnabled({ timeout: 30_000 });
  await pick.click();
  await (await $("#chat-badge")).waitForExist({ timeout: 10_000 });
}

/** Clicks a button by its text inside the last element matching `scope`. */
async function press(scope, label) {
  await waitFor(`no "${label}" button`, () =>
    browser.execute(
      (s, l) => {
        const all = document.querySelectorAll(s);
        const b = [...(all[all.length - 1]?.querySelectorAll("button") ?? [])].find((x) => x.textContent.trim().endsWith(l));
        b?.click();
        return Boolean(b);
      },
      scope,
      label,
    ),
  );
}

const before = JSON.parse(process.env.EMMET_E2E_SENTINELS ?? "{}");

for (const kind of ["claude", "codex"]) {
  describe(`chat with the fake ${kind === "claude" ? "Claude Code" : "Codex"}`, () => {
    it("shows the agent and who pays", async () => {
      await open(projects[kind]);
      await instances(8);
      const pick = await $(`#agent-${kind}`);
      await pick.waitForEnabled({ timeout: 30_000 });
      const badge = await browser.execute((k) => document.getElementById(`agent-${k}`)?.nextElementSibling?.textContent, kind);
      expect(badge).toBe(kind === "claude" ? "Claude · Max plan" : "Codex · API key");
      if (kind === "claude") {
        // A key in the user's environment bills instead of the plan (ADR 0001): the picker says so before a chat.
        writeFileSync(join(fakeDir, "claude-auth.json"), '{"loggedIn":true,"authMethod":"api_key","apiKeySource":"ANTHROPIC_API_KEY"}');
        await press(".picker", "Look again");
        await waitFor("the picker didn't show the key", async () =>
          (await browser.execute(() => document.querySelector(".picker")?.innerText ?? "")).includes("will bill the API key in ANTHROPIC_API_KEY"),
        );
        rmSync(join(fakeDir, "claude-auth.json"));
        await press(".picker", "Look again");
        await waitFor("the picker didn't go back to the plan", async () =>
          (await browser.execute(() => document.getElementById("agent-claude")?.nextElementSibling?.textContent)) === "Claude · Max plan",
        );
      }
    });

    it("edits model.py and the viewer updates", async () => {
      await start(kind, "derived-build");
      await say("Make the row 10 bricks wide, then build it.");
      await instances(10);
      await waitFor("no file-change card", () => browser.execute(() => !!document.querySelector('.card.file[data-file="model.py"]')));
      await idle();
      expect(await log()).toContain("the row is 10 bricks wide");
      // The end of a turn fetches plan usage (D-36): Claude's meters, Codex's billing state.
      // (0B's report has fixed reset dates, so only the percentages are checked.)
      await expect($("#usage-button")).toHaveText(
        expect.stringContaining(kind === "claude" ? "Used · Claude 4%" : "Codex · API key"),
        { wait: 10_000 },
      );
      await screenshots(`chat-${kind}`);
    });

    it("shows an approval card and answers it", async () => {
      await start(kind, kind === "claude" ? "permissions-ask-bash" : "permissions-untrusted");
      await say(kind === "claude" ? "Run the shell command `ls` and tell me its output." : "Change WIDTH in model.py from 8 to 10.");
      if (kind === "codex") {
        // untrusted (0B): the command is a card, then the file change.
        await press("[data-approval]", "Deny");
        await waitFor("the first card stayed open", () =>
          browser.execute(() => document.querySelectorAll("[data-approval]").length === 2),
        );
      }
      await press("[data-approval]", "Allow once");
      await waitFor("the card wasn't marked", async () => (await log()).includes("Allowed once"));
      await idle();
    });

    it("asks a question and uses the answer in the same turn", async () => {
      await start(kind, "derived-ask-user");
      await say("Build a wall.");
      await (await $("[data-question]")).waitForExist({ timeout: 30_000 });
      await expect($("#status")).toHaveText("Waiting for your answer");
      await press("[data-question]", "6 bricks");
      await waitFor("the question wasn't answered", async () => (await log()).includes("Answered"));
      await waitFor("the agent didn't carry on", async () => (await log()).includes("Six bricks tall"));
      await idle();
    });

    it("delivers a message sent mid-turn", async () => {
      await start(kind, kind === "claude" ? "mid-turn-message" : "mid-turn-steer");
      await say("Use the block tool with seconds=20. When it returns, reply with one short sentence.");
      await (await $('[data-tool="mcp__emmet__block"]')).waitForExist({ timeout: 30_000 });
      // A long tool call is quiet: the chat says the agent is still working.
      await (await $("#chat-working")).waitForExist({ timeout: 5_000 });
      await say("Also include the word banana in your reply.");
      await waitFor("the redirect wasn't delivered", () =>
        browser.execute(() => !!document.querySelector("[data-note] .delivered")),
      );
      await waitFor("the agent didn't act on it", async () => (await log()).toLowerCase().includes("banana"));
      await idle();
    });

    it("recovers a crashed agent with one click", async () => {
      await start(kind, "derived-crash-mid-stream");
      await say("Use the ping tool with the message hello.");
      const resume = await $("#chat-resume");
      await resume.waitForExist({ timeout: 30_000 });
      expect(await log()).toContain("crashed on purpose");
      await resume.click();
      await (await $("#chat-badge")).waitForExist({ timeout: 10_000 });
      await say(kind === "claude" ? "Reply with exactly the word: ready" : "What was the code word?");
      await waitFor("the resumed chat didn't answer", async () => (await log()).includes(kind === "claude" ? "ready" : "teal"));
      await idle();
    });
  });
}

describe("the chat overall", () => {
  // The window measures from the host having the event to the change being in the DOM; the next frame paints it, so
  // the budget is 50 ms less one 60 Hz frame.
  it("adds under 50 ms between an agent event and its pixels", async () => {
    const ms = await browser.execute(() => window.__emmet.chatLatency());
    expect(ms.length).toBeGreaterThan(50);
    const sorted = [...ms].sort((a, b) => a - b);
    const p95 = sorted[Math.floor(sorted.length * 0.95)];
    console.log(`chat event → DOM: median ${sorted[Math.floor(sorted.length / 2)]} ms, p95 ${p95} ms (${ms.length} events)`);
    expect(p95).toBeLessThan(50 - 17);
  });

  it("leaves the user's agent configs untouched", async () => {
    await (await $("#chat-new")).click();
    expect(agentConfigHashes()).toEqual(before);
    expect(Object.keys(before).length).toBeGreaterThan(3);
  });
});
