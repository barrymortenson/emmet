// SPDX-License-Identifier: AGPL-3.0-or-later
// Agent setup without a terminal (session 1F, D-41): from the chat pane's picker, with neither fake CLI installed,
// the provider notice comes before the first install (Not now runs and records nothing), and once accepted never
// again; offline, a checksum mismatch, and a cancel each say what happened with one Try again; the install then works,
// Claude Code's sign-in fails and is retried, and both agents end up ready, all by clicks. Antigravity's card adds
// Emmet's tools to the project. No agent config in the home changes.
import { $, expect } from "@wdio/globals";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { open, screenshots } from "../lib/app.mjs";
import { agentConfigHashes, fakeDir, install, installed } from "../lib/agents.mjs";
import { projectDir } from "../lib/projects.mjs";

const sentinels = JSON.parse(process.env.EMMET_E2E_SENTINELS ?? "{}");
const mode = (cli, m) => writeFileSync(join(fakeDir, `install-${cli}.mode`), m);
const login = (m) => writeFileSync(join(fakeDir, "claude-login.mode"), m);
const signedOut = () => writeFileSync(join(fakeDir, "claude-auth.json"), '{"loggedIn":false,"authMethod":"none"}');

async function click(selector) {
  const el = await $(selector);
  await el.waitForClickable({ timeout: 30_000 });
  await el.click();
}

/** The failure card's text, once it shows `words`, and its buttons. */
async function failure(kind, words) {
  const card = await $(`#failed-${kind}`);
  await card.waitForExist({ timeout: 30_000 });
  await browser.waitUntil(async () => (await card.getText()).includes(words), {
    timeout: 30_000,
    timeoutMsg: `#failed-${kind} never said "${words}": ${await card.getText()}`,
  });
  const buttons = await browser.execute(
    (id) => [...document.querySelectorAll(`#${id} > button`)].map((b) => b.textContent.trim()),
    `failed-${kind}`,
  );
  return { text: await card.getText(), buttons };
}

describe("agent setup without a terminal", () => {
  before(async () => {
    for (const cli of ["claude", "codex"]) {
      rmSync(installed[cli], { force: true });
      rmSync(join(fakeDir, `install-${cli}.log`), { force: true });
    }
    signedOut();
    rmSync(join(process.env.EMMET_DATA_DIR, "setup.json"), { force: true });
    rmSync(join(fakeDir, "opened.txt"), { force: true });
    await open(projectDir("setup"));
    // Detect again: the CLIs are gone.
    await browser.execute(() => [...document.querySelectorAll(".picker > button")].find((b) => b.textContent === "Look again")?.click());
    await (await $("#install-claude")).waitForExist({ timeout: 30_000 });
  });

  after(() => {
    // The specs after this one expect both CLIs, and Claude Code on its default Max plan.
    for (const cli of ["claude", "codex"]) if (!existsSync(installed[cli])) install(cli);
    rmSync(join(fakeDir, "claude-auth.json"), { force: true });
  });

  it("says which plans work, with the provider's page", async () => {
    await expect($("#setup-claude")).toHaveText(expect.stringContaining("Claude Pro, Max, Team, or Enterprise plan"));
    await expect($("#setup-codex")).toHaveText(expect.stringContaining("every ChatGPT plan, Free included"));
    await expect($("#agent-claude")).toBeDisabled();
    await screenshots("setup-picker");
    await click("#plans-claude");
    await browser.waitUntil(() => existsSync(join(fakeDir, "opened.txt")), { timeout: 10_000 });
    expect(readFileSync(join(fakeDir, "opened.txt"), "utf8")).toContain("https://code.claude.com/docs/en/setup#authenticate");
  });

  it("runs nothing and records nothing when the notice is put off", async () => {
    mode("claude", "offline");
    await click("#install-claude");
    await (await $("#provider-notice")).waitForDisplayed({ timeout: 10_000 });
    await browser.execute(() =>
      [...document.querySelectorAll("#provider-notice button")].find((b) => b.textContent.trim() === "Not now").click(),
    );
    await browser.waitUntil(async () => !(await (await $("#provider-notice")).isExisting()), { timeout: 10_000 });
    await browser.pause(500);
    expect(existsSync(join(fakeDir, "install-claude.log"))).toBe(false);
    expect(existsSync(join(process.env.EMMET_DATA_DIR, "setup.json"))).toBe(false);
  });

  it("shows the provider notice once, then says the install couldn't reach Anthropic", async () => {
    mode("claude", "offline");
    await click("#install-claude");
    await (await $("#provider-notice")).waitForDisplayed({ timeout: 10_000 });
    await expect($("#provider-notice")).toHaveText(expect.stringContaining("use is at your own risk"));
    expect(existsSync(join(fakeDir, "install-claude.log"))).toBe(false);
    await click("#provider-notice-ok");
    const f = await failure("claude", "couldn't reach Anthropic's download server");
    expect(f.buttons).toEqual(["Try again"]);
    await screenshots("setup-failed");
  });

  it("refuses a download that fails its checksum", async () => {
    mode("claude", "checksum");
    await click("#retry-claude");
    const f = await failure("claude", "didn't match the checksum Anthropic publishes");
    expect(f.buttons).toEqual(["Try again"]);
    // The notice came once.
    expect(await (await $("#provider-notice")).isExisting()).toBe(false);
  });

  it("cancels an install, and nothing lands afterward", async () => {
    mode("claude", "slow");
    await click("#retry-claude");
    await (await $("#job-claude")).waitForExist({ timeout: 10_000 });
    await expect($("#job-claude")).toHaveText(expect.stringContaining("Installing Claude Code"));
    await screenshots("setup-installing");
    await click("#cancel-claude");
    const f = await failure("claude", "Install canceled");
    expect(f.buttons).toEqual(["Try again"]);
    await browser.pause(4000);
    expect(existsSync(installed.claude)).toBe(false);
  });

  it("installs Claude Code, and a failed sign-in is retried", async () => {
    mode("claude", "ok");
    await click("#retry-claude");
    await (await $("#sign-in-claude")).waitForExist({ timeout: 30_000 });
    expect(existsSync(installed.claude)).toBe(true);

    login("fail");
    await click("#sign-in-claude");
    const f = await failure("claude", "Claude Code's sign-in didn't finish");
    expect(f.text).not.toContain("://");
    expect(f.buttons).toEqual(["Try again"]);
    await expect($("#terminal-sign-in")).toBeExisting();

    login("ok");
    await click("#retry-claude");
    await (await $("#agent-claude")).waitForEnabled({ timeout: 30_000 });
    const rec = JSON.parse(readFileSync(join(fakeDir, "claude-login.json"), "utf8"));
    expect(rec).toEqual({ args: ["auth", "login"], stdin_closed: true });
  });

  it("installs Codex", async () => {
    mode("codex", "ok");
    await click("#install-codex");
    // The file keychain holds an API key, so Codex is ready once it's installed.
    await (await $("#agent-codex")).waitForEnabled({ timeout: 30_000 });
    expect(await (await $("#setup-codex")).isExisting()).toBe(false);
  });

  it("adds Emmet's tools to the project for Antigravity", async () => {
    // Opened outright: a retried spec finds the card as the last attempt left it.
    await browser.execute(() => (document.getElementById("antigravity").open = true));
    await click("#antigravity-set-up");
    await (await $("#antigravity-ready")).waitForExist({ timeout: 10_000 });
    await screenshots("setup-antigravity");
    const config = JSON.parse(readFileSync(join(projectDir("setup"), ".agents", "mcp_config.json"), "utf8"));
    expect(config.mcpServers.emmet.args.slice(0, 2)).toEqual(["mcp", "--project"]);
  });

  it("left the user's agent configs untouched", () => {
    expect(agentConfigHashes()).toEqual(sentinels);
  });
});
