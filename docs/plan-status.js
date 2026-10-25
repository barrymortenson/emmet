// SPDX-License-Identifier: AGPL-3.0-or-later
// Execution status for docs/plan.html (plan section 20). Agents edit this on their session branch;
// scripts/plan-check.mjs validates it in CI. Never record progress by editing plan.html.
// Session status: todo | in-progress | review | done | blocked | skipped (Barry only) | backlog (Phase 8).
// done needs pr and at least one evidence line; skipped needs Barry's note; blocked needs an open blocker.
// Human status:   todo | waiting | done | not-needed
// Gate status:    todo | go | no-go | pivot
// A session Barry let start before its "after" sessions were done carries override: "<Barry's note>", which turns
// plan-check's error into a warning.
window.EMMET_STATUS = {
  updated: "2026-10-24",
  sessions: {
    "0A": {
      status: "done", pr: 1, branch: "bmortenson/s0a-foundation", started: "2026-10-24",
      evidence: [
        "just ci on this Mac: ci: ok",
        "CI ci check green #1 (mac on self-hosted emmet-mac, windows, plan)",
        "tests/it/tests/cli.rs, tests/scripts/{plan-check,gates,spdx-dco,ci,runner,justfile}.test.mjs"
      ],
      note: "1A and 1C must use the test HOME and data dir the runner guard sets; CLAUDE.md is a symlink, a text file on Windows without core.symlinks"
    },
    "0B": { status: "review", pr: 3, branch: "bmortenson/s0b-research-adrs", started: "2026-10-24" },
    "1A": { status: "todo" },
    "1B": { status: "todo" },
    "1C": { status: "todo" },
    "1D": { status: "todo" },
    "1E": { status: "todo" },
    "2A": { status: "todo" },
    "2B": { status: "todo" },
    "3A": { status: "todo" },
    "3B": { status: "todo" },
    "3C": { status: "todo" },
    "4A": { status: "todo" },
    "4B": { status: "todo" },
    "4C": { status: "todo" },
    "4D": { status: "todo" },
    "4E": { status: "todo" },
    "5A": { status: "todo" },
    "5B": { status: "todo" },
    "6A": { status: "todo" },
    "6B": { status: "todo" },
    "7A": { status: "todo" },
    "7B": { status: "todo" },
    "8D": { status: "backlog" },
    "8E": { status: "backlog" },
    "8F": { status: "backlog" },
    "8G": { status: "backlog" }
  },
  human: {
    "H-01": { status: "done", date: "2026-10-24", note: "repo created public; settings applied and read back in 0A" },
    "H-02": { status: "done", date: "2026-10-24", note: "runner emmet-mac registered by the agent with approval; PR #1 mac job green on it" },
    "H-03": { status: "done", date: "2026-10-24", note: "Claude Code 2.1.310 on Barry's plan; Codex 0.160.1 logged in using ChatGPT" },
    "H-04": { status: "todo" },
    "H-05": { status: "todo", note: "Apple Developer Program: enrolled" },
    "H-06": { status: "todo" },
    "H-07": { status: "todo" },
    "H-08": { status: "todo" },
    "H-09": { status: "todo" },
    "H-10": { status: "todo" },
    "H-11": { status: "done", date: "2026-10-24", note: "6743-1 Street Speeder, from the 0B shortlist (docs/spec/ladder.md)" },
    "H-12": { status: "todo" },
    "H-13": { status: "todo" },
    "H-14": { status: "not-needed", note: "releases publish from the main public repo (D-21)" },
    "H-15": { status: "todo", note: "emmetbuilder.com registered at Cloudflare 2026-10-24; API token still to do" },
    "H-16": { status: "done", note: "AGPL-3.0-or-later; Apache-2.0 for the DSL (2026-10-24)" },
    "H-17": { status: "done", note: "best-judgment search 2026-10-24; keep Emmet (D-30)" },
    "H-18": { status: "not-needed", note: "public from day one (D-12)" },
    "H-19": { status: "todo" },
    "H-20": { status: "waiting", date: "2026-10-24", note: "Commercial Terms accepted through the Console account; legal page read 2026-10-24. The description to Anthropic waits for the website and a business email, before 7B merges" },
    "H-21": { status: "done", date: "2026-10-24", note: "keychain items emmet-eval-anthropic and emmet-eval-openai exist, with spend caps set" },
    "H-22": { status: "todo" },
    "H-23": { status: "todo" },
    "H-24": { status: "todo" }
  },
  blockers: [],
  gates: {
    "G1": { status: "todo", note: "" },
    "G2": { status: "todo", note: "" }
  }
};
