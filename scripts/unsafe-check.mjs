#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Unsafe code only in the E2E build's one call into WebKit (D-50). The workspace lint denies unsafe code rather than
// forbidding it, so that file can allow it, and a `#[allow(unsafe_code)]` anywhere else would pass the compiler: this
// keeps every other Rust file free of it, as forbid did. `just check` runs it, so every Rust change does.
// Usage: node scripts/unsafe-check.mjs [repo dir]  (default: this repo)
import { execFileSync } from "node:child_process";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const ALLOWED = ["apps/desktop/src-tauri/src/e2e.rs"];
const repo = process.argv[2] ?? join(fileURLToPath(new URL(".", import.meta.url)), "..");

let found = "";
try {
  found = execFileSync("git", ["grep", "-l", "--untracked", "-w", "-e", "unsafe", "-e", "unsafe_code", "--", "*.rs"], { cwd: repo, encoding: "utf8" });
} catch (e) {
  // git grep exits 1 when nothing matches.
  if (e.status !== 1) throw e;
}
const bad = found.split("\n").filter((f) => f && !ALLOWED.includes(f));
if (bad.length) {
  console.error(`unsafe-check: unsafe code outside ${ALLOWED.join(", ")} (D-50):\n${bad.map((f) => `  ${f}`).join("\n")}`);
  process.exit(1);
}
console.log("unsafe-check: ok");
