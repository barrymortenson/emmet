// SPDX-License-Identifier: AGPL-3.0-or-later
// scripts/scrub-transcript: removes paths, names, and secrets (also when split across streaming chunks), is
// idempotent, and every committed replay transcript is small and already scrubbed.
import assert from "node:assert/strict";
import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { homedir, hostname, tmpdir, userInfo } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempDir } from "./helpers.mjs";

// The script is a CLI and a module: these tests use the module to give it another machine's home, temp, user, and host.
const { scrubText } = await import(join(ROOT, "scripts/scrub-transcript"));
const lines = (text) => text.trim().split("\n").map((l) => JSON.parse(l).msg);

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
    { t: 5, dir: "out", msg: { auto: `${ws}/home/.claude/projects/-private-tmp-emmet-spike-tool-call-Ab3dE9-project/memory/` } },
  ]);
  assert.equal(r.code, 0, r.out);
  const [init, text, secrets, ids, encoded, memory] = r.lines.map((l) => l.msg);
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
  assert.equal(memory.auto, "<WS>/home/.claude/projects/<WS>-project/memory/");
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

test("--portable keeps this machine's paths and names but still removes every secret", () => {
  const r = scrub([{ dir: "out", msg: { text: `${HOME}/a ${USER} ${hostname()} ${KEY} pat@example.com eyJhbGciOiJIUzI1.eyJzdWIiOiIxMjM0.SflKxwRJSMeKKF2QT4fw msg_011CgMy4YmEXcJtJyRmBHETo` } }], "--portable");
  assert.equal(r.code, 0, r.out);
  assert.equal(r.lines[0].msg.text, `${HOME}/a ${USER} ${hostname()} <REDACTED> <EMAIL> <REDACTED> msg_0001`);
});

test("a dotted hostname is scrubbed, and so is a long bare one, but never a short word like Mac", () => {
  const text = (host) => lines(scrubText(JSON.stringify({ msg: { t: `Mac OS 15 on ${host}` } }) + "\n", { host, user: "nobodyx" }))[0].t;
  assert.equal(text("Mac.localdomain"), "Mac OS 15 on <HOST>");
  assert.equal(text("Pats-MacBook-Pro.local"), "Mac OS 15 on <HOST>");
  assert.equal(text("buildbox7"), "Mac OS 15 on <HOST>");
  assert.equal(text("Mac"), "Mac OS 15 on Mac");
});

test("on Linux (tmpdir /tmp), an encoded spike workspace still becomes <WS>", () => {
  const msg = { auto: "/tmp/emmet-spike-tool-call-Ab3dE9/home/.claude/projects/-private-tmp-emmet-spike-tool-call-Ab3dE9-project/m" };
  const out = lines(scrubText(JSON.stringify({ msg }) + "\n", { tmp: "/tmp", home: "/home/someone", user: "someone", host: "ci-box.example" }))[0];
  assert.equal(out.auto, "<WS>/home/.claude/projects/<WS>-project/m");
});

test("streaming chunks are joined only within their own message and item", () => {
  const msgStart = { dir: "out", msg: { type: "stream_event", session_id: "s1", event: { type: "message_start" } } };
  const r = scrub([
    msgStart,
    delta(0, "text", "clean "),
    delta(0, "text", "first message"),
    msgStart,
    delta(0, "text", KEY.slice(0, 12)),
    delta(0, "text", KEY.slice(12)),
    { dir: "out", msg: { method: "item/agentMessage/delta", params: { itemId: "a", delta: "clean " } } },
    { dir: "out", msg: { method: "item/agentMessage/delta", params: { itemId: "b", delta: KEY.slice(0, 12) } } },
    { dir: "out", msg: { method: "item/agentMessage/delta", params: { itemId: "a", delta: "item" } } },
    { dir: "out", msg: { method: "item/agentMessage/delta", params: { itemId: "b", delta: KEY.slice(12) } } },
  ]);
  assert.equal(r.code, 0, r.out);
  const text = r.lines.map((l) => l.msg.event?.delta?.text ?? l.msg.params?.delta).filter((x) => x !== undefined);
  assert.deepEqual(text, ["clean ", "first message", "<REDACTED>", "", "clean ", "<REDACTED>", "item", ""]);
});

test("a token-like key is redacted inside JSON strings too, even split across streaming chunks", () => {
  const r = scrub([
    delta(0, "partial_json", '{"command":"login","pass'),
    delta(0, "partial_json", 'word":"hunter2-pl'),
    delta(0, "partial_json", 'ain","user":"x"}'),
    { dir: "out", msg: { type: "user", content: [{ type: "tool_result", content: '{"access_token":"opaque-value","name":"ok"}' }] } },
    { dir: "out", msg: { text: '{"inner":"{\\"api_key\\":\\"abc123plain\\"}"}' } },
  ]);
  assert.equal(r.code, 0, r.out);
  assert.equal(r.lines.map((l) => l.msg.event?.delta?.partial_json).filter(Boolean).join(""), '{"command":"login","password":"<REDACTED>","user":"x"}');
  assert.equal(r.lines[3].msg.content[0].content, '{"access_token":"<REDACTED>","name":"ok"}');
  assert.equal(r.lines[4].msg.text, '{"inner":"{\\"api_key\\":\\"<REDACTED>\\"}"}');
  assert.doesNotMatch(r.text, /hunter2|opaque-value|abc123plain/);
});

test("any value under a credential-like key is redacted, and counts like input_tokens are not", () => {
  const msg = { token: "t0ken", OPENAI_API_KEY: "k", ANTHROPIC_AUTH_TOKEN: "a", "x-api-key": "x", sessionToken: "s", password: 12345678, secret: { value: "hunter2" }, client_secret: "c", secretKey: "sk", input_tokens: 42, max_tokens: 5, apiKeySource: "ANTHROPIC_API_KEY", text: '{"sessionToken":"abc","password":1234,"input_tokens":7}' };
  const r = scrub([{ dir: "out", msg }]);
  assert.equal(r.code, 0, r.out);
  const out = r.lines[0].msg;
  for (const k of ["token", "OPENAI_API_KEY", "ANTHROPIC_AUTH_TOKEN", "x-api-key", "sessionToken", "password", "secret", "client_secret", "secretKey"]) assert.equal(out[k], "<REDACTED>", k);
  assert.deepEqual([out.input_tokens, out.max_tokens, out.apiKeySource], [42, 5, "ANTHROPIC_API_KEY"]);
  assert.equal(out.text, '{"sessionToken":"<REDACTED>","password":"<REDACTED>","input_tokens":7}');
});

test("--path without a token is an error, not a replacement with 'undefined'", () => {
  const r = scrub([{ dir: "out", msg: { text: "/opt/x" } }], "--path", "/opt/x");
  assert.equal(r.code, 2);
  assert.match(r.out, /--path takes/);
});

test("--redact applies with --portable too", () => {
  const r = scrub([{ dir: "out", msg: { text: "built by Pat" } }], "--portable", "--redact", "Pat");
  assert.equal(r.lines[0].msg.text, "built by <REDACTED>");
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
      // --portable: the fixtures were scrubbed on Barry's Mac, and this machine's own home, temp, user, and host
      // rules would rewrite harmless text elsewhere (Python's "using /tmp instead" on Linux).
      assert.equal(scrub(text, "--portable").text, text, `${cli}/${f} changes when scrubbed again`);
      assert.ok(!text.includes(HOME), `${cli}/${f} contains the home directory`);
      assert.ok(!new RegExp(`\\b${USER}\\b`, "i").test(text), `${cli}/${f} contains the username`);
      // Anyone's home directory, not just this machine's (the scrubbed <WS>/home/.claude form is fine).
      assert.doesNotMatch(text, /(?<![>\w])\/(?:Users|home)\/[^/<"\s]+|[A-Za-z]:\\\\Users\\\\/, `${cli}/${f} contains a home path`);
    }
  }
});
