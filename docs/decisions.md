# Decisions

Barry's answers, newest first, each with its date. Section 03 of the plan holds the decisions of record; ADRs in
`docs/adr/` change them. Sessions add an entry for every question they ask (plan section 19, rule 9).

## 2026-10-28, session 4A

- **Open… is one file picker.** keyboard.md says ⌘O opens "a project or an LDraw file", but the system's open dialogs
  (tauri-plugin-dialog) pick folders or files, never both in one panel. Asked through AskUserQuestion, Barry chose
  one file picker: Open… takes an LDraw `.ldr` or `.mpd` (which becomes an imported project) or a project's
  `emmet.toml` (which opens its folder). No design change. Over a second menu item, "Open LDraw File…" (a design
  change note), and over a new AppKit binding crate for a panel that takes either on macOS only.

- **Install command line tool asks for an admin password on macOS.** Neither the plan nor the frozen design said how
  Settings puts the bundled `emmet` on PATH (D-08). Asked through AskUserQuestion, Barry chose VS Code's way: on
  macOS a link at `/usr/local/bin/emmet` to the copy inside the app, made after macOS's own administrator prompt; on
  Windows the app's folder on the user's PATH, with no admin. No shell file is edited, and it works in every new
  terminal. Over a link in `~/.local/bin` with a line added to `~/.zprofile`, the same link with the line left for
  the user to add, or leaving it a stub.

- **The model names its knobs.** The first build of the Tweaks section showed every parameter the cottage has
  (width, depth, wall rows, colors, the baseplate), where the design session had ruled out a panel of raw
  parameters: Tweaks are a few character knobs, and the baseplate is the Base build setting. Barry pointed it out,
  and asked through AskUserQuestion, chose that the model marks its knobs: `@model(knobs=[...])` in emmet_dsl, which
  `get_params` reports per parameter and the panel shows alone, with a model that names none showing "This model has
  no knobs yet." The other parameters stay for the CLI, rung 1, and the tests. The doctrine (EMMET.md) tells the
  agent to name two to five. Over cutting the templates' parameters down to the knobs, and over the panel guessing
  which parameters are knobs from their names or types. No design change: the frozen Tweaks already draw only knobs.

## 2026-10-27, session 2B

- **The Steps tab is the model's tree.** B has no project tree, and section 14 and 4A still listed one. Asked where
  the model's structure lives, Barry chose the Steps tab: submodels show there as sub-assembly groups with how many
  times each is used and their own sub-steps; clicking a step shows the build up to it, and clicking a submodel
  highlights every copy. Over a fifth tab, Model (tight in the 384 px panel, and it repeats Steps), or no structure
  view in the MVP. Section 14 and 4A drop the separate tree.
- **No command palette.** Asked whether B gets C's ⌘K palette (commands only), Barry said no: the native menus on
  macOS, the title-bar menu on Windows, and the shortcuts are enough. The keyboard map has no ⌘K, and 4A doesn't
  build a palette.
- **H-08 (UX-2): signed as is.** Asked through AskUserQuestion whether he signs the frozen design, with a summary of
  all 47 screens and states and what 2B settled beyond 2A's B (15 colors a theme; Enter sends, ⇧↩ adds a line, ⌘↩
  still sends, Esc closes what's open before it stops the agent; Windows' full-width title row with its menu bar;
  1E's required ChatGPT UI on the frozen screens; plan usage on the plan badge, so D-36 and section 10 say so; no
  Chat focus layout), Barry signed it as is. From now on, a change to the frozen design needs a design change note
  (`docs/design/changes/`) and his yes.

## 2026-10-27, session 2A

- **H-07 (UX-1): B, as revised.** Asked through AskUserQuestion whether the refined B was done, Barry said yes
  ("B is done, record it"). B (Canvas), with the revisions recorded below, is the direction 2B turns into the design
  system and the frozen screens. A and C stay in `docs/design/directions/` as the record; the mix that review.html
  proposed before the revisions (A's Chat focus layout, C's ⌘K palette) wasn't part of the pick.
- **No step scrubber.** Barry, on B's "◀ Step 31/31 ▶" pill: it isn't needed. The Steps tab picks a step, and the
  step's number and parts box show over the viewer. Plan sections 13 and 14 and 4C drop the scrubber, and B's title
  row loses it. A and C keep theirs as the record of the directions.
- **No code in the app.** Barry, on the Checks card's excerpt and "Show in model.py": the user can't fix anything in
  Python, so the app never shows code or offers to open a code file. A finding is a plain sentence with its parts
  highlighted; a failed build says so in plain words; file-change cards in the chat say what changed, with no diff;
  and the untrusted banner drops "Read model.py". The source line, the error, and the hint still go to the agent with
  "Ask agent to fix". Law 2, section 14, and 4B and 4C say so now (4B takes the code out of 1D's Checks drawer and run
  panel), and the mock follows.
- **Steps read like an official instruction book.** Barry, from the books he builds from: a build goes bottom up a
  layer or a section (a wall) at a time, with a side open to put the inside in; steps are small groups of parts;
  a unit built apart is a sub-assembly in its own box with numbered sub-steps; each step shows its parts in a parts
  box with counts; and adding a feature redoes the step order. Most important for the PDF and on-screen
  instructions. Plan section 08 has the step rules, section 11 a `step_size` warning, and 3B, 4C, and 4E build them.
  The cottage template follows them now (31 steps, the same 163 parts), so the tests and the mock match it. The
  cottage's corners interlock row by row, so its four walls rise together and the room is furnished at the second
  row, open from above, rather than through an open side.
- **B is the starting design (H-07 in progress).** After reviewing the three directions, Barry took B (Canvas) as the
  starting point and asked to refine it until it is a fairly complete prototype: the viewer on the left majority of
  the window, the options panel on the right, and the chat along the bottom. H-07 is recorded when the refined B is
  done.
- **Rethink the parameters.** Barry: width, height, and even color aren't the best things to configure. Better: the
  target size (fewest and most pieces), the detail level (low, medium, high), the scale (figure scale, micro, mini
  town, a 1:8 car, and so on), and the build style (blocky, town, medieval, dinosaur park, and so on), the things that
  set a build's size, detail, complexity, and style. The prototype shows them as build settings that steer the agent,
  with the model's own tweaks under them; how they work is the next question. The style and scale names in the app are
  plain words, not brand or theme names (D-30).

- **Build settings plus character knobs.** Asked how build settings should work beside the model's sliders, Barry
  leaned to settings only, since width and length aren't worth a slider and a look like "a beige wall with random
  exposed brick" is a prompt. Shown that such a look makes a good knob once the agent writes it into model.py
  (weathering 0-30 %, Shuffle for another random arrangement), he chose character knobs: the model keeps instant
  sliders, but only two to five the agent picks because they change the model's character (weathering, features on
  or off, counts like floors or windows, Shuffle), never raw dimensions; colors and looks go through the prompt. Over
  settings only (dropping sliders, which would rewrite law 7, the sliders flow, the ladder, and the eval's Parameters
  score) and deciding after 3B. Build settings (style, scale, detail, target size, baseplate) steer the agent, and
  changing one is an agent turn. The plan's sections on sliders change with this PR.
- **Web requests are for the user's links.** The approval-card mock had the agent open a parts page to check colors;
  Barry asked why anything would need the web. Part, color, and availability questions go through Emmet's tools
  (3B's parts data), never the web, and the doctrine says so (plan section 10). The mock now shows the real case:
  the user pastes a link and the agent asks to open it.
- **The ant is the mark, and B wears it.** Barry chose to brand B with the ant (`brand/mark-ant.svg`, three studs
  for a body) over the brick: the project chip carries it and opens About, tips use it, first run shows the
  "Emmet Builder" wordmark, the accent is the brand orange from `brand/palette.css`, and About shows the macOS icon
  draft, what Emmet connects to, and the trademark line (D-30). The final typeface and the 16 px tuning are 2B's.
- **A better starter cottage.** Barry asked for a more interesting example with a real door, window glass, and some
  interior. The template (`crates/emmet-core/templates/cottage/model.py`) is now 10 x 8 studs and 163 parts: a
  stone base course, a Reddish Brown door in its frame, Trans-Clear glass in five windows (two in front when it's 10
  wide), a stone path with two flowers, a chimney over the stove, and a room inside (a bed, a table and stool, a
  stove), all in colors Rebrickable lists for those parts. It gains the knobs from the design (`roof`, `weathering`,
  `seed`), keeps width, depth, wall height, and colors for rung 1 and the tests, and validates clean at all 110
  width and depth combinations. Building it found an older bug: an odd width left a stud of ceiling unplated, so the
  roof above it floated; a Plate 1 x 2 now fills it, and the check test covers width 9. The Windows visual baseline
  for the cottage is stale until the Windows pass regenerates it (D-43).
- **Names, not subtext.** Barry: too much explanatory text under every option; naming should be intuitive. Options and
  knobs show a short name and their control; any help is a hover tooltip. A new build starts with every build setting
  on Any, so they don't box in the prompt, and style takes the user's own words (steampunk, zen garden) beside the
  suggestions. An on/off switch for build settings came and went the same day: with everything on Any, the prompt
  already decides, so the switch said nothing new. The settings run Base, Scale, Detail, Size, Style, then Age, which
  takes LEGO's age bands (4+, 6+, 9+, 13+, 18+); 1.5+ is DUPLO, left out because the checks can't stack DUPLO parts
  yet (their studs have no connectors, so a stacked DUPLO brick reads as floating). DUPLO and 1.5+ are backlog session
  8H. Scale uses the hobby's words, small to large: Micro, Micropolis (the community micro city standard, built on
  16 x 16 modules), Midi, Figure scale (what the hobby calls minifig scale; LEGO lists the Minifigure among its marks,
  so D-30 keeps the plain name), Large (Miniland is LEGOLAND's name, and no official sets come at its 1:20, so Large
  names no ratio; the large display cars, like the Ford Mustang at about 1:13, fit it), and Car 1:8. "Mini town" and
  "Display" weren't terms anyone uses. Each setting decides one thing, and together they set how the agent designs the
  model: Base what it stands on, Scale what the details are sized for (the 9,090-piece Titanic is Micro at 1:200),
  Detail the techniques, Pieces how many (shown as counts, 50–150, 150–500, 500–1,500, so it can't be read as how big
  it looks), Style the look, and Age who builds it. Scale isn't a strict ratio for the whole: like official sets, a
  figure-scale starship has figure-sized doors and seats but is far shorter than true scale, so the agent sizes what a
  figure touches for the figure and compresses the rest (the doctrine's "Sized for the figure, not to the ratio"). Age
  leads Detail and Pieces: picking an age sets them to what sets for that age usually have (4+ and 6+ Low and 50–250,
  9+ Medium and 250–1,000, 13+ High and 1,000–2,500, 18+ High and 2,500+), with no badge, and the user can change
  either. Pieces has no cap: its top band is 2,500+, since 18+ sets run 2,000 to 5,000 pieces, and the doctrine no
  longer asks before a build over 2,000 parts when the user asked for one that big. Emmet's side holds up there (a
  5,000-part model runs in 0.25 s and checks in 0.6 s on a debug build, and `validate` returns at most 50 findings);
  the agent's time and tokens grow with hand-placed detail, not the count.
- **Submodels and versions are MVP.** Barry: builds are made of submodels used many times, and a user needs to save a
  version and keep iterating without fear. Submodels: `m.submodel` defines one and `m.place` puts it in as often as
  needed, one `0 FILE` per submodel referenced by each placement, with the BOM, steps ("× 4"), the project tree, and
  the PDF counting every copy (sections 08 and 14; sessions 3B, 4A, 4E). Versions: a copy of the sources, the build,
  and a thumbnail in `history/`, saved by the user (⌘S) or automatically before every agent turn; restore saves the
  current state first, and any version can open as a new project (sections 07, 09, 14; sessions 4A and 4B).
- **Big builds after launch (8I).** Barry: add a 5,000+ part budget and a build that pushes the limits. Section 13
  gains a 5,000-part column (a target from 8I, not an MVP gate), the check gains 3.5 s for 5,000 parts, and the ladder
  gains rung 4, a landmark of 5,000+ parts that Barry picks: a large castle on landscaped terrain, an airport
  terminal, or a ballpark such as Fenway Park or Dodger Stadium. It measures limits and cost; it gates nothing.
- **Say each thing once.** Barry asked for a pass over every screen for duplicated or redundant UI. In B: one Stop
  (in the status, with Esc); the status shows only when it adds something (not "Idle", not when a card below says it);
  the viewer carries only the finding's pin (a failed build and a read-only project are told by the chat card and the
  banner); the panel head is the model's state in a word plus Export (counts are on the tabs); usage sits on the plan
  badge; New, Open, Export, and About live in the project menu; the build animation is one Skip button; and labels
  that repeat the card's own state ("Waiting for your answer" twice) are gone.

## 2026-10-26, session 1D

- **Up next shows one session.** Barry: the rail's Up next should name one session, not every unblocked one; fixed in
  1D. It names the first in plan order whose "after" sessions are done (sections 19 and 20).
- **A draft PR before the live evals and H-05.** With validate, the drawers, the eval harness, and the release dry
  run built and tested, Barry chose a draft PR now, so CI runs on macOS and Windows while he runs `just eval` and does
  H-05, over waiting until every Done when item holds or running CI by hand without a PR. The PR lists Done when 3
  and 4 as pending him, and turns ready when they hold.
- **The LDraw pin moves to 2026-10, and 6A mirrors each pinned library.** LDraw replaced `complete.zip` with 2026-10
  today and serves no older complete archive, so fresh installs failed the sha256 check (found in H-05's
  fresh-account check). Barry chose to bump the pin now and have 6A publish each pinned `complete.zip`, unmodified with
  its attribution, as a GitHub Release asset that the library manager falls back to (plan section 12; CC BY allows it),
  over accepting whatever release LDraw serves (installs would differ, and integrity would rest on HTTPS alone) or
  bumping with no lasting fix (fresh installs break each month).
- **G1: go.** After 1D merged, Barry called G1 a go ("mark g1 as a go."), on rung 1 passing on both agents, each after
  one rerun (`docs/evals/2026-10-26-rung1.md`). H-06's `scripts/terms-check` and `scripts/name-check` weren't run for
  this call. 3A and 3B can start.

## 2026-10-25, design notes from the 1C live check

- **Homes for the two features with no session.** Barry's notes are in `docs/design/notes.md`, required reading for
  2A. Of the two features with no session, Barry put the default Emmet folder for new projects (changeable in
  Settings) in 4A, and the baseplate on/off parameter for every template in 3B, over doing both in 1F or leaving them
  as notes. 4B's agent switching now follows nurb (a fresh conversation per switch, plus Start fresh), and section 13
  and 4C hide later steps when a step is chosen.

## 2026-10-25, setup for non-technical users and Google's agent

- **D-41: agent setup without a terminal.** Barry wants Emmet to be for people who aren't technical and want to use
  their AI plans, even small ones, so the barrier has to be low. Today a user installs Claude Code or Codex from a
  terminal and signs in to Claude there; nurb and Orca expect the same. Barry chose to add all of it to the plan: a
  new session, 1F (after 1C, beside 1D, 1E, and 2A), installs either CLI with the provider's own official installer
  after one click and runs each CLI's own sign-in with no terminal, with plain guidance on which plans work. 4A
  restyles it, and 4B adds a model and effort picker so small plans last longer.
- **D-37 amended: best reading, not written confirmation.** Barry: providers are unlikely to answer questions about
  their terms, so Emmet takes its best reading of the published text and tries it, instead of waiting for written
  confirmation. Users see a notice that Emmet doesn't control what Anthropic, OpenAI, or Google do: it tries to follow
  their policies, but they can change them or act on accounts, so use is at the user's own risk. The reasoning for
  each such row goes in `docs/terms/`, and a feature is still switched off if its provider objects.
- **H-20: no description to Anthropic.** For the same reason, Barry dropped sending Anthropic the description of how
  Emmet runs Claude Code. H-20 is done (the Commercial Terms were accepted on 2026-10-24), 7B no longer waits on it,
  and the draft (`docs/terms/anthropic-description.md`) is removed.
- **D-42: Google's agent.** Gemini CLI was replaced by Antigravity CLI for free and Google One users on 2026-06-18.
  Google bans third-party software that harvests or piggybacks on its CLI's sign-in; Emmet never touches it. Barry
  noted that tools like nurb and Orca already use these CLIs this way. He chose terminal mode in 1F (checked by hand
  in H-25) and the chat pane in a new session, 4F (after 1F, before 4B), running the user's unmodified `agy` in
  Google's documented headless mode on his best reading, over waiting for Google's written answer (no YOU step to ask
  Google) or keeping it in the backlog.

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
- **No note to Rebrickable (H-24 not needed).** Its downloads terms already allow the pack with credit (Q-02), and Q-14 is
  settled by reading: the Terms' clause is about training models on Rebrickable content, Emmet trains nothing, and its
  tools only answer the user's own agent with small facts. 3B no longer waits on a reply; if Rebrickable ever objects,
  that use is switched off. The draft note is deleted.
- **PR titles.** `<session id>: <outcome>` for a session, `<area>: <change>` otherwise, 50 characters or fewer, plain
  words, no version numbers or paths. In AGENTS.md (and its copy in plan section 25), agent rule 1, `/create-pr`, and
  the PR template; Dependabot's titles start `deps:`, and keep their version text because majors keep their own PR
  (section 06). Rejected: grouping Dependabot majors too; Conventional Commits.

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

## 2026-10-24, session 1A

- **New dependencies for 1A.** Barry approved `ureq` (a small blocking HTTPS client, for the resumable library
  download) and `toml` (for `emmet.toml` and `data/synonyms.toml`; already in `Cargo.lock` through tauri-build), over
  spawning the system curl or using reqwest. ADR 0007 records it. MCP output checks use a small validator in the tests,
  not a JSON Schema crate.
- **The demo render for 1A's Done when #1.** LDView 4.7 and LeoCAD 26.09 are only ad-hoc signed and not notarized, so
  Gatekeeper blocks them on this Mac (Homebrew disabled the LeoCAD cask on 2026-09-01 for that reason). Barry chose
  Emmet's own render instead: the demo .mpd rendered with three.js LDrawLoader through the 0B headless-render spike
  (ADR 0005) against the real 2026-09 library, in place of an LDView or LeoCAD screenshot.
- **1A's Windows CI run.** ci.yml runs only on pull requests, so Barry approved opening 1A's PR as a draft for CI and
  marking it ready (with 1A set to review) once every Done when item holds.

## 2026-10-25, session 1C

- **The keychain store for the Codex API key.** Plan section 06 names no keychain crate, the workspace forbids unsafe
  code, and Windows has no built-in command that reads a stored secret. Barry chose `keyring-core` with only the
  apple-native and windows-native stores (MIT/Apache-2.0; on macOS it uses security-framework, already in the tree)
  over shelling out to `/usr/bin/security` on macOS only, or Emmet's own FFI with an unsafe exception. ADR 0009 records
  it.
- **The Windows bus pipe's ACL.** Section 15 wants the pipe user-only. interprocess takes a security descriptor as
  SDDL in a `widestring::U16CStr`, and doesn't re-export the type. Barry chose widestring as a direct Windows-only
  dependency of emmet-bus (already in the tree through interprocess) over keeping Windows' default pipe ACL (owner,
  SYSTEM, and administrators may write; others may read) or FFI with an unsafe exception. ADR 0009 records it.
- **Barry's design and feature notes.** From the live check, Barry has notes on layout (where the agent's status shows,
  nurb-style agent switching with Start fresh, a model and effort picker, how useful the sliders are, how steps show)
  and features with no session yet (a default Emmet folder for new projects, changeable in Settings; a baseplate
  toggle). He chose a notes file, `docs/design/notes.md`, added in a small PR after 1C and made required reading for 2A,
  with a proposed session for each unhomed feature for him to approve, over folding them into 1C's PR or pasting them
  into 2A's prompt. 1C itself pins the "agent is working" line above the composer with a spinner.
- **1C's Windows CI run.** As for 1A and 1B, Barry approved opening 1C's PR (#13) as a draft so CI runs on Windows,
  marked ready only when every Done when item holds, including his live checks and H-04.

## 2026-10-25, session 1B

- **@types/three for the viewer.** three.js ships no TypeScript types, so Barry approved `@types/three` 0.186.0 (MIT,
  dev-only) over hand-written declarations or a plain-JS viewer. ADR 0000 lists it.
- **The leak test on macOS.** WKWebView has no JS heap API (`performance.memory` is Chromium-only), so Barry chose:
  Windows reads the JS heap (precise, after a forced GC); macOS reads the app's WebContent process footprint with the
  system's `footprint` tool, plus the viewer's resource counts. Both hold to section 13's ≤ 20 % growth.
- **0BSD in the license allowlist.** interprocess (emmet-bus, plan section 06) depends on doctest-file and recvmsg,
  licensed only 0BSD. Barry allowed 0BSD (OSI-approved, public-domain-like, AGPL-compatible) in deny.toml over
  per-crate exceptions or writing the socket and pipe by hand. ADR 0003 lists it.
- **The paste advisory.** specta and tauri-specta (section 06) depend on paste, which RUSTSEC-2024-0436 marks
  unmaintained (no vulnerability, no safe upgrade). Barry chose to ignore that one advisory in deny.toml, with a
  reason, over dropping tauri-specta or relaxing unmaintained checks for every transitive crate.
- **1B's Windows CI run.** As for 1A, Barry approved opening 1B's PR as a draft so CI runs on Windows, and marking it
  ready (with 1B set to review) once every Done when item holds.
- **The macOS leak baseline.** The WebContent footprint at first load varies 125–400 MB between runs, and the first
  pass through the widths adds about 25 MB of one-time allocations (part geometry, shaders, JIT), which put a low
  baseline at 20.1 %. Barry chose to take the macOS baseline after one warm-up cycle through every width (11 changes),
  then hold 100 more changes to ≤ 20 % with the steady resource counts; Windows keeps the first-load JS heap baseline.
- **The Windows CI budget.** With 1B's E2E specs the Windows job takes about 19 minutes (test 5.6, spike reports 2.1,
  E2E build 4.6, specs 4.2, bench about 1), over section 16's 15. Barry chose to run Windows E2E and the bench only
  on main and on PRs that change apps/, crates/, packages/, tests/e2e/, the justfile, or ci.yml (section 16 allows
  the paths filter), over moving the leak test and bench to a nightly run or raising the budget. Such PRs take
  about 19 minutes on Windows.

## 2026-10-25, CI scope

- **CI runs only what a change can break.** The 1B PR took about 12 minutes a push (Windows E2E 5, bench 2, setup 2)
  and ran everything again on main after the merge. Barry asked for CI that runs full or both-platform tests only when
  needed, so: `scripts/ci-scope.sh` picks the jobs from the changed paths (prose skips both OS jobs; the UI and viewer
  skip Windows; Windows E2E only for `apps/desktop/src-tauri/` and the specs); a push after a green run tests only
  its own commits; main no longer reruns a merged PR; the bench (no assertions) runs only weekly, by hand, or with the
  `full-ci` label; the 0B spike crates leave the workspace; and `just ci` is CI's job, not a step before each push.
  This replaces 1B's Windows CI budget decision above. A change to the toolchain, a lockfile, the justfile, or a
  workflow still runs everything.
- **Tests sized to real risk.** Rule 5 now asks for tests where a bug is likely or costly, not for every line, and
  allows deleting a redundant test with the reason in the commit.

## 2026-10-26, session 1E

- **ring for Sign in with ChatGPT.** OpenAI's ID tokens are RS256 (its JWKS has only RSA keys), and OAuth's state,
  nonce, PKCE verifier, and the host's `urn:uuid` need a secure random source. Barry approved `ring` 0.17.14 as a direct
  dependency (default features off, `alloc` on), already in the tree through ureq and rustls, over RustCrypto's `rsa`
  with `rand` (about 10 new crates) and `jsonwebtoken`. The fakes use it too, to sign test ID tokens with a committed
  test-only key. ADR 0011 records it.
- **OpenAI's logo in the approved button.** The "Continue with ChatGPT" button's approved format includes OpenAI's
  white ChatGPT logo, and the CSP forbids remote content. Barry chose to ship OpenAI's SVG unchanged, with no SPDX
  header, with TRADEMARKS.md and NOTICE saying it's OpenAI's mark, used only inside that button as OpenAI's UI
  guidelines direct, and not covered by the AGPL, over a text-only button (not the approved format) or waiting on
  OpenAI.

## 2026-10-26, session 1F

- **Claude's sign-in without a terminal, and its fallback.** Emmet runs Claude Code's own `claude auth login`,
  unmodified, from Rust: with stdin closed and no TTY it opens the browser itself and finishes on a `localhost`
  callback, so no pseudo-terminal dependency is needed. When the browser can't reach that callback, Claude's page shows
  a code to paste into Claude Code. Barry chose that Emmet never touches the code: its sign-in card explains what
  happened and offers the one-command Open Terminal card, over a field in Emmet that passes the code to Claude Code's
  stdin, which would have Emmet handle a sign-in credential.
- **Antigravity's MCP server goes in the project.** Antigravity has no command to add an MCP server; its docs name a
  global `~/.gemini/config/mcp_config.json` and a per-workspace `.agents/mcp_config.json`. Barry chose that a click in
  Emmet writes the project's `.agents/mcp_config.json` (a project file, like the AGENTS.md Emmet already writes;
  never `~/.gemini`), so no step asks the user to edit a file, over a card that shows the JSON to paste.
- **The Mac app comes first (D-43).** Barry, after the Windows run found 1E's credential bug: skip the Windows steps
  for now, focus on getting the Mac app working (function, design, features), and go back through for Windows once it
  works. Windows code is still written where a feature needs it. For the next few phases the Windows CI jobs are
  paused too, for quicker progress: `scripts/ci-scope.sh` turns `windows`, `e2e_windows`, and `nsis` off unless
  `EMMET_CI_WINDOWS=1`, which a manual run sets with its "windows" box. 1F's Windows fresh-account run and H-10 wait
  for that pass.
- **Antigravity, Cursor, and Grok are out of the MVP (D-43).** They come after it (Phase 8). Barry chose to remove 1F's
  Antigravity terminal mode from the PR (the card, `agy` detection, the project-file writer, and their tests), over
  keeping it hidden or shipping it unverified; it's in git history for later. This replaces the project-file decision
  above for the MVP. 4F is skipped and H-25 isn't needed.

## 2026-10-27, session 3A

- **The LDCad shadow data is compiled into the binary.** The connector table built from the LDCad shadow library
  (CC BY-SA 4.0) stays its own file in the source, `data/connections/ldcad-shadow.ldr`, with its license and
  attribution; emmet-core compiles that file's bytes in unchanged, as it does `data/synonyms.toml`, and NOTICE (and
  About, from 4A) credit it. Barry chose this over shipping it as a separate file read at run time (ADR 0003's wording, with a
  lookup now and a packaging step in 6A) and over not using it (overrides only). CC lists GPLv3 as compatible with
  BY-SA 4.0, and GPLv3 combines with the AGPL. ADR 0003 records it.
## 2026-10-28, session 3A

- **Connector match tolerance: 1 LDU for round joints, 0.5 LDU for studs.** Official OMR files place parts by hand,
  and round joints (wheels on holder pins, hands on clips, hinge fingers, posed arms) are often 1 LDU (0.4 mm) off
  their mates; plan section 11's 0.5 LDU reported those as floating. Barry chose 1 LDU across the axis for pins,
  axles, bars, clips, fingers, and generic plugs, keeping 0.5 LDU for studs so stud-built work keeps a tight grid, over
  1 LDU everywhere and over 0.5 LDU with those models left out of the corpus. Faults agents make are 4 LDU or more, so
  recall doesn't change; LDCad data found wrong for a part gets an exact override, never a looser rule.
- **Collision threshold: over 1 LDU.** Collision proxies are 1 LDU voxels, and a sloped or curved face can stick out
  half a voxel, so touching slopes and official files' hand-placement slop read as 1 LDU overlaps (22 of the 33 false
  collisions left on the nightly corpus). Barry chose to report overlaps of more than 1 LDU, past the measurement's own
  resolution, over plan section 11's 0.5 LDU (with the 1 LDU findings chased one at a time) and over 0.5 LDU voxels
  (about four times the build time and cache, and still quantized). Injected faults overlap by 8 LDU or more.
- **Closing the nightly corpus.** 19 of the 33 nightly models validated clean; the other 14 had verified file faults
  (21022-1's attic resting on tile edges, 10124-1's rigging drawn with primitives, a hinge stick 2 LDU off its pivot
  in 4 models) or part-specific data gaps. Barry chose to fix the gaps on parts agents will use (dishes on round
  bricks, container doors, turntables, steering wheels, antennas) with exact overrides, and to take models whose only
  errors are file faults or rare parts (train magnets, jet engines, the excavator) off the list, each with its reason
  in nightly.json and an issue per gap, replacing them with clean models from the cached OMR pool to keep 20–40; over
  fixing every gap first (several more hours) and over trimming to the clean models alone.

## 2026-10-28, session 3B

- **How the BOM exports were proved against the real sites.** Done when 4 asks for the Rebrickable CSV and the
  BrickLink XML to import cleanly. Barry chose that Claude drive his signed-in Chrome, over uploading them himself,
  BrickLink only, or leaving the item open: the rung 2 reference's wanted list went through BrickLink's upload to its
  verify page, which recognized all 29 lines and saved nothing, and its parts list was imported into a new private
  Rebrickable part list (29 lines, 154 pieces, no errors; two parts moved to newer molds by Rebrickable's own "Fix
  Molds" option), which was then deleted. Barry signed in to Rebrickable himself. Screenshots are in
  `docs/spec/sessions/3B/`.
- **No credit line in the Rebrickable parts-list CSV.** Plan section 12 puts Rebrickable's credit in BOM exports as a
  header line in CSV and a comment in XML. The plain CSV and the BrickLink XML carry it; the Rebrickable import file
  goes back into Rebrickable, and any line besides `Part,Color,Quantity` and the parts risks breaking its import.
  Barry chose to leave the credit out of that one file (it's in the other exports, the Parts list, Settings → Data,
  About, and NOTICE), over adding it and testing the import again, or putting it in the file name. Section 12 says so.

## 2026-10-28, session 3C

- **Mined techniques aren't derived from OMR models (D-48).** Every model the miner reads (the 10 committed
  fixtures and the 33 nightly ones) is "Redistributable under CCAL version 2.0", which LDraw reads as CC BY 2.0: that
  would allow snippets derived from them under our Apache-2.0, with each model's author, title, and URL credited.
  Barry chose not to derive from them at all, over crediting every model a motif was found in and over crediting
  only the examples a snippet was written from. The mining ranks ideas (which sub-assemblies recur, and how widely),
  and each technique is written from general building knowledge and Barry's own instruction books (sets 21335,
  10350, 43217, and 60440), which are read for methods only: no page, picture, or text from them goes into the repo.
  Each technique's provenance note says that, citing the books by set number. ADR 0003 records it.
- **What proves 3C's Done when 2 (D-49).** It asks that rung 2's replay and dry-run eval scores don't regress, but no
  rung 2 eval exists yet: `emmet eval` runs rung 1, and rung 2 arrives with 5A. Barry chose to prove it with what
  exists (the rung 1 dry-run eval with both fake agents, every replay test, and the rung 2 reference model clean at
  each of its params), with the real rung 2 scores moving to 5A and G2, over building a rung 2 dry-run in this
  session and over leaving the item unticked.
- **Review pages show our own sketches.** Cropped renders of official models where a candidate's key parts meet
  often showed those parts used for something else (the window hood's match was a tower's siding), and pages of the
  instruction books can't be shown. Barry chose that each candidate gets a quick sketch written in the DSL and
  rendered, so the picture is what Emmet would build; accepted sketches become the techniques and rejected ones are
  dropped. Rejected: the crops labeled as "may not be this technique", and pages with no pictures. The miner still
  gives each candidate how widely official sets join its key parts.
- **H-22, batch 1: all 20 accepted** (Barry, 2026-10-28, on `docs/techniques/candidates/batch-1.html`): quoin_corner,
  wall_top_tie, lift_off_roof_seat, lift_off_floor, jettied_floor, masonry_wall, clapboard_siding, half_timber_band,
  window_sill, window_hood, shuttered_window, shop_window, door_hood, roof_dormer, ridge_cap, chimney_stack, downpipe,
  porch_posts, sidewalk, wall_vent. Barry asked for techniques for every kind of build, not only houses.
- **H-22, batch 2 (vehicles and trains): 18 accepted, 2 rejected** (Barry, 2026-10-28, on
  `docs/techniques/candidates/batch-2.html`). Rejected: wheel_axle and wheel_arch, because the wheel on the 2 x 2 plate
  with wheel pins (4870, 4624c01) reads as aircraft landing gear, not a car's wheels. Accepted: headlights,
  tail_lights, windscreen, curved_hood, trailer_hitch, front_bumper, bracket_side_panel, cab_seat, light_bar,
  front_grille, tilting_hood, spliced_chassis, rear_spoiler, exhaust_stack, cargo_pallet, train_carriage,
  hinged_tailgate, technic_spine. That makes 38 accepted, past Done when 1's 30.
- **H-22, batch 3 (landscape and nature): all 20 accepted** (Barry, 2026-10-28, on
  `docs/techniques/candidates/batch-3.html`): rock_spire, rock_facet, cliff_wall, rock_outcrop, surf_line, river,
  waterfall, pond, grass_edge, hillside, flagstone_paving, dirt_path, pine_tree, bush, flower_bed, cave_mouth, snow_cap,
  footbridge, woodpile, landing_zone. 58 accepted in all.
- **H-22, batch 4 (castles and ships): all 20 accepted** (Barry, 2026-10-28, on
  `docs/techniques/candidates/batch-4.html`): battlements, machicolation, arrow_slit, buttress, round_tower, portcullis,
  drawbridge, torch_sconce, wall_banner, wooden_hoarding, bartizan, thick_wall, ship_hull, mast_and_yard, crows_nest,
  cannon_port, rudder_hinge, oars, dock_pier, portholes, with the cards' notes as written (round_tower's seams line up;
  ship_hull's bow wedges are tied only by the deck). 78 accepted in all.
- **H-22, batch 5 (sci-fi and mechanisms): all 20 accepted** (Barry, 2026-10-28, on
  `docs/techniques/candidates/batch-5.html`): greeble_strip, engine_nozzle, wedge_wing, angled_hull_panel,
  hatch_on_hinge, cockpit_canopy, turret, radar_dish, control_console, antenna_mast, solar_panel, landing_leg, gear_pair,
  worm_drive, crank_winch, technic_pin_hinge, lift_arm_linkage, technic_brace, rotating_platform, ball_joint_arm.
  98 accepted in all, of 100 candidates in five batches.

## 2026-10-29, CI

- **E2E beside Barry's work (D-50).** WebKit stops drawing a covered window, so E2E on the self-hosted Mac failed with
  "the app's window is covered" whenever Barry worked over it. Barry first chose hosted `macos-latest` for E2E; a
  trial run (40950731531) passed but shrank the window to 1024 × 674, so the 52 window baselines couldn't be shared
  with his Mac (the renders matched), took 7.5 minutes of specs to the Mac's 4, and needed retries in the chat and
  keyboard specs. Barry then chose to keep E2E on his Mac with the E2E build out of his way: no focus at launch, no
  Dock icon, and WKWebView's private `_setWindowOcclusionDetectionEnabled:` so a covered page keeps drawing (60 fps
  covered in a test). The call needs unsafe code: the workspace lint is `deny` instead of `forbid`, `e2e.rs` allows
  it, and `scripts/unsafe-check.mjs`, run by `just check`, keeps every other file free of it. It calls through `objc2`, already in the
  tree through tauri and the WebDriver plugin, so no new crate. Rejected: hosted E2E, its own baselines, or a hosted
  PR run with a weekly run on the Mac; and only stopping the app from taking focus.

## 2026-10-29, session 3D

- **Session 3D before 4B (D-51).** Barry asked, before 4B, for detailed technique mining from all his instruction
  books (92 booklets in his Dropbox, about 55 sets: houses and town buildings, a voxel village, game playsets, cars
  at every width and scale, architecture and micro models, display ships, and sculptures), so Emmet has many wall,
  roof, and interior treatments for a house, and knows the blocky and video-game styles, cars at each scale, and
  micro and micropolis; then how the agent should use them, practiced on the Cottage. The final Cottage for the MVP
  is a detailed figure-scale building with character, landscaping, a detailed interior, and a removable roof. The
  books are read for methods only, under D-48: no page, picture, or text in the repo; techniques cite set numbers.
- **The agent reads guides for its build settings.** One guide per scale, style, and subject (vehicles, interiors,
  landscaping), in `crates/emmet-cli/doctrine/guides/`, served by `emmet guide` and the MCP `guide` tool: with no
  name, the guides for the project's `[build]` scale and style. Rejected: putting every style's rules in the
  doctrine, which every session reads in full. No trademark appears in a guide's names: the doctrine tells the agent
  to map a game or brand the user names to the guide for its look.
- **H-26, batch 6 (house walls and facades): all 20 accepted** (Barry, 2026-10-29, on
  `docs/techniques/candidates/batch-6.html`): wainscot_split_wall, plaster_with_exposed_stones, two_tone_masonry,
  string_course, dentil_band, timber_frame_panels, panel_pilaster, rusticated_stone_pilaster, color_block_facade,
  wall_top_cornice, lattice_skirt_foundation, curved_wall_end, rough_stone_ledge, two_story_color_split,
  white_ashlar_wall, exposed_stud_stonework, irregular_stone_wall, balustrade_parapet, bermed_wall_reveal,
  corbelled_balcony. Barry's note on wall_top_cornice: the stud left for the roof to grip should stand at the tiles'
  height; each lug is now a 1 x 1 plate flush with the tiles, its stud up.
- **Technique examples are checked standing on their lowest level.** Loose on the table, the validator lets a group of
  parts that floats beside something pass as a separate toy, which hid floating parts in three accepted techniques
  (headlights' lenses 0.2 studs too far out, bracket_side_panel's brackets turned the wrong way, grass_edge's example
  cliff on nothing) and two candidates. The technique tests, `scripts/technique-candidates`, and the sketch checks now
  mark the example's lowest parts as ground, as a baseplate would.
- **H-26, batch 7 (windows, doors, and porches): all 20 accepted** (Barry, 2026-10-29, on
  `docs/techniques/candidates/batch-7.html`): bay_window, window_box, round_window, twin_arched_window,
  eyebrow_arch_hood, dome_window_hood, slope_hood_small_window, lattice_shopfront, round_corner_bay, plank_door,
  doorstep_and_mat, panel_garage_door, round_window_frame, turned_post_porch, wing_plate_porch_roof, scalloped_awning,
  wall_lamp_bracket, hanging_planter, blade_sign, signboard_beam.
- **H-26, batch 8 (roofs, chimneys, and roof details): all 20 accepted** (Barry, 2026-10-29, on
  `docs/techniques/candidates/batch-8.html`): tiled_roof_panel, plate_shingle_roof, tile_stripe_shingles,
  studs_up_snow_roof, mansard_module, cross_gable, curved_dormer, mini_dormer, roof_cresting, pyramid_spire_tower,
  ribbed_hip_roof, tapered_tower_cap, banded_chimney, sideways_chimney_column, roof_hatch, lean_to_awning,
  half_roof_hinge, overlap_tile_roof, roof_eaves_overhang_frame, eaves_fascia.
- **H-26, batch 9 (rooms and furniture): all 20 accepted** (Barry, 2026-10-29, on
  `docs/techniques/candidates/batch-9.html`): hearth_fireplace, wood_stove, figure_bed, armchair_and_sofa,
  kitchen_counter, table_and_chairs, wall_shelf, wall_picture_and_clock, plank_floor, floor_rug, floor_lamp_and_candle,
  interior_arch_doorway, loft_half_floor, cross_display_stand, curved_booth, bar_counter, display_case_counter,
  upright_piano, dresser_shelf, bookcase.
- **H-26, batch 10 (gardens and landscaping): all 20 accepted** (Barry, 2026-10-29, on
  `docs/techniques/candidates/batch-10.html`): organic_ground_outline, curved_garden_path, stepping_stones, garden_gate,
  lawn_with_plant_sockets, contour_terraces, garden_boulder, tiled_pond_water, sunken_pond_bank, lily_and_reeds,
  layered_conifer, blossom_tree, palm_tree, bamboo_grove, potted_plant, clipped_hedge, stone_lantern, snow_drift_edge,
  gnarled_tree, cypress. garden_gate takes an `id` (the Cottage's gate), as figure_bed, hearth_fireplace, and
  table_and_chairs now do.
- **H-26, batch 11 (blocky worlds): all 20 accepted** (Barry, 2026-10-30, on `docs/techniques/candidates/batch-11.html`):
  block_column, raised_module_base, module_link_block, grass_block, stepped_block_terrain, block_tree,
  speckled_block_wall, mossy_stone_block, bookshelf_block, cactus_column, reed_stalks, block_water_and_falls, lava_inset,
  crop_field, block_fence_post, stick_torch, plank_and_stone_wall, stepped_plank_roof, flat_jumper_roof, block_beast.
- **H-26, batch 12 (video-game worlds and retro hardware): all 20 accepted** (Barry, 2026-10-30, on
  `docs/techniques/candidates/batch-12.html`): pixel_mosaic_panel, tile_brick_texture, item_block, float_on_clear,
  cube_head_character, brick_enemy, chunky_kart, curb_rumble_strip, start_gantry, bolted_pillar, flip_bridge,
  a_frame_tent, brick_campfire, crt_cabinet, console_shell, vent_fins, controller_slab, game_pipe, polka_dot_cube,
  item_launcher. game_pipe was "warp_pipe" until its sketcher flagged the name as one game's term.
