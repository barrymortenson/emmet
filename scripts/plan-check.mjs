#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Checks docs/plan-status.js against docs/plan.html (plan section 20). Node only, no dependencies.
// Usage: node scripts/plan-check.mjs [plan.html] [plan-status.js]. Exits 1 on any error.
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";

const SESSION = new Set(["todo", "in-progress", "review", "done", "blocked", "skipped", "backlog"]);
const HUMAN = new Set(["todo", "waiting", "done", "not-needed"]);
const GATE = new Set(["todo", "go", "no-go", "pivot"]);
const SATISFIES_AFTER = new Set(["done", "skipped"]);
const STARTED = new Set(["in-progress", "review", "done"]);

const words = (s) => (s || "").split(/\s+/).filter(Boolean);

/** Session ids with their `after` and YOU-step lists, human step ids, and gate ids, read from the plan. */
function readPlan(html) {
  const sessions = new Map();
  for (const [tag] of html.matchAll(/<div class="session"[^>]*>/g)) {
    const attr = (name) => tag.match(new RegExp(` ${name}="([^"]*)"`))?.[1];
    sessions.set(attr("data-session"), {
      after: words(attr("data-after")),
      waits: [...words(attr("data-waits")), ...words(attr("data-waits-merge"))],
    });
  }
  const human = new Set([...html.matchAll(/<tr data-h="([^"]+)"/g)].map((m) => m[1]));
  const gates = new Set([...html.matchAll(/class="pill gate">(G\d+)</g)].map((m) => m[1]));
  return { sessions, human, gates };
}

function readStatus(js) {
  const window = {};
  runInNewContext(js, { window });
  return window.EMMET_STATUS;
}

function check(plan, status) {
  const errors = [];
  const warnings = [];
  const sessions = status.sessions || {};
  const human = status.human || {};
  const gates = status.gates || {};
  const blockers = status.blockers || [];

  const sameIds = (kind, planIds, statusIds) => {
    for (const id of planIds) if (!statusIds.includes(id)) errors.push(`${kind} ${id} is in the plan but not in the status file`);
    for (const id of statusIds) if (!planIds.includes(id)) errors.push(`${kind} ${id} is in the status file but not in the plan`);
  };
  sameIds("session", [...plan.sessions.keys()], Object.keys(sessions));
  sameIds("human step", [...plan.human], Object.keys(human));
  sameIds("gate", [...plan.gates], Object.keys(gates));

  for (const [id, s] of plan.sessions) {
    for (const a of s.after) if (!plan.sessions.has(a)) errors.push(`session ${id}: data-after names ${a}, which doesn't exist`);
    for (const h of s.waits) if (!plan.human.has(h)) errors.push(`session ${id}: waits on ${h}, which doesn't exist`);
  }

  const stat = (id) => sessions[id]?.status;
  for (const [id, r] of Object.entries(sessions)) {
    if (!SESSION.has(r.status)) {
      errors.push(`session ${id}: invalid status "${r.status}"`);
      continue;
    }
    if (r.status === "done" && !(Number.isInteger(r.pr) && Array.isArray(r.evidence) && r.evidence.length > 0))
      errors.push(`session ${id}: done needs a pr number and at least one evidence line`);
    if (r.status === "skipped" && !r.note) errors.push(`session ${id}: skipped needs Barry's note`);
    if (r.status === "blocked" && !blockers.some((b) => b.session === id && !b.closed))
      errors.push(`session ${id}: blocked needs an open blocker for ${id}`);
    if (STARTED.has(r.status) && plan.sessions.has(id)) {
      for (const a of plan.sessions.get(id).after) {
        if (SATISFIES_AFTER.has(stat(a))) continue;
        const msg = `session ${id} is ${r.status} but ${a}, which it comes after, is ${stat(a) || "missing"}`;
        if (r.override) warnings.push(`${msg} (allowed: ${r.override})`);
        else errors.push(msg);
      }
    }
  }
  for (const [id, r] of Object.entries(human)) if (!HUMAN.has(r.status)) errors.push(`human step ${id}: invalid status "${r.status}"`);
  for (const [id, r] of Object.entries(gates)) if (!GATE.has(r.status)) errors.push(`gate ${id}: invalid status "${r.status}"`);
  for (const b of blockers) {
    if (!/^BLK-\d{3}$/.test(b.id || "")) errors.push(`blocker ${b.id}: the id must look like BLK-001`);
    if (!plan.sessions.has(b.session)) errors.push(`blocker ${b.id}: session ${b.session} doesn't exist`);
  }
  return { errors, warnings };
}

const [planPath = "docs/plan.html", statusPath = "docs/plan-status.js"] = process.argv.slice(2);
let plan, status;
try {
  plan = readPlan(readFileSync(planPath, "utf8"));
  status = readStatus(readFileSync(statusPath, "utf8"));
} catch (e) {
  console.error(`plan-check: can't read the plan or the status file: ${e.message}`);
  process.exit(1);
}
if (!status || typeof status !== "object") {
  console.error(`plan-check: ${statusPath} doesn't set window.EMMET_STATUS`);
  process.exit(1);
}
const { errors, warnings } = check(plan, status);
for (const w of warnings) console.warn(`warning: ${w}`);
for (const e of errors) console.error(`error: ${e}`);
if (errors.length) {
  console.error(`plan-check: ${errors.length} error(s)`);
  process.exit(1);
}
console.log(`plan-check: ok (${plan.sessions.size} sessions, ${plan.human.size} human steps, ${plan.gates.size} gates)`);
