// SPDX-License-Identifier: AGPL-3.0-or-later
// Codex on the user's ChatGPT plan (session 1E): OpenAI's UI rules in the app, against fake-openai and the fake browser.
// An API-key user sees the invitation; Continue with ChatGPT signs in (timed from the click); the one-time "You're using
// your ChatGPT plan" notice; the account picker with the active account; "Using ChatGPT plan · Manage usage" by the
// composer and in the status bar; the usage-limit card with Manage usage first; a decline; and the notice never again.
// Then no token fake-openai issued is anywhere in the app's data, the project, or the run's logs.
import { $, expect } from "@wdio/globals";
import { existsSync, readdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { join } from "node:path";
import { instances, invoke, open, screenshots } from "../lib/app.mjs";
import { chatgptRecord, fakeDir, issuedTokens, opened, scenario, setChatgptRecord, setOpenAi } from "../lib/agents.mjs";
import { out } from "../lib/paths.mjs";
import { projectDir } from "../lib/projects.mjs";

const text = (sel) => browser.execute((s) => document.querySelector(s)?.innerText ?? "", sel);

async function waitFor(what, test, timeout = 30_000) {
  await browser.waitUntil(test, { timeout, interval: 25, timeoutMsg: what });
}

const mod = process.platform === "darwin" ? "Meta" : "Control";

async function settings() {
  if (!(await (await $("#settings")).isExisting())) await browser.keys([mod, ","]);
  await (await $("#codex-billing")).waitForExist({ timeout: 10_000 });
}

async function closeSettings() {
  if (await (await $("#settings")).isExisting()) await (await $("#settings-done")).click();
}

/** Waits until the turn is over. */
const idle = () => browser.waitUntil(() => browser.execute(() => !window.__emmet.chat().running), { timeout: 30_000, timeoutMsg: "the turn never ended" });

/** Text into a field the way typing does (see chat.e2e.mjs). */
async function say(t) {
  const el = await $("#chat-input");
  await browser.execute(
    (e, v) => {
      Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value").set.call(e, v);
      e.dispatchEvent(new Event("input", { bubbles: true }));
    },
    el,
    t,
  );
  await (await $("#chat-send")).click();
}

/** Every file under `dir`, skipping `skip` (the fakes' own folder: the keychain and what the provider recorded). */
function files(dir, skip, outList = []) {
  if (!existsSync(dir)) return outList;
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (skip.includes(p)) continue;
    if (statSync(p).isDirectory()) files(p, skip, outList);
    else outList.push(p);
  }
  return outList;
}

describe("Codex on the ChatGPT plan", () => {
  // A first-time user: no host id or accounts yet (the data dir outlives a retried attempt).
  before(() => {
    rmSync(join(process.env.EMMET_DATA_DIR, "chatgpt"), { recursive: true, force: true });
    setOpenAi({});
  });
  after(async () => {
    // Leave the API key chosen for any spec after this one.
    await invoke("chatgpt_choose", { account: null });
  });

  it("invites an API-key user, signs in, and shows the notice once", async () => {
    await open(projectDir("chatgpt"));
    await instances(8);
    await settings();
    await (await $("#chatgpt-invite")).waitForExist({ timeout: 10_000 });
    expect(await text("#chatgpt-invite")).toContain("Use your ChatGPT plan in this app.");
    await screenshots("chatgpt-invite");

    const started = Date.now();
    await (await $("#chatgpt-continue")).click();
    await (await $("#plan-notice")).waitForExist({ timeout: 30_000 });
    const ms = Date.now() - started;
    console.log(`sign-in: click → signed in in ${ms} ms (Emmet's share; the person's time in the browser is on top)`);
    expect(ms).toBeLessThan(30_000);
    expect(await text("#plan-notice")).toContain("You're using your ChatGPT plan");
    expect(await text("#plan-notice")).toContain("Eligible usage in this app uses your ChatGPT plan.");
    await screenshots("chatgpt-notice");
    await (await $("#plan-notice-ok")).click();
    await waitFor("the notice stayed open", async () => !(await (await $("#plan-notice")).isExisting()));

    const picker = await text("#codex-pays");
    expect(picker).toContain("ChatGPT · builder@example.com");
    expect(picker).toContain("Active");
    expect(picker).toContain("OpenAI API key");
    await screenshots("chatgpt-accounts");
  });

  it("says Using ChatGPT plan by the composer and in the status bar", async () => {
    await closeSettings();
    // resume.jsonl: the first app-server, then the one a renewal restarts, which resumes the thread.
    scenario("codex", "resume");
    // The agent picker says who pays before the switch.
    await (await $("#agent-picker")).click();
    await browser.keys("Enter");
    const item = await $('[data-menu="agent-codex"]');
    await item.waitForDisplayed({ timeout: 10_000 });
    expect(await item.getText()).toContain("ChatGPT plan");
    await item.click();
    await (await $("#plan-label")).waitForExist({ timeout: 10_000 });
    expect(await text("#plan-label")).toContain("Using ChatGPT plan");
    await (await $("#plan-manage-usage")).click();
    await waitFor("Manage usage didn't open ChatGPT's usage page", () =>
      opened().includes("https://chatgpt.com/settings/usage"),
    );
  });

  it("stays responsive while a turn renews the token", async () => {
    // Ten minutes left, and a refresh that takes 2 s: the turn renews first, off the window's thread.
    const r = chatgptRecord("oaiapp_test1");
    setChatgptRecord("oaiapp_test1", { ...r, expires_at: Math.floor(Date.now() / 1000) + 600 });
    setOpenAi({ refresh_delay_ms: 2000 });
    // Timed from the send: a command on the window's thread blocks the click itself until the renewal is done.
    const started = Date.now();
    await say("Reply with exactly the word: ready");
    await browser.execute(() => 1);
    const ms = Date.now() - started;
    console.log(`window answered in ${ms} ms during the renewal`);
    expect(ms).toBeLessThan(1500);
    await idle();
    expect(chatgptRecord("oaiapp_test1").access_token).not.toBe(r.access_token);
    setOpenAi({});
  });

  it("shows the usage-limit card with Manage usage first", async () => {
    scenario("codex", "derived-plan-usage-limit");
    await (await $("#start-fresh")).click();
    await browser.waitUntil(async () => (await browser.execute(() => window.__emmet.chat().session)) == null, { timeout: 10_000 });
    await (await $("#plan-label")).waitForExist({ timeout: 10_000 });
    await say("Reply with exactly the word: ready");
    await (await $("#limit-card")).waitForExist({ timeout: 30_000 });
    const card = await text("#limit-card");
    expect(card).toContain("ChatGPT");
    expect(card).toContain("Usage limit reached");
    expect(card).toContain("Review your plan or this app's limit in ChatGPT settings.");
    expect(await (await $("#limit-manage-usage")).getAttribute("data-variant")).toBe("primary");
    expect(card).not.toMatch(/resets/i);
    await screenshots("chatgpt-limit");
  });

  it("handles a decline, and never shows the notice twice", async () => {
    await settings();
    setOpenAi({ decline: true });
    await (await $("#chatgpt-add")).click();
    await waitFor("no decline message", async () => (await text("#chatgpt-message")).includes("isn't enabled"));
    setOpenAi({});

    await (await $("[data-account] .sign-out")).click();
    await waitFor("not signed out", async () => (await text("#codex-pays")).includes("Signed out"));
    // Signing out stops requests: the chat on that account ended.
    await waitFor("the plan chat kept running", async () => (await browser.execute(() => window.__emmet.chat().session)) == null);
    await (await $("[data-account] .sign-in")).click();
    await waitFor("not signed in again", async () => (await text("#codex-pays")).includes("Signed in"));
    expect(await (await $("#plan-notice")).isExisting()).toBe(false);
    await (await $("[data-account] .sign-out")).click();
    await waitFor("not signed out", async () => (await text("#codex-pays")).includes("Signed out"));
    await closeSettings();
  });

  it("leaves no token in the app's data, the project, or the logs", async () => {
    const tokens = issuedTokens();
    expect(tokens.length).toBeGreaterThan(3);
    const skip = [fakeDir, join(out, "openai")];
    const all = [...files(process.env.EMMET_DATA_DIR, skip), ...files(projectDir("chatgpt"), skip), ...files(out, [...skip, join(out, "projects"), join(out, "screens")])];
    expect(all.length).toBeGreaterThan(3);
    for (const f of all) {
      const body = readFileSync(f, "latin1");
      for (const t of tokens) expect([f, body.includes(t)]).toEqual([f, false]);
    }
  });
});
