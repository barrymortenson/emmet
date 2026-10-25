// SPDX-License-Identifier: AGPL-3.0-or-later
// Shared plumbing for the 0B recorders (ADR 0001): an isolated workspace per run, the eval key from the keychain,
// and a transcript writer that keeps both directions of the protocol with relative timestamps.

import { spawn, execFileSync } from "node:child_process";
import { appendFileSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, writeFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";

export const SPIKE_DIR = dirname(fileURLToPath(import.meta.url));
export const STUB = join(SPIKE_DIR, "stub-mcp.mjs");
export const RAW = process.env.SPIKE_RAW_DIR || join(tmpdir(), "emmet-0b-raw");

/** A fresh workspace: home/ (the CLI's HOME), project/ (cwd), session/ (MCP config), and a file outside the project. */
export function workspace(scenario) {
  // Under /tmp, not $TMPDIR: macOS's per-user temp dir has a machine-specific id that would end up in transcripts.
  const root = realpathSync(mkdtempSync(join(process.platform === "win32" ? tmpdir() : "/tmp", `emmet-spike-${scenario}-`)));
  const ws = { root, marker: root.split("/").pop(), home: join(root, "home"), project: join(root, "project"), session: join(root, "session") };
  for (const d of [ws.home, ws.project, join(ws.project, "models"), ws.session]) mkdirSync(d, { recursive: true });
  writeFileSync(join(ws.project, "model.py"), "# A spike project.\nWIDTH = 8\n");
  writeFileSync(join(ws.project, "CARD.md"), "# Spike\n");
  writeFileSync(join(root, "outside.txt"), "This file is outside the project.\n");
  ws.log = join(ws.session, "stub.log");
  // The marker goes to a file, never to stdout, so the session running the spikes doesn't echo it (config-snapshot).
  mkdirSync(RAW, { recursive: true });
  appendFileSync(join(RAW, "markers.txt"), ws.marker + "\n");
  return ws;
}

/** The environment a user's terminal would give, minus the parent agent's variables, plus `extra`. */
export function cleanEnv(ws, extra) {
  const env = {};
  for (const [k, v] of Object.entries(process.env))
    if (!/^(CLAUDE|CODEX_|ORCA_|ANTHROPIC_|OPENAI_|MCP_)/.test(k)) env[k] = v;
  return { ...env, HOME: ws.home, ...extra };
}

export function keychain(service) {
  return execFileSync("security", ["find-generic-password", "-s", service, "-w"], { encoding: "utf8" }).trim();
}

/**
 * Spawns `cmd` and records every line. Returns helpers to send lines and wait for output that matches a predicate.
 * Transcript lines: {"t": ms since start, "dir": "in" | "out" | "err" | "stub" | "note", "msg": parsed JSON or text}.
 */
export function record({ cli, scenario, cmd, args, ws, env, meta, append }) {
  mkdirSync(join(RAW, cli), { recursive: true });
  const file = join(RAW, cli, `${scenario}.jsonl`);
  if (!append) writeFileSync(file, "");
  const t0 = Date.now();
  const write = (dir, msg) => appendFileSync(file, JSON.stringify({ t: Date.now() - t0, dir, msg }) + "\n");
  write("note", { meta: { scenario, cli, date: new Date().toISOString().slice(0, 10), args, ...meta } });

  const child = spawn(cmd, args, { cwd: ws.project, env, stdio: ["pipe", "pipe", "pipe"] });
  const events = [];
  const waiters = [];
  const listeners = [];
  const check = () => {
    for (const w of [...waiters]) {
      const hit = events.slice(w.from).find(w.pred);
      if (hit) {
        waiters.splice(waiters.indexOf(w), 1);
        clearTimeout(w.timer);
        w.resolve(hit);
      }
    }
  };
  createInterface({ input: child.stdout }).on("line", (line) => {
    let msg = line;
    try {
      msg = JSON.parse(line);
    } catch {
      // not JSON: keep the line as text
    }
    write("out", msg);
    events.push(msg);
    for (const fn of listeners) fn(msg);
    check();
  });
  createInterface({ input: child.stderr }).on("line", (line) => write("err", line));
  const exited = new Promise((resolve) =>
    child.on("exit", (code, signal) => {
      write("note", { exit: { code, signal } });
      resolve({ code, signal });
    }),
  );

  // Copy the stub's call log into the transcript as it grows, so the recording shows what each tool was asked.
  let stubSeen = 0;
  const stubPoll = setInterval(() => {
    if (!existsSync(ws.log)) return;
    const lines = readFileSync(ws.log, "utf8").split("\n").filter(Boolean);
    for (const l of lines.slice(stubSeen)) write("stub", JSON.parse(l));
    stubSeen = lines.length;
  }, 200);

  return {
    file,
    events,
    exited,
    send(obj) {
      const line = typeof obj === "string" ? obj : JSON.stringify(obj);
      write("in", obj);
      child.stdin.write(line + "\n");
    },
    note(msg) {
      write("note", msg);
    },
    /** Calls `fn` with every output message from now on (the Codex recorder answers server requests with it). */
    on(fn) {
      listeners.push(fn);
    },
    /** Resolves with the first event from now on that matches `pred`; rejects after `ms`. */
    waitFor(pred, ms = 120_000, label = "event") {
      return new Promise((resolve, reject) => {
        const w = { pred, resolve, from: events.length };
        w.timer = setTimeout(() => {
          waiters.splice(waiters.indexOf(w), 1);
          reject(new Error(`timed out after ${ms} ms waiting for ${label}`));
        }, ms);
        waiters.push(w);
        check();
      });
    },
    stubLog() {
      return existsSync(ws.log) ? readFileSync(ws.log, "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l)) : [];
    },
    async end(ms = 30_000) {
      child.stdin.end();
      const timer = setTimeout(() => child.kill("SIGTERM"), ms);
      const r = await exited;
      clearTimeout(timer);
      clearInterval(stubPoll);
      for (const l of this.stubLog().slice(stubSeen)) write("stub", l);
      return r;
    },
    kill() {
      child.kill("SIGTERM");
    },
  };
}

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
