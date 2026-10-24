// SPDX-License-Identifier: AGPL-3.0-or-later
// scripts/plan-check.mjs: passes on the real plan and fails on each broken case in plan section 20.
import assert from "node:assert/strict";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempDir, writeFiles } from "./helpers.mjs";

const PLAN = `
<span class="pill gate">G1</span>
<div class="session" id="session-0a" data-session="0A" data-phase="0" data-title="A" data-after="" data-waits="H-01" data-waits-merge="H-02">
<div class="session" id="session-0b" data-session="0B" data-phase="0" data-title="B" data-after="0A" data-waits="" data-waits-merge="">
<div class="session" id="session-1a" data-session="1A" data-phase="1" data-title="C" data-after="0B" data-waits="" data-waits-merge="">
<tr data-h="H-01"><td>H-01</td></tr>
<tr data-h="H-02"><td>H-02</td></tr>`;

const good = () => ({
  sessions: { "0A": { status: "todo" }, "0B": { status: "todo" }, "1A": { status: "todo" } },
  human: { "H-01": { status: "todo" }, "H-02": { status: "todo" } },
  blockers: [],
  gates: { G1: { status: "todo", note: "" } },
});

function check(status, plan = PLAN) {
  const dir = tempDir();
  const js = typeof status === "string" ? status : `window.EMMET_STATUS = ${JSON.stringify(status)};`;
  writeFiles(dir, { "plan.html": plan, "status.js": js });
  return run("node", [join(ROOT, "scripts/plan-check.mjs"), join(dir, "plan.html"), join(dir, "status.js")]);
}

function fails(t, mutate, message, plan) {
  test(`fails when ${t}`, () => {
    const s = good();
    mutate(s);
    const r = check(s, plan);
    assert.equal(r.code, 1, r.out);
    assert.match(r.out, message);
  });
}

test("passes on the real plan and status file", () => {
  const r = run("node", ["scripts/plan-check.mjs"], { cwd: ROOT });
  assert.equal(r.code, 0, r.out);
  assert.match(r.out, /plan-check: ok/);
});

test("passes on a good fixture", () => {
  const r = check(good());
  assert.equal(r.code, 0, r.out);
});

fails("a status session isn't in the plan", (s) => (s.sessions["9Z"] = { status: "todo" }), /session 9Z is in the status file but not in the plan/);
fails("a plan session is missing from the status file", (s) => delete s.sessions["1A"], /session 1A is in the plan but not in the status file/);
fails("a human step isn't in the plan", (s) => (s.human["H-99"] = { status: "todo" }), /human step H-99 is in the status file/);
fails("a gate isn't in the plan", (s) => (s.gates.G7 = { status: "todo" }), /gate G7 is in the status file/);
fails("a session status is invalid", (s) => (s.sessions["0A"].status = "finished"), /session 0A: invalid status "finished"/);
fails("a human status is invalid", (s) => (s.human["H-01"].status = "yes"), /human step H-01: invalid status/);
fails("a gate status is invalid", (s) => (s.gates.G1.status = "maybe"), /gate G1: invalid status/);
fails("done has no PR", (s) => (s.sessions["0A"] = { status: "done", evidence: ["just ci: ok"] }), /0A: done needs a pr/);
fails("done has no evidence", (s) => (s.sessions["0A"] = { status: "done", pr: 1, evidence: [] }), /0A: done needs a pr number and at least one evidence/);
fails("skipped has no note", (s) => (s.sessions["0A"] = { status: "skipped" }), /0A: skipped needs Barry's note/);
fails("blocked has no blocker", (s) => (s.sessions["0A"].status = "blocked"), /0A: blocked needs an open blocker/);
fails(
  "blocked has only a closed blocker",
  (s) => {
    s.sessions["0A"].status = "blocked";
    s.blockers = [{ id: "BLK-001", session: "0A", text: "x", opened: "2026-10-24", closed: "2026-10-25" }];
  },
  /0A: blocked needs an open blocker/,
);
fails("a blocker names a missing session", (s) => (s.blockers = [{ id: "BLK-001", session: "9Z", closed: null }]), /BLK-001: session 9Z doesn't exist/);
fails("a session starts before the one it comes after is done", (s) => (s.sessions["0B"].status = "in-progress"), /session 0B is in-progress but 0A, which it comes after, is todo/);
fails("a session is in review before the one it comes after is done", (s) => (s.sessions["0B"] = { status: "review", pr: 2 }), /session 0B is review but 0A, which it comes after, is todo/);
fails(
  "a session is done before the one it comes after is done",
  (s) => {
    s.sessions["0A"].status = "in-progress";
    s.sessions["0B"] = { status: "done", pr: 2, evidence: ["ci green"] };
  },
  /session 0B is done but 0A, which it comes after, is in-progress/,
);
fails("data-after names a missing session", () => {}, /session 1A: data-after names 0C, which doesn't exist/, PLAN.replace('data-after="0B"', 'data-after="0C"'));
fails("a session waits on a missing human step", () => {}, /session 0A: waits on H-03, which doesn't exist/, PLAN.replace('data-waits-merge="H-02"', 'data-waits-merge="H-03"'));

test("an out-of-order start Barry allowed is a warning, not a failure", () => {
  const s = good();
  s.sessions["0B"] = { status: "in-progress", override: "Barry 2026-10-24: start early" };
  const r = check(s);
  assert.equal(r.code, 0, r.out);
  assert.match(r.out, /warning: session 0B is in-progress but 0A.*\(allowed: Barry 2026-10-24: start early\)/);
});

test("a session that hasn't started may wait on unfinished sessions", () => {
  const s = good();
  s.sessions["0B"].status = "blocked";
  s.blockers = [{ id: "BLK-001", session: "0B", text: "x", opened: "2026-10-24", closed: null }];
  const r = check(s);
  assert.equal(r.code, 0, r.out);
});

test("done and skipped both satisfy after", () => {
  const s = good();
  s.sessions["0A"] = { status: "done", pr: 1, evidence: ["ci green"] };
  s.sessions["0B"] = { status: "skipped", note: "Barry: not needed" };
  s.sessions["1A"] = { status: "review", pr: 3 };
  assert.equal(check(s).code, 0);
});

test("fails when the status file doesn't set EMMET_STATUS", () => {
  const r = check("var x = 1;");
  assert.equal(r.code, 1);
  assert.match(r.out, /doesn't set window.EMMET_STATUS/);
});

test("fails when the status file isn't valid JavaScript", () => {
  const r = check("window.EMMET_STATUS = {");
  assert.equal(r.code, 1);
  assert.match(r.out, /can't read/);
});
