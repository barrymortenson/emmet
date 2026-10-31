// SPDX-License-Identifier: AGPL-3.0-or-later
// The projects each spec opens: fresh copies under tests/e2e/out/projects/, trusted with this data dir and put on the
// recents list, so the spec can open them with open_recent.
import { execFileSync } from "node:child_process";
import { cpSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { bin, out, repo } from "./paths.mjs";

/** Each spec's projects: [suffix, fixture, params]. The first is the spec's own (projectDir(spec)); the others are
 * projectDir(`${spec}-${suffix}`). "cottage" is the template emmet new makes. */
const PROJECTS = {
  hello: [],
  viewer: [["", "cottage"]],
  leak: [["", "cottage"]],
  render: [["", "cottage"], ["other", "scene-colors"]],
  runs: [["", "scene-grid"]],
  visual: [["", "cottage"], ["scene-colors", "scene-colors"], ["scene-grid", "scene-grid"]],
  bench: [["", "cottage"], ["grid500", "scene-grid"], ["grid2000", "scene-grid", { count: 2000 }]],
  chat: [["", "agent"], ["codex", "agent"]],
  dock: [["", "cottage"]],
  bus: [["", "cottage"]],
  chatgpt: [["", "agent"]],
  checks: [["", "broken-collision"]],
  setup: [["", "agent"]],
  panel: [["", "cottage"], ["floating", "broken-floating"]],
  trust: [["", "cottage-untrusted"]],
  import: [["", "cottage"]],
  versions: [["", "cottage"]],
  keyboard: [["", "cottage"]],
  screens: [["", "cottage"], ["floating", "broken-floating"], ["untrusted", "cottage-untrusted"]],
};

export function emmet(args, options = {}) {
  return execFileSync(bin("emmet"), ["--data-dir", process.env.EMMET_DATA_DIR, ...args], { encoding: "utf8", ...options });
}

export function projectDir(name) {
  return join(out, "projects", name);
}

/** Empties `dir`. On Windows the app's watcher can hold the folder itself open (a retried spec), so when the folder
 * can't go, its contents do. */
function clear(dir) {
  try {
    rmSync(dir, { recursive: true, force: true });
  } catch {
    for (const name of readdirSync(dir)) rmSync(join(dir, name), { recursive: true, force: true, maxRetries: 5 });
  }
}

function make(fixture, dir, params) {
  clear(dir);
  if (fixture === "cottage" || fixture === "cottage-untrusted") {
    emmet(["new", dir]);
    // Made here but not trusted, as a project from another computer is (D-27).
    if (fixture === "cottage-untrusted") rmSync(join(dir, ".emmet", "trust"), { force: true });
  } else {
    cpSync(join(repo, "tests", "fixtures", "projects", fixture), dir, { recursive: true });
    emmet(["--project", dir, "trust"]);
  }
  if (params) writeFileSync(join(dir, "params.json"), JSON.stringify(params));
}

/** The path Open… picks next (src-tauri/src/projects.rs's E2E seam). */
export function pickNext(path) {
  writeFileSync(process.env.EMMET_E2E_PICK_FILE, path);
}

/** Makes the spec's projects and lists them as its recents. */
export function prepareProject(spec) {
  const list = PROJECTS[spec];
  if (!list) throw new Error(`tests/e2e/lib/projects.mjs doesn't know the spec ${spec}`);
  const recent = list.map(([suffix, fixture, params]) => {
    const dir = projectDir(suffix ? `${spec}-${suffix}` : spec);
    make(fixture, dir, params);
    return { path: dir, name: suffix || spec };
  });
  writeFileSync(join(process.env.EMMET_DATA_DIR, "recent.json"), JSON.stringify(recent, null, 2));
}
