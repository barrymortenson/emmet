# ADR 0012: Agent setup without a terminal

Status: accepted, 2026-10-26 (session 1F)

## Context

D-41: someone who has never opened a terminal installs Claude Code or Codex and signs in from Emmet's first run.
Plan section 10 has Emmet run each provider's official installer, unmodified, only after a click, and Claude Code's
own sign-in in a way that needs no terminal, within Anthropic's terms (`docs/terms/`, re-read 2026-10-26). D-42 added
Antigravity in terminal mode; D-43 took it out of the MVP before 1F merged. Codex's sign-in is 1E's (ADR 0011): the
Codex card's Sign in opens Settings → Agents, where Continue with ChatGPT and the API key live, so the window never has
two copies of them.

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
  searches.
- Neither installer needs admin rights; both refuse or don't need sudo. What they write (the CLI, `~/.claude/downloads`,
  a PATH line in the shell profile) is theirs (D-41). Emmet's own code writes no agent config: the sentinel tests
  (`agent_sentinel.rs`, `setup.e2e.mjs`) hash `.claude`, `.claude.json`, `.codex`, and `.gemini` around install and
  sign-in.
- Detection runs first; an installed CLI isn't installed again.

**Progress and cancel.** Neither installer reports progress ("The install command shows no progress while it
downloads"), so the card shows the elapsed time and the installer's last line. The installer runs in its own process
group on Unix and is ended with `kill -TERM -<group>` (then KILL); on Windows, `taskkill /T /F`. Both are the OS's own
tools, so no new dependency. Killing only the shell would leave curl running and the CLI landing after Cancel; the
cancel test (`tests/it/tests/setup.rs`) catches that, and was seen to fail when the group kill was removed.

Emmet waits for the installer's exit, not for its output to close: a helper the installer started may hold the output
open. Once the installer has exited (or was ended on Cancel or at the time limit), Emmet reads what's left for 2 s and
stops. A Cancel that lands after the installer succeeded doesn't turn the install into a failure. One install or
sign-in runs per agent: a second click (the start screen and the picker each have a card) is told it's already
running, with Cancel, instead of ending the first.

**Failures in plain words.** The installers' own messages (curl's, PowerShell's, and both checksum failures) map to
`Failure::{Offline, Checksum, Failed, Canceled, TimedOut, Signature, Busy}`, each a sentence and one Try again (Busy:
Cancel it, or Look again). A failed installer's own words decide the kind; one that succeeds is a success, since
OpenAI's falls back to GitHub after a bad download. The tool's last line goes under "What the installer said".
Installs stop after 15 minutes.

### Signatures where the provider publishes them

Both installers check SHA-256 against a manifest fetched over the same HTTPS channel. Anthropic also publishes who
signs Claude Code ("signed by \"Anthropic PBC\" and notarized by Apple"; Windows "signed by \"Anthropic, PBC\""), so
after an install Emmet checks the binary it found with the OS's own tool: `codesign --verify --strict` and team
`Q6L2SF6YDW` on macOS, `Get-AuthenticodeSignature` with status `Valid` and that common name on Windows. A copy that
isn't Anthropic's is reported as such ("Don't use it: delete it, then install again"), and Try again checks it again;
a check that couldn't run (PowerShell blocked, say) is a plain failure with the tool's words, never "not signed".
OpenAI publishes no signing identity for Codex (its macOS binary is
Developer ID signed today, team `2DC432GLL2`, but nothing says so), so Codex rests on its installer's checksums.
Verifying the manifest's GPG signature would need an OpenPGP crate, for no more than the platform signature proves.

### Sign-in: `claude auth login`, with no terminal and no new dependency

Anthropic documents `claude auth login` as the sign-in "in your shell before your first session". 1F ran it (2.1.311)
in a throwaway home with stdin closed and no TTY: it printed "Opening browser to sign in…", a fallback link, and
"Paste code here if prompted >", opened the browser itself (`open` on macOS) on a `http://localhost:<port>/callback`
redirect, and listened on 127.0.0.1. So Emmet starts it from Rust, unmodified, waits for it to exit (10 minutes at
most), and detects again. No pseudo-terminal crate is needed.

- Its output isn't streamed to the window, since one line carries the sign-in link; on a failure, Claude Code's last
  line without `://` in it becomes the detail. An exit 0 that `claude auth status` doesn't confirm says so.
- stdin is closed: when the browser can't reach the callback and shows a code to paste, Emmet doesn't take it (Barry
  2026-10-26, docs/decisions.md). The sign-in card says so and offers Open Terminal with the one step: `claude`, then
  `/login`.
- "Use a Claude Console account instead" runs `claude auth login --console`: Emmet offers both of Claude Code's
  browser sign-ins and restricts none (an API key in the user's shell still wins, as before).

### Antigravity: built, then removed before merge (D-43)

Antigravity has no `agy mcp add`; its docs name `~/.gemini/config/mcp_config.json` (global) and
`<workspace>/.agents/mcp_config.json`. 1F built a card that wrote `mcpServers.emmet` into the open project's file
(keeping every other key, refusing a `.agents` that's a link or leads outside the project, and showing it as set up
only when the entry matched), then Barry took Antigravity, Cursor, and Grok out of the MVP and the code was removed
from the PR. It's in the branch's history for when Antigravity comes back.

### The provider notice and the plans

The notice (D-37) is a dialog the window asks for at each Install or Sign in click (Codex's Sign in that opens
Settings included), on the start screen and under the picker, until the user clicks Continue, which records
`provider_notice: true` in `<data>/setup.json`; Not now records nothing. Each card says which plans work, from the provider's page, with a
"Which plans work" link the host opens (no prices).

### Test switches

In debug builds only: `EMMET_FAKE_INSTALL=<fake-install>` replaces the installer; `EMMET_FAKE_SIGNATURE=ok` accepts the
unsigned fakes; `EMMET_DETECT_HOME_ONLY=1` leaves `/opt/homebrew/bin` and `/usr/local/bin` out of detection, where the
CI Mac's own CLIs live.

In every build, as before: `EMMET_CLAUDE_BIN` and `EMMET_CODEX_BIN` name the CLI outright.
They now count only when the file is there (with no further search), so the E2E app can start "not installed" and
fake-install puts the CLI back at that path. fake-claude answers `auth
login` per `claude-login.mode` and records its arguments and whether stdin was closed.

## Consequences

- No new dependency. The window gains seven commands (install, sign-in, cancel, the notice and its answer, plans, and
  the terminal), each in `build.rs` and `capabilities/default.json`.
- The real installers aren't run by tests (they'd download from the providers); the fresh-account runs on macOS and
  Windows (1F's Done when) are where they're proven.
- Detection's fix text no longer names terminal commands; the cards carry the actions.

## Rerun

```sh
cargo test -p emmet-it --test setup --test agent_sentinel
just e2e setup
scripts/terms-check
```
