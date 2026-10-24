// Execution status for docs/plan.html (plan section 20). Agents edit this on their session branch;
// scripts/plan-check.mjs validates it in CI. Never record progress by editing plan.html.
// Session status: todo | in-progress | review | done | blocked | skipped (Barry only) | backlog (Phase 8)
// Human status:   todo | waiting | done | not-needed
// Gate status:    todo | go | no-go | pivot
window.EMMET_STATUS = {
  updated: "2026-10-24",
  sessions: {
    "0A": { status: "todo" },
    "0B": { status: "todo" },
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
    "H-01": { status: "todo" },
    "H-02": { status: "todo" },
    "H-03": { status: "todo", note: "Claude Code 2.1.310 and Codex 0.160.1 are installed; confirm sign-in" },
    "H-04": { status: "todo" },
    "H-05": { status: "todo", note: "Apple Developer Program: enrolled" },
    "H-06": { status: "todo" },
    "H-07": { status: "todo" },
    "H-08": { status: "todo" },
    "H-09": { status: "todo" },
    "H-10": { status: "todo" },
    "H-11": { status: "todo" },
    "H-12": { status: "todo" },
    "H-13": { status: "todo" },
    "H-14": { status: "not-needed", note: "releases publish from the main public repo (D-21)" },
    "H-15": { status: "todo" },
    "H-16": { status: "done", note: "AGPL-3.0-or-later; Apache-2.0 for the DSL (2026-10-24)" },
    "H-17": { status: "todo" },
    "H-18": { status: "not-needed", note: "public from day one (D-12)" },
    "H-19": { status: "todo" },
    "H-20": { status: "todo" },
    "H-21": { status: "todo" },
    "H-22": { status: "todo" },
    "H-23": { status: "todo" }
  },
  blockers: [],
  gates: {
    "G1": { status: "todo", note: "" },
    "G2": { status: "todo", note: "" }
  }
};
