# Decisions

Barry's answers, newest first, each with its date. Section 03 of the plan holds the decisions of record; ADRs in
`docs/adr/` change them. Sessions add an entry for every question they ask (plan section 19, rule 9).

## 2026-10-24, session 0B

- **H-03.** Both CLIs are signed in on this Mac: Claude Code on Barry's plan, Codex with ChatGPT
  (`codex login status`: "Logged in using ChatGPT").
- **H-21.** The eval keys are in the keychain as `emmet-eval-anthropic` and `emmet-eval-openai`, each with a
  spend cap.
- **H-20: send the description to Anthropic later.** Barry accepted the Commercial Terms through his Console account
  and read the legal page on 2026-10-24. Sending Anthropic the description of how Emmet runs Claude Code waits until
  Emmet has its website and a business email. Until then H-20 is `waiting`; the description must be sent before 7B
  merges. The draft is kept for that step. Accepting the Commercial Terms is what lets Emmet run Claude Code, so 0B
  starts now. Plan sections 10, 21, and 22 and 7B's merge wait are updated to match.
- **D-40: permissions are sandbox first, like nurb.** 0B found Claude Code runs read-only shell commands without
  asking, and Codex on `on-request` asks almost nothing. Barry asked what nurb does: it answers every permission request
  "allow once" and relies on an OS sandbox (macOS Seatbelt: read anything, write only to the project, the app's data,
  temp, caches, and the agents' own state) instead of approval cards. Decision: each CLI's own sandbox guards the
  chat pane: Claude Code through `--settings` with `sandbox.enabled` and `failIfUnavailable`, Codex with
  `workspace-write` and `/tmp` excluded. Emmet approves what runs inside the sandbox; cards appear only for what the
  sandbox can't contain, and for every shell command under Claude Code on native Windows, which has no sandbox (ask
  rules there). Plan section 10 and section 03 change to match; 1C proves it on both OSes. Rejected: an ask rule with
  cards for every shell command; accepting the gap.
- **Section 10's OpenAI quotes.** Two phrases in the Codex column changed wording since planning ("is available to all
  open-source partners" is gone; "without a client secret or partner API key" moved to the sign-in page as "needs
  neither a client secret nor a partner API key"). The basis is the same, so section 10 quotes today's text and the
  1E row stays Build with conditions. Rejected: holding 1E for OpenAI's written confirmation; dropping ChatGPT plan use.
- **Q-01: the LDCad shadow library ships as CC BY-SA data.** It is CC BY-SA 4.0 (ADR 0003). The connector table built
  from it ships as its own data file under CC BY-SA 4.0 with attribution to the LDCad Shadow Library, kept separate
  from the AGPL code; overrides cover what it misses (track, the 9V switch). Rejected: building it on the user's
  machine from a first-run download; not using it.
- **License credits are allowed.** LDraw part and OMR model headers name their authors, and CC BY requires keeping
  them. The no-real-names invariant gets a second exception for third-party authors' names a license requires (LDraw
  parts, OMR models, their attribution files), in AGENTS.md, its copy in plan section 25, and agent rule 7 in section
  19. Rejected: keeping the names without changing the text; committing no named files (no ldraw-mini).
- **A draft PR so CI runs.** Approved `gh pr create --draft` for 0B (PR #3), as in 0A: CI runs only on PRs, and the
  Windows evidence (E2E, render, binaries) needs it. It is marked ready only when every Done when item holds.
- **H-11: rung 3 rebuilds 6743-1 Street Speeder** (165 pieces, 69 part and color lines, 92% common parts), from the
  shortlist of five in `docs/spec/ladder.md`. Its OMR main model matches the Rebrickable inventory exactly.
- **G1 without a demo or posts.** Barry skips the 60-second demo video and the community posts at G1 (Eurobricks,
  r/lego, the LDraw forum), and decides go, no-go, or pivot from the eval reports and his own testing. Plan sections
  02, 03 (D-18's wording), 21 (Phase 1, 1D: no demo kit; the signed build stays), 22 (H-06 and its guide), and 23 (the
  risk row) are updated. The launch posts and video at 7A/7B are unchanged.

## 2026-10-24, session 0A

- **Personal-details pass.** Before the first push: drop the full name from the plan header and the H-05 example,
  remove the portfolio notes, reword the personal nurb reference, rename the runner `emmet-mac`, and trim the H-17
  findings to the outcome. Keep the first name "Barry" as the maintainer role, `~/Sites/emmet`, the toolchain names,
  and the status notes. Delete the three local backup branches that held the old email once `main` is pushed (done).
- **H-01: create the repo.** Approved `gh repo create barrymortenson/emmet --public --source <repo> --remote origin`
  with the description "Emmet Builder: a desktop app where your own Claude Code or Codex agent builds brick models
  from real LDraw parts".
- **D-39: the one push of `main`.** Approved pushing the empty initial commit `7fb7df7` to `main`. The first attempt
  failed on a typo in the SHA and pushed nothing; the corrected command was approved again and ran once.
- **Repo security settings.** Approved private vulnerability reporting, approval for every outside contributor's
  workflow runs (was first-time contributors only), Dependabot alerts, and requiring actions pinned by SHA. Secret
  scanning, push protection, and read-only workflow tokens were already on.
- **Protection for `main`, step 1.** Approved a ruleset on the default branch: PR required with 0 approvals, no
  force-push, no deletion, empty bypass list. The required `ci` check is added after the first green run.
- **H-02: the runner.** Barry asked the agent to register it. Approved, and run once: download runner v2.338.1
  (sha256 checked), register `emmet-mac` with the label `emmet` using a token minted by `gh` (never printed), install
  and start the login service, and fill its `.path` with `scripts/runner-env.sh`. PR #1's mac job passed on it.
- **Protection for `main`, step 2.** After the first green run, approved adding the required `ci` check (GitHub
  Actions) to the ruleset, not strict (a branch needn't be up to date with `main` to merge).
- **The first CI run.** `main` is empty, so CI can only run on a PR. Open PR #1 as a draft so CI runs, and mark it
  ready for review only when every Done when item holds.

## 2026-10-24, planning session

The answers recorded in plan section 03 as "Barry 2026-10-24" (the full text is there):

- **D-03 Frontend.** React 19 + TypeScript strict + Vite.
- **D-06 Agent bridge.** Both agents in the spike: Claude Code (claude -p, stream-JSON over stdio) and Codex (codex app-server, JSON-RPC over stdio), behind one AgentAdapter trait.
- **D-10 Status tracker.** A committed docs/plan-status.js, hand-edited by agents on their session branch and checked by scripts/plan-check.mjs in CI.
- **D-11 Test cases.** A ladder of three models: Cottage → Station + track loop → OMR set rebuild, all official parts.
- **D-12 Visibility.** The repo is public from day one (0A), under AGPL-3.0 (D-14), once the name check is done (H-17).
- **D-13 CI.** A self-hosted macOS runner on Barry's Mac (warm caches, $0, with runner guards) plus hosted windows-latest.
- **D-14 License.** AGPL-3.0-or-later for the app, engine, CLI, MCP servers, and viewer: anyone may use, change, and share Emmet, but whoever distributes it or runs it as a service must publish their full source under the same license and keep the copyright notices.
- **D-15 Agent tests.** Every PR replays recorded agent transcripts through fake claude/codex binaries: deterministic, no tokens.
- **D-16 Design.** The agent builds clickable HTML mockups of three directions.
- **D-17 Website.** A static Astro site in site/ in this repo, deployed to Cloudflare Workers static assets with PR previews.
- **D-18 macOS signing.** Barry is enrolled in the Apple Developer Program.
- **D-19 Windows checks.** Barry's hands-on Windows checks run in a Windows 11 ARM VM on this Mac. x64 behavior is proven by hosted CI and by beta testers.
- **D-21 Distribution.** Release binaries and the updater manifest are GitHub Releases on the public repo, and the parts data pack is its own release series there.
- **D-25 Rebrickable data.** Part, color, element, and availability data come from Rebrickable's catalog downloads ("Free for personal and commercial use, provided you acknowledge Rebrickable as your data source").
- **D-30 Name and branding.** Keep the name, decided on a best-judgment search (H-17, 2026-10-24; no attorney).
- **D-32 Question cards.** The agent asks the user through an ask_user tool on emmet_host: one question, 2–4 options with optional descriptions, single or multi choice, and always an Other free-text answer.
- **D-33 Redirect and stop.** The composer stays open while the agent works.
- **D-34 Build animation.** Agent-driven runs animate in the viewer: added parts drop in brick by brick in build order (step, then bottom-up), capped in total time and skippable.
- **D-35 Plans and keys.** Claude: the chat pane runs the user's own unmodified Claude Code with whatever sign-in the user gave it in their own terminal (a Pro or Max plan, or an API key).
- **D-36 Plan usage display.** Claude: status bar meters from Claude Code's documented /usage report and rate_limit_event, fetched only when the user opens the usage popover, presses refresh, or finishes a turn.
- **D-37 Terms gate.** A feature that touches an AI provider (accounts, sign-in, credentials, billing, usage data, model requests, or names and logos) is built only if the provider's current published text permits that exact use.
- **D-38 Scripted-run budget.** Scripted runs (anything that drives a real agent CLI automatically, billed to Barry's API keys) are kept to a minimum.
- **D-39 Repo bootstrap and protection.** main gets exactly one direct push, ever: in 0A, with Barry's approval of the exact commands, the empty initial commit is pushed to create the remote, before the hooks and protection exist.
