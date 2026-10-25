# ADR 0001: Agent protocols

Status: accepted, 2026-10-24 (session 0B). Settles Q-05 (from the docs; 1C verifies hands-on), Q-11, and Q-12.

## Context

Emmet drives the user's own Claude Code (`claude -p`, stream-JSON) and Codex (`codex app-server`, JSON-RPC) through
one adapter trait (plan section 10). Before 1C builds the adapters, 0B recorded what both CLIs really do on this Mac:
Claude Code 2.1.310 and Codex 0.160.1. Each scenario was recorded once per CLI (D-38) and is committed, scrubbed, under
`tests/replay/{claude,codex}/` for 1C's fake agents.

## How the recordings were made

- **Spike code:** `tests/spikes/agents/`. `stub-mcp.mjs` is a dependency-free stdio MCP server with `ping`, `approve`
  (Claude's permission prompt answer: allow, except Bash and paths outside the project), and `block` (sleeps for the
  timeout tests); it logs every call. `record-claude.mjs` and `record-codex.mjs` drive one scenario each and keep both
  directions of the protocol with timestamps; `summarize.mjs` prints a transcript one message per line.
- **Flags:** section 10's, exactly: for Claude, `-p --input-format stream-json --output-format stream-json --verbose
  --include-partial-messages --mcp-config <session>/mcp.json --strict-mcp-config --permission-prompt-tool
  mcp__emmet_host__approve --allowedTools … --disallowedTools AskUserQuestion --append-system-prompt …`, with two stub
  instances as `emmet` and `emmet_host`; for Codex, the API-key provider block, `-c mcp_servers.emmet…` and
  `mcp_servers.emmet_host…` (with `tool_timeout_sec=3600`), `sandbox_mode="workspace-write"`,
  `approval_policy="on-request"`, plus `--strict-config`.
- **Credentials (D-37, H-21):** every scripted run used Barry's eval keys from the keychain (`emmet-eval-anthropic`,
  `emmet-eval-openai`) and its own empty HOME, so neither CLI could see a plan sign-in or touch `~/.claude` or
  `~/.codex`. The parent agent's variables (`CLAUDE*`, `CODEX_*`, `ANTHROPIC_*`, `OPENAI_*`, `MCP_*`) were removed.
  Guards: Claude runs started only after `claude auth status --json` reported `authMethod: api_key` /
  `apiKeySource: ANTHROPIC_API_KEY` in that exact environment, and `system/init` was checked again; Codex runs started
  only after `initialize` returned the isolated `codexHome` and `account/read` showed no ChatGPT account.
- **Models:** `--model haiku` (`claude-haiku-5-5`) and `gpt-6-luna` with `effort: "low"`: the cheapest that call tools
  reliably. Protocol behavior doesn't depend on the model.
- **Scrubbing:** `scripts/scrub-transcript` (tested in `tests/scripts/scrub-transcript.test.mjs`); gitleaks is clean
  on `tests/replay/`.

## Claude Code 2.1.310 (stream-JSON)

| Scenario | File | What happened |
|---|---|---|
| Plain turn | `plain-turn.jsonl` | `system/init` (with `apiKeySource`, `model`, `claude_code_version`, `mcp_servers` connected), `system/status`, `stream_event` partials, `assistant`, `result/success` with `total_cost_usd` and `usage` |
| Tool call | `tool-call.jsonl` | MCP tools are deferred: the model first calls the built-in `ToolSearch` (`select:mcp__emmet__ping`), then the tool. `ToolSearch` needs no permission |
| Permission prompt | `permissions.jsonl` | An edit to `model.py` (allowed) runs with no prompt. A read of `../outside.txt` goes to `mcp__emmet_host__approve`, is denied, and the tool result is the denial message. Creating `notes.txt` goes to `approve` and is allowed. **`grep`, `ls`, and `echo hi` ran through Bash without any prompt**: Claude Code auto-allows commands it classes as read-only |
| Ask rule | `permissions-ask-bash.jsonl` | With `--settings '{"permissions":{"ask":["Bash"]}}'`, `ls` and `echo hi` both go to `approve` |
| Interrupt | `interrupt.jsonl` | During `block`, `{"type":"control_request","request_id":…,"request":{"subtype":"interrupt"}}` gets a `control_response` of `success` within 10 ms; the running tool's result becomes a rejection, a `[Request interrupted by user for tool use]` user message follows, and `result/error_during_execution`. Closing stdin then exits with code 1 |
| Interrupt, then a new turn | `interrupt-new-turn.jsonl` | A user message written right after the interrupt starts a normal new turn (`after`) |
| Message mid-turn | `mid-turn-message.jsonl` | A user message written 2 s into a 20 s tool call reaches the model at its next step, in the same turn: the one result includes "banana", and no second turn starts |
| Resume | `resume.jsonl` | A second process with `--resume <session_id>` remembers the first turn ("teal") |
| Tool timeout, default | `block-default.jsonl` | A 600 s tool call completed: the default MCP tool timeout is longer than 10 minutes. `tool_progress` heartbeats arrive every 30 s while it runs |
| Tool timeout, raised | `block-raised.jsonl` | With `MCP_TOOL_TIMEOUT=86400000` (24 h), the 600 s call completed |
| Timeout past 32 bits | `block-overflow.jsonl` | `MCP_TOOL_TIMEOUT=2147483648` (one past the largest 32-bit timer) didn't turn into an instant timeout: a 30 s call completed |

**By hand, on Barry's Max plan** (`tests/spikes/agents/hand-check.mjs`, real HOME, the parent's variables removed;
reduced fixtures in `tests/replay/claude/{usage-report,plan-turn,plan-and-key-turn}.jsonl` with Barry's hooks,
plugins, and usage breakdown removed):

- **`/usage` in a `-p` stream-JSON session** returns an `assistant` message carrying `usage_report`
  (`session` and `rate_limits.limits[]`: `kind`, `group`, `percent`, `resets_at`, `scope`, `severity`, `is_active`,
  plus `extra_usage`) and `local_command_run`, then a `result` with **0 tokens, `total_cost_usd: 0`, `num_turns: 0`**:
  it spends no model tokens. Without `--no-session-persistence` **it saves a session** (one file in
  `~/.claude/projects/<cwd>/`); with it, none. The probe passes `--no-session-persistence`.
- **A turn on the plan:** `system/init` says `apiKeySource: "none"`; `claude auth status --json` says
  `authMethod: "claude.ai"`, `subscriptionType: "max"`. A **`rate_limit_event`** arrives with `rate_limit_info`
  (`status`, `resetsAt`, `rateLimitType`, `overageStatus`, and `unifiedWindows.five_hour` / `seven_day` with
  `utilization`). The result still reports a notional `total_cost_usd`, which isn't money spent on a plan.
- **`-p` uses the plan sign-in without `--bare`**, so this version did not make `--bare` the default (`--help` lists it
  as an opt-in that never reads OAuth or the keychain).
- **Plan sign-in plus `ANTHROPIC_API_KEY`:** the key wins in `-p`. `system/init` says `apiKeySource:
  "ANTHROPIC_API_KEY"`, `auth status` says `authMethod: "api_key"`, and no `rate_limit_event` arrives. So the account
  label comes from `system/init`'s `apiKeySource` ("none" means the plan), and Emmet warns when a key in the user's
  environment will bill instead of their plan (section 10).

## Codex 0.160.1 (app-server JSON-RPC)

The protocol schema is committed at `crates/emmet-agent/schema/codex/0.160.1/` (`codex app-server
generate-json-schema --out …`, stable surface; `--experimental` adds about 60 methods). Messages are JSON-RPC without
a `jsonrpc` field. Relevant methods: `initialize`, `account/read`, `thread/start`, `thread/resume`, `turn/start`,
**`turn/steer`** (`expectedTurnId` precondition), `turn/interrupt`; server requests
`item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, `mcpServer/elicitation/request`,
`item/permissions/requestApproval`.

| Scenario | File | What happened |
|---|---|---|
| Startup | every file | `--strict-config` rejects a misspelled `-c` key ("unknown configuration field … in -c/--config override") and accepts section 10's provider keys. `initialize` returns `codexHome`; `account/read` returns `account: null, requiresOpenaiAuth: false` under the provider block |
| Plain turn | `plain-turn.jsonl` | `thread/start` → `turn/start` → `turn/started`, `item/started`/`item/completed` (userMessage, agentMessage with `phase: final_answer`), `account/rateLimits/updated` (all null on a key), `turn/completed` |
| Tool call | `tool-call.jsonl` | **Each MCP tool call needs approval**, even on `on-request`: an `mcpServer/elicitation/request` with `_meta.codex_approval_kind: "mcp_tool_call"` (and a session-persist option). Accepted, the call runs |
| Permissions, `on-request` | `permissions.jsonl` | With `/tmp` and `$TMPDIR` excluded from the sandbox: an edit to `model.py`, a read of `../outside.txt`, creating `notes.txt`, and `echo hi` all ran with no approval request. The append to `../outside.txt` was blocked by the sandbox, and Codex asked to escalate (`item/commandExecution/requestApproval`), which was declined |
| Permissions, `untrusted` | `permissions-untrusted.jsonl` | **`approval_policy = "untrusted"` in config is refused** ("is no longer supported; remove this setting"), but `thread/start`'s `approvalPolicy: "untrusted"` is accepted: then every command (even `rg` and `head`) and every file change asks |
| Interrupt | `interrupt.jsonl` | `turn/interrupt` during the tool call ends the turn with `status: "interrupted"` |
| Interrupt, then a new turn | `interrupt-new-turn.jsonl` | A `turn/start` right after the interrupt runs normally |
| Steer mid-turn | `mid-turn-steer.jsonl` | `turn/steer` during a 20 s tool call reaches the model at its next step, in the same turn ("…banana") |
| `turn/start` mid-turn | `mid-turn-start.jsonl` | A `turn/start` while a turn runs returns **the running turn's id** and folds into it, like a steer; no second turn |
| Resume | `resume.jsonl` | `thread/resume` in a new process, then a turn, remembers "teal" |
| Tool timeout, default | `block-default.jsonl` | **The default MCP tool timeout is 300 s**: the call failed with "timed out awaiting tools/call after 300s", and the agent reported the error |
| Tool timeout, raised | `block-raised.jsonl` | On `emmet_host`, with `tool_timeout_sec=3600`, the 600 s call completed |
| Bad key | `auth-401.jsonl` | A truncated key: five "Reconnecting… n/5" errors with `httpStatusCode: 401`, `thread/status/changed` to `systemError`, `turn/completed` with `status: "failed"` |
| No credit | `quota-exceeded.jsonl` | An account without credit: `error` with `codexErrorInfo: "usageLimitExceeded"`, `willRetry: false`, then a failed turn |

**By hand, on Barry's ChatGPT login** (`hand-check.mjs codex-account`; no turn): `account/read` returns
`{type: "chatgpt", planType: "prolite", email}` and `requiresOpenaiAuth: true`, and the same with `OPENAI_API_KEY`
set: Codex uses a key only through a provider's `env_key`. So "Codex · API key" comes from the provider block, and
Emmet never starts the app-server on this login (D-35).

## Decisions

- **Redirect paths (D-33, Q-11): native on both.** Claude: write the note to stdin as a stream-JSON user message; it
  arrives at the next step. Codex: `turn/steer` with the active turn's id. The queued path (notes on the next Emmet
  tool result) stays as the fallback when a CLI rejects the native one.
- **Timeouts for `ask_user` (D-32, Q-12):** Claude: set `MCP_TOOL_TIMEOUT=3600000` (1 h) for parity; no cap showed up
  (24 h and 2³¹ ms were accepted; 0B waited 10 minutes, not longer). Codex: keep `tool_timeout_sec=3600` on
  `emmet_host` only, since the default is 300 s. If a call reaches the cap, `ask_user` returns `reason: "timeout"`
  just before it, the agent ends its turn, the card stays open, and the answer goes out as the next turn (section 10).
- **Codex policy: `on-request`** with the sandbox as the guard, per D-40 (Barry, 2026-10-24, after asking what nurb
  does): commands and edits inside the project run without asking, escalations out of the sandbox are cards, and Emmet
  accepts the `mcp_tool_call` approvals for its own two servers. `untrusted` would match the old ask-first table, but
  0.160.1 accepts it only through the protocol and calls the setting unsupported.
- **Claude permissions:** sandbox first per D-40: `--settings` with `sandbox.enabled` and `failIfUnavailable` on macOS,
  the ask rule above on native Windows. The adapter must also handle `ToolSearch` calls (deferred MCP tools) and
  `tool_progress` heartbeats.
- **The usage probe** (D-36): `/usage` with `--no-session-persistence`, no MCP servers, in an empty directory.
- **Account label:** Claude's `system/init` `apiKeySource` ("none" = the plan; a variable name = a key); Codex is
  "Codex · API key" under the provider block.

## No write to the user's agent configs

`tests/spikes/agents/config-snapshot.mjs` saved every file under `~/.claude`, `~/.claude.json`, and `~/.codex` (size
and mtime; credential files are never opened) and the sha256 of the config files (settings, instructions, MCP and
hook config, agents, commands, skills, rules) before the first CLI run (23:52 UTC) and after the last scripted run
(01:29 UTC), before Barry's hand checks. Each run's workspace name was appended to a markers file (never printed).
The diff:

```
config-snapshot: 2026-10-24T23:52:19.315Z → 2026-10-25T01:29:57.911Z
  config files hashed: 277, changed: 0
  other files new or changed: 179, removed: 33 (the user's own live sessions)
    .claude.json: 1
    .claude/.credentials.json: 1
    .claude/.last-cleanup: 1
    .claude/backups: 5
    .claude/cache: 1
    .claude/file-history: 31
    .claude/history.jsonl: 1
    .claude/mcp-needs-auth-cache.json: 1
    .claude/plugins: 16
    .claude/projects: 23
    .claude/security: 7
    .claude/session-env: 3
    .claude/sessions: 4
    .claude/shell-snapshots: 1
    .claude/skills: 2
    .claude/teams: 1
    .codex/cache: 3
    .codex/goals_1.sqlite: 1
    .codex/logs_2.sqlite: 1
    .codex/logs_2.sqlite-shm: 1
    .codex/logs_2.sqlite-wal: 1
    .codex/memories_1.sqlite: 1
    .codex/models_cache.json: 1
    .codex/plugins: 7
    .codex/queue_1.sqlite-shm: 1
    .codex/queue_1.sqlite-wal: 1
    .codex/session_index.jsonl: 1
    .codex/sessions: 2
    .codex/shell_snapshots: 2
    .codex/skills: 49
    .codex/state_5.sqlite: 1
    .codex/thread-writer-locks: 2
    .codex/thread_history_1.sqlite: 1
    .codex/tmp: 4
  content scan skipped under: .claude/projects/-Users-<user>-Sites-emmet/
  markers checked: 33; hits: 0
config-snapshot: ok, no write by the spike
```

The files that changed belong to Barry's own live Claude Code and Codex sessions (history, logs, synced skills with
identical content). The content scan skips the transcript of the session that ran the spikes, which records their
output by design; the path check still covers it. The hand checks ran afterwards, on Barry's real HOME, by design.

## Windows (Q-05, from the current docs; 1C verifies)

- **Claude Code** (https://code.claude.com/docs/en/setup, read 2026-10-24): Windows 10 1809+ or Server 2019+, x64 or
  ARM64, 4 GB+ RAM; install from PowerShell (`irm https://claude.ai/install.ps1 | iex`), CMD, or WinGet
  (`winget install Anthropic.ClaudeCode`), no administrator needed. Git for Windows is optional: with it the Bash tool
  uses Git Bash (`CLAUDE_CODE_GIT_BASH_PATH` if it isn't found); without it Claude Code uses its PowerShell tool. The
  sandbox isn't supported on native Windows (https://code.claude.com/docs/en/sandboxing), only in WSL 2.
- **Codex** (https://learn.chatgpt.com/docs/windows/windows-sandbox, read 2026-10-24): Windows 11 recommended, recent
  Windows 10 best effort, 1809 or newer required; runs natively in PowerShell with its own Windows sandbox, no WSL
  needed; `winget` should be available. The sandbox's `unelevated` mode needs no administrator setup but has weaker
  network isolation and no denied read paths; `elevated` needs administrator-approved setup.

## Run count (D-38)

Scripted, on the eval keys: **Claude 13 runs** (11 scenarios, the ask-rule check, and one retry each of the two
10-minute runs, whose first recordings stopped early on a recorder bug), **Codex 17 runs** (13 scenarios; a first
plain turn that hit the truncated key and a second that hit the empty account, both kept as failure fixtures; one
retry of the permission runs, whose first `on-request` recording was inside the writable `/tmp` and whose first
`untrusted` attempt was refused at startup; and one retry of each 10-minute run, whose first recordings stopped
waiting at 300 s), plus two one-request checks of the OpenAI key. By hand on Barry's plan and login: 4 checks. Every
retry is one per recording, for an unusable recording only. The first recordings are kept locally, not committed.

## Rerun

```sh
# Baseline before any run, then each scenario, then the diff
node tests/spikes/agents/config-snapshot.mjs save /tmp/before.json
for s in plain-turn tool-call permissions permissions-ask-bash interrupt interrupt-new-turn mid-turn-message resume \
         block-default block-raised block-overflow; do SPIKE_RAW_DIR=/tmp/raw node tests/spikes/agents/record-claude.mjs $s; done
for s in plain-turn tool-call permissions interrupt interrupt-new-turn mid-turn-steer mid-turn-start resume \
         block-default block-raised; do SPIKE_RAW_DIR=/tmp/raw node tests/spikes/agents/record-codex.mjs $s; done
SPIKE_RAW_DIR=/tmp/raw node tests/spikes/agents/record-codex.mjs permissions --policy untrusted
node tests/spikes/agents/config-snapshot.mjs save /tmp/after.json
node tests/spikes/agents/config-snapshot.mjs diff /tmp/before.json /tmp/after.json /tmp/raw/markers.txt \
  --exclude-content .claude/projects/<the session running this>/
for f in /tmp/raw/*/*.jsonl; do scripts/scrub-transcript "$f" "tests/replay/$(basename "$(dirname "$f")")/$(basename "$f")"; done
node tests/spikes/agents/summarize.mjs tests/replay/codex/mid-turn-steer.jsonl

# The Codex schema
HOME=$(mktemp -d) codex app-server generate-json-schema --out crates/emmet-agent/schema/codex/$(codex --version | cut -d' ' -f2)

# By hand, on your own sign-ins
node tests/spikes/agents/hand-check.mjs claude-usage   # also: claude-turn, claude-turn-key, codex-account
```
