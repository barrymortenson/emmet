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

## 2026-10-28, planning session 6C

- **Technique library v2 is session 6C, after 5A.** Asked where the next technique session goes, now that 3C's
  library is in: it needs 5A's eval to measure whether agents use techniques and whether builds vary. Barry chose
  a Phase 6 session after 5A, beside 6A and 6B, over a third Phase 5 session before G2 and over a backlog session
  after launch. 7B comes after it, so 1.0 ships with it. He added that Emmet must stay creative: it comes up with
  its own wall designs, or has many options, so its builds don't look alike. That is 6C's variety work and its
  variety eval.
- **Technique sources for 6C (D-54).** Asked where v2 may take its methods from, Barry chose the miner's top
  motifs as ideas (D-48 stands), more of his own instruction books (then a 6C step; 3D later read all 92 booklets, D-51), and LEGO's online building
  instructions, and added forums, blogs, and videos where builders explain how they build (roads and track
  dressing are methods he learned that way): "many people do things different but that is what we want to
  collect." All are read for methods only, and each technique cites its sources by set number or URL; no text,
  picture, model file, or author's name goes into the repo. He didn't choose deriving techniques from OMR models
  with credit, so D-48 holds.
- This plan never merged. The 2026-11-03 entry below rewrote it, and its D-50, H-26, and H-27 are now D-54, H-28,
  and H-29.

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
- **H-26, batch 13 (vehicles at figure scale): all 20 accepted** (Barry, 2026-10-30, on
  `docs/techniques/candidates/batch-13.html`): car_wheels, truck_spine_chassis, six_wide_truck_cab, cab_over_sleeper,
  racing_scale_base, quarter_tile_headlights, side_stud_lights, curved_wedge_nose, tapered_hood_wedges, snot_side_skirt,
  rounded_car_end, window_band, shipping_container, lift_off_window_cab, vintage_chassis, vintage_radiator, street_cart,
  four_wide_car, offroad_fender_flare, glass_box_cab. car_wheels puts car rims and tires on flush pins inside mudguard
  arches at the body's sides, not on the plate with hanging wheel pins that 3C's rejected wheel_axle used.
- **H-26, batch 14 (vehicles at large scale): all 20 accepted** (Barry, 2026-10-30, on
  `docs/techniques/candidates/batch-14.html`): large_display_wheel, technic_brick_ring_chassis, liftarm_ladder_frame,
  beam_spine_floor, hang_on_side_panel, rolled_sill, fender_crown, inverted_arch_wheel_well, tapered_deck, canopy_hump,
  front_hinged_hood, dashboard_and_wheel, pleated_seat, round_tail_light_cluster, ring_light_grille, side_vent_grille,
  flared_exhaust, steering_rack, removable_soft_top, engine_bay_greebles. removable_soft_top and engine_bay_greebles
  stand in for a wishbone suspension and a V engine, whose parts (5294, 2909/2910, 2853a/2854) have no connector data
  yet; front_hinged_hood was "front_hinged_bonnet" until the US English sweep.
- **H-26, batch 15 (micro and micropolis): all 20 accepted** (Barry, 2026-10-30, on
  `docs/techniques/candidates/batch-15.html`): micropolis_block, micro_tower, tower_setback_plinth, tower_crown,
  tower_side_fins, floor_band_bricks, stepped_tower_setbacks, bar_spire, paired_colonnade, spiral_dish_stack,
  micro_tree_cluster, micro_traffic, micro_boat, water_tile_field, statuette_group, micro_house_on_wall,
  raked_seating_panel, mowed_pitch, curtain_wall_glazing, skyline_depth_rows.
- **H-26, batch 16 (sculptures, characters, and letters): all 20 accepted** (Barry, 2026-10-30, on
  `docs/techniques/candidates/batch-16.html`): snot_core, curved_slope_skin, contour_slab, layered_plate_cap,
  sphere_from_caps, dome_from_quarters, rounded_column, hover_stand, side_stud_eyes, big_round_eye, muzzle_and_nose,
  teeth_row, floppy_ear, brick_letters, posable_neck_ball, hinged_limb, pyramid_pedestal, countershade_belly,
  feather_shingles, smooth_display_base. hover_stand replaced egg_from_panels, which read as a box.
- **H-26, batch 17 (sci-fi rooms, ships, and fantasy stonework): all 20 accepted** (Barry, 2026-10-30, on
  `docs/techniques/candidates/batch-17.html`): chamfered_door_portal, hangar_chevron_floor, recessed_glow_panel,
  lit_window_panel, checker_light_wall, glow_trim_edge, pipe_column_cluster, ribbed_side_panel, lattice_tower,
  bar_truss_brace, round_deck_dome, clamshell_pod, plank_deck, davit_lifeboat, round_funnel, rock_mosaic_panel,
  double_arch_colonnade, fin_merlon_crenels, round_bastion, fountain_court.
- **H-26, batch 18 (midi: display ships and architecture): all 20 accepted** (Barry, 2026-10-30, on
  `docs/techniques/candidates/batch-18.html`): stave_cylinder, stepped_curve_hull, bare_stud_greeble, porthole_band,
  saucer_petal_panels, radial_spoke_saucer, axle_round_stack, angled_display_stand, lancet_window, rose_window_ring,
  flying_buttress_arch, pinnacle_stack, pointed_buttress_strip, belfry_louvers, steep_nave_roof, slender_column_capital,
  triangle_tile_floor, banded_round_turret, architecture_base, rounded_pier_nose. 260 accepted in all, of 260
  candidates in thirteen batches; the library has 378 techniques.
- **H-26, the Cottage: approved as is** (Barry, 2026-10-30, through AskUserQuestion, on the renders in
  `docs/spec/sessions/3D/`): 903 parts, a stone-and-plaster house with a bay window, a porch, a dormer, an outside
  chimney, and a roof that lifts off as one sub-assembly; two furnished rooms; a garden with a fence and gate, a path,
  a mailbox, raised lawns, a bench, hedges, and a tall tree. Knobs: season, roof, weathering, seed.
- **The viewer budget holds on the new Cottage** (Barry, 2026-10-30: "speed it up now"). The 903-part Cottage took
  1,221 to 1,250 ms from a slider change to the updated viewer in the E2E (debug) build, past 1B's 1 s budget; 430 ms
  of it was the validator running unoptimized. `emmet-core` now builds at opt-level 2 in debug builds too, as every
  dependency already did, and the same change takes 738 to 797 ms (three runs). Sending the scene before its findings
  and caching the schema read, the other two ideas offered, aren't needed for the budget; they're left for 4C.

## 2026-10-30, session 3E

- **Session 3E before 5A (D-52).** In his 4B hand session of rung 2 on Claude Code, Barry found the model rendered
  but wasn't built well: the canopy's eaves sat below the ticket booth's door top, and the benches stood against the
  canopy's columns (`docs/spec/sessions/3E/before-4b-station.png`). He wants Emmet to get this kind of thing right
  without extra prompting, and asked for a session that takes a complete look at building with good technique and
  makes sure the technique library doesn't hold models back. The causes were in the library as much as the agent:
  `canopy` hard-codes 5-brick columns under a 6-brick door, `porch_posts` and `turned_post_porch` are low the same
  way, `canopy` doesn't say where its columns stand, `validate` only sees overlaps, and the guides give sizes but no
  clearances. 5A waits for 3E, so G2's live eval runs on the fixed library.
- **The `figure_clearance` rule (D-53, proposed).** A warning, not an error, so 3A's zero-error gate on the official
  corpus is untouched; 3E holds the rule to zero findings on that corpus too. Keep-clear boxes come from door frames,
  from techniques and the agent through `m.keep_clear(box, why)`, at figure scale and large only. Rejected: rules in
  the doctrine alone, which the agent skims, while it fixes every finding `validate` gives it.

## 2026-10-30, session 4B

- **Codex's rung 2 hand session is taken as it is** (Barry, through AskUserQuestion). On his ChatGPT plan, at Codex's
  default low effort, it left a clean model of 210 parts: under rung 2's 250, with no door on the ticket booth, and
  standing on the table after a Rebuild for Base None took the baseplates out. 4B's Done when asks for a clean build,
  and this one is; the shortfalls go to 3E (technique) and 5A (rung 2's eval). Its transcript is
  `tests/replay/codex/rung2.jsonl` and its project `tests/fixtures/projects/rung2-codex`.
- **A highlight has to be seen, and an agent's says whose it is** (Barry, through AskUserQuestion, after his terminal
  agent's highlight of the station's two benches showed nothing he could see). The viewer dimmed the rest to 35 %
  and left the highlighted parts as they were, and their edges at full strength, so on a dark model, or a few small
  parts in a big one, nothing stood out. Now the rest fades 80 % toward the background, the highlighted parts draw over
  it in their own colors. An agent's highlight puts a bar over the viewer: "Claude is pointing at 10
  parts", Zoom to them, Clear. The agent is named from its MCP client's own name (the bus's new optional `from`). The
  camera moves only when the user asks. Rejected: zooming to the parts on every agent highlight, which moves the
  user's view without asking; and leaving it to 4C. Barry then had the accent edges and the silhouette of a hidden
  part taken out (2026-10-31): the parts keep their own colors. A hidden part isn't seen until Zoom to them or a turn
  of the view.

## 2026-10-31, session 3E

- **The building rules are approved as written (H-27, part 1; Barry, through AskUserQuestion).**
  `docs/spec/building-rules.md`: headroom at or above the door top beside it, a door's approach clear on both sides
  (its width, 3 studs out, from a plate above its bottom up to its top), a seat's ends, front, and headroom, walkways 3
  studs clear with no post in line with a door, how parts meet, and the scales they hold at (figure scale, large where
  the model has figure doors or seats, and Any). Each rule names what enforces it. Rejected: a 2-stud approach on a
  door's room side, which only the `door_frame` technique could tell from the street side.
- **D-53 is decided** with the rules: `figure_clearance` is a warning, from keep-clear boxes declared by door frames
  (the parts index), by techniques, and by the agent with `m.keep_clear`, at figure scale, large, and Any.
- **The technique prompts' baseline runs with `main`'s library** (session prompt): `emmet eval` learns the five prompts
  in `docs/spec/ladder.md` (T1 to T5) and keeps each run's project, and that is the only change before the ten
  baseline runs. They are the scripted runs D-52 asks for, on top of D-38's gate budget: ten now and ten at the end.
- **Door spaces come from the techniques that place doors, not the parts index** (Barry, through AskUserQuestion).
  Found from door frames in the parts index, rule 2's space gave 269 findings on the 67 official models in the 3C pool
  that have 1 x 4 x 6 door frames, and still 90 when cut to a 1-stud, 2-wide doorway: counters, stairs, and crates a
  stud behind a door, walls built right behind a facade's decorative door, upper-floor doors opening under a slope.
  The committed and nightly OMR corpus has no figure doors, so it would have passed without testing anything. Now
  `door_frame` and the other door techniques declare the approved space with `m.keep_clear`, and the doctrine asks the
  agent to declare one for a door frame it places by hand. Rejected: the parts index with the narrow doorway, which
  leaves official models with findings and changes D-53's bar to errors only; and both sources.
- **The 4B station is checked two ways** (Barry, through AskUserQuestion). On the fixed library the station's canopy,
  called without a height, clears the booth's door by itself, so `tests/fixtures/projects/rung2-claude` as it is gives
  the two bench findings only. `ladder.rs` checks the station with its canopy at 4B's height (`height=5`, the old
  5-brick column): exactly the three findings Done when 2 names, the canopy at the booth's door and a column against
  each bench; and the fixture as it is: exactly the two bench findings. Rejected: the fixture alone with Done when 2
  reworded, and a copy of the old canopy pinned in the fixture.
- **The Cottage drops its wall clock** (Barry, through AskUserQuestion). Now that the Cottage declares its front door's
  way through with `m.keep_clear`, figure_clearance found the clock in the cell beside the door hanging half a stud
  over the doorway at head height, at every width (the living room is the door plus one cell wide on that wall).
  Rejected: a picture in its place, which starts at the same cell, and moving the clock to another wall.
- **The Cottage's porch stands taller** (rule 1, approved with the building rules). Its beam started 2 bricks under the
  front door's top, the 4B canopy's mistake: the posts are 7 round bricks instead of 5, the beam starts at the door's
  top, and the porch roof's back row, which ran into the house's eaves once raised, is gone, so the porch roof tucks
  under them (rule 5). `docs/spec/sessions/3E/cottage-{before,after}.png`.
- **The narrowest Cottage has no hearth** (Barry, through AskUserQuestion). At width 14 the living room is the front
  door and a stud wide, and the fireplace and the firewood stood a stud inside the door's way in. At width 14 only, the
  living room leaves them out; the chimney stays outside, and the armchair and rug stay. Rejected: a width knob from
  16, and moving the armchair to make room. The snowman (winter) stands a stud further back, clear of the garden
  bench's legroom, which its widest garden had crowded.
- **The Cottage's kitchen table only in the widest house** (Barry, through AskUserQuestion). `interior_arch_doorway`
  now keeps its way through clear, and in a kitchen 7 wide (widths 16 and 18) the counter and the table set fill it
  to the partition, in front of the doorway. The table and chairs go in only where they leave it 3 studs (width 20).
  The same pass moved the partition's doorway a stud back (the lamp behind the bay clears it), left the kitchen lamp
  out where the table is, stood the garden lamp behind the gate's post (it stood in the gate's opening), and planted
  the lawn's flower bed only where it leaves the bench's end a stud. Rejected: a small table by the bay, and a
  2-stud way through for interior doorways.
- **H-27, part 2: the after renders are better** (Barry, through AskUserQuestion). The technique prompts went from 4
  figure_clearance findings in the baseline to 0 at the end on both agents, and both stations centered their benches
  between the columns and stood the canopy at the booth's door height, unprompted (`docs/evals/2026-10-31-technique.md`).

## 2026-10-31, session 4C

- **Box select and the layer cut-away** (Barry, through AskUserQuestion; the frozen viewer design doesn't show
  them). Shift-drag in the viewer draws a box that selects the parts in it. The cut-away is in the View menu, Cut Away
  Above (`⌥↓` lowers the cut a layer, `⌥↑` raises it), and while it's on a bar over the viewer names the layers shown,
  with lower, raise, and Show all, as the version bar does. Rejected: a slider always on the viewer's right edge, and
  two more mode buttons in the view switch.
- **Screenshot copies, and the toast offers Save** (Barry, through AskUserQuestion; the frozen view switch has one
  Screenshot button). A click copies a PNG of what the viewer shows to the clipboard at once, and the toast, "Copied a
  picture of the model", offers Save to file, which writes it where exports go and then offers Show in folder. The
  View menu has Copy Screenshot and Save Screenshot. Rejected: a two-item menu on the button, and saving first with
  Copy in the toast.
- **One bucket per part and finish, not per part and color** (4C, from the profile). The 2,000-part town block drew
  in 647 calls, against section 13's ~300, because each (part, color) bucket draws its faces, edges, and conditional
  lines. The finish's material is now white and each instance carries its color (faces, and the edges' softened edge
  color), so a part in many solid colors is one bucket: 229 draw calls for the 2,018-part block and 272 for the
  Cottage. Transparent colors bucket by alpha and glowing ones by color, since the material holds those. Section 13
  and ADR 0008 say so. Rejected: merging parts into one geometry per color, which breaks the diff by id and the
  per-part highlight.
- **Renders always draw the full detail and the ground shadow** (4C), whatever the window's quality, so an agent's
  render, a baseline, and an export look the same on every machine.
- **Pins for the worst five findings** (4C): a pin for each finding with parts, errors first, at most five. A model
  with hundreds of findings made a pin each and slowed every frame the camera moved.
- **Design change note 004 approved as written** (Barry, through AskUserQuestion, 2026-11-01): Replay build in the
  view switch, the hover tooltip, shift-drag's outline, the cut-away bar, up to five finding pins, the step's parts box
  with sub-assembly boxes and a leader, and Settings' quality pick and Build animation row
  (`docs/design/changes/004-4c-viewer.md`).
- **4C's Windows half: the PR opens now, and Windows CI runs once on the branch** (Barry, through AskUserQuestion,
  2026-11-01). The macOS halves of Done when 1 and 2 are ticked; a manual CI run with its "windows" box checked
  reports the hosted Windows numbers and writes Windows baselines from its artifact. Rejected: leaving Windows wholly
  to the Windows pass, as 4A and 4B did, and holding the PR until then.
- **One pin per rule, and only for rules about a place** (Barry, through AskUserQuestion, 2026-11-02). A model with
  many step-size warnings showed five pins, each "A step adds more parts than the age allows", stacked over the
  model's middle. Now each rule about where parts are (floating, collision, weak connection, figure clearance) gets
  one pin over all its findings' parts, naming their count; a click opens the rule's first finding in Checks. Step
  size, step order, and the library rules get no pin. Replaces 4C's "pins for the worst five findings". Rejected: one
  pin per rule including step size, and a pin per place finding up to five.
- **Two CI faults fixed in 4C's PR, then Windows CI once more** (Barry, through AskUserQuestion, 2026-11-02). The
  manual Windows run stopped before E2E: hosted Windows took 720 ms for a warm `emmet run` against tests/it's 400 ms,
  and the hosted Mac's `just check` ran clippy before a fresh checkout had built apps/desktop/dist (main's scheduled
  run failed the same way). Now `just check` builds the frontend when dist is missing, and warm_run_is_fast reports
  on hosted runners and enforces 400 ms on the self-hosted Mac, as perf.e2e.mjs does. Rejected: a separate `ci:` PR
  first, and leaving 4C's Windows half unticked.
- **Hosted CI jobs get 45 minutes** (Barry, through AskUserQuestion, 2026-11-02). The rerun passed all of Windows'
  `just test` (21 minutes, cold) and was stopped at 25 building the E2E app; the hosted Mac stopped at 20. A full run
  on a hosted runner now has 45 minutes for the windows job and the mac job; the warm self-hosted Mac keeps 20.
- **Windows E2E runs perf, and Windows CI runs once more on 4C's branch** (Barry, through AskUserQuestion,
  2026-11-03). Run 41566073449's E2E failed in setup on Windows' locked fake claude.exe (fixed in bbb0e63), and main's
  Windows tier had no perf spec, so no Windows run could report section 13's numbers. `perf` joins
  `tests/e2e/tiers.mjs`'s Windows list (hosted runners report the budgets without failing on them), and one manual run
  with Windows checks the fix, the Windows numbers, and the six Windows baselines. Rejected: the run without perf, which
  leaves Done when 1 without Windows numbers, and merging with both Windows halves unticked.
- **Windows' budgets go to the Windows pass; its baselines hold in 4C** (Barry, through AskUserQuestion, 2026-11-03).
  Run 41773913493 passed all of Windows' visual tests against the six baselines. Its perf spec reported a first load
  of 9.0 s (budget 6 s) and slider to viewer 1.9 s (budget 3 s) for the 2,018-part block, on a hosted runner with no
  GPU; the build animation never started there (nor on the hosted Mac, most likely reduced motion in the runner
  images), and the job hit its 45-minute cap before the orbit and animation numbers. Done when 2 is ticked for
  Windows; Done when 1's Windows half stays unticked, with those numbers, for the Windows pass on real hardware.
  Rejected: forcing the animation under reduced motion for the measurement and running Windows again, and profiling
  the hosted runner's first load in 4C.

## 2026-11-03: CI runs what proves the app works

- **Lean tests and scoped CI** (Barry, through AskUserQuestion: "the ci should also only run what is needed for the
  changes in the app"). PR runs had passed section 16's own budget (a full Mac run took 16 minutes, and 19 of the
  last 60 runs failed, nearly all in Mac E2E). Now:
  - **E2E in tiers** (`tests/e2e/tiers.mjs`): core (opening a project, slider to viewer, a broken model and its
    recovery, chat with each fake agent, trust, a killed run, versions, import, Checks, render through MCP, and the
    visual scenes) for any change that reaches the app; each area's specs when its code changes; and an extra tier
    (the frozen screens and dock states as screenshots, memory over 100 changes, chat's two timing checks) that runs
    by hand (`just e2e all`), and in CI only when its own spec or baselines change. Nothing was deleted. `hello` runs first and
    alone, stops the run when it fails, and fails in a second when WebKit throttles the window to 30 fps; `just e2e`
    stops at once on a locked screen and keeps the display awake.
  - **The Rust integration tests are one binary.** macOS scanned each newly built test binary on its first launch,
    about 10 s each (measured 2026-11-03); the 36 binaries spent more time starting than testing. Three timing checks
    run alone after the suite, and the technique library's 408 builds run only when the techniques, the DSL, the
    engine, or ldraw-mini change. `emmet eval`'s test gets its own data dir, since the agent tests' bus now holds the
    shared one for the whole binary.
  - **"Everything" means a toolchain or lockfile change** (and weekly, and `full-ci`). A justfile or workflow change
    runs the checks and core E2E. The release-protocol build runs when `apps/desktop/src-tauri/` changes, as before; the bench runs by hand.
  - **Windows E2E runs the specs for paths and processes** (`just e2e windows`), and hosted runners report timing
    budgets without failing on them.
  - **No daily E2E job** (Barry, 2026-11-03: "I don't want a daily job to run"). The first version of this PR added
    one to `nightly.yml` at 19:00 UTC; it's gone, and `nightly.yml` runs only the validator corpus, as before.
  Rejected: deleting the extra tier outright, a daily E2E job, and doing this on 4C's branch.

## 2026-11-03, session 6C rewritten

- **6C is rewritten against what 3D and 3E built** (Barry, through AskUserQuestion: "Rewrite 6C"). The 6C plan of
  2026-10-28 never merged, and later decisions took its ids (D-50 for CI, H-26 and H-27 for 3D and 3E). It keeps the variety work (several
  approaches for each of 25 jobs, style arguments, a seed from the project, "Make it yours", and the variety eval), the
  online sources (now D-54), project techniques, technique search, measuring technique use, and the principles and
  assemblies. It drops what 3D
  did: Barry's instruction books (all 92 booklets) and the guides for each scale and style. Its YOU steps are H-28
  (the videos, forums, and blogs he learned from, no longer the books) and H-29. Its size is L, down from XL.
  Its new techniques meet 3E's building rules (an audit row, keep-clear boxes, and T1 to T5 still clean); a project's
  seed lives in `emmet.toml`, and a project without one builds as before; `techniques/` becomes a project source
  (section 07). These came from the PR's review; Barry approved all three (2026-11-03, through AskUserQuestion).
  Rejected: merging the old branch as written, and dropping 6C.

## 2026-11-04, the photo rung's subjects

- **Photo to model takes a photo of anything, and the photo rung tests real objects** (Barry, through
  AskUserQuestion: "Three real objects"). The rung's three subjects (H-23) are real objects of different kinds,
  none of them brick models: for example a building, a vehicle, and a small household object. Brick models were the
  easy case (the agent can count studs in the photo) and would mostly have measured that, not turning real things
  into bricks. 4D's goal now says "a photo of anything"; the rung's scoring (section 16.1) is unchanged.
  Rejected: one brick model as a baseline plus two real objects, and the plan's two brick models and one building.
- **The photo rung's images stay local** (Barry, 2026-11-04, through AskUserQuestion: "Keep these, local only").
  Barry's subjects are web images he doesn't hold the rights to, so they never enter the public repo. They live in
  a gitignored `data/ladder/photo/local/`; the repo commits only `data/ladder/photo/subjects.toml` (each subject's
  name, Barry's description, its answers, which image is angle a or b, and each image's SHA-256, so a G2 run proves
  it used the same photos). CI's dry run uses generated images. Barry scores on his Mac with the compare view; the
  G2 report shows the renders, not the reference photos. Rejected: openly licensed photos from Wikimedia Commons
  (the subjects would likely change), and Barry taking his own photos.
- **Four subjects; the rung passes at 3 of 4** (Barry, 2026-11-04, through AskUserQuestion). A leg lamp (the tall
  floor version), a moto-style e-bike, the Bradbury Building, and a small pickup truck. Each subject
  is one more run per agent. Rejected: 2 of 4, and dropping a subject.
- **D-38's G2 budget is 14 runs, up from 12** (Barry, 2026-11-04, through AskUserQuestion: "Four subjects, 14
  runs"). D-38 counts each photo subject as a run per agent; the agent first said four subjects cost the same, which
  was wrong, and asked again. Rejected: three subjects and 12 runs.
- **The Bradbury is a detailed modular building at minifigure scale, 1,500-3,000 parts, and the agent has to ask**
  (Barry, 2026-11-04, through AskUserQuestion: "Answer it when asked"). The prompt stays "Build this."; the subject's
  answer script gives the scale when the agent asks. Rejected: saying it in the prompt, and keeping it at 50-300.
- **The images for each angle** (Barry, 2026-11-04, through AskUserQuestion: "Floor lamp"): the floor leg lamp, with
  the table version as its angle b, labeled as a smaller version of the same lamp; the truck's front and rear
  three-quarter shots (the shot with dimension labels is dropped, since it hands the agent the measurements); the
  e-bike's three-quarter shot, then its side profile; the Bradbury's street corner, then the aerial.
- **G2's live eval runs are Barry's hand sessions on his plans, not scripted runs on API keys** (Barry, 2026-11-04:
  "These runs should be using my plans not the api"; then through AskUserQuestion: "Hand sessions on plans"). A
  scripted `just eval` run on a plan has no basis in the terms matrix: Anthropic says developers building products
  "should use API key authentication", and the chat pane bills a plan only for turns the user starts. So at G2 Barry
  runs each rung himself in the app on his Claude and ChatGPT plans, types the prompt, gives the answer script's
  answers himself (attaching a photo's second angle when asked), and `emmet eval --score <project>` scores the
  finished project and its transcript, sending nothing to an agent. 14 hand sessions. The terms matrix splits its
  "Automated runs" row: scripted spikes stay on the API keys (H-21), and live evals from G2 are the chat pane on his
  plan. D-38's G2 line, section 16's Live eval row, 16.1, H-09, and the `emmet eval` row in section 09 say so.
  Rejected: keeping scripted runs on the API keys, and plans by hand with the keys for any rung left over.

## 2026-11-04, what the first hand check showed

- **Emmet builds a finished, detailed model by default, not a basic one** (Barry, 2026-11-04, on the e-bike hand
  check: "we want this to build a detailed complex design not something so basic"). The agent built a 165-part e-bike
  from stacked plain bricks; a good display set of a dirt bike has its fork, brakes, chain, panels, and stand. The
  doctrine's "Keep it lean" now means few tool calls, never few parts, and a new "Finished, not basic" section has the
  agent build at High detail and a set's size when Detail and Pieces are Any, list what makes the subject itself and
  give each part real parts, and shape it rather than stack it. "Show a draft in the first minute" now covers any
  subject, and says not to plan the whole model before writing any of it (the e-bike's first build came at 7:36).
- **The photo rung's part ranges are set-sized** (Barry, 2026-11-04, through AskUserQuestion: "Set-sized ranges"):
  the lamp 300-1,000, the e-bike 600-2,000, the truck 800-2,500, and the Bradbury 1,500-3,000 as before, so a basic
  model fails. Rejected: floors only, and keeping 50-300 with the rubric judging detail.
- **A command's card says what it did** (Barry, 2026-11-04: "ran a command is too technical, we should either having
  it say in plain words what it is doing or have a set of words like sorting bricks, choosing colors, finding
  pieces"). The card reads the command and says it in words ("Looked through the project", "Tried the model",
  "Installed a tool"); one it can't read says "Worked on the project", with bench words ("Sorting parts", "Choosing
  colors") while it runs. The status after a step says "Thinking", then a new bench word every 15 seconds, so a long
  quiet stretch still shows the agent is working. "Pieces" stays "parts", as content.md has it.
- **A double click on the title row maximizes the window** (Barry, 2026-11-04). The capability allows Tauri's
  drag-region command for it (`core:window:allow-internal-toggle-maximize`); without it the double click was refused.
- **A turn cut off by Emmet closing says so** (Barry, 2026-11-04: "on the screen there was no indication it was still
  working"). The agent's rebuild of `just dev` restarted the app mid-turn; read back, the chat now ends with "Claude's
  turn stopped when Emmet closed." instead of going quiet.
- **A new session, 4G, before G2: Technic frames and display vehicles** (Barry, 2026-11-04, through
  AskUserQuestion: "New session before G2"). The e-bike retry, with the new doctrine, aimed at a Technic frame and
  spent about 20 minutes test-building: Emmet can't say which way a beam's holes face or whether a pin is seated, and
  has no motorbike techniques. 4G adds the connection data to `part_info`, beams placed by their holes with seated
  pins, the validator's pin connections, and display-vehicle techniques. It follows 4D, runs beside 4E, and 5A and 5B
  wait for it. Rejected: moving it into 6C after G2, and waiting for G2 to show the gap.
- **Design change note 005 is accepted as built** (Barry, 2026-11-04, through AskUserQuestion: "Yes, as built"),
  after the screenshots and his hand checks: the photo chips, the first-use line, Send with a photo and no words, the
  paperclip waiting while the agent works, the sent photo in its message, and the compare view.
- **4D's hand checks** (Barry, 2026-11-04). Claude Code, the e-bike side shot: it sized the bike from the photo,
  asked for a front or back photo at the end, and finished at 39:57 with 426 parts and 0 errors; its first build came
  about 2:35 into its own time (the scale question waited 5:49 for Barry), so "a first draft within a minute" isn't
  met yet, and 4G is the fix. Codex, the truck's front three-quarter shot: "A light blue Ford Maverick crew-cab pickup
  with black trim, silver steel wheels, and a short open bed", in 0:22; the saved copy in `refs/` has no metadata.

## 2026-11-05, session 4E

- **The step number goes over the parts box** (Barry, 2026-11-05, on the rung 1 PDF prototype: "the step number
  should be above the parts"), in the PDF and, through AskUserQuestion ("Match it in the app"), in the viewer's step
  view too, so the screen and the printout read the same. Plan section 08 says so; design change note 006 records it.
  Rejected: the PDF only.
- **Emmet writes the PDF itself** (D-55, 2026-11-05; ADR 0014): no PDF crate (a new dependency) and no webview
  print-to-PDF (two native paths that don't paginate alike); the pictures go in as the webview's JPEGs, and the text in
  Helvetica, unembedded. Within plan section 06's options, with no new dependency (rule 6).
- **The page baselines' pixel width is 200** (Barry, 2026-11-05, through AskUserQuestion: "Every page, 200 px
  wide"): every page of both books, at about 6 MB a set per OS. Rejected: the first, 320 px set (15 MB), and a sample
  of pages.
- **The hosted Windows job gets 75 minutes** (Barry, 2026-11-05, through AskUserQuestion: "Raise cap, run once"): run
  42086110115 reached the instructions spec at minute 35 of 45, and its books draw in software on that runner. The job
  runs by hand only while D-43 pauses Windows. Rejected: leaving the Windows baselines to the Windows pass, and rung 2
  only on Windows.
- **Rung 2 is legible printed on A4 and Letter** (Barry, 2026-11-05, through AskUserQuestion, on pages 1, 2, 32, 45,
  and 48 of each: "it is legible on both"). His one change: "the blur around the edges of the image ... makes the part
  look blurry", so a close frame's cut edges are no longer faded (ADR 0014). 4E's Done when 1.

## 2026-11-06, session 4E

- **4E's Windows page baselines wait for the Windows pass** (Barry, 2026-11-06, through AskUserQuestion: "Defer to
  Windows pass"). On the hosted Windows runner, the PDF read-back tests pass through render mode (run 42122932036),
  and the app's Export → PDF saves both books (run 42159781016: the Cottage in 171 s, rung 2 in 88 s, drawn in
  software); the page baselines were never written there, as the spec missed Windows' "Saved" toast (it names the
  folder by its path; fixed in 75661bb). Done when 2b stays unticked for Windows in the PR. Rejected: a fifth manual
  Windows run now.

## 2026-11-06: CI's Mac jobs move to GitHub's hosted Macs

- **The PR's Mac jobs run on GitHub's hosted `macos-15`, not the self-hosted Mac** (Barry, 2026-11-06, through
  AskUserQuestion: "GitHub's Macs"). Of about 32 PR runs on the self-hosted Mac since 2026-10-30, about 17 failed,
  and at least 9 of those were the Mac rather than the code. E2E needs the window on screen, so these all failed a
  run: a locked screen, a sleeping display, the window on another Space, a local `just e2e` driving CI's app over the
  same WebDriver port, and WebKit's 30 fps throttle. The runner is a laptop, too, so a shut lid left runs queued for
  7 to 18 hours. Hosted Macs are free for this public repo, and nothing Barry does on his Mac can fail them. This
  changes D-13 and D-50 for ci.yml only: nightly.yml's corpus job and releases keep the self-hosted Mac.
  `ci-hosted-mac.yml`, the manual fallback, is gone, since ci.yml is now that.
  Rejected: a dedicated Mac mini (the steadiest, but it costs the machine), a macOS VM on this MacBook (a new tool,
  and it still stops with the lid shut), and keeping the laptop runner with only guards.
- **What changes with it.** The checks and tests (`just ci checks`) and the E2E (`just ci e2e`) run as two jobs side
  by side, `mac` and `mac-e2e`, and Windows splits the same way (`windows` and `windows-e2e`). Each builds the
  workspace once: the techniques and timing passes use `--workspace --test it`, since `-p emmet-it` picked other
  features and rebuilt about 60 crates. A failed run saves its Rust cache too, and Windows turns Defender's scan off
  for the build. The E2E window is 1024 × 640 everywhere (a hosted Mac's 1024 × 768 screen fits 1024 × 672 at most,
  which is what D-50's trial hit), and window baselines shrink by the screen's pixel ratio, so one set holds on a
  Retina Mac and a 1x hosted one. Section 13's budgets are enforced only off hosted runners, as before, so they're
  checked by hand on real hardware (`just test`, `just e2e perf`, `just bench`); hosted runs report the numbers.

## 2026-11-08, session 4G

- **A display bike's fork steers, on a triple tree and a pivot** (Barry, 2026-11-08, on batch 19's review page: "the
  handle bars should be able to turn, it needs a what is known as a triple tree, and pivit point"). The first fork
  hung on a horizontal axle, so it could only nod. Now the frame's head is a bearing on the raked steering axis (a
  cross block on the head axle), and `suspension_fork` turns in it: a steerer with upper and lower triple clamps, the
  fork tubes through them, a slider and shock on each tube, and the front wheel. The bars sit on the tubes' tops. The
  fork, brakes, and bars join the frame at the steerer alone, and a test checks it.
- **The seven display-bike techniques are accepted** (Barry, 2026-11-08, through AskUserQuestion: "Accept all 7"):
  motorbike_frame, swingarm_shock, suspension_fork, chain_drive, disc_brake, handlebars, and
  display_stand_nameplate, as built after the steering change (`docs/techniques/candidates/batch-19.html`).
- **The e-bike is a large display model, and the agent has to ask** (Barry, 2026-11-08, through AskUserQuestion:
  "Large display model"). The rung's general scale answer, "Minifigure scale", makes an e-bike about 6 studs long, short
  of its 600-part floor; the e-bike's own answer in `data/ladder/photo/subjects.toml` is now "A large display model on
  a stand, about 40 studs long, with lots of pieces", as the Bradbury has its own. Rejected: keeping "Minifigure
  scale", and no answer.
- **The bike techniques fit the photo, then the e-bike runs again** (Barry, 2026-11-09, through AskUserQuestion: "Fit
  the photo, rerun"). His hand session (New model 8) built by holes with no probe build and a first build at 26 s,
  validated with 0 findings, and stopped at 399 parts; his rubric was proportions 0, recognizability 1, buildability 2.
  The techniques had fixed sizes (one thin motorcycle wheel, one frame and wheelbase, one rake, no bar height), so
  every bike came out the same long, low motorcycle, and the agent read their source to work around them. They get
  size options, the guide has the agent measure the photo first, and Barry runs the e-bike again. Rejected: opening
  the PR with Done when 4 unticked.
- **A display bike stands on its kickstand, not on a base** (Barry, 2026-11-09, before his second e-bike session: "it
  is building a plate for the bike to sit on, this is not needed. a kickstand on the bike is all that is needed to make
  the bike stand up. it is using a lot of the parts to make this and not focusing on the detail of the bike").
  `display_stand_nameplate` is gone, and `kickstand` takes its place: a beam swung down from the frame's left side to
  the table, level with the tires. The large guide says a bike stands on its wheels and kickstand with no base, and
  the parts go into the bike; plan section 21's 4G text says so. The e-bike's scale answer in
  `data/ladder/photo/subjects.toml` says "standing on its kickstand" for "on a stand".
- **New model asks its name and build settings first** (Barry, 2026-11-09: "when creating a new file it does not ask
  for a name or give you and chance to name it. Also I am thinking that the project settings should be be asked as a
  wizard or modal style new file setup, so the expectations of the new project, size, style, etc. are set at the
  beginning, then the project settings are in the small summary at the top of the tweeks sidebar and can be edit
  later if needed"; through AskUserQuestion, it goes in 4G: "Fold into 4G"). This reverses 4A's New with no dialog
  (Barry's note 7) for a new model from a prompt: ⌘N (File → New) and the start screen's card open the New model
  dialog, which needs a name and offers the six build settings on one page, each Any until set, as Options has them
  (Age still brings its Detail and Pieces). Barry, the same day, on a two-page version: "this should not be a two step
  option. Just open the dialog for the new blank model, do not show the welcome screen again. also make all the
  questions on a single step"; so ⌘N no longer shows the start screen once the parts are set up. Create
  makes the folder by that name (a taken one gets "name 2") with `[build]` written. Options shows the settings as a
  line of words with Change on every model, an empty one too. The Cottage example, on first run's start screen, still opens with no dialog.
  Rejected: its own branch and PR, and waiting until 4G merges.
- **The bike techniques get fixed again, then one more e-bike run** (Barry, 2026-11-09, through AskUserQuestion: "Fix
  techniques, rerun"). His second hand session (ebike1) validated with 0 errors and scored proportions 1,
  recognizability 2, buildability 1 (4 of 6), but stopped at 260 parts: the project said Pieces Medium (250–1,000) and
  no scale, so the agent took the range's floor as the target and asked nothing. His notes: "the forks and kickstand
  and rear suspension are not very well designed, they are design to be non-functioning event though there are
  components like a spring, or they porportions are off and could be improved. The over all bike still lacks the
  detail I was expecting for this type of model." In the session he said the fork "is not stable", the axles need
  "something to retain them", and the kickstand goes "of the rear left swing arm"; the agent then dropped the stock
  frame, swingarm, and fork because their axles were flush. So: `suspension_fork` guides the wheel on its tubes and
  the shocks only spring it; every axle end in the bike techniques is held (a bush, a half bush, or a crank); the
  kickstand pins to the left swingarm arm; the rear shock can be one in the middle; bodywork techniques add the detail
  (fenders, a rack, a battery case); and the agent aims at the subject's part range and the upper half of the Pieces
  setting, not its floor. Barry reruns once with Scale Large and Pieces Large. Rejected: rerunning with no fixes, and
  opening the PR with Done when 4 unmet.
- **The ten reworked bike techniques are accepted** (Barry, 2026-11-10, on the review page
  `docs/techniques/candidates/batch-19.html`, through AskUserQuestion: "Accept all 10"). Before the port he saw the
  prototype's renders and said it was the right direction (2026-11-09, "Right direction"). motorbike_frame,
  swingarm_shock, suspension_fork, chain_drive, disc_brake, handlebars, and kickstand, reworked so the wheels spring
  against their shocks and every axle is held, and bike_seat_rack, battery_case, and front_fender, new. Rejected:
  marking them one by one.
- **A new model starts on the base its Base setting names, and the viewer draws a ground grid** (Barry, 2026-11-10,
  starting his third e-bike session: "when I created the parts I said none for the base but it still created a plate
  in the view ... this could be a ground transparent grid instead"; through AskUserQuestion: "Under every model"). The
  New model dialog's Base now picks the starter model: None starts with no parts, on the table at z = 0, and 16 × 16,
  32 × 32, or 48 × 48 starts on that green baseplate; Any keeps the 32 × 32 one, as `emmet new` does. The live viewer
  draws a faint grid on the table under every model, a square a stud with a stronger line every 8 studs, fading out
  with distance, in both themes. Pictures (renders for the agent, exports, instructions, and the camera button) keep
  the plain background with only the shadow. Rejected: a grid only when the model has no baseplate, and only while
  it's empty.
- **Detail never caps the part count; one more e-bike run** (Barry, 2026-11-10, through AskUserQuestion: "Fix
  doctrine, run 4"). His third session (E-bike 4g 3) picked Pieces Medium by mistake ("I made a mistake chosing
  medium") and ended at 261 parts with no findings. The agent had read that a Medium model is 600 to 1,000 parts, and
  said: "That's low for the Medium pieces setting because Medium detail doesn't allow building sideways." So the
  doctrine says Detail is the kind of finish, not the part count; building sideways for the shape is fine at any
  Detail (only sideways textures wait for High); and under the Pieces range's upper half the agent keeps building at
  the Detail it has, never stopping short and blaming Detail. The large guide says the same for its faces and its
  display bikes. Barry runs a fourth session with Scale Large, Detail Medium, and Pieces Large. Rejected: running
  again with no change, and rebuilding the third session's bike with Pieces Large.
- **The build settings are Size and Detail; the part count is the outcome** (Barry, 2026-11-10, after his fourth
  e-bike session, through AskUserQuestion: "Size + Detail"). Run 4 had Scale Large, Detail Medium, and Pieces Large and
  ended at 301 parts, the agent saying it was "well under your Large pieces setting ... mostly because this bike is an
  open frame with little bodywork." Barry: "even with the part count it is did not build it with more pieces, in fact
  the model came out almost identical to the last few ... seems like that piece count is a hard goal with a medium
  detail for object like a bike or a car, where as a large building or a sailing ship or very complex item could meet
  it easier. I think our setting options need to have some realative contraints or becombined into a few options that
  lets the ai know what we are going for with out the conflicting contraints, so may be focus on size and complexity
  and let the piece count be open ended? This would make it simpler for the user also." So Scale and Pieces merge into
  one setting, Size (what the model is sized to: Micro, Midi, Figure scale, Display, or 1:8), Detail stays as the
  complexity, and Age, Style, and Base stay. No setting names a part count: the agent builds to the size and the
  detail and says what the count came to, and still asks before a build over 2,000 parts. Rejected: keeping Scale and
  turning Pieces into a Size by dimensions (the two can still disagree), and keeping Pieces as a soft guide.
- **4G's Done when 4 is the photo rung's pass bar, on a new run** (Barry, 2026-11-10, through AskUserQuestion: "Rubric
  bar, new run"). "600+ parts" set a part count the build settings no longer name. Done when 4 becomes: after the
  settings change, in Barry's hand session, the e-bike at Size Display has 0 validator errors and scores 4 of 6 or
  more (the photo rung's bar, docs/spec/ladder.md), its part count reported, not required. Rejected: scoring run 4 as
  it is, and requiring Detail High.
- **Shift-drag pans the viewer; ⌥-drag draws the selection box** (Barry, 2026-11-10, in his fifth e-bike session: "the
  shift key used to allow you to drag the object into view ... the screen can no longer be repositioned"; through
  AskUserQuestion: "Shift pans, ⌥ boxes"). Since 4C shift-drag had drawn the box that picks parts, which took the
  pan OrbitControls gives a shift-drag. Now shift-drag, ⌘-drag, and a right drag pan, and ⌥-drag (Alt on Windows)
  draws the box. The same run showed a model that opens empty (Base None) framed on the empty table and built off to
  one side: a model that opens empty is now framed when its first parts come, which build up as any run's do.
  Rejected: keeping shift-drag for the box.
