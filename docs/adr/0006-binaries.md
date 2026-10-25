# ADR 0006: Two executables on Windows

Status: accepted, 2026-10-24 (session 0B). Confirms D-08.

## Context

D-08 ships two executables: the GUI app, and a console `emmet` CLI as a Tauri sidecar (which is also the stdio MCP
server agents spawn). It amended the brief's "one binary" because a Windows GUI-subsystem exe can't print to a
terminal. 0B checks both halves of that on Windows: a GUI-subsystem exe can't serve a terminal, and a console sidecar
serves stdio MCP fine.

## Evidence

`tests/spikes/binaries/` builds the same line-oriented server twice: `console-echo` (the default console subsystem)
and `gui-echo` (`#![windows_subsystem = "windows"]`, like the app's release exe). It answers a JSON-RPC style
`initialize` the way a stdio MCP server does, echoes other lines, and reports whether the process has a console (it
can open `CONOUT$` only when one is attached). `tests/spikes/binaries/tests/subsystems.rs`:

- `both_subsystems_answer_over_pipes` (every OS): each exe, spawned with piped stdin and stdout the way an agent
  spawns an MCP server, answers `initialize` and echoes a line.
- `only_the_console_subsystem_has_a_console` (Windows only): `console-echo` reports `console=true`, `gui-echo`
  reports `console=false`.

Windows CI (`windows-latest`, the 0B PR's `windows` job): WINDOWS_RESULT

## Decision

D-08 stands, with one nuance recorded:

- **A GUI-subsystem exe can't serve a terminal.** Windows gives it no console, so a user typing `emmet` in a terminal
  would see no output, and an interactive shell doesn't wait for it to finish. The CLI has to be a console-subsystem
  exe.
- **A console sidecar serves stdio MCP fine** over the pipes an agent gives it.
- **The nuance:** a GUI-subsystem exe also serves stdio over pipes, so MCP alone wouldn't need a second binary. The
  terminal CLI does, and since `emmet mcp` is a subcommand of that CLI, one console sidecar serves both. Section 03
  doesn't change.

## Rerun

```sh
cargo test -p emmet-spike-binaries -- --nocapture   # on Windows for the console test; it prints both reports
```
