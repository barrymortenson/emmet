// SPDX-License-Identifier: AGPL-3.0-or-later
// scripts/scrub-transcript: removes paths, names, and secrets (also when split across streaming chunks), is
// idempotent, and every committed replay transcript is small and already scrubbed.
import assert from "node:assert/strict";
import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { homedir, hostname, tmpdir, userInfo } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempDir } from "./helpers.mjs";

const SCRUB = join(ROOT, "scripts/scrub-transcript");
const HOME = homedir();
const USER = userInfo().username;
const KEY = "sk-ant-api03-" + "x".repeat(40);

function scrub(lines, ...args) {
  const dir = tempDir();
  const input = join(dir, "in.jsonl");
  const output = join(dir, "out.jsonl");
  writeFileSync(input, typeof lines === "string" ? lines : lines.map((l) => JSON.stringify(l)).join("\n") + "\n");
  const r = run("node", [SCRUB, input, output, ...args]);
  return { ...r, text: r.code === 0 ? readFileSync(output, "utf8") : "", lines: r.code === 0 ? readFileSync(output, "utf8").trim().split("\n").map((l) => JSON.parse(l)) : [] };
}

const delta = (index, key, value) => ({ dir: "out", msg: { type: "stream_event", session_id: "s1", event: { type: "content_block_delta", index, delta: { type: key === "text" ? "text_delta" : "input_json_delta", [key]: value } } } });

test("scrubs home and temp paths, spike workspaces, the username, the hostname, and secrets", () => {
  const ws = "/private/tmp/emmet-spike-tool-call-Ab3dE9";
  const r = scrub([
    { t: 0, dir: "out", msg: { type: "system", subtype: "init", cwd: `${ws}/project`, apiKeySource: "ANTHROPIC_API_KEY", session_id: "b9909fb8-ae19-48f5-ac07-f898b2fb07d2" } },
    { t: 1, dir: "out", msg: { text: `see ${HOME}/notes and ${join(tmpdir(), "x")} as ${USER} on ${hostname()}` } },
    { t: 2, dir: "out", msg: { error: `Incorrect API key provided: sk-proj-****************abcd`, key: KEY, apiKey: "plain-looking-value", auth: "Bearer abcdefghijklmnopqrstuvwxyz" } },
    { t: 3, dir: "out", msg: { who: "someone@example.com", jwt: "eyJhbGciOiJIUzI1.eyJzdWIiOiIxMjM0.SflKxwRJSMeKKF2QT4fw" } },
    { t: 4, dir: "out", msg: { memory: `${HOME.replace(/[^A-Za-z0-9]/g, "-")}-projects` } },
  ]);
  assert.equal(r.code, 0, r.out);
  const [init, text, secrets, ids, encoded] = r.lines.map((l) => l.msg);
  assert.equal(init.cwd, "<WS>/project");
  assert.equal(init.apiKeySource, "ANTHROPIC_API_KEY", "the name of a variable isn't a secret");
  assert.equal(init.session_id, "b9909fb8-ae19-48f5-ac07-f898b2fb07d2");
  assert.equal(text.text, "see <HOME>/notes and <TMP>/x as <USER> on <HOST>");
  assert.equal(secrets.error, "Incorrect API key provided: <REDACTED>");
  assert.equal(secrets.key, "<REDACTED>");
  assert.equal(secrets.apiKey, "<REDACTED>");
  assert.equal(secrets.auth, "Bearer <REDACTED>");
  assert.deepEqual(ids, { who: "<EMAIL>", jwt: "<REDACTED>" });
  assert.equal(encoded.memory, "<HOME>-projects");
});

test("scrubs a path or a key split across streaming chunks, keeping the joined content exact", () => {
  const path = `${HOME}/cottage/model.py`;
  const cut = Math.floor(HOME.length / 2);
  const r = scrub([
    delta(0, "partial_json", `{"file_path":"${path.slice(0, cut)}`),
    delta(0, "partial_json", path.slice(cut, cut + 3)),
    delta(0, "partial_json", `${path.slice(cut + 3)}"}`),
    delta(1, "text", "untouched "),
    delta(1, "text", "chunks"),
    { dir: "out", msg: { method: "item/agentMessage/delta", params: { itemId: "i1", delta: KEY.slice(0, 10) } } },
    { dir: "out", msg: { method: "item/agentMessage/delta", params: { itemId: "i1", delta: KEY.slice(10) } } },
  ]);
  assert.equal(r.code, 0, r.out);
  const parts = r.lines.map((l) => l.msg.event?.delta ?? l.msg.params);
  assert.deepEqual(parts.slice(0, 3).map((d) => d.partial_json), ['{"file_path":"<HOME>/cottage/model.py"}', "", ""]);
  assert.deepEqual(parts.slice(3, 5).map((d) => d.text), ["untouched ", "chunks"], "a clean group keeps its chunking");
  assert.deepEqual(parts.slice(5).map((d) => d.delta), ["<REDACTED>", ""]);
  assert.doesNotMatch(r.text, new RegExp(HOME.slice(1, cut)));
});

test("is idempotent", () => {
  const first = scrub([{ dir: "out", msg: { text: `${HOME}/a ${KEY}` } }, delta(0, "text", HOME.slice(0, 4)), delta(0, "text", HOME.slice(4))]);
  const second = scrub(first.text);
  assert.equal(second.code, 0, second.out);
  assert.equal(second.text, first.text);
});

test("renames Anthropic message ids consistently, so gitleaks' generic-api-key rule has nothing to flag", () => {
  const a = "msg_011CgMy4YmEXcJtJyRmBHETo";
  const b = "msg_022DhNz5ZnFYdKuKzSnCIFUp";
  const r = scrub([
    { dir: "out", msg: { message: { id: a }, api_message_id: a } },
    { dir: "out", msg: { message: { id: b }, api_message_id: b } },
    { dir: "out", msg: { api_message_id: a } },
  ]);
  assert.equal(r.code, 0, r.out);
  assert.deepEqual(r.lines.map((l) => l.msg.api_message_id), ["msg_0001", "msg_0002", "msg_0001"]);
  assert.equal(r.lines[0].msg.message.id, "msg_0001");
});

test("--path and --redact add mappings", () => {
  const r = scrub([{ dir: "out", msg: { text: "/opt/projects/Cottage built by Pat" } }], "--path", "/opt/projects/Cottage=<PROJECT>", "--redact", "Pat");
  assert.equal(r.code, 0, r.out);
  assert.equal(r.lines[0].msg.text, "<PROJECT> built by <REDACTED>");
});

test("fails on a line that isn't JSON", () => {
  const r = scrub('{"ok":true}\nnot json\n');
  assert.equal(r.code, 1);
  assert.match(r.out, /line 2 is not JSON/);
});

test("every committed replay transcript is under 200 KB and already scrubbed", () => {
  for (const cli of ["claude", "codex"]) {
    const dir = join(ROOT, "tests/replay", cli);
    const files = readdirSync(dir).filter((f) => f.endsWith(".jsonl"));
    assert.ok(files.length > 0, `no transcripts in tests/replay/${cli}`);
    for (const f of files) {
      const path = join(dir, f);
      assert.ok(statSync(path).size < 200 * 1024, `${cli}/${f} is ${statSync(path).size} bytes`);
      const text = readFileSync(path, "utf8");
      assert.equal(scrub(text).text, text, `${cli}/${f} changes when scrubbed again`);
      assert.ok(!text.includes(HOME), `${cli}/${f} contains the home directory`);
    }
  }
});
