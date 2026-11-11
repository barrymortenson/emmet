# The test matrix

Every behavior the plan names, and the test that proves it (session 5A). The sections follow `docs/plan.html`: 14
(the window, its flows, states, and keys), 09 (the CLI, the MCP tools, the error codes), 11 (the validator and its
rules), and 15 (the security and privacy controls). To find a test, find the behavior's row, open the file, and
search for the test's name.

`tests/scripts/test-matrix.test.mjs` keeps this file honest: every row names at least one test, each file exists
and holds a test of that name, and CI runs it. A test CI runs only by hand (the E2E extra tier, an `extraIt`, an
ignored Rust test other than the nightly corpus) can't be a row's proof.

Where a test runs: Rust tests under `tests/it/tests/` and `apps/desktop/src-tauri/tests/`, and the viewer's, run in
`just test` on macOS and Windows; the scripts' tests on macOS and in the Linux plan job; an E2E spec on macOS, and on
Windows when `tests/e2e/tiers.mjs` lists it for Windows. While D-43 holds, the Windows jobs run only by hand, so
"Windows" below means "when the Windows jobs run".

## 14 · UI and UX: the window

| Behavior | Test | Runs on |
|---|---|---|
| Title row: the project menu's New, Open, Save version, Export, and Settings open their surfaces | `tests/e2e/specs/keyboard.e2e.mjs` › `⌘S, ⌘E, ⌘,, ⌘/, and ⌘N open their surfaces, and Esc closes each`<br>`tests/e2e/specs/keyboard.e2e.mjs` › `⌘O opens what the file picker returns` | macOS |
| Title row: the view switch (Iso, Front, Top, Back, Left, Right) and Frame | `tests/e2e/specs/interact.e2e.mjs` › `F frames the pick, the views turn to each side, and the viewer's keys orbit and zoom only when it has focus` | macOS |
| Title row: Screenshot, to the clipboard or a file | `tests/e2e/specs/interact.e2e.mjs` › `Screenshot puts a picture of the view on the clipboard, and the toast saves it to a file` | macOS |
| Title row on Windows: the menu bar's menus | `tests/e2e/specs/keyboard.e2e.mjs` › `Windows: the title row's menus open, with the same actions` | macOS |
| Title row: a double click maximizes the window | `tests/e2e/specs/keyboard.e2e.mjs` › `a double click on the title row maximizes the window, and another restores it` | macOS |
| Viewer: draws every part of the model | `tests/e2e/specs/viewer.e2e.mjs` › `opens the fixture project and draws every part` | macOS, Windows |
| Viewer: never moves the camera on its own (law 1) | `tests/e2e/specs/viewer.e2e.mjs` › `a wider cottage changes the count in place and keeps the camera`<br>`tests/e2e/specs/photo.e2e.mjs` › `opens a sent photo beside the model without moving the camera` | macOS, Windows |
| Viewer: shows the last good model while a new one builds or fails (law 1) | `tests/e2e/specs/runs.e2e.mjs` › `shows what happened and keeps the last good model`<br>`tests/e2e/specs/viewer.e2e.mjs` › `deleting build/ keeps the model on screen, and the next change rebuilds it` | macOS, Windows |
| Viewer: hovering names the part and its color | `tests/e2e/specs/interact.e2e.mjs` › `hovering a part names it and its color, and leaving clears it` | macOS |
| Viewer: click, shift-click, and box select | `tests/e2e/specs/interact.e2e.mjs` › `a click picks a part, shift-click adds another, and a click on nothing clears`<br>`tests/e2e/specs/interact.e2e.mjs` › `⌥-drag draws a box and picks the parts whose centers are in it, without turning the camera` | macOS |
| Viewer: a finding's pin | `tests/e2e/specs/interact.e2e.mjs` › `a finding's pin sits at its parts and opens it in Checks`<br>`tests/e2e/specs/interact.e2e.mjs` › `findings of one rule share a pin, and a step's size gets none` | macOS |
| Viewer: a step's parts box and sub-assembly box | `tests/e2e/specs/interact.e2e.mjs` › `a step shows the build up to it, outlines its new parts, and draws its parts box`<br>`tests/e2e/specs/interact.e2e.mjs` › `a step that puts on a sub-assembly draws its box with numbered sub-steps and a leader to the copies` | macOS |
| Viewer: a sub-assembly shown on its own, with its frame and bar (note 008) | `tests/e2e/specs/panel.e2e.mjs` › `a sub-step shows its sub-assembly on its own, built up to it, and the model comes back: ${name}` | macOS |
| Viewer: the layer cut-away | `tests/e2e/specs/interact.e2e.mjs` › `the cut-away hides what's above a layer, its bar lowers and raises it, and Show all brings it back` | macOS |
| Viewer: toasts say what was written and where | `tests/e2e/specs/panel.e2e.mjs` › `exports the parts list into Downloads, and says so` | macOS |
| Viewer: updates from the project folder (an edit outside the app) | `tests/e2e/specs/viewer.e2e.mjs` › `an external edit to model.py updates the viewer in place`<br>`apps/desktop/src-tauri/tests/watcher.rs` › `only_sources_trigger_a_run`<br>`apps/desktop/src-tauri/tests/watcher.rs` › `a_burst_of_changes_runs_once_after_it_goes_quiet` | macOS, Windows |
| Versions: Save version names one, with its picture | `tests/e2e/specs/versions.e2e.mjs` › `saves a named version with its thumbnail`<br>`tests/it/tests/versions.rs` › `save_change_restore_gives_the_same_bytes` | macOS, Windows |
| Versions: a version opens read-only to compare, and Restore saves the current state first | `tests/e2e/specs/versions.e2e.mjs` › `opens the version to compare, and Restore gives back the same bytes after saving the state before` | macOS, Windows |
| Versions: Open as new project | `tests/e2e/specs/versions.e2e.mjs` › `opens a version as a new project`<br>`tests/it/tests/versions.rs` › `open_as_new_project_is_trusted_and_builds` | macOS, Windows |
| Versions: an automatic version before an agent's turn, and Undo turn | `tests/e2e/specs/chat.e2e.mjs` › `builds, says so in words, and Undo turn puts the sources back byte for byte`<br>`tests/it/tests/versions.rs` › `a_turn_that_changed_nothing_is_told_apart_and_only_emmets_own_version_goes` | macOS, Windows |
| Chat: the agent picker and who pays (the plan badge) | `tests/e2e/specs/chat.e2e.mjs` › `shows the agent and who pays`<br>`tests/it/tests/agent_claude.rs` › `billing_on_a_plan_with_live_limits` | macOS, Windows |
| Chat: plan usage meters and the ChatGPT plan line | `tests/e2e/specs/dock.e2e.mjs` › `the agent picker, the switch, the model and effort, and the plan usage`<br>`tests/e2e/specs/chatgpt.e2e.mjs` › `says Using ChatGPT plan by the composer and in the status bar`<br>`tests/it/tests/agent_claude.rs` › `usage_report_meters` | macOS, Windows |
| Chat: Start fresh and switching agents keep the old conversation | `tests/e2e/specs/chat.e2e.mjs` › `starts a fresh conversation and keeps the old one` | macOS, Windows |
| Chat: tool cards say what happened in plain words (law 3) | `tests/e2e/specs/chat.e2e.mjs` › `builds rung 1 from a live run, and the chat says what each step did` | macOS, Windows |
| Chat: the model and effort picker | `tests/it/tests/agent_claude.rs` › `the_model_and_effort_reach_the_cli`<br>`tests/it/tests/agent_codex.rs` › `the_model_and_effort_reach_the_cli` | macOS, Windows |
| Chat: attach a photo (pick, paste, drop), and the note on where it goes | `tests/e2e/specs/photo.e2e.mjs` › `attaches a dropped and a pasted photo, says once where they go, and sends them cleaned` | macOS |
| Chat: the compare view | `tests/e2e/specs/photo.e2e.mjs` › `opens a sent photo beside the model without moving the camera` | macOS |
| Chat: a turn cut off by Emmet closing says so | `tests/e2e/specs/chat.e2e.mjs` › `says so when a turn was cut off by Emmet closing` | macOS, Windows |
| Options panel: the model's state in a word | `tests/e2e/specs/checks.e2e.mjs` › `lists the collision the validator found, in plain words`<br>`tests/e2e/specs/runs.e2e.mjs` › `shows what happened and keeps the last good model` | macOS, Windows |
| Options tab: build settings wait for Rebuild | `tests/e2e/specs/panel.e2e.mjs` › `holds a build setting change until Rebuild, which writes emmet.toml` | macOS |
| Options tab: a knob is a draft until Keep; Shuffle | `tests/e2e/specs/panel.e2e.mjs` › `keeps a moved knob as a draft until Keep writes it into model.py`<br>`tests/it/tests/run.rs` › `the_knobs_change_the_build_and_a_seed_reproduces_it` | macOS, Windows |
| Checks tab: findings in plain words, click to highlight | `tests/e2e/specs/checks.e2e.mjs` › `lists the collision the validator found, in plain words`<br>`tests/e2e/specs/checks.e2e.mjs` › `clicking the finding highlights its parts, and again clears them` | macOS |
| Checks tab: It's fine, and Check it again | `tests/e2e/specs/panel.e2e.mjs` › `says a finding in plain words, and It's fine allows it until Check it again` | macOS |
| Parts tab: the list with pictures, colors, and counts | `tests/e2e/specs/checks.e2e.mjs` › `the Parts tab lists each part and color with its count`<br>`tests/e2e/specs/panel.e2e.mjs` › `lists the parts with their pictures, most first`<br>`tests/e2e/specs/viewer.e2e.mjs` › `clicking a part in the Parts list highlights it` | macOS, Windows |
| Steps tab: the model's tree; a step shows the build up to it; a sub-assembly lights every copy | `tests/e2e/specs/panel.e2e.mjs` › `shows the build up to a step, and every copy of a sub-assembly` | macOS |
| Settings: the parts data and color names | `tests/e2e/specs/checks.e2e.mjs` › `Settings says what the parts data is, and changes the color names` | macOS |

## 14 · Key flows

| Behavior | Test | Runs on |
|---|---|---|
| First run: welcome → library download → Python → agents → the Cottage or a prompt | `tests/e2e/specs/first-run.e2e.mjs` › `welcomes, with no project and nothing set up`<br>`tests/e2e/specs/first-run.e2e.mjs` › `sets the agents up without a terminal, then offers the Cottage or a prompt`<br>`tests/e2e/specs/first-run.e2e.mjs` › `builds the Cottage in Documents/Emmet with no dialog` | macOS |
| First run: a dropped download picks up where it stopped | `tests/e2e/specs/first-run.e2e.mjs` › `says where a dropped download stopped, and Try again picks up there`<br>`tests/it/tests/library.rs` › `interrupted_download_resumes` | macOS, Windows |
| First run: agents installed and signed in without a terminal | `tests/e2e/specs/setup.e2e.mjs` › `installs Claude Code, and a failed sign-in is retried`<br>`tests/e2e/specs/setup.e2e.mjs` › `installs Codex`<br>`tests/it/tests/setup.rs` › `claude_installs_and_signs_in_without_a_terminal` | macOS, Windows |
| First run: Codex on a ChatGPT plan (Continue with ChatGPT) | `tests/e2e/specs/chatgpt.e2e.mjs` › `invites an API-key user, signs in, and shows the notice once`<br>`tests/it/tests/chatgpt.rs` › `first_sign_in_registers_emmet` | macOS, Windows |
| New model from a prompt: the dialog asks its name and build settings | `tests/e2e/specs/first-run.e2e.mjs` › `makes a new model from a prompt with ⌘N: one dialog asks its name and build settings first`<br>`tests/it/tests/run.rs` › `a_new_model_starts_on_the_base_it_was_given` | macOS, Windows |
| Tweak with knobs: the viewer updates in place; Keep writes it | `tests/e2e/specs/panel.e2e.mjs` › `keeps a moved knob as a draft until Keep writes it into model.py`<br>`tests/e2e/specs/viewer.e2e.mjs` › `two parameter changes at once both land` | macOS, Windows |
| Fix a finding: click → highlight → Ask agent to fix → the fix shows | `tests/e2e/specs/checks.e2e.mjs` › `asking the agent to fix it sends the finding in plain words, and its details to the agent`<br>`tests/e2e/specs/checks.e2e.mjs` › `fixing the model clears the finding and its highlight`<br>`tests/it/tests/agent_claude.rs` › `a_collision_finding_is_fixed_at_its_line` | macOS, Windows |
| Export: the parts list as CSV, Rebrickable, BrickLink XML | `tests/e2e/specs/panel.e2e.mjs` › `exports the parts list into Downloads, and says so`<br>`tests/it/tests/bom.rs` › `exports_without_the_pack` | macOS, Windows |
| Export: the model as LDraw | `tests/e2e/specs/states.e2e.mjs` › `exports the model as an LDraw file into Downloads, and says so`<br>`tests/it/tests/export.rs` › `every_format_places_the_parts_the_build_does` | macOS, Windows |
| Export: instructions as a PDF, or an LPub3D-ready file | `tests/e2e/specs/instructions.e2e.mjs` › `exports ${rung} as a PDF into Downloads, and every page matches its baseline`<br>`tests/it/tests/export.rs` › `lpub3d_puts_a_sub_assemblys_copies_in_one_callout` | macOS, Windows |
| Open an existing LDraw file: it becomes an imported project | `tests/e2e/specs/import.e2e.mjs` › `imports an .ldr as a project with its steps and no knobs`<br>`tests/it/tests/import.rs` › `an_imported_ldr_builds_with_its_steps` | macOS, Windows |
| Open a project from its emmet.toml | `tests/e2e/specs/import.e2e.mjs` › `opens a project from its emmet.toml` | macOS, Windows |
| Switch agent mid-project: a new session, a divider, the old one kept | `tests/e2e/specs/chat.e2e.mjs` › `starts a fresh conversation and keeps the old one` | macOS, Windows |
| Answer a question: click, a number key, or Other; the agent carries on in the same turn | `tests/e2e/specs/chat.e2e.mjs` › `answers a question with Other, in the same turn`<br>`tests/it/tests/agent_claude.rs` › `a_question_answered_in_the_same_turn`<br>`tests/it/tests/agent_codex.rs` › `a_question_answered_in_the_same_turn` | macOS, Windows |
| Watch: the model grows brick by brick | `tests/e2e/specs/animate.e2e.mjs` › `an edit's run builds up part by part, and Skip ends it at once` | macOS |
| Redirect: a message typed mid-turn reaches the agent at its next step | `tests/e2e/specs/chat.e2e.mjs` › `delivers a redirect sent mid-turn`<br>`tests/it/tests/agent_claude.rs` › `a_redirect_delivered_through_user_notes`<br>`tests/it/tests/agent_codex.rs` › `a_redirect_delivered_natively_mid_turn` | macOS, Windows |
| Stop: the turn ends at once, and a pending message goes out as the next turn | `tests/e2e/specs/chat.e2e.mjs` › `stops the turn at once`<br>`tests/it/tests/agent_claude.rs` › `a_redirect_still_pending_at_stop`<br>`tests/it/tests/agent_claude.rs` › `a_redirect_sent_as_the_next_turn` | macOS, Windows |

## 14 · States

| Behavior | Test | Runs on |
|---|---|---|
| Empty | `tests/e2e/specs/hello.e2e.mjs` › `starts at Start with no project` | macOS |
| First run | `tests/e2e/specs/first-run.e2e.mjs` › `welcomes, with no project and nothing set up` | macOS |
| Loading the library | `tests/e2e/specs/first-run.e2e.mjs` › `says where a dropped download stopped, and Try again picks up there` | macOS |
| Building | `tests/e2e/specs/runs.e2e.mjs` › `shows what happened and keeps the last good model` | macOS, Windows |
| Build failed, in plain words, with its line for the agent | `tests/e2e/specs/viewer.e2e.mjs` › `a broken model.py says the model didn't build, with its line for the agent, and the last good model stays`<br>`tests/e2e/specs/viewer.e2e.mjs` › `fixing model.py recovers` | macOS, Windows |
| Checks with findings | `tests/e2e/specs/checks.e2e.mjs` › `lists the collision the validator found, in plain words` | macOS |
| Agent thinking and building | `tests/e2e/specs/dock.e2e.mjs` › `thinking, building, a redirect, and built` | macOS |
| Waiting for approval | `tests/e2e/specs/chat.e2e.mjs` › `denies an approval, and the agent carries on`<br>`tests/e2e/specs/dock.e2e.mjs` › `an approval stays over the composer when a step follows it, and under a redirect` | macOS, Windows |
| Question waiting for an answer | `tests/e2e/specs/dock.e2e.mjs` › `a question waiting for an answer`<br>`tests/e2e/specs/chat.e2e.mjs` › `answers a question with Other, in the same turn` | macOS, Windows |
| Redirect pending | `tests/e2e/specs/dock.e2e.mjs` › `thinking, building, a redirect, and built`<br>`tests/e2e/specs/chat.e2e.mjs` › `delivers a redirect sent mid-turn` | macOS, Windows |
| Stopped by the user | `tests/e2e/specs/chat.e2e.mjs` › `stops the turn at once`<br>`tests/e2e/specs/dock.e2e.mjs` › `stopped, and crashed` | macOS, Windows |
| Build animating, with Skip | `tests/e2e/specs/animate.e2e.mjs` › `an edit's run builds up part by part, and Skip ends it at once`<br>`tests/e2e/specs/animate.e2e.mjs` › `Replay build plays the whole model from empty, and a render mid-way shows the finished model` | macOS |
| Agent not installed or not signed in | `tests/e2e/specs/setup.e2e.mjs` › `installs Claude Code, and a failed sign-in is retried`<br>`tests/e2e/specs/dock.e2e.mjs` › `a build that failed, and an agent that isn't signed in`<br>`tests/it/tests/agent_claude.rs` › `detection_reads_the_version_and_sign_in` | macOS, Windows |
| Agent crashed: one-click restart, project untouched | `tests/e2e/specs/chat.e2e.mjs` › `restarts a crashed agent with one click, on the same conversation`<br>`tests/it/tests/agent_claude.rs` › `a_crash_mid_stream_then_resume`<br>`tests/it/tests/agent_codex.rs` › `a_crash_mid_stream_then_resume` | macOS, Windows |
| Plan limit reached | `tests/e2e/specs/dock.e2e.mjs` › `the plan limits: Claude's, and ChatGPT's for Codex`<br>`tests/e2e/specs/chatgpt.e2e.mjs` › `shows the usage-limit card with Manage usage first`<br>`tests/it/tests/agent_codex.rs` › `plan_usage_limit` | macOS, Windows |
| Offline: everything except the agent and downloads works | `tests/e2e/specs/states.e2e.mjs` › `offline says so, and a knob still rebuilds the model`<br>`tests/it/tests/setup.rs` › `offline_says_so_and_the_retry_works` | macOS, Windows |
| Untrusted project: read-only, with Trust | `tests/e2e/specs/trust.e2e.mjs` › `opens an untrusted project read-only, and runs nothing`<br>`tests/e2e/specs/trust.e2e.mjs` › `trusts it, builds it, and unlocks it` | macOS, Windows |
| An old library version | `tests/e2e/specs/states.e2e.mjs` › `an old library says which release it is, and Update the library picks up where a dropped download stopped` | macOS |

## 14 · Keyboard

| Behavior | Test | Runs on |
|---|---|---|
| ⌘N, ⌘S, ⌘E, ⌘,, ⌘/ and Esc to close | `tests/e2e/specs/keyboard.e2e.mjs` › `⌘S, ⌘E, ⌘,, ⌘/, and ⌘N open their surfaces, and Esc closes each` | macOS |
| ⌘O open | `tests/e2e/specs/keyboard.e2e.mjs` › `⌘O opens what the file picker returns` | macOS |
| ↩ sends, ⌘L the message box, Esc stops the agent | `tests/e2e/specs/chat.e2e.mjs` › `stops the turn at once` | macOS, Windows |
| ⌥⌘Z undo the agent's last turn | `tests/e2e/specs/chat.e2e.mjs` › `builds, says so in words, and Undo turn puts the sources back byte for byte` | macOS, Windows |
| F frame, 1–6 views | `tests/e2e/specs/keyboard.e2e.mjs` › `1–6 pick the views, and F frames what's selected` | macOS |
| [ and ] step | `tests/e2e/specs/keyboard.e2e.mjs` › `[ and ] step through the steps, opening Steps` | macOS |
| ⌘1–⌘4 panel tabs | `tests/e2e/specs/keyboard.e2e.mjs` › `⌘1–⌘4 pick the panel's tabs` | macOS |
| F6 next region | `tests/e2e/specs/keyboard.e2e.mjs` › `F6 moves between the viewer, the chat, and the panel` | macOS |
| ⌘Z and ⇧⌘Z on a knob's draft | `tests/e2e/specs/keyboard.e2e.mjs` › `⌘Z and ⇧⌘Z undo and redo a knob's draft` | macOS |
| A key that types doesn't fire in a text field | `tests/e2e/specs/keyboard.e2e.mjs` › `a key that types a character doesn't fire in a text field` | macOS |

## 14 · Content, tone, accessibility

| Behavior | Test | Runs on |
|---|---|---|
| Plain words, no code on screen (law 2) | `tests/e2e/specs/viewer.e2e.mjs` › `a broken model.py says the model didn't build, with its line for the agent, and the last good model stays`<br>`tests/e2e/specs/checks.e2e.mjs` › `asking the agent to fix it sends the finding in plain words, and its details to the agent` | macOS, Windows |
| One tip at a time, taught once | `tests/e2e/specs/dock.e2e.mjs` › `teaches the first knob once, and then never again (tips, kept per user)` | macOS |
| WCAG 2.2 AA contrast in both themes | `tests/scripts/contrast-check.test.mjs` › `every text token is checked against every surface, and UI tokens at 3:1`<br>`tests/scripts/contrast-check.test.mjs` › `the command fails on a bad stylesheet and passes docs/design as it stands` | macOS, Linux |
| Full keyboard reach and visible focus | `tests/e2e/specs/keyboard.e2e.mjs` › `every control Tab reaches shows a visible focus ring` | macOS |
| Reduced motion: no build animation | `tests/e2e/specs/animate.e2e.mjs` › `reduced motion turns the build animation off`<br>`tests/scripts/design.test.mjs` › `tokens.css keeps to 15 colors, defines both themes alike, and turns motion off when asked` | macOS, Linux |
| Screen reader labels on every control | `tests/e2e/specs/states.e2e.mjs` › `every control has a name a screen reader can say: the window, each panel tab, Export, and Settings` | macOS |
| Only the design system's tokens and components | `tests/scripts/style-lint.test.mjs` › `the app's styles use only tokens`<br>`tests/scripts/design.test.mjs` › `components.css takes every size and duration from tokens.css` | macOS, Linux |

## 09 · The envelope and exit codes

| Behavior | Test | Runs on |
|---|---|---|
| The success envelope (`ok`, `data`, `warnings`, `meta`) | `tests/it/tests/cli.rs` › `version_json_is_the_success_envelope`<br>`tests/it/tests/run.rs` › `new_then_run_matches_the_golden` | macOS, Windows |
| The failure envelope, with the user's file and line | `tests/it/tests/errors.rs` › `model_exec_failed_name_error`<br>`tests/it/tests/runner.rs` › `errors_point_into_the_project_never_at_python_internals` | macOS, Windows |
| Findings aren't errors; `--strict` exits 1 on error findings only | `tests/it/tests/check.rs` › `strict_exits_1_on_errors_only` | macOS, Windows |
| Exit 2: a usage error, and PROJECT_NOT_FOUND | `tests/it/tests/cli.rs` › `no_command_is_a_usage_error`<br>`tests/it/tests/errors.rs` › `project_not_found` | macOS, Windows |
| Exit 3: a runtime error | `tests/it/tests/errors.rs` › `model_exec_failed_syntax_error` | macOS, Windows |
| Exit 4: the environment (library, Python) | `tests/it/tests/errors.rs` › `library_missing`<br>`tests/it/tests/errors.rs` › `python_missing` | macOS, Windows |
| Reproducible: a clean build/ regenerates the same bytes | `tests/it/tests/run.rs` › `clean_rebuild_is_byte_identical`<br>`tests/it/tests/check.rs` › `validation_json_is_reproducible` | macOS, Windows |
| Global flags `--project`, `--json`, `--data-dir` | `tests/it/tests/paths.rs` › `a_project_path_with_spaces_and_accents`<br>`tests/it/tests/run.rs` › `new_then_run_matches_the_golden` | macOS, Windows |

## 09 · Error codes

| Behavior | Test | Runs on |
|---|---|---|
| PROJECT_NOT_FOUND | `tests/it/tests/errors.rs` › `project_not_found` | macOS, Windows |
| PROJECT_UNTRUSTED | `tests/it/tests/errors.rs` › `project_untrusted`<br>`tests/it/tests/errors.rs` › `a_marker_copied_from_created_by_doesnt_trust` | macOS, Windows |
| MODEL_EXEC_FAILED | `tests/it/tests/errors.rs` › `model_exec_failed_syntax_error`<br>`tests/it/tests/errors.rs` › `model_exec_failed_name_error`<br>`tests/it/tests/errors.rs` › `model_exec_failed_in_a_submodule` | macOS, Windows |
| MODEL_TIMEOUT | `tests/it/tests/errors.rs` › `model_timeout`<br>`tests/it/tests/runner.rs` › `the_kill_backstop_ends_a_model_that_stops_its_own_watchdog` | macOS, Windows |
| MODEL_TOO_LARGE | `tests/it/tests/errors.rs` › `model_too_large`<br>`tests/it/tests/dsl.rs` › `a_submodel_counts_toward_the_part_limit_once` | macOS, Windows |
| UNKNOWN_PART | `tests/it/tests/errors.rs` › `unknown_part`<br>`tests/it/tests/errors.rs` › `unknown_part_from_part_info` | macOS, Windows |
| UNKNOWN_COLOR | `tests/it/tests/errors.rs` › `unknown_color` | macOS, Windows |
| PARAM_INVALID | `tests/it/tests/errors.rs` › `param_invalid_out_of_range`<br>`tests/it/tests/errors.rs` › `param_invalid_unknown_name_and_bad_json` | macOS, Windows |
| LIBRARY_MISSING | `tests/it/tests/errors.rs` › `library_missing` | macOS, Windows |
| LIBRARY_CORRUPT | `tests/it/tests/errors.rs` › `library_corrupt`<br>`tests/it/tests/library.rs` › `a_damaged_install_is_corrupt_not_missing` | macOS, Windows |
| PYTHON_MISSING | `tests/it/tests/errors.rs` › `python_missing` | macOS, Windows |
| RENDER_UNAVAILABLE | `tests/it/tests/render.rs` › `a_stale_bus_file_falls_back_to_render_mode` | macOS, Windows |
| IO_ERROR | `tests/it/tests/errors.rs` › `io_error_build_is_a_link`<br>`tests/it/tests/errors.rs` › `io_error_new_into_a_full_folder` | macOS, Windows |
| INTERNAL | `tests/it/tests/errors.rs` › `internal_a_build_file_emmet_cant_read` | macOS, Windows |

## 09 · CLI commands

| Behavior | Test | Runs on |
|---|---|---|
| `emmet new <dir> [--template cottage|empty]` | `tests/it/tests/run.rs` › `new_then_run_matches_the_golden`<br>`tests/it/tests/run.rs` › `a_new_model_starts_on_the_base_it_was_given`<br>`tests/it/tests/errors.rs` › `io_error_new_into_a_full_folder` | macOS, Windows |
| `emmet run [--params k=v …]` | `tests/it/tests/run.rs` › `new_then_run_matches_the_golden`<br>`tests/it/tests/run.rs` › `params_get_set_and_run_overrides` | macOS, Windows |
| `emmet check [--strict] [--rule r] [--params k=v …]` | `tests/it/tests/check.rs` › `each_broken_project_gets_exactly_its_findings`<br>`tests/it/tests/check.rs` › `strict_exits_1_on_errors_only`<br>`tests/it/tests/check.rs` › `check_with_params_leaves_params_json`<br>`tests/it/tests/check.rs` › `check_runs_a_stale_model_first` | macOS, Windows |
| `emmet bom [--format …] [--color-names …]` | `tests/it/tests/bom.rs` › `bom_with_the_pack`<br>`tests/it/tests/bom.rs` › `exports_without_the_pack`<br>`tests/it/tests/bom.rs` › `color_names_follow_the_users_naming` | macOS, Windows |
| `emmet steps` | `tests/it/tests/dsl.rs` › `loose_parts_get_steps_a_layer_at_a_time`<br>`tests/it/tests/dsl.rs` › `a_submodel_placed_four_times` | macOS, Windows |
| `emmet render [--view] [--highlight ids|F-id] [--size] [--out]` | `tests/it/tests/render.rs` › `render_without_the_app_uses_render_mode`<br>`tests/it/tests/render.rs` › `render_runs_a_model_that_is_out_of_date_first`<br>`tests/e2e/specs/render.e2e.mjs` › `emmet render renders through the open window` | macOS, Windows |
| `emmet search <query> [--category c] [--limit n]` | `tests/it/tests/search.rs` › `golden_queries`<br>`tests/it/tests/search.rs` › `a_category_narrows_the_search` | macOS, Windows |
| `emmet part <id>` and `emmet colors [--part id]` | `tests/it/tests/search.rs` › `part_and_colors_through_the_cli`<br>`tests/it/tests/parts_data.rs` › `part_info_colors_and_search_use_the_pack` | macOS, Windows |
| `emmet params [--set k=v] [--write-default k]` | `tests/it/tests/run.rs` › `params_get_set_and_run_overrides`<br>`tests/it/tests/dsl.rs` › `write_default_rewrites_one_literal` | macOS, Windows |
| `emmet technique [name]` | `tests/it/tests/techniques.rs` › `every_technique_has_a_test` | macOS, Windows |
| `emmet rules` | `tests/it/tests/techniques.rs` › `the_doctrine_or_a_guide_names_every_technique` | macOS, Windows |
| `emmet api` | `tests/it/tests/dsl.rs` › `every_api_call_is_documented_and_used` | macOS, Windows |
| `emmet guide [name]` | `tests/it/tests/guides.rs` › `with_no_name_it_gives_the_guides_for_the_build_settings`<br>`tests/it/tests/guides.rs` › `a_guide_answers_to_its_other_names` | macOS, Windows |
| `emmet export --format ldr|mpd|lpub3d` | `tests/it/tests/export.rs` › `every_format_places_the_parts_the_build_does`<br>`tests/it/tests/export.rs` › `lpub3d_puts_a_sub_assemblys_copies_in_one_callout` | macOS, Windows |
| `emmet export --format pdf [--paper a4|letter]` | `tests/it/tests/instructions.rs` › `the_rung_2_station_matches_its_steps`<br>`tests/it/tests/instructions.rs` › `the_cottage_exports_in_under_30_seconds_and_matches_its_steps` | macOS, Windows |
| `emmet version save|list|restore|open` | `tests/it/tests/versions.rs` › `save_change_restore_gives_the_same_bytes`<br>`tests/it/tests/versions.rs` › `a_missing_version_or_name_says_so`<br>`tests/it/tests/versions.rs` › `open_as_new_project_is_trusted_and_builds` | macOS, Windows |
| `emmet library status|install` | `tests/it/tests/library.rs` › `install_downloads_checks_and_indexes`<br>`tests/it/tests/library.rs` › `install_checks_the_sha256` | macOS, Windows |
| `emmet library update` | `tests/it/tests/library.rs` › `update_installs_another_release_and_then_has_nothing_to_do` | macOS, Windows |
| `emmet library verify` | `tests/it/tests/library.rs` › `verify_detects_a_changed_file_and_reindexes` | macOS, Windows |
| `emmet doctor [--fix]` | `tests/it/tests/python_setup.rs` › `doctor_fix_recovers_from_a_broken_python_folder`<br>`tests/it/tests/library.rs` › `a_damaged_install_is_corrupt_not_missing` | macOS, Windows |
| `emmet trust` | `tests/it/tests/errors.rs` › `projects_trusted_at_once_on_a_fresh_install_stay_trusted`<br>`tests/it/tests/errors.rs` › `a_marker_copied_from_created_by_doesnt_trust` | macOS, Windows |
| `emmet mcp [--session id]` | `tests/it/tests/mcp.rs` › `lists_exactly_the_tools_built_so_far`<br>`tests/it/tests/mcp.rs` › `a_session_tool_result_carries_the_users_notes` | macOS, Windows |
| `emmet mcp-host --session id` | `tests/it/tests/host.rs` › `approve_allows_reads_and_model_edits_and_asks_for_the_rest`<br>`tests/it/tests/host.rs` › `ask_user_needs_two_to_four_options_and_says_why_there_is_no_answer` | macOS, Windows |
| `emmet eval --rung … --agent … --dry-run` | `tests/it/tests/eval.rs` › `a_dry_run_scores_rung_1_with_each_fake_agent`<br>`tests/it/tests/eval.rs` › `a_dry_run_scores_the_photo_rung_with_each_fake_agent` | macOS, Windows |
| `emmet eval --score <project>`: a hand session of rung 2, scored on its track, station, and knobs | `tests/it/tests/eval.rs` › `a_hand_session_of_rung_2_is_scored_on_its_track_its_station_and_its_knobs` | macOS, Windows |
| `emmet eval --score <project>`: a hand session of rung 3, scored against the set (BOM F1, placement match) | `tests/it/tests/eval.rs` › `a_hand_session_of_rung_3_is_scored_against_the_set` | macOS, Windows |
| `emmet eval --score <project>`: a hand session of the photo rung, with the rubric | `tests/it/tests/eval.rs` › `scoring_a_hand_session_reads_its_chat_and_checks_the_photos` | macOS, Windows |
| `emmet eval`: a rung that isn't on the ladder is a usage error | `tests/it/tests/eval.rs` › `a_rung_that_isnt_on_the_ladder_is_a_usage_error` | macOS, Windows |

## 09 · MCP tools

| Behavior | Test | Runs on |
|---|---|---|
| The tool list, and each tool's schema | `tests/it/tests/mcp.rs` › `lists_exactly_the_tools_built_so_far`<br>`tests/it/tests/mcp.rs` › `mcp_schemas_are_fresh`<br>`tests/it/tests/mcp.rs` › `the_schema_validator_catches_mistakes` | macOS, Windows |
| `search_parts` | `tests/it/tests/mcp.rs` › `search_parts` | macOS, Windows |
| `part_info` | `tests/it/tests/mcp.rs` › `part_info`<br>`tests/it/tests/technic.rs` › `part_info_s_holes_match_the_parts_ldraw_geometry` | macOS, Windows |
| `run_model`, and its error with file and line | `tests/it/tests/mcp.rs` › `run_model`<br>`tests/it/tests/mcp.rs` › `run_model_error_has_file_and_line` | macOS, Windows |
| `validate` | `tests/it/tests/mcp.rs` › `validate_tool` | macOS, Windows |
| `render` | `tests/it/tests/mcp.rs` › `render`<br>`tests/e2e/specs/render.e2e.mjs` › `MCP render returns image content from the open window` | macOS, Windows |
| `bom` | `tests/it/tests/mcp.rs` › `bom_tool` | macOS, Windows |
| `steps` | `tests/it/tests/mcp.rs` › `steps_tool` | macOS, Windows |
| `save_version` | `tests/it/tests/mcp.rs` › `save_version_tool` | macOS, Windows |
| `get_params` and `set_params` | `tests/it/tests/mcp.rs` › `get_params`<br>`tests/it/tests/mcp.rs` › `set_params` | macOS, Windows |
| `technique` | `tests/it/tests/mcp.rs` › `technique_tool` | macOS, Windows |
| `guide` | `tests/it/tests/mcp.rs` › `guide_tool` | macOS, Windows |
| `list_colors` | `tests/it/tests/mcp.rs` › `list_colors` | macOS, Windows |
| `highlight` | `tests/it/tests/mcp.rs` › `highlight`<br>`tests/it/tests/mcp.rs` › `highlight_a_finding`<br>`tests/e2e/specs/render.e2e.mjs` › `MCP highlight reaches the open viewer`<br>`tests/e2e/specs/bus.e2e.mjs` › `highlights parts in the app's viewer and says who is pointing` | macOS, Windows |
| `camera` and `selection` | `tests/it/tests/mcp.rs` › `camera_and_selection`<br>`tests/e2e/specs/bus.e2e.mjs` › `turns the view and reads it back`<br>`tests/e2e/specs/bus.e2e.mjs` › `reads what the user picked` | macOS, Windows |
| `rules` and `api` | `tests/it/tests/mcp.rs` › `rules_and_api_return_the_doctrine_and_the_dsl` | macOS, Windows |
| `user_notes` on a session's tool results | `tests/it/tests/mcp.rs` › `a_session_tool_result_carries_the_users_notes` | macOS, Windows |
| `emmet_host` `approve` | `tests/it/tests/host.rs` › `approve_allows_reads_and_model_edits_and_asks_for_the_rest` | macOS, Windows |
| `emmet_host` `ask_user` | `tests/it/tests/host.rs` › `ask_user_needs_two_to_four_options_and_says_why_there_is_no_answer` | macOS, Windows |

## 11 · Validator: the three numbers

| Behavior | Test | Runs on |
|---|---|---|
| Precision: zero error findings on the official-model corpus | `tests/it/tests/validator_corpus.rs` › `the_omr_fixtures_have_no_errors`<br>`tests/it/tests/validator_corpus.rs` › `corpus_nightly` | macOS, Windows |
| Recall: injected faults are caught (≥ 95 % overall, ≥ 90 % per error rule) | `tests/it/tests/validator_mutations.rs` › `injected_faults_are_caught`<br>`tests/it/tests/validator_mutations.rs` › `figure_faults_are_caught` | macOS, Windows |
| Speed: ≤ 300 ms for 500 parts, ≤ 1.5 s for 2,000 | `tests/it/tests/validator_bench.rs` › `validator_bench` | macOS, Windows |
| Connector sources: primitives, the shadow library, and overrides | `tests/it/tests/validator_corpus.rs` › `each_override_holds_in_its_test_model`<br>`tests/it/tests/validator_corpus.rs` › `track_joins_exactly` | macOS, Windows |
| Technic points: a pin or axle in its hole | `tests/it/tests/technic.rs` › `a_beam_on_a_5_12_13_triangle_seats_a_pin_at_each_end`<br>`tests/it/tests/technic.rs` › `every_technic_beam_brick_pin_and_axle_has_points` | macOS, Windows |
| Finding ids are stable, and build/validation.json is reproducible | `tests/it/tests/check.rs` › `validation_json_is_reproducible` | macOS, Windows |
| `m.allow` and `m.ground` | `tests/it/tests/check.rs` › `an_allowed_finding_moves_to_allowed`<br>`tests/it/tests/check.rs` › `the_models_allow_and_ground_count` | macOS, Windows |

## 11 · Rules

| Behavior | Test | Runs on |
|---|---|---|
| `collision` | `tests/it/tests/check.rs` › `each_broken_project_gets_exactly_its_findings`<br>`tests/it/tests/technic.rs` › `a_wheel_on_an_axle_through_its_hub_checks_clean` | macOS, Windows |
| `floating` | `tests/it/tests/check.rs` › `each_broken_project_gets_exactly_its_findings`<br>`tests/it/tests/validator_mutations.rs` › `injected_faults_are_caught` | macOS, Windows |
| `disconnected_submodel` | `tests/it/tests/validator_corpus.rs` › `a_submodel_placed_off_its_model_is_disconnected` | macOS, Windows |
| `step_order` | `tests/it/tests/check.rs` › `each_broken_project_gets_exactly_its_findings`<br>`tests/it/tests/technic.rs` › `a_part_slid_onto_an_axle_after_both_its_ends_are_shut_is_out_of_order` | macOS, Windows |
| `step_size` | `tests/it/tests/check.rs` › `each_broken_project_gets_exactly_its_findings`<br>`tests/it/tests/check.rs` › `the_age_setting_sets_the_step_target` | macOS, Windows |
| `weak_connection` | `tests/it/tests/check.rs` › `each_broken_project_gets_exactly_its_findings` | macOS, Windows |
| `off_grid` | `tests/it/tests/check.rs` › `each_broken_project_gets_exactly_its_findings` | macOS, Windows |
| `figure_clearance` | `tests/it/tests/check.rs` › `each_broken_project_gets_exactly_its_findings`<br>`tests/it/tests/check.rs` › `figure_clearance_is_off_where_no_figure_fits`<br>`tests/it/tests/techniques.rs` › `figure_clearance_leads_the_warnings_and_can_be_allowed` | macOS, Windows |
| `unknown_part` | `tests/it/tests/validator_corpus.rs` › `a_part_the_library_lacks_is_unknown`<br>`tests/it/tests/errors.rs` › `unknown_part` | macOS, Windows |
| `obsolete_part` and `alias_part` | `tests/it/tests/check.rs` › `each_broken_project_gets_exactly_its_findings` | macOS, Windows |
| `part_unavailable` and `color_unavailable` | `tests/it/tests/check.rs` › `broken_color_unavailable_needs_the_pack`<br>`tests/it/tests/parts_data.rs` › `availability_is_honest` | macOS, Windows |
| A pin that misses its hole names the hole | `tests/it/tests/technic.rs` › `a_pin_out_of_its_hole_is_a_finding_that_names_the_hole` | macOS, Windows |

## 15 · Security and privacy controls

| Behavior | Test | Runs on |
|---|---|---|
| Workspace trust: an untrusted project opens read-only and runs nothing | `tests/e2e/specs/trust.e2e.mjs` › `opens an untrusted project read-only, and runs nothing`<br>`tests/it/tests/errors.rs` › `project_untrusted`<br>`tests/it/tests/errors.rs` › `a_marker_copied_from_created_by_doesnt_trust`<br>`tests/it/tests/runner.rs` › `untrusted_projects_run_no_code_for_params_either` | macOS, Windows |
| Model code: no inherited environment | `tests/it/tests/runner.rs` › `the_model_gets_no_inherited_environment` | macOS, Windows |
| Model code: the 30 s timeout and the kill on cancel | `tests/it/tests/errors.rs` › `model_timeout`<br>`tests/it/tests/cancel.rs` › `closing_stdin_cancels_the_run`<br>`tests/it/tests/runner.rs` › `the_kill_backstop_ends_a_model_that_stops_its_own_watchdog` | macOS, Windows |
| Model code: output caps (20,000 parts, 10 MB of log) | `tests/it/tests/errors.rs` › `model_too_large`<br>`tests/it/tests/runner.rs` › `output_past_10_mb_is_cut_and_drained` | macOS, Windows |
| The agent: only Emmet's tools and model-file edits are pre-allowed; the rest asks | `tests/it/tests/host.rs` › `approve_allows_reads_and_model_edits_and_asks_for_the_rest`<br>`tests/it/tests/agent_claude.rs` › `an_approval_card_denied_then_allowed`<br>`tests/it/tests/agent_claude.rs` › `allowing_a_command_for_the_session_doesnt_allow_others`<br>`tests/it/tests/agent_codex.rs` › `every_approval_kind_answered` | macOS, Windows |
| The agent: Emmet never writes a user-level agent config | `tests/it/tests/agent_sentinel.rs` › `user_level_agent_configs_are_untouched`<br>`tests/it/tests/agent_sentinel.rs` › `setup_writes_no_agent_config`<br>`tests/e2e/specs/chat.e2e.mjs` › `leaves the user's agent configs untouched` | macOS, Windows |
| The webview: every command is in the capabilities, and no more | `apps/desktop/src-tauri/tests/bindings.rs` › `every_command_is_allowed` | macOS, Windows |
| Paths: the library protocol refuses every way out | `apps/desktop/src-tauri/tests/paths.rs` › `the_library_protocol_refuses_every_way_out`<br>`apps/desktop/src-tauri/tests/paths.rs` › `the_library_protocol_refuses_a_link_out_of_the_library`<br>`tests/e2e/specs/viewer.e2e.mjs` › `emmet-lib serves the library and refuses paths outside it` | macOS, Windows |
| Paths: a project opens only from the recent list or a picker | `apps/desktop/src-tauri/tests/paths.rs` › `open_recent_opens_only_a_project_on_the_list`<br>`apps/desktop/src-tauri/tests/paths.rs` › `open_recent_refuses_a_link_to_an_unlisted_folder`<br>`tests/e2e/specs/viewer.e2e.mjs` › `open_recent refuses a folder that isn't on the list` | macOS, Windows |
| Paths: links out of the project are refused | `tests/it/tests/versions.rs` › `a_linked_models_folder_or_source_is_refused`<br>`tests/it/tests/instructions.rs` › `an_export_never_writes_through_a_linked_folder`<br>`tests/it/tests/agent_claude.rs` › `a_linked_transcripts_folder_isnt_written_through`<br>`tests/it/tests/refs.rs` › `only_a_photo_in_the_projects_own_refs_resolves` | macOS, Windows |
| Downloads: HTTPS only, sha256 pinned, zip-slip guarded | `tests/it/tests/library.rs` › `plain_http_is_refused_except_to_this_machine`<br>`tests/it/tests/library.rs` › `install_checks_the_sha256`<br>`tests/it/tests/library.rs` › `zip_slip_entry_is_refused`<br>`tests/it/tests/setup.rs` › `a_checksum_mismatch_installs_nothing`<br>`tests/it/tests/parts_data.rs` › `a_bad_pack_is_refused_and_the_installed_one_stays` | macOS, Windows |
| The socket: user-only, with a per-launch token that never crosses it | `tests/it/tests/bus.rs` › `the_socket_is_private`<br>`tests/it/tests/bus.rs` › `a_wrong_token_is_refused`<br>`tests/it/tests/bus.rs` › `an_answer_from_a_process_that_doesnt_know_the_token_is_refused`<br>`tests/it/tests/bus.rs` › `a_request_can_be_sent_once_and_only_as_itself`<br>`tests/it/tests/bus.rs` › `the_token_file_is_private_and_goes_with_the_app` | macOS, Windows |
| Secrets in the repo: gitleaks, and scrubbed transcripts | `tests/scripts/gates.test.mjs` › `gitleaks fails on a committed token`<br>`tests/scripts/gates.test.mjs` › `the pre-commit hook blocks a staged token`<br>`tests/scripts/scrub-transcript.test.mjs` › `every committed replay transcript is under 200 KB and already scrubbed` | macOS, Linux |
| An AI provider's credential never reaches a file, a log, a transcript, or the chat | `tests/it/tests/chatgpt.rs` › `no_token_reaches_any_output`<br>`tests/it/tests/chatgpt.rs` › `a_crash_report_hides_the_token`<br>`tests/e2e/specs/chatgpt.e2e.mjs` › `leaves no token in the app's data, the project, or the logs`<br>`tests/it/tests/agent_codex.rs` › `the_api_key_comes_from_the_keychain` | macOS, Windows |
| Sign in with ChatGPT: state, nonce, PKCE, and ID-token checks | `tests/it/tests/chatgpt.rs` › `callback_problems_are_refused`<br>`tests/it/tests/chatgpt.rs` › `bad_id_tokens_are_refused` | macOS, Windows |
| Supply chain: the license allowlist, actions pinned by SHA, the runner guard | `tests/scripts/gates.test.mjs` › `cargo deny rejects a dependency outside the license allowlist`<br>`tests/scripts/ci.test.mjs` › `every action in every workflow is pinned by a full commit SHA`<br>`tests/scripts/runner.test.mjs` › `runner-guard refuses ${why}` | macOS, Linux |
| Privacy: a photo is saved and sent without its metadata | `tests/it/tests/refs.rs` › `a_geotagged_photo_is_saved_without_its_metadata`<br>`tests/it/tests/agent_claude.rs` › `a_photo_reaches_claude_code_without_its_metadata`<br>`tests/it/tests/agent_claude.rs` › `a_photo_that_still_has_metadata_is_refused` | macOS, Windows |

## Waiting for another session

Controls the plan gives to a later session, which don't exist yet. The check fails once that session is done in
`docs/plan-status.js` and its row is still here: it moves into a table above, with its test.

| Behavior | Session |
|---|---|
| 15 · The webview's CSP: a script in a part name, chat markdown, or a model name doesn't run | 5B |
| 15 · The capabilities audit, written up | 5B |
| 15 · Update hijack: the updater's signature check | 6A |
| 15 · A provider's credential never reaches a diagnostics bundle | 6B |
