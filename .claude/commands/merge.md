---
description: Evaluate the plan, set the session to done on the PR branch, then merge the PR once the required checks pass
argument-hint: "[PR number; default: the current branch's PR]"
disable-model-invocation: true
---

Merge PR $ARGUMENTS (no number: the open PR for the current branch) into main the way plan section 19 says: the
status file's `done` lands as the PR's last commit, then the merge. Barry running this command authorizes merging this
one PR after its required checks pass. It never covers a release, a tag, a gate, a YOU step, making anything public,
`--admin`, or any other PR.

Stop and report (don't merge) at the first step that fails. Ask with AskUserQuestion only where a step says so.

## 1. Find the PR and the session

- `gh pr view <n> --json number,state,isDraft,title,headRefName,headRefOid,baseRefName,mergeable,mergeStateStatus,body`.
  It must be open, not a draft, based on `main`, and `MERGEABLE`.
- `git fetch origin`. The working tree is clean, the current branch is the PR's head branch (switch to it if not), and
  local `HEAD` equals `headRefOid` (pull it if behind; stop if it has commits the PR doesn't).
- The session is the `docs/plan-status.js` entry whose `pr` is this number or whose `branch` is the head branch, else
  the id in a `bmortenson/s<id>-…` branch name. No session (a reconcile, tooling, or Dependabot PR): skip to step 4
  and change the status file only if this PR's work makes it wrong.

## 2. Evaluate the plan for this session (read only)

Read the session's block in `docs/plan.html` (`<div class="session" id="session-<id>"`: Done when, What good looks
like, `data-after`, `data-waits-merge`) and the phase box it sits in. Check, and list each result:

1. Every session in `data-after` is `done` or `skipped` (or this session has Barry's `override`).
2. Every YOU step in `data-waits-merge` is `done` or `not-needed` under `human`. If not, the YOU step blocks the
   merge: stop and say which step and what Barry has to do (plan section 22 has the guide).
3. Every Done when item is ticked in the PR description with evidence, and the evidence holds on this branch: the
   named files and tests exist, and cheap commands it quotes (`node scripts/plan-check.mjs`, a single test) still
   give that result. Spot-check What good looks like the same way. An unticked or broken item: stop and ask.
4. No open blocker in `blockers` names this session.
5. If every other session in this phase is already `done` or `skipped`, this is the phase's last PR: its description
   needs the phase exit check (plan section 19, "Running a whole phase") against the phase's Delivers line. If it's
   missing, write it with evidence for each clause and add it to the PR description with `gh pr edit`.

## 3. Update docs/plan-status.js on the branch

- The session: `status: "done"`, keeping `pr`, `branch`, and `started`; add `evidence` (2-4 short lines taken from the
  PR, such as the `just ci` result line, "CI ci check green #<n>", and the key test files) and a `note` only if
  something is worth knowing later. No `sha`: the merge SHA doesn't exist yet. A reconcile run can add it.
- YOU steps this PR finished or verified, as its description shows: `done` with today's date and a short note.
- Blockers this PR resolved: set `closed` to today's date.
- `updated`: today's date.
- Never set a gate (G1, G2) unless Barry gave his call in this conversation; then record his words in its note.
  Never change another session's status. Never record progress in `docs/plan.html` (rule 13).
- Write US English, and flag British spellings you see in the file rather than copying them.

Then run `node scripts/plan-check.mjs` and `just ci`; both must pass. Show the status diff, commit it with
`git commit -s -m "Set <id> to done before the merge"`, and push. If nothing in the status file changed, don't make
an empty commit.

## 4. Wait for the required checks

`gh pr checks <n> --watch --fail-fast` on the pushed head. Right after a push it can say "no checks reported" before
the run registers; retry until the jobs appear. Then `gh pr checks <n> --required` must show the `ci` check passing.
A failure stops here: report the failing job and its log link, and don't merge. Never bypass a required check or
branch protection.

## 5. Merge

`gh pr merge <n> --merge --match-head-commit <the pushed head SHA>`. A merge commit keeps every signed-off commit as it
is (the DCO check skips merge commits); a squash would put one unsigned commit on main. Don't pass `--delete-branch`:
PR descriptions link screenshots on the branch. If the merge is refused (a new commit, a ruleset), stop and report.

## 6. After the merge

- `git switch main && git pull --ff-only` (in a worktree whose main is checked out elsewhere, `git fetch origin` only),
  and confirm `origin/main` contains the merge commit and `node scripts/plan-check.mjs` passes there.
- Report: the merge SHA; each status change; the plan's progress (sessions done of the total, excluding Phase 8);
  which sessions can start now (their "after" sessions done, their before-start YOU steps done); and which YOU steps
  or gates the next sessions wait on. If every session outside Phase 8 is done, say the plan is complete.
- Don't start the next session.
