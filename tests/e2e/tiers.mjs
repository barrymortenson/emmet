// SPDX-License-Identifier: AGPL-3.0-or-later
// Which E2E specs a change runs (plan section 16). Every spec file is in exactly one tier:
//
// - core: the flows that prove the app works. Any change that can reach the app runs them.
// - an area: specs for one part of the app, run when that part changes (or the spec itself does).
// - extra: design states, memory, and timing checks, run by hand (`just e2e all`), and by CI only when their own spec
//   or baselines change. A test inside a core or area spec can be extra too (extraIt).
//
// scripts/ci-scope.sh and `just e2e` both ask this file, and tests/scripts/e2e-tiers.test.mjs checks that every spec
// file has a tier. `node tests/e2e/tiers.mjs [--changed | all | core | windows | <spec,...>]` prints the specs to run,
// comma-separated, with hello first: changed paths come on stdin for --changed; no argument is core and every area.
import { pathToFileURL } from "node:url";

/** The app's first window, run on its own first: when it fails, the machine can't run E2E, and nothing after it is
 * worth waiting for. */
export const GATE = "hello";

export const CORE = ["viewer", "panel", "chat", "checks", "trust", "runs", "versions", "import", "render", "visual", "states"];

/** Each area: the paths that are its code, and the specs that cover it beyond core. */
export const AREAS = {
  // The viewer package: picking and the camera, the build animation, and the load and frame budgets (session 4C).
  viewer: { paths: /^packages\/viewer\//, specs: ["interact", "animate", "perf"] },
  // The agent bridge, the bus, the MCP host, and the chat UI around them, with photo to model (4D).
  agent: {
    paths:
      /^(crates\/emmet-(agent|bus)\/|crates\/emmet-cli\/src\/(mcp|host)\.rs$|crates\/emmet-core\/src\/refs\.rs$|tests\/(fake-agents|replay)\/|tests\/fixtures\/photos\/|apps\/desktop\/src\/(Agents|Chat|ChatGpt|Usage|Compare)\.tsx$|apps\/desktop\/src\/photos\.ts$|apps\/desktop\/src-tauri\/src\/(chat|chatgpt|setup)\.rs$)/,
    specs: ["chatgpt", "setup", "bus", "photo", "dock"],
  },
  // The shell: menus and keys, which every screen shares, and what the window may do (the capabilities).
  shell: { paths: /^apps\/desktop\/(src\/|src-tauri\/src\/menu\.rs$|src-tauri\/capabilities\/)/, specs: ["keyboard"] },
  // Building instructions (4E): the pictures drawn for a book, its layout, and the PDF, against per-page baselines; and
  // the export command that writes it (projects.rs) and the steps it's laid out from.
  instructions: {
    paths:
      /^(crates\/emmet-core\/src\/(instructions\/|steps\.rs$)|packages\/viewer\/src\/|apps\/desktop\/src\/(instructions|render)\.ts$|apps\/desktop\/src-tauri\/src\/projects\.rs$|tests\/e2e\/(baselines\/[^/]+\/instructions\/|specs\/instructions\.e2e\.mjs$))/,
    specs: ["instructions"],
  },
  // First run: the library download and the Python setup, on a fresh data dir.
  "first-run": {
    paths:
      /^(apps\/desktop\/src\/FirstRun\.tsx$|apps\/desktop\/src-tauri\/src\/(setup|library)\.rs$|crates\/emmet-core\/src\/(library|python|data)\.rs$|crates\/emmet-agent\/src\/setup\.rs$)/,
    specs: ["first-run"],
  },
};

/** Windows E2E (a manual run while D-43 pauses Windows): what behaves differently there, paths, processes, the
 * sidecars, and Python under uv, plus perf, since section 13's budgets hold on both OSes and WebView2 draws with its
 * own GPU path. The UI runs the same JS in WebView2, so the Mac covers the rest. */
export const WINDOWS = ["viewer", "chat", "setup", "runs", "trust", "versions", "import", "render", "visual", "perf", "instructions"];

/** By hand (and when they change): the frozen screens as screenshots against their baselines, memory over 100 changes.
 * The chat's states (dock) moved to the agent area in 5A: they assert the tips, the pending redirect, and the limit
 * and sign-in cards, compare no baseline, and the test matrix needs CI to run them. */
export const EXTRA = ["screens", "leak"];

/** Not an E2E run: `just bench` runs it, by hand. */
export const BENCH = "bench";

/** First run starts the app on its own fresh data dir, so it's a run of its own (wdio.conf.mjs). */
export const FIRST_RUN = "first-run";

/** A change to any of these can reach the app: it runs core. */
export const APP = /^(apps\/|crates\/|packages\/|python\/emmet_dsl\/src\/|data\/|tests\/(e2e|fixtures|replay|fake-agents)\/)/;

/** The harness itself: it can break any spec, so it runs core and every area. */
const HARNESS = /^tests\/e2e\/(lib\/|wdio\.conf\.mjs$|package\.json$|tiers\.mjs$)/;

const areaSpecs = () => Object.values(AREAS).flatMap((a) => a.specs);

/** Every spec in a tier, for the check that none is missing. */
export function tiered() {
  return [GATE, ...CORE, ...areaSpecs(), ...EXTRA, BENCH];
}

/** True when the extra tests run: `just e2e all`. */
export const extra = process.env.EMMET_E2E_ALL === "1";

/** A test that runs only with `just e2e all`: it shows as pending in a PR's run, never silently gone. */
export function extraIt(title, fn) {
  return (extra ? globalThis.it : globalThis.it.skip)(title, fn);
}

/** A quarantined test (plan section 16's flake policy): one that needed its retry twice in a week, skipped until
 * it's fixed, which must be within the phase. `issue` is the GitHub issue that tracks it, in full; a quarantine without
 * one fails tests/scripts/ci.test.mjs, and the job summary lists every quarantined test (scripts/ci-summary.mjs).
 * Nothing is quarantined silently. */
export function quarantine(issue, title, fn) {
  return globalThis.it.skip(`${title} (quarantined: ${issue})`, fn);
}

/** The quarantine's issue: this repo's, by number. */
export const ISSUE = /^https:\/\/github\.com\/barrymortenson\/emmet\/issues\/\d+$/;

/** CI's Mac E2E runs as two jobs side by side, so a full run fits the time budget (plan section 16): `core` takes the
 * core specs, `areas` everything else in the list (the areas, first run, a by-hand spec that changed). Each starts
 * with hello; a shard with nothing but hello has nothing to run. */
export const SHARDS = ["core", "areas"];
export function shard(specs, which) {
  if (!SHARDS.includes(which)) throw new Error(`tests/e2e/tiers.mjs: no shard ${which} (${SHARDS.join(" or ")})`);
  const mine = specs.filter((s) => s !== GATE && CORE.includes(s) === (which === "core"));
  return mine.length ? order([GATE, ...mine]) : [];
}

/** The specs a set of changed paths runs: nothing when none can reach the app, else core plus each area changed,
 * plus any spec whose own file changed. */
export function forChanges(paths) {
  const list = paths.filter(Boolean);
  if (!list.some((p) => APP.test(p))) return [];
  const pick = new Set([GATE, ...CORE]);
  const harness = list.some((p) => HARNESS.test(p));
  for (const area of Object.values(AREAS)) {
    if (harness || list.some((p) => area.paths.test(p))) for (const s of area.specs) pick.add(s);
  }
  for (const p of list) {
    const own = p.match(/^tests\/e2e\/specs\/([\w-]+)\.e2e\.mjs$/)?.[1];
    if (own && own !== BENCH) pick.add(own);
    // A screen baseline changed: the spec that compares it.
    if (/^tests\/e2e\/baselines\/[^/]+\/screens\//.test(p)) pick.add("screens");
  }
  return order([...pick]);
}

/** What `just e2e` runs with no argument (and a full CI run): core and every area. `all` adds the extra tier. */
export function every(withExtra = false) {
  return order([GATE, ...CORE, ...areaSpecs(), ...(withExtra ? EXTRA : [])]);
}

/** hello first, first-run last (its own run), the rest in tier order. */
function order(specs) {
  const rank = tiered();
  return [...new Set(specs)].sort((a, b) => {
    const key = (s) => (s === GATE ? -1 : s === FIRST_RUN ? 1e6 : rank.indexOf(s));
    return key(a) - key(b);
  });
}

// The command line: what to run.
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const arg = process.argv[2] ?? "";
  let specs;
  if (arg === "--changed") {
    const input = await new Promise((resolve) => {
      let text = "";
      process.stdin.on("data", (d) => (text += d)).on("end", () => resolve(text));
    });
    specs = forChanges(input.split("\n").map((l) => l.trim()));
  } else if (arg === "--shard") {
    // `--shard core hello,viewer,…`: that shard's part of the list.
    specs = shard((process.argv[4] ?? "").split(/[,\s]+/).filter(Boolean), process.argv[3]);
  } else if (arg === "" || arg === "all") {
    specs = every(arg === "all");
  } else if (arg === "core") {
    specs = order([GATE, ...CORE]);
  } else if (arg === "windows") {
    specs = order([GATE, ...WINDOWS]);
  } else {
    const known = new Set(tiered());
    const asked = arg.split(/[,\s]+/).filter(Boolean);
    for (const s of asked) if (!known.has(s) || s === BENCH) throw new Error(`tests/e2e/tiers.mjs: no E2E spec ${s}`);
    specs = order([GATE, ...asked]);
  }
  process.stdout.write(`${specs.join(",")}\n`);
}
