// SPDX-License-Identifier: AGPL-3.0-or-later
// Proves a spike run didn't write to the user's agent config (D-26, ADR 0001).
//
//   node config-snapshot.mjs save <out.json>
//   node config-snapshot.mjs diff <before.json> <after.json> <markers file> [--exclude-content <path prefix>]...
//
// `save` records every file under ~/.claude, ~/.codex and ~/.claude.json by size and mtime, and the sha256 of the
// config files proper (settings, instructions, MCP and hook config). Credential files are recorded by size and mtime
// only and are never opened. Both directories are live (the user's own sessions write to them), so `diff` doesn't
// expect zero changes. It fails if a config file's hash changed, or if any new or changed path, or the content of
// any new or changed non-credential file, contains a marker: the spike's unique working-directory name (the recorders
// append one per run to the markers file). --exclude-content skips the content scan (never the path check) under a
// prefix: the transcript of the interactive session running the spikes, which records their output by design.

import { createHash } from "node:crypto";
import { lstatSync, readdirSync, readFileSync, writeFileSync, existsSync, realpathSync } from "node:fs";
import { homedir } from "node:os";
import { join, relative } from "node:path";

const home = homedir();
const roots = [join(home, ".claude"), join(home, ".codex"), join(home, ".claude.json")];
const CONFIG = [
  ".claude/settings.json",
  ".claude/settings.local.json",
  ".claude/CLAUDE.md",
  ".claude.json#mcpServers",
  ".codex/config.toml",
  ".codex/hooks.json",
  ".codex/AGENTS.md",
];
const CONFIG_DIRS = [".claude/agents", ".claude/commands", ".claude/skills", ".codex/rules", ".codex/skills"];
const isCredential = (rel) => /(^|\/)(\.credentials\.json|auth\.json)$|token|secret/i.test(rel);

function walk(path, out) {
  let st;
  try {
    st = lstatSync(path);
  } catch {
    return;
  }
  const rel = relative(home, path);
  if (st.isDirectory()) {
    let names;
    try {
      names = readdirSync(path);
    } catch {
      return;
    }
    for (const n of names) walk(join(path, n), out);
  } else {
    out[rel] = { size: st.size, mtime: st.mtimeMs };
  }
}

const sha = (buf) => createHash("sha256").update(buf).digest("hex");

function configHashes() {
  const h = {};
  for (const c of CONFIG) {
    const [file, key] = c.split("#");
    const p = join(home, file);
    if (!existsSync(p)) {
      h[c] = null;
      continue;
    }
    const buf = readFileSync(realpathSync(p));
    h[c] = key ? sha(JSON.stringify(JSON.parse(buf)[key] ?? null)) : sha(buf);
  }
  for (const d of CONFIG_DIRS) {
    const files = {};
    walk(join(home, d), files);
    for (const rel of Object.keys(files).sort()) h[rel] = sha(readFileSync(join(home, rel)));
  }
  return h;
}

const [cmd, ...args] = process.argv.slice(2);
if (cmd === "save") {
  const files = {};
  for (const r of roots) walk(r, files);
  writeFileSync(args[0], JSON.stringify({ at: new Date().toISOString(), config: configHashes(), files }));
  console.log(`config-snapshot: saved ${Object.keys(files).length} files, ${Object.keys(configHashes()).length} config hashes`);
} else if (cmd === "diff") {
  const [a, b, markersFile, ...rest] = args;
  const markers = readFileSync(markersFile, "utf8").split("\n").filter(Boolean);
  if (!markers.length) throw new Error("diff needs at least one marker");
  const exclude = [];
  for (let i = 0; i < rest.length; i++) if (rest[i] === "--exclude-content") exclude.push(rest[++i]);
  const before = JSON.parse(readFileSync(a, "utf8"));
  const after = JSON.parse(readFileSync(b, "utf8"));
  const bad = [];
  for (const k of new Set([...Object.keys(before.config), ...Object.keys(after.config)]))
    if (before.config[k] !== after.config[k]) bad.push(`config changed: ${k}`);
  const changed = Object.keys(after.files).filter(
    (k) => !before.files[k] || before.files[k].size !== after.files[k].size || before.files[k].mtime !== after.files[k].mtime,
  );
  const removed = Object.keys(before.files).filter((k) => !after.files[k]);
  for (const k of [...changed, ...removed]) {
    if (markers.some((m) => k.includes(m))) bad.push(`path names the spike: ${k}`);
    if (isCredential(k) || !after.files[k] || exclude.some((x) => k.startsWith(x))) continue;
    let buf;
    try {
      buf = readFileSync(join(home, k));
    } catch {
      continue;
    }
    for (const m of markers) if (buf.includes(m)) bad.push(`content names the spike: ${k}`);
  }
  const top = {};
  for (const k of changed) top[k.split("/").slice(0, 2).join("/")] = (top[k.split("/").slice(0, 2).join("/")] || 0) + 1;
  console.log(`config-snapshot: ${before.at} → ${after.at}`);
  console.log(`  config files hashed: ${Object.keys(after.config).length}, changed: ${bad.filter((x) => x.startsWith("config")).length}`);
  console.log(`  other files new or changed: ${changed.length}, removed: ${removed.length} (the user's own live sessions)`);
  for (const [d, n] of Object.entries(top).sort()) console.log(`    ${d}: ${n}`);
  for (const x of exclude) console.log(`  content scan skipped under: ${x}`);
  console.log(`  markers checked: ${markers.length}; hits: ${bad.filter((x) => !x.startsWith("config")).length}`);
  if (bad.length) {
    for (const x of bad) console.log(`  FAIL ${x}`);
    process.exit(1);
  }
  console.log("config-snapshot: ok, no write by the spike");
} else {
  console.error("usage: config-snapshot.mjs save <out> | diff <before> <after> <marker>...");
  process.exit(2);
}
