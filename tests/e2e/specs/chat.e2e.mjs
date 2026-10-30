// SPDX-License-Identifier: AGPL-3.0-or-later
// The chat with each fake agent (sessions 1C and 4B, plan sections 10, 14, and 16), replayed from tests/replay/: a full
// rung 1 build, told in plain words; a build that edits model.py, which Undo turn takes back byte for byte; an
// approval denied; a question answered with Other; a redirect mid-turn; an interrupt; a crash and Restart; the model
// and effort reaching the CLI; switching agents, which keeps the old conversation. Then: under 50 ms between an agent
// event and its pixels, a 1,000-message history that stays smooth, and the user's agent configs untouched.
import { $, expect } from "@wdio/globals";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { instances, invoke, open, screenshots } from "../lib/app.mjs";
import { agentConfigHashes, fakeDir, scenario } from "../lib/agents.mjs";
import { projectDir } from "../lib/projects.mjs";

const projects = { claude: projectDir("chat"), codex: projectDir("chat-codex") };
const NAME = { claude: "Claude", codex: "Codex" };

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

const chat = () => browser.execute(() => window.__emmet.chat());
/** The whole dock's words, with the history open. */
const log = () => browser.execute(() => document.getElementById("dock")?.innerText ?? "");

async function waitFor(what, test, timeout = 30_000) {
  await browser.waitUntil(test, { timeout, interval: 50, timeoutMsg: `${what}; the dock says:\n${await log()}` });
}

/** Waits until the turn is over. */
const idle = () => waitFor("the turn never ended", async () => !(await chat()).running);

async function history(open = true) {
  const shown = await (await $("#chat-log")).isExisting();
  if (shown !== open) await (await $("#history-toggle")).click();
}

/** Starts a fresh conversation with `kind`, whose fake plays `name` when the first message starts it. */
async function start(kind, name) {
  // A turn a failed test left running ends first.
  if ((await chat()).running) {
    await invoke("chat_stop").catch(() => {});
    await idle();
  }
  await history(false);
  scenario(kind, name);
  await waitFor("no agent", async () => (await chat()).agent != null);
  if ((await chat()).agent === kind) await (await $("#start-fresh")).click();
  else await pick(kind);
  await waitFor("the fresh conversation never came", async () => (await chat()).session == null && (await chat()).agent === kind);
}

/** Opens a Radix menu or select by keyboard (Radix opens on pointerdown, which the embedded driver doesn't send), then
 * picks `item`. */
async function choose(trigger, item) {
  await (await $(trigger)).click();
  await browser.keys("Enter");
  const it = await $(item);
  await it.waitForDisplayed({ timeout: 5_000 });
  await it.click();
}

/** The agent picker's `kind`. */
const pick = (kind) => choose("#agent-picker", `[data-menu="agent-${kind}"]`);

/** Clicks a button by its words inside the last element matching `scope`. */
async function press(scope, label) {
  await waitFor(`no "${label}" button in ${scope}`, () =>
    browser.execute(
      (s, l) => {
        const all = document.querySelectorAll(s);
        const b = [...(all[all.length - 1]?.querySelectorAll("button") ?? [])].find((x) => x.textContent.trim().replace(/^\d/, "").startsWith(l));
        b?.click();
        return Boolean(b);
      },
      scope,
      label,
    ),
  );
}

/** sha256 of each of a project's sources (plan section 07): what Undo turn must put back. */
function sources(dir) {
  const out = {};
  const add = (rel) => (out[rel] = createHash("sha256").update(readFileSync(join(dir, rel))).digest("hex"));
  for (const f of ["model.py", "params.json", "emmet.toml", "CARD.md"]) if (existsSync(join(dir, f))) add(f);
  const walk = (rel) => {
    for (const n of readdirSync(join(dir, rel), { withFileTypes: true })) {
      if (n.isDirectory()) walk(`${rel}/${n.name}`);
      else if (n.name.endsWith(".py")) add(`${rel}/${n.name}`);
    }
  };
  if (existsSync(join(dir, "models"))) walk("models");
  return out;
}

/** The closed line of every card in the history: none may be a tool's name, a path, or code (What good looks like 1). */
const closedLines = () =>
  browser.execute(() =>
    [...document.querySelectorAll("#chat-log .tool-card summary, #chat-log .tool-card-row, #chat-log .tool-run > summary")].map((s) =>
      [...s.querySelectorAll(".tool-card-title, .tool-card-detail, span")].map((x) => x.textContent.trim()).filter(Boolean).join(" · "),
    ),
  );

const before = JSON.parse(process.env.EMMET_E2E_SENTINELS ?? "{}");

for (const kind of ["claude", "codex"]) {
  describe(`chat with the fake ${kind === "claude" ? "Claude Code" : "Codex"}`, () => {
    it("shows the agent and who pays", async () => {
      await open(projects[kind]);
      await instances(8);
      if ((await chat()).agent !== kind) await pick(kind);
      await waitFor("the picker didn't show the agent", async () => (await $("#agent-picker").getText()) === NAME[kind]);
      if (kind === "claude") {
        await expect($("#plan-badge")).toHaveText("Max plan", { wait: 10_000 });
        // A key in the user's environment bills instead of the plan (ADR 0001): the dock says so before a turn.
        writeFileSync(join(fakeDir, "claude-auth.json"), '{"loggedIn":true,"authMethod":"api_key","apiKeySource":"ANTHROPIC_API_KEY"}');
        await choose("#agent-picker", '[data-menu="agents-again"]');
        await waitFor("the dock didn't say the key bills", async () => (await log()).includes("will bill the API key in ANTHROPIC_API_KEY"));
        rmSync(join(fakeDir, "claude-auth.json"));
        await choose("#agent-picker", '[data-menu="agents-again"]');
        await waitFor("the warning stayed", async () => !(await log()).includes("will bill the API key"));
      } else {
        await expect($("#plan-badge")).toHaveText("API key", { wait: 10_000 });
      }
    });

    it("builds, says so in words, and Undo turn puts the sources back byte for byte", async () => {
      await start(kind, "derived-build");
      const was = sources(projects[kind]);
      await say("Make the row 10 bricks wide, then build it.");
      await (await $("#chat-status")).waitForExist({ timeout: 10_000 });
      await instances(10);
      await idle();
      await history();
      const lines = await closedLines();
      expect(lines.join("\n")).toContain("Changed the model");
      expect(lines.join("\n")).toMatch(/Built 10 parts/);
      for (const l of lines) expect(l).not.toMatch(/mcp__|model\.py|[{}()=]|\//);
      expect(await log()).toContain("the row is 10 bricks wide");
      // The end of a turn fetches plan usage (D-36): Claude's meters, Codex's billing state.
      if (kind === "claude") await expect($("#plan-badge")).toHaveText(expect.stringMatching(/^Max plan · \d+%$/), { wait: 10_000 });
      await expect($("#chat-status")).toHaveText(expect.stringContaining("Done in"));
      expect(sources(projects[kind])).not.toEqual(was);
      await screenshots(`chat-built-${kind}`);
      // Undo turn by its button, and by ⌥⌘Z (docs/design/keyboard.md).
      if (kind === "claude") await (await $("#undo-turn")).click();
      else {
        await (await $("#viewer")).click();
        await browser.keys([process.platform === "darwin" ? "Meta" : "Control", "Alt", "z"]);
      }
      await waitFor("Undo turn didn't put the sources back", async () => JSON.stringify(sources(projects[kind])) === JSON.stringify(was));
      await instances(8);
      await expect($("#toast")).toHaveText(expect.stringContaining(`Undid ${NAME[kind]}'s last turn`), { wait: 5_000 });
      expect(await (await $("#undo-turn")).isExisting()).toBe(false);
    });

    it("denies an approval, and the agent carries on", async () => {
      await start(kind, kind === "claude" ? "permissions-ask-bash" : "permissions-untrusted");
      await say(kind === "claude" ? "Run the shell command `ls` and tell me its output." : "Change WIDTH in model.py from 8 to 10.");
      await (await $("#dock-active .approval-card")).waitForExist({ timeout: 30_000 });
      // The card says it waits on you, so the status is quiet.
      expect(await (await $("#chat-status")).isExisting()).toBe(false);
      await screenshots(`chat-approval-${kind}`);
      await press("#dock-active .approval-card", "Don't allow");
      if (kind === "codex") {
        // untrusted (0B): the command is a card, then the file change.
        await (await $("#dock-active .approval-card")).waitForExist({ timeout: 30_000 });
        await press("#dock-active .approval-card", "Allow once");
      }
      await idle();
      await history();
      expect(await log()).toContain("You said no");
    });

    it("answers a question with Other, in the same turn", async () => {
      await start(kind, "derived-ask-user");
      await say("Build a wall.");
      await (await $("#dock-active [data-question]")).waitForExist({ timeout: 30_000 });
      expect(await (await $("#chat-status")).isExisting()).toBe(false);
      await screenshots(`chat-question-${kind}`);
      await type("#dock-active .question-other input", "Five bricks, in tan");
      await press("#dock-active .question-other", "Answer");
      await history();
      await waitFor("the question didn't fold to its answer", async () => (await log()).includes("Five bricks, in tan"));
      await waitFor("the agent didn't carry on", async () => (await log()).includes("Six bricks tall"));
      await idle();
    });

    it("delivers a redirect sent mid-turn", async () => {
      await start(kind, kind === "claude" ? "mid-turn-message" : "mid-turn-steer");
      await say("Use the block tool with seconds=20. When it returns, reply with one short sentence.");
      // The replay waits for the redirect at the long tool call, as the CLI took it in 0B's recording.
      await expect($("#chat-status")).toHaveText(expect.stringContaining("Working"), { wait: 30_000 });
      await say("Also include the word banana in your reply.");
      await (await $("#pending")).waitForExist({ timeout: 10_000 }).catch(() => {});
      await waitFor("the redirect wasn't delivered", async () => (await log()).includes(`Delivered at ${NAME[kind]}'s next step`), 40_000);
      await waitFor("the agent didn't act on it", async () => (await log()).toLowerCase().includes("banana"));
      await idle();
    });

    it("stops the turn at once", async () => {
      await start(kind, "interrupt");
      // The keyboard map's chat rows: ⌘L to the message box, Enter sends, and Esc stops (Claude: the Stop button).
      await (await $("#viewer")).click();
      await browser.keys([process.platform === "darwin" ? "Meta" : "Control", "l"]);
      expect(await browser.execute(() => document.activeElement?.id)).toBe("chat-input");
      await type("#chat-input", "Use the block tool with seconds=60.");
      await browser.keys("Enter");
      await (await $("#chat-stop")).waitForExist({ timeout: 30_000 });
      if (kind === "claude") await (await $("#chat-stop")).click();
      else await browser.keys("Escape");
      await idle();
      await expect($("#chat-status")).toHaveText(expect.stringContaining("Stopped by you"));
      await history();
      expect(await log()).toContain("You stopped the turn at");
      await screenshots(`chat-stopped-${kind}`);
    });

    it("restarts a crashed agent with one click, on the same conversation", async () => {
      await start(kind, "derived-crash-mid-stream");
      await say("Use the ping tool with the message hello.");
      await (await $("#crash-card")).waitForExist({ timeout: 30_000 });
      expect(await log()).toContain(`${kind === "claude" ? "Claude Code" : "Codex"} stopped unexpectedly`);
      await screenshots(`chat-crashed-${kind}`);
      const conversation = (await invoke("chat_state")).sessions.at(-1).info.id;
      await (await $("#chat-restart")).click();
      await waitFor("the agent didn't start again", async () => (await chat()).session === conversation && !(await chat()).crashed);
      await say(kind === "claude" ? "Reply with exactly the word: ready" : "What was the code word?");
      await waitFor("the resumed chat didn't answer", async () => (await log()).includes(kind === "claude" ? "ready" : "teal"));
      await idle();
    });

    it("builds rung 1 from its replay, and the chat says what each step did", async () => {
      await start(kind, "derived-rung1");
      await say("Build rung 1.");
      await (await $("#dock-active [data-question]")).waitForExist({ timeout: 30_000 });
      await press("#dock-active [data-question]", "Classic");
      await idle();
      await history();
      for (const l of await closedLines()) expect(l).not.toMatch(/mcp__|\.py\b|[{}=]|\/\w/);
      expect(await log()).toContain("The cottage is up");
    });
  });
}

describe("the model and effort", () => {
  it("reach Claude Code, which starts again on the same conversation", async () => {
    await start("claude", "resume");
    await say("Reply with exactly the word: ready");
    await idle();
    await (await $("#model-chip")).click();
    await (await $("#model-popover")).waitForExist({ timeout: 5_000 });
    await screenshots("chat-model-picker");
    await choose("#model-select", '[role="option"]*=Opus');
    await choose("#effort-select", '[role="option"]*=High');
    await browser.keys("Escape");
    await expect($("#model-chip")).toHaveText(expect.stringContaining("Opus · High"));
    await say("What was the code word?");
    await idle();
    const n = readdirSync(fakeDir).filter((f) => /^claude-\d+\.json$/.test(f)).length;
    const args = JSON.parse(readFileSync(join(fakeDir, `claude-${n - 1}.json`), "utf8")).args;
    const after = (flag) => args[args.indexOf(flag) + 1];
    expect(after("--model")).toBe("opus");
    expect(after("--effort")).toBe("high");
    expect(args.some((a) => a.startsWith("--resume="))).toBe(true);
  });
});

describe("switching agents", () => {
  it("starts a fresh conversation and keeps the old one", async () => {
    await start("claude", "plain-turn");
    await say("Reply with exactly the word: ready");
    await idle();
    await pick("codex");
    await waitFor("no switch", async () => (await chat()).agent === "codex");
    await history();
    expect(await log()).toContain("Codex · a fresh conversation. The Claude one is kept.");
    await screenshots("chat-switched");
    await pick("claude");
  });
});

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

  it("stays smooth at 1,000 messages", async () => {
    // A project whose history holds 1,000 messages: 500 asks and 500 replies, in one conversation.
    const dir = projects.codex;
    const chatDir = join(dir, ".emmet", "chat");
    mkdirSync(chatDir, { recursive: true });
    const id = `c${"ab".repeat(16)}`;
    writeFileSync(join(chatDir, "sessions.json"), JSON.stringify({ sessions: [{ id, kind: "codex", started: Date.now() }] }));
    const lines = [];
    for (let i = 0; i < 500; i++) {
      lines.push(JSON.stringify({ at: i, user: `Message ${i}: make the walls a little taller and the roof steeper.` }));
      lines.push(JSON.stringify({ at: i, event: { type: "text", message: `m${i}`, delta: `Reply ${i}: the walls are 7 bricks tall now, and the roof is at 45 degrees. **163 parts**, 0 errors.` } }));
    }
    writeFileSync(join(chatDir, `${id}.jsonl`), `${lines.join("\n")}\n`);
    await open(dir);
    // The project may be open already: the dock reads its history again.
    await browser.execute(() => window.__emmet.reloadChat());
    await history();
    const log = await $("#chat-log");
    await expect(log).toHaveAttribute("data-rows", "1000");
    const drawn = Number(await log.getAttribute("data-drawn"));
    expect(drawn).toBeLessThan(100);
    // Scrolling from the bottom to the top and back, a frame at a time: no frame takes long.
    const worst = await browser.execute(async () => {
      const el = document.getElementById("chat-log");
      const frame = () => new Promise((r) => requestAnimationFrame(r));
      let worst = 0;
      for (const dir of [-1, 1]) {
        for (let i = 0; i < 40; i++) {
          const t = performance.now();
          el.scrollTop += dir * el.scrollHeight / 40;
          el.dispatchEvent(new Event("scroll"));
          await frame();
          worst = Math.max(worst, performance.now() - t);
        }
      }
      return worst;
    });
    console.log(`1,000 messages: ${drawn} rows drawn, worst scroll frame ${Math.round(worst)} ms`);
    expect(worst).toBeLessThan(100);
    expect(await browser.execute(() => document.getElementById("chat-log").innerText)).toContain("Reply 499");
    rmSync(join(chatDir, "sessions.json"));
  });

  it("leaves the user's agent configs untouched", async () => {
    expect(agentConfigHashes()).toEqual(before);
    expect(Object.keys(before).length).toBeGreaterThan(3);
  });
});
