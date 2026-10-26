# ADR 0009: The agent bridge as built

Status: accepted, 2026-10-25 (session 1C)

## Context

Session 1C builds plan section 10's agent bridge: the `emmet-agent` engine and its Claude Code and Codex protocols,
`emmet mcp-host`, redirect notes in `emmet mcp --session`, the chat pane, and the replay harness. A few things the
plan didn't settle had to be decided, two of them dependencies Barry approved, and review of the bus and the approve
policy hardened both. ADR 0001 keeps what the CLIs do; this ADR keeps how Emmet uses them.

## Decisions

### Dependencies (approved by Barry, docs/decisions.md 2026-10-25)

| Crate | Version | License | For |
|---|---|---|---|
| keyring-core, apple-native-keyring-store (feature `keychain`), windows-native-keyring-store (no default features) | 1.0.1, 1.0.2, 1.1.0 | MIT OR Apache-2.0 | The Codex API key in the macOS Keychain and the Windows Credential Manager (D-35). On macOS it uses security-framework, already in the tree |
| widestring (Windows only) | 1.2.1 | MIT OR Apache-2.0 | The bus pipe's SDDL access list, which interprocess takes as a wide string. Already in the tree through interprocess |

Rejected: shelling out to `/usr/bin/security` (macOS only, no Windows path), and FFI with an `unsafe` exception.

### The fake agents are their own crate

`tests/fake-agents` (section 05 names the directory) holds `fake-claude` and `fake-codex`, never shipped. Tests pick
them with `EMMET_CLAUDE_BIN` and `EMMET_CODEX_BIN` in the session's environment (or the app's), and their scenario
with `EMMET_FAKE_DIR`. A fake replays one of 0B's transcripts, or one derived from them by `tests/replay/derive.mjs`
(a crash, a malformed line, an unknown event, Emmet's own tools, a question), as the CLI would:

- It starts the MCP servers the adapter configured and calls the real ones for every tool they have; 0B's stub tools
  (`ping`, `block`) keep their recorded results. Where 0B's stub logged a permission prompt, the fake asks the real
  `emmet_host approve`. It makes Claude's Edit and Write changes and Codex's approved `fileChange` items.
- Two orders the recorder used but Emmet doesn't are accepted: a message the recorder wrote right after an interrupt
  (Emmet sends it once the turn has ended), and, for Codex, independent lines in either order (a redirect typed as a
  tool-call approval goes out). A Codex `turn/start` mid-turn may be a `turn/steer`, which Codex folds the same way.
- Tests never touch the real keychain: in a debug build, `EMMET_FAKE_KEYCHAIN` keeps the key in a file.
- Detection and Settings only check that the key exists (`keychain::has`, an attribute lookup); the key is read when
  a Codex session starts. On macOS, reading it asks for the login password whenever the app's signature changed since
  the key was saved, which is every rebuild of a dev build.

### Session API beyond section 10's sketch

`Session` adds `answer(question, answer)` (question cards), `cancel_note` and `edit_note` (a pending redirect's
Cancel and Edit), and `close()`. `steer(text)` returns the note; `NoteDelivered { path }` says how it went out.
`AgentEvent` adds `ApprovalClosed`, `QuestionClosed`, and `Exited`, and `Started` carries who pays.

### Redirects wait for the agent's next step

A redirect stays in Emmet, editable, until a tool call is in progress (or the next one starts), then goes out
natively; that's when either CLI takes it anyway (ADR 0001), so delivery is no later, and Edit and Cancel mean
something until then. With native delivery off, or refused by Codex, the note waits in the hub for the next Emmet tool
result (`user_notes`). If the turn ends first, it's the next turn.

### File changes come from the project, not the protocol

After each tool result and at the end of a turn, the engine compares the project's files (size and modification time,
outside `build/`, `.emmet/`, `.git/`) with before. That sees an edit however it was made (Claude's Edit, Codex's
`apply_patch` or a shell command), the same way for both CLIs. Every change is reported by the end of its turn; one
made very fast after a tool result may be reported a step early.

### The approve policy

`emmet_host approve` allows reads, Emmet's own tools, and edits inside the project, judged after following links, by
the path field the tool writes. Everything else is a card. Inside the project, agent config is a card too: a dot
folder or file (`.claude/`, `.codex/`, `.mcp.json`, `.git/`), `AGENTS.md`, `CLAUDE.md`, `CLAUDE.local.md`, a name
with trailing dots or spaces or a `:`, any non-ASCII name (both file systems fold Unicode case), a dangling link, and
`..` past a missing folder. "Allow for this session" covers that one command, host, or file, never a whole tool.

### The bus authenticates both ways

The token never crosses the socket. A request carries a random nonce and `sha256("client:token:nonce:id:message")`;
the reply carries `sha256("server:token:nonce:reply")`, and the client believes nothing without it. The server
refuses a nonce it has seen. So a process squatting on the socket's name (say, after a crash left `bus.json`) can't
drive the app or answer `approve` for it. The macOS socket is chmod 0600 as soon as it's bound (macOS refuses fchmod
on a socket before bind); the Windows pipe's ACL grants only its owner and SYSTEM.

### Doctrine through tools

Section 10 has the doctrine "read once through a tool", and section 09 had no such tool, so `emmet mcp` gains
`rules` (EMMET.md) and `api` (the DSL vocabulary, generated from emmet_dsl's docstrings), the same text as
`emmet rules` and `emmet api`.

### Tests pin what the design promises

The replay outlines keep three orders apart: the CLI's own events in order, the hub's cards in order among
themselves (they race the CLI's stdout), and the set of files changed by the end of the turn.

## Consequences

- `cargo deny check` passes with the new crates.
- `scripts/ci-scope.sh` runs the Windows E2E when `crates/emmet-agent/` or `tests/fake-agents/` change, since the
  bridge spawns processes and talks over pipes.
- The chat's event-to-pixels budget is measured at the DOM commit (p95 5 ms on the development Mac), with one 60 Hz
  frame allowed inside the 50 ms.

## Rerun

```sh
node tests/replay/derive.mjs                 # rewrite the derived transcripts
cargo test -p emmet-it --test agent_claude --test agent_codex --test agent_sentinel --test host --test bus
just e2e chat
```
