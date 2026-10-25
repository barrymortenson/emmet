// SPDX-License-Identifier: AGPL-3.0-or-later
// The project each spec opens: a fresh copy under tests/e2e/out/projects/, trusted with this data dir.
import { execFileSync } from "node:child_process";
import { cpSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { bin, out, repo } from "./paths.mjs";

/** Which project each spec file opens: the cottage template, or a fixture from tests/fixtures/projects/. */
const PROJECTS = {
  hello: null,
  viewer: "cottage",
  leak: "cottage",
  render: "cottage",
  runs: "scene-grid",
  visual: "cottage",
  bench: "cottage",
};

/** The scenes the visual and bench specs open after the first, through the recents list. */
export const EXTRA = ["scene-colors", "scene-grid"];

export function emmet(args, options = {}) {
  return execFileSync(bin("emmet"), ["--data-dir", process.env.EMMET_DATA_DIR, ...args], { encoding: "utf8", ...options });
}

export function projectDir(name) {
  return join(out, "projects", name);
}

function make(name, dir) {
  rmSync(dir, { recursive: true, force: true });
  if (name === "cottage") {
    emmet(["new", dir]);
  } else {
    cpSync(join(repo, "tests", "fixtures", "projects", name), dir, { recursive: true });
    emmet(["--project", dir, "trust"]);
  }
}

/** Makes the spec's project and returns its path, or null for a spec that opens none. */
export function prepareProject(spec) {
  const name = PROJECTS[spec];
  if (name === undefined) throw new Error(`tests/e2e/lib/projects.mjs doesn't know the spec ${spec}`);
  if (!name) return null;
  const dir = projectDir(spec);
  make(name, dir);
  // The extra scenes are on the recents list, so open_recent may open them.
  const recent = [{ path: dir, name: spec }];
  if (spec === "visual" || spec === "bench") {
    for (const extra of EXTRA) {
      const d = projectDir(`${spec}-${extra}`);
      make(extra, d);
      recent.push({ path: d, name: extra });
    }
  }
  writeFileSync(join(process.env.EMMET_DATA_DIR, "recent.json"), JSON.stringify(recent, null, 2));
  return dir;
}
