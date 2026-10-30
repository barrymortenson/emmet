// SPDX-License-Identifier: AGPL-3.0-or-later
// Every chat state the frozen design draws (docs/design/frozen/screens.js, session 4B), on the Cottage, saved as
// screens/dock-<frozen id>-<theme>.png for the side-by-sides. States a replay passes through too quickly to see (the
// agent thinking, a build running, a redirect waiting) are played into the dock as the host would send them (the
// test-only chatPlay hook); the rest come from the app itself: a build that fails, an agent that isn't signed in, the
// picker, the plan usage, and the plan limits.
import { $, expect } from "@wdio/globals";
import { rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { open, screenshots } from "../lib/app.mjs";
import { fakeDir } from "../lib/agents.mjs";
import { projectDir } from "../lib/projects.mjs";

const dir = projectDir("dock");
const PROMPT = "Build a small cottage on a green 32 × 32 baseplate: four walls with a door and some windows, a sloped roof, and a chimney.";

const play = (events, user, o) => browser.execute((e, u, x) => window.__emmet.chatPlay(e, u, x), events, user ?? null, o ?? {});
const chat = () => browser.execute(() => window.__emmet.chat());
/** A fresh conversation with `kind`, and nothing in it. */
async function fresh(kind = "claude") {
  await browser.execute((k) => window.__TAURI_INTERNALS__.invoke("chat_fresh", { kind: k }), kind);
  await browser.execute(() => window.__emmet.reloadChat());
  await browser.waitUntil(async () => (await chat()).items === 0 && (await chat()).agent === kind, { timeout: 10_000 });
}
async function choose(trigger, item) {
  await (await $(trigger)).click();
  await browser.keys("Enter");
  await (await $(item)).waitForDisplayed({ timeout: 5_000 });
}

const think = { type: "reasoning", message: "r1", delta: "Planning the cottage: the baseplate, then the walls a row at a time with a door and windows, the room furnished while the walls are low, a gable roof, and a chimney that goes up with the walls." };
const tool = (id, name, input) => ({ type: "tool_call", id, name: `mcp__emmet__${name}`, input });
const done = (id, summary, ok = true) => ({ type: "tool_result", id, ok, summary });
const start = [
  tool("t1", "rules", {}),
  done("t1", "How to build with Emmet…"),
  tool("t2", "search_parts", { query: "brick 1x2", more: ["slope 45 2x4", "door 1x4x6", "window 1x2x2"] }),
  done("t2", "12 results"),
  { type: "tool_call", id: "t3", name: "Write", input: { file_path: "model.py", content: "…" } },
  { type: "file_change", path: "model.py", kind: "modified" },
];
const stage1 = [tool("t4", "run_model", {}), done("t4", "Built 70 parts in 14 steps (0.10 s), generation 2."), tool("t5", "validate", {}), done("t5", "No findings: the model is clean.")];
const finish = [
  tool("t6", "run_model", {}),
  done("t6", "Built 147 parts in 31 steps (0.10 s), generation 3."),
  tool("t7", "validate", {}),
  done("t7", "No findings: the model is clean."),
  tool("t8", "render", { view: "iso" }),
  done("t8", "Rendered the iso view."),
  { type: "text", message: "a1", delta: "The cottage is up: 10 x 8 studs, 6 bricks tall, **147 parts**, 0 errors: a real door, glass in the windows, and a room inside. Knobs: weathering, roof on or off, and Shuffle." },
];

describe("the dock's frozen states", () => {
  before(async () => {
    await open(dir);
  });

  it("thinking, building, a redirect, and built", async () => {
    await fresh();
    await play([think], PROMPT);
    await screenshots("dock-thinking");
    await play([...start, tool("t4", "run_model", {})]);
    await expect($("#chat-status")).toHaveText(expect.stringContaining("Building"));
    await screenshots("dock-building");
    await play([done("t4", "Built 70 parts in 14 steps (0.10 s), generation 2."), tool("t5", "run_model", {})], "put the chimney at the back, like the stone one on this house: example.com/stone-cottage.jpg", { note: "n1" });
    await expect($("#pending")).toHaveText(expect.stringContaining("Goes to Claude at its next step"));
    await screenshots("dock-redirect");
    await play([{ type: "note_delivered", id: "n1", path: "native" }, done("t5", "Built 114 parts in 22 steps (0.10 s), generation 3."), { type: "text", message: "a0", delta: "Got it: the chimney goes at the back. I'd like to look at your photo first." }, { type: "approval", id: "p1", kind: "permission", tool: "WebFetch", detail: "example.com/stone-cottage.jpg" }]);
    await screenshots("dock-approval");
    await play([{ type: "approval_closed", id: "p1" }, ...finish, { type: "turn_done", reason: "completed" }]);
    await expect($("#chat-status")).toHaveText(expect.stringContaining("Done in"));
    await screenshots("dock-built");
    await (await $("#history-toggle")).click();
    await (await $("#chat-log")).waitForExist();
    await screenshots("dock-history");
    await (await $("#history-toggle")).click();
  });

  it("a question waiting for an answer", async () => {
    await fresh();
    await play([think, { type: "question", id: "q1", question: { question: "Which style should the cottage have?", header: "Style", options: [{ label: "Classic: tan walls and a dark red roof", description: "Recommended" }, { label: "Modern: white walls and a black roof" }], multi: false } }], PROMPT);
    await (await $("#dock-active [data-question]")).waitForExist();
    await screenshots("dock-question");
    await play([{ type: "question_closed", id: "q1", reason: "stopped" }, { type: "turn_done", reason: "interrupted" }]);
  });

  it("fixing a finding, sent from Checks", async () => {
    await fresh();
    await play([tool("f1", "validate", {}), done("f1", "1 error: 1 floating.\nF-0001 error floating model.py:40 …"), { type: "tool_call", id: "f2", name: "Edit", input: { file_path: "model.py" } }, tool("f3", "run_model", {})], "Fix this: 9 parts aren't held by anything. The chimney starts 9 plates above the baseplate, so nothing holds it up.", { from: "checks" });
    await screenshots("dock-fixing");
    await play([done("f3", "Built 147 parts in 31 steps (0.10 s), generation 4."), { type: "turn_done", reason: "completed" }]);
  });

  it("stopped, and crashed", async () => {
    await fresh();
    await play([...start, ...stage1, tool("s1", "run_model", {}), { type: "turn_done", reason: "interrupted" }], PROMPT, { at: Date.now() - 41_000 });
    await screenshots("dock-stopped");
    await fresh();
    await play([...start, tool("c1", "run_model", {}), { type: "exited", code: 1 }], PROMPT);
    await (await $("#crash-card")).waitForExist();
    await screenshots("dock-crashed");
  });

  it("the agent picker, the switch, the model and effort, and the plan usage", async () => {
    await fresh();
    await play([...start, ...finish, { type: "turn_done", reason: "completed" }], PROMPT);
    await choose("#agent-picker", '[data-menu="agent-codex"]');
    await screenshots("dock-switch-agent");
    await browser.keys("Escape");
    await (await $("#model-chip")).click();
    await (await $("#model-popover")).waitForDisplayed();
    await screenshots("dock-model-picker");
    await browser.keys("Escape");
    await (await $("#plan-badge")).click();
    await (await $("#usage-popover")).waitForDisplayed();
    await expect($("#usage-popover")).toHaveText(expect.stringContaining("resets in"), { wait: 10_000 });
    await screenshots("dock-usage");
    await browser.keys("Escape");
    await fresh("codex");
    await play([{ type: "started", resume: "t", model: "gpt-6.1-sol", billing: { kind: "api_key", source: "keychain" } }, tool("w1", "get_params", {}), done("w1", "…"), { type: "reasoning", message: "r2", delta: "Joining the garage to the cottage's right wall and widening it to fit two cars." }], "make the garage and cottage one building, make it larger and make a door from the cottage to the garage, make the garage fit two cars.");
    await (await $("#history-toggle")).click();
    await screenshots("dock-switched");
    await (await $("#history-toggle")).click();
    await play([{ type: "turn_done", reason: "interrupted" }]);
  });

  it("the plan limits: Claude's, and ChatGPT's for Codex", async () => {
    await fresh();
    const resets = Math.round(Date.now() / 1000) + 2 * 3600 + 14 * 60;
    await play([{ type: "limits", snapshot: { agent: "claude", billing: { kind: "plan", name: "max" }, windows: [{ kind: "five_hour", group: "five_hour", label: "5-hour", percent: 100, resets_at: resets, scope: null }, { kind: "seven_day", group: "seven_day", label: "Week", percent: 41, resets_at: resets + 86400 * 3, scope: null }], credits: null, source: "rate_limit_event", updated_at: Date.now(), status: "limited", failure: null } }, { type: "turn_done", reason: "failed" }], PROMPT);
    await (await $("#claude-limit")).waitForExist();
    await screenshots("dock-limit");
    await fresh("codex");
    await play([{ type: "started", resume: "t", model: "gpt-6.1-sol", billing: { kind: "plan", name: "ChatGPT" } }, { type: "error", code: "quota_exceeded", message: "You've hit your usage limit." }, { type: "turn_done", reason: "failed" }], "make the garage bigger");
    await (await $("#limit-card")).waitForExist();
    await screenshots("dock-codex-limit");
  });

  it("a build that failed, and an agent that isn't signed in", async () => {
    await fresh("claude");
    writeFileSync(join(dir, "model.py"), "this isn't python\n");
    await (await $("#build-failed")).waitForExist({ timeout: 30_000 });
    await screenshots("dock-failed");
    writeFileSync(join(fakeDir, "claude-auth.json"), '{"loggedIn":false}');
    await choose("#agent-picker", '[data-menu="agents-again"]');
    await (await $('[data-menu="agents-again"]')).click();
    await (await $("#agent-fix")).waitForExist({ timeout: 10_000 });
    await screenshots("dock-signed-out");
    rmSync(join(fakeDir, "claude-auth.json"));
    await choose("#agent-picker", '[data-menu="agents-again"]');
    await (await $('[data-menu="agents-again"]')).click();
  });
});
