// SPDX-License-Identifier: AGPL-3.0-or-later
// A command the agent runs is said in words, from what it does, never "Ran a command" (Barry 2026-11-04). Node strips
// the types from summaries.ts, so the table the chat uses is the one tested.
import assert from "node:assert/strict";
import { test } from "node:test";
import { doing, toolCard } from "../src/summaries.ts";

const done = (name, command) => toolCard(name, { command }, { ok: true, summary: "" }).title;

test("a command is said by what it does, with each agent's shell", () => {
  const cases = [
    ["Bash", "ls -la", "Looked through the project"],
    ["Bash", "cd /tmp/project && cat model.py | head -40", "Looked through the project"],
    ["Bash", "emmet run --json", "Built the model"],
    ["Bash", "emmet check", "Checked the model"],
    ["Bash", "/Applications/Emmet.app/Contents/MacOS/emmet search 'slope 45 2 x 2'", "Looked for parts"],
    ["Bash", "python3 model.py", "Tried the model"],
    ["Bash", "uv run python -c 'from emmet_dsl import *'", "Tried the model"],
    ["Bash", "python3 - <<'EOF'\nprint(24 * 0.4)\nEOF", "Worked out the sizes"],
    ["Bash", "pip install numpy", "Installed a tool"],
    ["Bash", "curl -sL https://example.com/x.png -o ref.png", "Downloaded a file"],
    ["Bash", "git log --oneline", "Looked at the versions"],
    ["Bash", "mkdir -p models && cp a.py models/", "Tidied the project's files"],
    ["Bash", "sleep 2", "Waited"],
    ["PowerShell", "Get-ChildItem -Recurse", "Looked through the project"],
    ["shell", "/bin/zsh -lc 'rg -n wall models/'", "Looked through the project"],
    ["shell", "bash -lc \"EMMET_X=1 emmet render --view front\"", "Looked at the model"],
    ["commandExecution", "emmet bom", "Counted the parts"],
    // What matters most wins: checking after a build is still the build.
    ["Bash", "emmet run && emmet check && ls", "Built the model"],
  ];
  for (const [name, command, want] of cases) assert.equal(done(name, command), want, `${name}: ${command}`);
});

test("a command no one can read says it worked on the project, and bench words while it runs", () => {
  assert.equal(done("Bash", "frobnicate --all"), "Worked on the project");
  const bench = ["Sorting parts", "Finding parts", "Choosing colors", "Counting studs", "Lining up the rows", "Snapping parts together"];
  assert.ok(bench.includes(doing("Bash", { command: "frobnicate --all" })));
  // The same command keeps its words, so the status doesn't flicker while it runs.
  assert.equal(doing("Bash", { command: "frobnicate --all" }), doing("Bash", { command: "frobnicate --all" }));
  assert.equal(doing("Bash", { command: "emmet run" }), "Building");
});

test("no command's line shows code, a path, or the word command", () => {
  const titles = ["ls", "emmet run", "python3 x.py", "frobnicate", "git status", "rm -rf build"].flatMap((c) =>
    ["Bash", "shell"].flatMap((n) => [done(n, c), doing(n, { command: c }), toolCard(n, { command: c }, { ok: false, summary: "" }).title]),
  );
  for (const t of titles) {
    assert.doesNotMatch(t, /command|[/\\`]|\.py|emmet /i, t);
  }
});
