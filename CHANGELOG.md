# Changelog

Notable changes to Emmet. Versions follow semver; the minor version bumps per phase, and 1.0.0 is the public launch.

## Unreleased

### Added

- The repo foundation (session 0A): the AGPL-3.0-or-later license (Apache-2.0 for `python/emmet_dsl`), NOTICE,
  TRADEMARKS.md, CONTRIBUTING.md (DCO), SECURITY.md, and the code of conduct.
- The skeleton: four Rust crates, the Tauri 2 desktop app (one window showing the name and version), the viewer package,
  the Python DSL package, and an integration test for each.
- `emmet --version [--json]`.
- The `justfile` (every command in plan section 18), git hooks, and CI on the self-hosted Mac and hosted Windows, with
  SPDX, DCO, gitleaks, cargo-deny, and plan-tracker checks.
- Claude Code commands for plan section 19: `/create-pr` opens the session's PR, sets the session to review, and runs
  the PR review toolkit on it; `/merge` evaluates the session against the plan, sets it to done on the PR branch,
  waits for the required checks, and merges.
- Research spikes and ADRs 0001–0006 (session 0B): how Claude Code and Codex really behave (recorded, scrubbed
  transcripts in `tests/replay/`, the Codex app-server schema in `crates/emmet-agent/schema/codex/0.160.1/`), E2E with
  WebdriverIO and the embedded WebDriver server (`just e2e`), licenses and reuse, LDraw ↔ Rebrickable ↔ BrickLink
  mapping, headless rendering in a hidden window, and two executables on Windows.
- `tests/fixtures/ldraw-mini/` (141 LDraw parts in 177 part files, with aliases, and their attribution, checked in CI) and
  `tests/fixtures/rebrickable-mini/`, with `scripts/make-ldraw-mini` and `scripts/make-rebrickable-mini`.
- `scripts/scrub-transcript`, `scripts/terms-check` (with the provider quotes in `docs/terms/`), and the model ladder
  spec (`docs/spec/ladder.md`), with rung 3 rebuilding 6743-1 Street Speeder.

- The engine walking skeleton (session 1A): an LDraw parser and writer with body boxes (studs left out); the
  library manager (`emmet library status|install|verify`: the pinned 2026-09 `complete.zip`, sha256-checked,
  resumable, with a zip-slip guard); the parts index in SQLite FTS5 with dimension parsing, synonyms
  (`data/synonyms.toml`), and ranking (`emmet search`, `emmet part`, `emmet colors`); emmet_dsl v0 (`@model`,
  `Model.add`, `Model.step`, parameters from the signature, the Z-up stud and plate frame with footprint placement,
  ids with file:line, color names, error codes); the Python bootstrap through uv; `emmet new` (cottage and empty
  templates), `emmet run`, `emmet params`, `emmet trust`, `emmet doctor`; and `emmet mcp` with `search_parts`,
  `part_info`, `run_model`, `get_params`, `set_params`, and `list_colors`, whose schemas are in `docs/spec/mcp/`.
  New dependencies: ureq and toml (ADR 0007).

- The app shell and live viewer (session 1B): `packages/viewer` builds its own three.js scene from placements, each
  unique part loaded once through LDrawLoader and instanced per (part, color) with instanced edges, diff updates by
  Emmet id, highlight, views, frame, offscreen PNG renders, and stats (ADR 0008 confirms D-23). The Tauri host opens,
  creates, and reopens projects, watches sources (150 ms debounce), runs the model through the emmet sidecar (latest
  wins, stale runs canceled, older generations dropped), serves the library over `emmet-lib://` in one request per
  model, and types every command and event through tauri-specta. A plain UI: project tree, viewer, params drawer,
  run panel with the traceback, status line. `emmet render` and MCP `render` and `highlight` go through the open app
  over emmet-bus v0 (`docs/spec/bus.md`), or the app's hidden render mode. `just dev [project]`, `just bench`, and
  E2E specs with visual baselines and a leak test. New dependencies, all named in plan section 06 but
  `@types/three` (approved): three, tauri-specta, notify, interprocess, tauri-plugin-dialog, pixelmatch.

- The agent bridge (session 1C): from the app's chat pane, talk to your own Claude Code or Codex. `emmet-agent` runs
  either CLI (Claude's stream-JSON, Codex's app-server, with its types checked against the committed schema) through one
  shared engine: streaming text, tool-call and file-change cards, approval cards (Allow once, Allow for this session,
  Deny), question cards (`emmet_host ask_user`), redirects while the agent works (delivered natively at its next step,
  through `user_notes` on the next Emmet tool result, or as the next turn), Stop, resume, and crash recovery in one
  click. The agent picker and chat header say who pays ("Claude · Max plan", "Codex · API key"); Claude runs with your
  login shell's environment unchanged, and Codex on the OpenAI API key in your keychain (Settings → Agents). The status
  bar shows Claude's plan usage, fetched only when you ask. Chats and transcripts stay in the project's `.emmet/`.
- `emmet rules` and `emmet api` (the doctrine and the DSL vocabulary, also MCP tools for agents), the hidden
  `emmet mcp-host`, and `emmet mcp --session` for redirect notes. New projects' AGENTS.md points agents at the rules.
- The replay harness: `tests/fake-agents` (fake-claude, fake-codex) replays 0B's transcripts and derived failure cases
  through the real MCP servers; replay, sentinel, and E2E chat tests for both agents. New dependencies (approved,
  ADR 0009): keyring-core with the macOS and Windows stores, and widestring on Windows.

- Closing the loop (session 1D): validate v0 checks collisions (shrunk body boxes as oriented boxes), floating parts
  (stud connections; the stud primitive table starts here), and unknown, obsolete, and alias parts, and writes
  findings with ids, source lines, and hints to `build/validation.json` (`emmet check [--strict] [--rule]`, MCP
  `validate`). bom v0 lists part × color × quantity (`emmet bom [--format json|csv]`, MCP `bom`). MCP `highlight` and
  `emmet render --highlight` take a finding id. `[[check.allow]]` in emmet.toml allows a finding (ADR 0010).
- The app's Checks drawer lists the findings; a click highlights the parts and shows the `model.py` lines that placed
  them, and "Ask agent to fix" sends one to the chat. A Parts drawer lists the BOM. Agent runs build up in the viewer
  part by part (`viewer.update(…, { animate: "build" })`, D-34); sliders stay instant. A Set up button installs the
  library and Python on first run.
- The doctrine's loop is run → validate → render → fix, with a draft in the first minute.
- `just eval --rung 1 --agent claude|codex` (`emmet eval`): one live run through the real CLI on the eval API key,
  scored per section 16.1 and written to `docs/evals/` (D-37, D-38). `--dry-run` runs it with the fake agents in CI.
- `just release-dry [--sign]`: a universal macOS `.dmg` with the emmet and uv sidecars, signed with Developer ID,
  notarized, and stapled with `--sign`; CI's `nsis` job builds the unsigned Windows installer.
- Validator fixtures: `broken-*` projects, an unknown-part `.mpd`, and the five smallest stud-built OMR models
  (`tests/fixtures/omr/`, CC BY 2.0, credited in its ATTRIBUTION.md).

- Codex on your ChatGPT plan (session 1E): Settings → Agents offers Continue with ChatGPT, OpenAI's Sign in with
  ChatGPT for open-source apps, in the system browser. An account picker shows the active account, with sign-out and
  enabling plan use after a decline; Codex runs on the chosen account or on an API key. The chat shows "Using ChatGPT
  plan · Manage usage" by the composer and OpenAI's card for a usage limit, and the status bar shows "Codex · Using
  ChatGPT plan". The token renews between turns (the app-server restarts and resumes the thread). New dependency
  (approved, ADR 0011): ring. OpenAI's ChatGPT logo ships unchanged inside the button (TRADEMARKS.md).
- `fake-openai` (tests/fake-agents) stands in for OpenAI's sign-in endpoints in the tests.
- Agent setup without a terminal (session 1F, ADR 0012): the start screen and the chat picker install Claude Code or
  Codex with one click, running Anthropic's native installer or OpenAI's standalone installer word for word, with the
  elapsed time, Cancel, and one Try again for every failure, said in plain words. Claude Code's Anthropic signature is
  checked after an install. "Sign in to Claude" runs Claude Code's own `claude auth login`, which opens the browser;
  if the browser shows a code to paste, the card offers Open Terminal instead. "Sign in to Codex" opens Settings →
  Agents (1E's sign-in). Each card says which plans work, with the provider's page, and the provider notice comes
  before the first install or sign-in, until the user accepts it.
- `fake-install` (tests/fake-agents) stands in for the installers, and fake-claude answers `auth login`.
- A better starter cottage (`emmet new`, "Start with the Cottage example"): a real door, glass in the windows, a
  stone base, a path and flowers, and a room inside with a bed, a table, and a stove, 163 parts at its new 10 x 8
  default; knobs for the roof (off shows the room), weathering, and Shuffle.
- Design research and three clickable directions (session 2A, `docs/design/`): a teardown of nurb and six brick-CAD
  tools, storyboards of the nine key flows, three static HTML prototypes of every screen and state in light and dark
  (real renders, run and check output, and transcript text), `review.html` comparing them, and a brand starter.
  `scripts/contrast-check.mjs` checks the mockups' colors for WCAG AA, and each prototype's `#audit` checks every
  text element it renders.
- The design system and the frozen screens (session 2B): `apps/desktop/src/styles/tokens.css` (15 colors a theme,
  type, space, radius, elevation, motion with reduced motion, and the macOS and Windows adjustments; the site imports
  it as is), 41 components and 8 regions, on Radix primitives where one exists (`docs/design/components.md`, every state in
  `docs/design/frozen/components.html`), 47 frozen screens and states at 1440 × 900 and 1280 × 800, light and dark,
  macOS and Windows, each listing the components and tokens it uses (`docs/design/frozen/`), the final keyboard map
  and menus, the content and tone guide, the icon set (lucide-react plus Emmet's brick glyphs), and the final app
  icon, which `scripts/app-icon.mjs` turns into every platform size. `docs/design/SIGNOFF.md` freezes it (H-08).
- validate v1 (session 3A, `docs/spec/validator.md`, ADR 0013): a connection model of studs, anti-studs, holes, pins,
  axles, bars, clips, hinges, and plugs from a primitive table, the LDCad Shadow Library (CC BY-SA 4.0, credited in
  NOTICE), and overrides with test models (`data/connections/`), including exact train track and 9V switch ends;
  collisions by collision proxies cached in the parts index; and new rules: disconnected_submodel, step_order,
  step_size (from emmet.toml's `[build] age`), weak_connection, and off_grid, each finding with a concrete hint. The
  MCP `validate` summary lists findings like a compiler. placements.json may carry `allow`, `ground`, and submodel
  `copies`.
- The validator corpus: ten committed OMR models, a nightly job over 33 larger ones
  (`.github/workflows/nightly.yml`, `tests/fixtures/omr/nightly.json`), a seeded mutation generator that reports
  recall per rule, and `just bench validator`.
- emmet_dsl v1 (session 3B, `docs/spec/dsl.md`): `m.wall`, `m.row`, `m.grid`, `m.stack`, `m.fill`, `m.repeat`,
  `m.mirror`, submodels built once and placed many times (`m.submodel`, `m.place`: one `0 FILE` each, ids prefixed by
  the copy, every copy counted), `m.use` with the technique library, `m.allow`, `m.ground`, `import_ldraw`, `Rot` and
  `m.add_raw` for any angle, and the part helpers `Brick`, `Plate`, `Tile`, and `Slope`. With the parts data pack,
  Color params take only colors parts were made in. `emmet params --write-default` rewrites one default in model.py
  and nothing else.
- Steps v1: `build/steps.json` with each step's parts box and sub-assemblies ("× 4"), a layer-a-step fallback for
  parts outside any step, `emmet steps`, and the MCP `steps` tool. The cottage's door and windows are sub-assemblies,
  and every template has a baseplate switch.
- The technique library v0 (`data/techniques/`): 20 techniques, from roofs and walls to track, each with its doc, a
  render, and a test that it validates clean; `emmet technique` and the MCP `technique` tool.
  `data/ladder/rung2_reference.py`: a station beside a loop of track with a siding.
- The parts data pack (plan section 12): `scripts/build-parts-data` turns Rebrickable's catalog into a checked SQLite
  pack with a manifest; the app installs the pack it bundles (sha256-checked, swapped in atomically, the previous one
  kept for rollback); signed packs are checked against a key (a test key until 6A, and none in a release). Availability
  (available, never made in the color, never made, or unknown) feeds the validator's `color_unavailable` and
  `part_unavailable` warnings, `part_info`'s produced colors and alternates, and search's color counts.
- BOM v1: Rebrickable ids and names, availability, the color naming you choose (LDraw, Rebrickable, or BrickLink), and
  exports to CSV, Rebrickable's parts list CSV, and a BrickLink wanted list; `emmet export` writes a flat .ldr, an
  .mpd, and an LPub3D-ready .mpd. The Parts list shows availability and Rebrickable's credit, and Settings has a Data
  section.
- The window to the frozen design (session 4A): the viewer with its title row (the project chip and its menu, the view
  switch), the chat docked along its bottom, and the options panel behind a resizable separator: the model's state,
  Export, and the Options (build settings and knobs with Keep and Reset), Checks (findings in plain words, It's fine,
  Check it again), Parts (each part's picture), and Steps (the model's tree: a step shows the build up to it, a
  sub-assembly highlights every copy) tabs. macOS gets its native menus and Windows a menu bar in the title row; every
  shortcut in the keyboard map that works without an agent works (the chat's own come with 4B). Banners for a read-only project (Trust this project), offline, an old library,
  and an imported file. Settings and About (with the licenses and attributions) are dialogs.
- First run: welcome, the parts library download with its progress (a stopped download picks up where it stopped),
  Python, your agent, and the Cottage or a new model from a prompt. New makes a project in Documents/Emmet with no
  dialog; Open… takes a project's emmet.toml or an LDraw .ldr or .mpd, which becomes an imported project.
- Versions: `emmet version save|list|restore|open`, and in the app Save version (⌘S), the versions list, a version
  opened to compare, Restore (which saves the current state first), and Open as new project.
- `emmet new --import <file>`, Install command line tool (`/usr/local/bin/emmet` on macOS, the user PATH on Windows),
  Copy diagnostics, and `scripts/style-lint.mjs` (no raw colors or pixel sizes outside the tokens).
- Technique mining (session 3C): `tests/it/src/bin/mine-techniques.rs` finds the sub-assemblies official OMR models
  share on the validator's connection graph and ranks ideas by how widely sets use them; techniques aren't derived from
  the models (D-48, ADR 0003). Candidates are reviewed in batches of 20 on `docs/techniques/candidates/<batch>.html`
  (`scripts/technique-candidates`), each a sketch in the DSL with its render. 98 accepted techniques join the library
  (118 in all): houses and town buildings (stone corners, siding, half-timbering, sills, hoods, shutters, dormers,
  chimneys, lift-off roofs and floors), vehicles and trains (headlights, grilles, cabs, hinged hoods and tailgates, a
  hitch, a carriage), landscape and nature, castles and ships, and sci-fi and mechanisms. Each technique has a `PROVENANCE` note, which `emmet technique` and MCP `technique` return.
- Building styles from all of Barry's instruction books (session 3D, D-51): ten readers took methods (never pages)
  from 92 booklets of about 55 sets, and 260 techniques joined the library in thirteen reviewed batches (378 in all):
  house walls and facades, windows, doors, and porches, roofs and chimneys, rooms and furniture, gardens, blocky
  worlds, video-game worlds and retro hardware, vehicles at figure scale (with car wheels at last) and at large scale,
  micro and micropolis, sculptures and brick-built letters, sci-fi rooms and ships, and midi ships and architecture.
- Build guides: one for each scale (micro, micropolis, midi, figure, large, car 1:8), each style (town, medieval,
  blocky, video game, dinosaur park, space, pirate, modern, fantasy, sculpture), and each subject (vehicles, interiors,
  landscaping), in plain rules with the techniques to reach for. `emmet guide` and the MCP `guide` tool give the ones
  for the project's build settings, or one by name, and the doctrine reads them before the first draft.
- Options → Style offers Video game and Sculpture (design change note 002).
- LDraw files saved with a byte order mark (from Windows editors, and in some OMR files) read like any other, in the
  engine and in `import_ldraw`.
- The doctrine says how official sets are built (no joint on a joint, tie the layers, smooth faces on side studs) and
  names every technique under what it's for.

### Security

- emmet-bus authenticates both ways: the token never crosses the socket, each request and reply proves it bound to a
  fresh nonce and the message, and a seen nonce is refused. The Windows pipe is user-only (ADR 0009).
- ChatGPT tokens live only in the OS keychain, and a session's API key or access token is replaced with `<redacted>`
  in its transcript and chat wherever the CLI echoed it; leak tests grep every output for every token (ADR 0011).

### Fixed

- headlights' lenses floated 0.2 studs in front of their bricks, bracket_side_panel hung its panel on the brackets'
  backs, and grass_edge's example cliff stood on nothing; the checks that let them through are stricter now (3D).

- Versions never read or write through a link, and an untrusted project's versions can't be restored or copied out as
  a trusted project. An export's file name can't leave Downloads, whatever the project calls itself (4A).
- The cottage template at an odd width (7, 9, …): the ceiling left one stud unplated, and the roof above it floated.
- Sign in with ChatGPT on Windows: the credential record (three tokens) is longer than the 2,560 bytes a Windows
  credential holds, so the sign-in failed after OpenAI's page. Long secrets are now split across credentials on
  Windows (1F, found in its Windows hand check).

### Changed

- The Cottage template is rebuilt as the MVP's example (3D, Barry's yes): a stone-and-plaster cottage with a bay
  window, a porch, a dormer, an outside chimney, two furnished rooms, a roof that lifts off as one piece, and a garden,
  903 parts. Its eight knobs are season, roof style (tiles, slate, shingles, thatch), wall finish (plaster, brick,
  timber, siding), garden (tidy, wild), lights, roof (lifted off to show the rooms), weathering, and seed; width and
  depth are 14–20 and 10–12. A detailed model may name up to eight knobs (two to five otherwise).
- Technique examples are checked standing on their lowest level, as on a baseplate, so a floating part can't pass as a
  separate toy on the table.
- Search ranks corner shapes ("Corner Round") below plain ones unless the query says "corner".

- `m.wall` lays each course in bond over the joints of the course below that a `m.wall` laid, so no joint sits on a
  joint and the course over a door or a window bridges its edges. The cottage template is rebuilt to it: long bricks
  in bond, interlocking corners, a roof and ceiling whose seams step across, and gable ends in the wall color (147
  parts instead of 165).
- The cottage template splits its walls, ceiling, and roof into steps of 8 parts or fewer.
- The E2E app stays out of the way on the Mac running it (D-50): it doesn't take focus or show in the Dock, and it
  keeps drawing while other windows cover it, so CI's Mac E2E runs while Barry works.
- The Mac app comes first (D-43): CI's Windows jobs are paused (a manual run's "windows" box runs them), and
  Antigravity, Cursor, and Grok wait until after the MVP.
- CI runs only what a change can break (`scripts/ci-scope.sh`): prose skips the OS jobs, the UI skips Windows, a push
  after a green run tests only its own commits, and main doesn't rerun merged PRs. Everything runs weekly, by hand, or
  with the `full-ci` label. `just test` builds only when a suite needs it, and the 0B spike crates left the workspace.
- Agent permissions are sandbox first (D-40), section 10 quotes OpenAI's current wording, and the LDCad shadow
  library's connector data will ship as its own CC BY-SA 4.0 file (Q-01).
- The plan adds setup without a terminal (D-41, session 1F: one-click install of Claude Code or Codex and sign-in
  with no terminal) and Google's agent (D-42: Antigravity in terminal mode in 1F, and in the chat pane in 4F).
  The terms gate (D-37) now builds on Barry's best reading of each provider's published text, with a use-at-your-own-risk
  notice for users, instead of waiting for written confirmation.
- `docs/design/notes.md` holds the design and feature notes from the 1C live check, required reading for 2A. 4A gets
  a default Emmet folder for new projects, 3B a baseplate on/off parameter in every template, 4B nurb-style agent
  switching with Start fresh, and choosing a step hides the later ones (section 13, 4C).
- Body boxes keep underside tubes (a 1 x 1 round plate measured 3 LDU tall) and leave out hand-drawn corner studs
  (door and window frames measured a stud taller than they stack); the parts index rebuilds itself (schema 2).
- The LDraw library pin moves to 2026-10: LDraw replaced `complete.zip` and serves no older one, so fresh installs
  failed the 2026-09 sha256. From 6A, Emmet mirrors each pinned release as a GitHub Release asset (plan section 12).
- The app fills its window: the panes take the height left by the title bar, any notices, and the status bar.
- The plan's rail shows one session under Up next, the first in plan order.
- Checking for the Codex API key no longer reads it, so a rebuilt dev app doesn't ask for the login keychain password
  at launch; the key is read only when a Codex session starts.
