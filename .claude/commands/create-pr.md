---
description: Open the PR for the current branch once the work is ready, set the session to review, then run the PR review toolkit on it
argument-hint: "[anything Barry wants said in the PR]"
---

Open one PR from the current branch into main (plan section 19, rules 1 and 12), then review it with
/pr-review-toolkit:review-pr. $ARGUMENTS

## When this may run

- **Barry ran `/create-pr`:** that authorizes opening this one PR now, ready or not. Items that don't hold yet stay
  unticked in the description with the reason.
- **You decided to run it:** only inside a session run whose prompt has you open the PR (rule 12), and only when the
  work is ready: `just ci` passes and every Done when item holds. If an item can't be finished, ask Barry with
  AskUserQuestion before opening, and leave it unticked with the reason. Otherwise keep working and don't open the PR.
- Never on main, never a second PR for the same branch, and never a merge. Merging is `/merge`, run by Barry.
  `gh pr view --json number,state,isDraft` finds the branch's PR: an `OPEN` one that's a draft gets this command's
  steps 1-4 with `gh pr edit` in place of `gh pr create`, then `gh pr ready <n>`; any other open PR, report it and
  stop. A closed or merged one doesn't count.

## 1. Check the branch

- `git fetch origin`. The current branch isn't main, the working tree is clean (commit or ask about stray changes),
  and the branch is pushed with every commit signed off (`scripts/dco-check.sh origin/main HEAD`).
- The session is the `docs/plan-status.js` entry whose `branch` is this branch, else the id in a `bmortenson/s<id>-…`
  branch name, uppercased (`s0a` → `0A`). No session (tooling, docs, a reconcile): use the template's sections that apply and skip the session
  steps below.
- `node scripts/plan-check.mjs` and `just ci` pass. Keep the `ci:` result line for the description.

## 2. Write the description

Follow `.github/PULL_REQUEST_TEMPLATE.md`. For a session:

- **Title:** `<id>: <outcome>`, 50 characters or fewer, plain words, no version numbers or paths ("0B: Research
  spikes and ADRs"). Other PRs: `<area>: <change>`.
- **Done when:** the session's list from `docs/plan.html` (`<div class="session" … data-session="<id>"`), word for word,
  each item ticked with its evidence (test name, command plus result line, screenshot, file) after checking it holds
  on this branch. Not ticked without evidence.
- **What good looks like:** how the work meets each item, with numbers where the plan asks for them.
- **Checks:** the template's list, with the `just ci` result line. UI work needs light and dark screenshots on macOS
  and Windows.
- Then, where they apply: a CSP or capabilities section (any loosening said plainly), things to know (decisions made,
  deviations from the plan, anything Barry should look at), and the YOU steps this PR needs from Barry.
- US English. No secrets, real names, emails, or personal paths.

Write it to a temp file and run `gh pr create --base main --head <branch> --title <title> --body-file <file>`.

## 3. Set the session to review

On the branch: the session gets `status: "review"` and `pr: <number>` (keep `branch` and `started`), and `updated`
is today. Run `node scripts/plan-check.mjs`, commit with `git commit -s -m "Set <id> to review with PR #<n>"`, and
push. If the description says the status shows review, that's now true.

## 4. Review it

Run the `pr-review-toolkit:review-pr` skill with the Skill tool, args `all`. Tell it the PR number and that the
changes are `git diff origin/main...HEAD` (the working tree is clean, so plain `git diff` shows nothing). If the
skill isn't installed, say so and skip this step.

Then act on what it finds:

- Check each finding against the code before acting on it.
- Fix the critical and important ones that hold up on this branch, each bug with a test that would have caught it
  (rule 5). Never weaken a test, lint, or gate to settle a finding.
- A fix that would change the plan, an ADR, a section 03 decision, or the frozen design: ask Barry (rule 10).
- Run `just ci`, commit with sign-off, push, and update the description where the fixes change its evidence. Add a
  short "Review" section: what was fixed, and what was left and why.

## 5. Stop

Report: the PR link, the Done when items ticked and not, what the review fixed and left, the YOU steps the PR waits
on, and that `/merge` is Barry's next step. Don't start the next session.
