# ADR 0012: Agent setup without a terminal

Status: accepted, 2026-10-26 (session 1F)

## Context

D-41: someone who has never opened a terminal installs Claude Code or Codex and signs in from Emmet's first run.
Plan section 10 has Emmet run each provider's official installer, unmodified, only after a click, and Claude Code's
own sign-in in a way that needs no terminal, within Anthropic's terms (`docs/terms/`, re-read 2026-10-26). D-42 adds
Antigravity in terminal mode. Codex's sign-in is 1E's (ADR 0011).

## Decisions

### Install: the documented command, word for word

`emmet_agent::setup::install` runs, in a shell Emmet starts with the user's own environment (`env::login_env`):

| Agent | macOS | Windows |
|---|---|---|
| Claude Code | `/bin/bash -c "set -o pipefail; curl -fsSL https://claude.ai/install.sh \| bash"` | `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "irm https://claude.ai/install.ps1 \| iex"` |
| Codex | `/bin/bash -c "set -o pipefail; curl -fsSL https://chatgpt.com/codex/install.sh \| sh"` | `powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "irm https://chatgpt.com/codex/install.ps1 \| iex"` |

- `pipefail` is the outer shell's: without it a failed download feeds the installer nothing and looks like success.
  PowerShell comes from `%SystemRoot%\System32` by full path.
- Codex gets `CODEX_NON_INTERACTIVE=1`, the installer's documented switch ("Set to 1, true, or yes to skip prompts"):
  nobody can answer its "Start Codex now?" from Emmet's window.
- OpenAI's standalone installer replaces 1C's npm advice: it needs no Node, checks each archive's SHA-256, and
  installs to `~/.local/bin` (macOS) or `%LOCALAPPDATA%\Programs\OpenAI\Codex\bin` (Windows), which detection now
  searches, with Antigravity's `%LOCALAPPDATA%\agy\bin`.
- Neither installer needs admin rights; both refuse or don't need sudo. What they write (the CLI, `~/.claude/downloads`,
  a PATH line in the shell profile) is theirs (D-41). Emmet's own code writes no agent config: the sentinel tests
  (`agent_sentinel.rs`, `setup.e2e.mjs`) hash `.claude`, `.claude.json`, `.codex`, and `.gemini` around install and
  sign-in.
- Detection runs first; an installed CLI isn't installed again.

**Progress and cancel.** Neither installer reports progress ("The install command shows no progress while it
downloads"), so the card shows the elapsed time and the installer's last line. The installer runs in its own process
group on Unix and is ended with `kill -TERM -<group>` (then KILL); on Windows, `taskkill /T /F`. Both are the OS's own
tools, so no new dependency. Killing only the shell would leave curl running and the CLI landing after Cancel; the
cancel test (`setup.rs`) catches that, and was seen to fail when the group kill was removed.

**Failures in plain words.** The installers' own messages (curl's, PowerShell's, and both checksum failures) map to
`Failure::{Offline, Checksum, Failed, Canceled, TimedOut, Signature}`, each a sentence and one Try again. The tool's
last line goes under "What the installer said". Installs stop after 15 minutes.

### Signatures where the provider publishes them

Both installers check SHA-256 against a manifest fetched over the same HTTPS channel. Anthropic also publishes who
signs Claude Code ("signed by \"Anthropic PBC\" and notarized by Apple"; Windows "signed by \"Anthropic, PBC\""), so
after an install Emmet checks the binary it found with the OS's own tool: `codesign --verify --strict` and team
`Q6L2SF6YDW` on macOS, `Get-AuthenticodeSignature` with status `Valid` and that common name on Windows. A failure says
the copy isn't Anthropic's and Emmet won't run it. OpenAI publishes no signing identity for Codex (its macOS binary is
Developer ID signed today, team `2DC432GLL2`, but nothing says so), so Codex rests on its installer's checksums.
Verifying the manifest's GPG signature would need an OpenPGP crate, for no more than the platform signature proves.

### Sign-in: `claude auth login`, with no terminal and no new dependency

Anthropic documents `claude auth login` as the sign-in "in your shell before your first session". 1F ran it (2.1.311)
in a throwaway home with stdin closed and no TTY: it printed "Opening browser to sign in…", a fallback link, and
"Paste code here if prompted >", opened the browser itself (`open` on macOS) on a `http://localhost:<port>/callback`
redirect, and listened on 127.0.0.1. So Emmet starts it from Rust, unmodified, waits for it to exit (10 minutes at
most), and detects again. No pseudo-terminal crate is needed.

- Emmet never reads, shows, or keeps its output beyond telling how it ended: one line carries the sign-in link. A
  failure's detail is Claude Code's own words, never a line with `://` in it.
- stdin is closed: when the browser can't reach the callback and shows a code to paste, Emmet doesn't take it (Barry
  2026-10-26, docs/decisions.md). The sign-in card says so and offers Open Terminal with the one step: `claude`, then
  `/login`.
- "Use a Claude Console account instead" runs `claude auth login --console`: Emmet offers both of Claude Code's
  browser sign-ins and restricts none (an API key in the user's shell still wins, as before).

### Antigravity: the project's `.agents/mcp_config.json`

Antigravity has no `agy mcp add`; its docs name `~/.gemini/config/mcp_config.json` (global) and
`<workspace>/.agents/mcp_config.json`. "Add Emmet's tools to this project" writes `mcpServers.emmet` =
`{"command": <the emmet sidecar>, "args": ["mcp", "--project", <root>]}` into the open project's file, keeping every
other key; a file that isn't a JSON object is left alone (Barry 2026-10-26). Emmet never writes `~/.gemini`. The card
opens Terminal (or PowerShell) in the project folder, where the user types `agy`. agy has no status command, so its
sign-in is its own first run. H-25 checks that agy loads the file.

### The provider notice and the plans

The notice (D-37) is a dialog before the first Install or Sign in, from either card, recorded as
`provider_notice: true` in `<data>/setup.json`. Each card says which plans work, from the provider's page, with a
"Which plans work" link the host opens (no prices).

### Test switches (debug builds only)

`EMMET_FAKE_INSTALL=<fake-install>` replaces the installer; `EMMET_FAKE_SIGNATURE=ok` accepts the unsigned fakes;
`EMMET_DETECT_HOME_ONLY=1` leaves `/opt/homebrew/bin` and `/usr/local/bin` out of detection, where the CI Mac's own
CLIs live. `EMMET_CLAUDE_BIN` and `EMMET_CODEX_BIN` now count only when the file is there (with no further search), so
the E2E app can start "not installed" and fake-install puts the CLI back at that path. fake-claude answers `auth
login` per `claude-login.mode` and records its arguments and whether stdin was closed.

## Consequences

- No new dependency. The window gains nine commands (install, sign-in, cancel, the notice, plans, terminal,
  Antigravity), each in `build.rs` and `capabilities/default.json`.
- The real installers aren't run by tests (they'd download from the providers); the fresh-account runs on macOS and
  Windows (1F's Done when) are where they're proven.
- Detection's fix text no longer names terminal commands; the cards carry the actions.

## Rerun

```sh
cargo test -p emmet-it --test setup --test agent_sentinel
just e2e setup
scripts/terms-check
```
