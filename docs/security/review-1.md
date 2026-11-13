# Security review 1 (session 5B, 2026-11-13)

A walk through every row of plan section 15: the threat, the control in the code, the tests that hold it, and what
they say. Then the Tauri capabilities, the dependencies, and the repo's history. The review read the code on
`bmortenson/s5b-windows-perf-security`; a test named here runs in CI (`tests/it/tests/` in the `mac` and `windows`
jobs, `tests/e2e/specs/` in `mac-e2e` and `windows-e2e`, `tests/scripts/` in `mac` and `plan`).

Out of scope, as SECURITY.md says: someone who already controls the user's account or environment, and what the
agent CLIs do with the user's own sign-in.

## Findings

Seven findings; the high and the two mediums are fixed in this session, each with a test.

| # | Severity | Finding | Status |
|---|---|---|---|
| F-1 | High | The chat started an agent in a project that wasn't trusted. `chat_send` checked nothing, and Claude Code and Codex read the project's own `CLAUDE.md`, `AGENTS.md`, and `.claude/settings.json` (hooks included) as they start, so a shared project could run its own commands on the first message, before the user trusted it. | Fixed: `ensure_live` (`apps/desktop/src-tauri/src/chat.rs`) refuses with `PROJECT_UNTRUSTED`. Test: `trust.e2e.mjs` "opens an untrusted project read-only, and runs nothing" sends a message and finds no conversation or transcript. |
| F-2 | Medium | Secrets were scanned only in the `mac` job, which a prose-only PR skips. A token pasted into a doc would have merged unscanned (GitHub's push protection aside). | Fixed: the `plan` job, which no change skips, runs gitleaks over the full history. Test: `ci.test.mjs` "the plan job, which no change skips, scans the whole history for secrets". |
| F-3 | Medium | rmcp 3.7.0 buffered an HTTP response without limit (RUSTSEC-2026-0462). Emmet uses rmcp over stdio only, so the affected client isn't built in, but the audit gate was red. | Fixed: rmcp 3.9.0. `cargo deny check`: advisories ok. |
| F-4 | Low | The secret behind trust markers (`machine-id`) was written with the default file mode, readable by other accounts on a shared Mac; with it, another account could mark a project trusted for this user. | Fixed: written private (0600), as the bus token is. Test: `errors.rs::projects_trusted_at_once_on_a_fresh_install_stay_trusted`. |
| F-5 | Low | The capabilities test checked that every command was allowed, not that nothing else was; three permission files had no command behind them. | Fixed: `bindings.rs::the_capabilities_allow_nothing_else`, with a short list of core permissions that a change must edit on purpose; the stale files are gone. |
| F-6 | Low | Dev-tool advisories: glob, minimatch, and basic-ftp (moderate) and extract-zip (high) in the trees of WebdriverIO and ESLint. None ships in the app (`pnpm audit --prod` is clean). | glob, minimatch, and basic-ftp are pinned to patched versions (`pnpm-workspace.yaml`). extract-zip has no patched release; it's reached only when WebdriverIO downloads a browser driver, which Emmet's E2E never does (the embedded driver, ADR 0002; install scripts off). Open, accepted. |
| F-7 | Low | A kill (timeout or cancel) ends the Python process, not processes the model started (`run.rs`). A trusted model is the user's own code and can outlive any kill it wants to; this is tidiness, not a boundary. | Accepted; `runner.rs::a_process_the_model_leaves_running_doesnt_hold_up_the_run` holds that such a process can't hang the run. |

Notes that aren't findings:

- **What the approval tool allows without a card** is wider than section 15's one line said ("model-file edits"):
  reads anywhere, and edits to any file inside the project that isn't agent or tool configuration. That is section
  10's policy (D-40, sandbox first), and `host.rs::approve_allows_reads_and_model_edits_and_asks_for_the_rest` pins
  it; section 15's row now says the same. On Windows, where Claude Code has no sandbox, shell commands and web
  requests are cards, so a read can't leave the machine without one.
- **Trust this project** is a button in the window, and the host acts on its command without a second, native
  confirmation. A script running in the window could press it; the controls against a script running there are the
  ones under "Webview compromise" below.
- **On Windows the token and trust-secret files** take their access list from the folder they're in. `%APPDATA%`
  is the user's alone; a `--data-dir` somewhere shared is the user's choice. The pipe itself is the owner's and
  SYSTEM's only, whatever the folder.
- **SECURITY.md** lists signed updates; the updater arrives in 6A.

## Section 15, row by row

| Threat | Control | Tests | Result |
|---|---|---|---|
| A shared project's `model.py` runs code on open | Workspace trust (D-27): `.emmet/trust` must hold sha256 of a secret in app data and the project's canonical path (`project.rs`). Run, params, versions' restore, every panel command, and now the chat refuse without it. | `trust.e2e.mjs` (both tests); `errors.rs::project_untrusted`, `a_marker_copied_from_created_by_doesnt_trust`, `projects_trusted_at_once_on_a_fresh_install_stay_trusted`; `runner.rs::untrusted_projects_run_no_code_for_params_either`; `versions.rs::an_untrusted_projects_versions_cant_be_restored_or_opened`; `paths.rs::a_project_named_in_another_case_is_the_same_project` | Pass. F-1 and F-4 fixed |
| Model code misbehaves (loops, huge output, reads secrets from the environment) | `python -I -B`, cwd the project, a cleared environment (`SYSTEMROOT` and `WINDIR` only), a 30 s timeout with a kill 5 s after, 20,000 parts, a 10 MB log, a kill on cancel (`run.rs`) | `runner.rs::the_model_gets_no_inherited_environment` (sentinel `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GITHUB_TOKEN`, `PYTHONPATH`, and `PYTHONSTARTUP` in the parent; none visible, and a planted `sitecustomize.py` doesn't run), `the_kill_backstop_ends_a_model_that_stops_its_own_watchdog`, `output_past_10_mb_is_cut_and_drained`; `errors.rs::model_timeout`, `model_too_large`; `cancel.rs::closing_stdin_cancels_the_run` | Pass. F-7 accepted |
| Agent does something unexpected | Section 10's policy: Emmet's tools, reads, and edits inside the project (never agent or tool configuration) go without a card, everything else asks; Claude Code's sandbox on macOS and ask rules on Windows; Codex in `workspace-write` with `on-request` | `host.rs::approve_allows_reads_and_model_edits_and_asks_for_the_rest`; `agent_claude.rs::permissions`, `an_approval_card_denied_then_allowed`, `allowing_a_command_for_the_session_doesnt_allow_others`; `agent_codex.rs::permissions_on_request`, `every_approval_kind_answered`, `windows_every_command_asks`; `agent_sentinel.rs` (both) | Pass |
| Webview compromise escalates (a script from a part id, chat markdown, or a model name) | No HTML sink: names render as React text and the chat's markdown builds elements (`markdown.tsx`). CSP: `default-src 'self'; script-src 'self'`, `connect-src` the IPC and `emmet-lib:` only, `object-src`, `base-uri`, `form-action`, and `frame-ancestors` `'none'`. Capabilities name each command; no shell, fs, or dialog permission. The window's parts list comes from the host's own index, not a project's `build/bom.json`. | `csp.e2e.mjs`: a project whose name, part id, placement id, step titles, and findings are `<img onerror>` and `<script>` shows them as words and runs nothing; inline script, an inline handler, a `javascript:` link, `eval`, and a fetch to another host are refused. `chat.e2e.mjs` "a script in an agent's reply, a tool's result, or the user's own message doesn't run". `bindings.rs::every_command_is_allowed`, `the_capabilities_allow_nothing_else` | Pass. F-5 fixed |
| Path traversal through IPC or a project file | Every path from the window is canonicalized and must sit under the project, app data, or the recents list (`paths.rs`, `recent.rs`, `refs.rs`); Emmet never writes through a link a project carries (`fsx::own_file`); `emmet-lib:` looks names up in an index of the library and joins nothing | Desktop `tests/paths.rs` (the library protocol's every way out, a link out, `open_recent` on and off the list); `refs.rs::only_a_photo_in_the_projects_own_refs_resolves`; `versions.rs::a_linked_models_folder_or_source_is_refused`; `instructions.rs::an_export_never_writes_through_a_linked_folder`; `agent_claude.rs` (linked transcripts); `errors.rs::io_error_build_is_a_link`; `viewer.e2e.mjs` (emmet-lib, `open_recent`); `fsx` unit test for `\\?\` and `\\?\UNC\` paths | Pass |
| Malicious library or data download | HTTPS only (plain HTTP to this machine alone, for tests), sha256 pinned per release and checked before unzip, entries that escape the target refused, 1 GiB download and 4 GiB unzipped limits (`library.rs`); parts data packs signed (`parts_data.rs`) | `library.rs::install_checks_the_sha256`, `zip_slip_entry_is_refused` (`..`, absolute, backslash, and drive-letter names), `plain_http_is_refused_except_to_this_machine`; `setup.rs::a_checksum_mismatch_installs_nothing`, `an_unsigned_claude_code_is_refused`; `parts_data.rs::a_bad_pack_is_refused_and_the_installed_one_stays` | Pass |
| Another local process drives the app through the socket | The socket is 0600 on macOS; the Windows pipe's access list names its owner and SYSTEM only. A per-launch token in a private file never crosses the socket: each request and reply proves it with a hash bound to a nonce and the message (`emmet-bus`, `docs/spec/bus.md`) | `bus.rs::a_wrong_token_is_refused`, `an_answer_from_a_process_that_doesnt_know_the_token_is_refused`, `a_request_can_be_sent_once_and_only_as_itself`, `the_socket_is_private` (macOS), `the_pipe_is_private` (Windows, new), `the_token_file_is_private_and_goes_with_the_app`, `the_data_folders_case_doesnt_matter` (new) | Pass |
| Update hijack | Tauri updater signatures and notarization | None yet: there is no updater until 6A, whose release smoke test is this row's test. The one signature check in the code today is the parts data pack's (above) | Not built (6A) |
| Secrets or personal data in the repo | gitleaks before each commit and in CI over the full history, GitHub secret scanning and push protection, transcript scrubbing | `gates.test.mjs` (gitleaks fails on a committed token; the pre-commit hook blocks a staged one); `scrub-transcript.test.mjs`; `ci.test.mjs` (the plan job scans). `gitleaks git` on this branch: 574 commits, no leaks | Pass. F-2 fixed |
| An AI provider's credential leaks | Credentials in the OS keychain only; the window never sees one; every transcript line and event has the session's secrets replaced; Sign in with ChatGPT's state, nonce, PKCE, and ID-token checks (ADR 0011) | `chatgpt.rs::no_token_reaches_any_output`, `a_crash_report_hides_the_token`, `callback_problems_are_refused`, `bad_id_tokens_are_refused`; `agent_codex.rs::the_api_key_comes_from_the_keychain`; `chatgpt.e2e.mjs` (no token in the app's data, the project, or the logs) | Pass. The diagnostics bundle is 6B's |
| Supply chain | Lockfiles committed; `cargo deny` (advisories, licenses, sources); `pnpm audit --prod`; Dependabot; Actions pinned by SHA; the self-hosted runner's guard | `gates.test.mjs` (cargo-deny's allowlist); `ci.test.mjs` (every action pinned by SHA); `runner.test.mjs`. This branch: `cargo deny check`: advisories ok, bans ok, licenses ok, sources ok; `pnpm audit --prod`: no known vulnerabilities | Pass. F-3 fixed, F-6 accepted |

Privacy: photos are saved without metadata (`refs.rs::a_geotagged_photo_is_saved_without_its_metadata`) and go only
to the agent the user picked; the app makes no network call of its own beyond the library and data downloads.

## Tauri capabilities

Two capability files, one window each. Nothing is granted by a wildcard, and no plugin's permissions (fs, shell,
dialog, opener) are granted to any window: the host opens dialogs and starts processes itself.

**`capabilities/render.json`** (render mode's hidden window): `render_job`, `render_done`, `render_failed`,
`render_book_done`, `render_progress`. It reads its one job and hands back pictures.

**`capabilities/default.json`** (the main window): ten core permissions and 71 commands by name.

Core: listen for the host's events and stop (`core:event:allow-listen`, `allow-unlisten`), and work its own title
row: drag, close, minimize, maximize and its double click, read whether it's maximized, and go full screen and back
(`core:window:allow-start-dragging`, `allow-close`, `allow-minimize`, `allow-toggle-maximize`,
`allow-internal-toggle-maximize`, `allow-is-maximized`, `allow-is-fullscreen`, `allow-set-fullscreen`). The two
full-screen permissions are new in 5B, for Windows' F11.

| Area | Commands | What they can reach |
|---|---|---|
| The open project and viewer (`app.rs`) | `viewer_state`, `app_info`, `ui_state`, `open_recent`, `get_params`, `set_params`, `list_colors`, `data_settings`, `set_color_names`, `rebuild`, `bus_reply` | The open project only. `open_recent` takes a path, which must be on the recents list; the rest take values, not paths |
| Projects, versions, exports (`projects.rs`) | `new_project`, `open_dialog`, `trust_project`, `set_build_settings`, `allow_finding`, `unallow_finding`, `keep_params`, `reset_params`, `save_version`, `list_versions`, `version_scene`, `restore_version`, `open_version_as_new`, `export`, `instructions_jobs`, `export_instructions`, `last_book`, `save_screenshot`, `reveal_export` | The open project; new projects go in the projects folder under a cleaned name; exports and screenshots go in Downloads under a cleaned name. Open… is the system's own dialog, opened by the host |
| Chat (`chat.rs`) | `agents`, `chat_send`, `chat_attach`, `chat_photo`, `chat_steer`, `chat_note`, `chat_stop`, `chat_respond`, `chat_answer`, `chat_fresh`, `chat_model`, `chat_restart`, `chat_undo`, `chat_state`, `usage`, `openai_key_saved`, `openai_key_set` | The agent CLI the user picked, in the open, trusted project. Photos resolve inside the project's `refs/` only. `openai_key_set` writes the keychain; nothing reads a key back to the window |
| Sign in with ChatGPT (`chatgpt.rs`) | `chatgpt_state`, `chatgpt_sign_in`, `chatgpt_cancel`, `chatgpt_sign_out`, `chatgpt_choose`, `chatgpt_notice_seen`, `open_chatgpt_usage` | The sign-in flow and the keychain; the window gets the account's state, never a token |
| Settings (`prefs.rs`) | `get_prefs`, `set_prefs`, `tips_seen`, `tip_seen`, `pick_projects_folder` | `settings.json` in app data; the folder comes from the system's dialog |
| Agent setup (`setup.rs`) | `agent_install`, `claude_sign_in`, `agent_cancel`, `provider_notice`, `provider_notice_seen`, `open_terminal`, `open_plans`, `setup` | The providers' own installers from fixed URLs with checksums or signatures checked; fixed pages in the browser |
| Tools (`tools.rs`) | `cli_status`, `install_cli`, `diagnostics`, `report_problem` | The `emmet` command on the user's PATH; a diagnostics text with no secrets |

`bindings.rs::every_command_is_allowed` and `the_capabilities_allow_nothing_else` hold the two lists equal: a
command without a permission, a permission without a command, a core permission off the list above, or any plugin
permission fails the build.

## Dependencies

- Rust: `cargo deny check` on `deny.toml` (advisories with one named exception, RUSTSEC-2024-0436, unmaintained
  `paste`; a ten-license allowlist; no wildcards; crates.io only). Clean after F-3.
- JavaScript: `pnpm audit --prod` clean. The full audit, dev tools included, leaves F-6's extract-zip.
- Python: `emmet_dsl` has no runtime dependencies; uv's lockfile pins the dev tools.
- GitHub Actions: every `uses:` pinned by commit SHA.
- Dependabot: three alerts were open on main for the dev-tool packages in F-6; this branch's lockfile clears them.

## History

`gitleaks git --redact --no-banner .` over all 574 commits on this branch: no leaks found. The same scan now runs in
the `plan` job of every PR.
