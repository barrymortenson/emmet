// SPDX-License-Identifier: AGPL-3.0-or-later
// Shared helpers for the script tests: run a command, make a temp dir or a temp git repo.
import { spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync, mkdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

/** Runs a command; returns { code, out } with stdout and stderr together. */
export function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, { encoding: "utf8", ...opts, env: { ...process.env, ...opts.env } });
  if (r.error) throw r.error;
  return { code: r.status, out: `${r.stdout}${r.stderr}` };
}

export function tempDir() {
  return mkdtempSync(join(tmpdir(), "emmet-test-"));
}

/** Writes { "path": "content" } under dir. */
export function writeFiles(dir, files) {
  for (const [path, content] of Object.entries(files)) {
    mkdirSync(dirname(join(dir, path)), { recursive: true });
    writeFileSync(join(dir, path), content);
  }
}

/** A fresh git repo with a fixed test identity and no hooks or signing from the user's config. */
export function tempRepo() {
  const dir = tempDir();
  const git = (...args) => {
    const r = run("git", args, { cwd: dir, env: { GIT_CONFIG_GLOBAL: "/dev/null", GIT_CONFIG_NOSYSTEM: "1" } });
    if (r.code !== 0) throw new Error(`git ${args.join(" ")}: ${r.out}`);
    return r.out;
  };
  git("init", "-q", "-b", "main");
  git("config", "user.name", "Test Author");
  git("config", "user.email", "test@example.com");
  git("config", "commit.gpgsign", "false");
  return { dir, git };
}
