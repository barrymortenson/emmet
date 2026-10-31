# emmet-bus v1

How `emmet`, `emmet mcp`, and `emmet mcp-host` reach the running app (plan sections 04, 09, 10, and 15). The crate is
`crates/emmet-bus`; its tests are `tests/it/tests/bus.rs`, the agent replay tests, and `tests/e2e/specs/render.e2e.mjs`
and `chat.e2e.mjs`.

## Finding the app

While the app runs, it keeps `<data dir>/bus.json`, readable by this user only:

```json
{ "v": 1, "name": "emmet-0123456789abcdef.sock", "token": "<64 hex digits, new each launch>", "pid": 4242 }
```

`name` is a local socket: a named pipe on Windows (its ACL grants only the pipe's owner and SYSTEM), `/tmp/<name>` on
macOS (chmod 0600 as soon as it's bound). It's derived from the data dir, so the dev, test, and installed apps never
meet. The app removes `bus.json` when it quits. A stale file (the app crashed) fails to connect, or reaches nothing
that can answer for the app (below).

## Messages

One JSON object per line (NDJSON), one reply per request, on one connection. The token itself never crosses the
socket: each request has a new random `nonce` and `auth = sha256("client:" + token + ":" + nonce + ":" + id + ":" +
message)`, where `message` is the request's `type` and fields as JSON; the reply has `auth = sha256("server:" + token
+ ":" + nonce + ":" + reply)`, the reply as JSON without `auth`. The server refuses a wrong `auth` (and ends the
connection) and a nonce it has seen; the client believes no delivered reply without the server's `auth`. So only a
process that read `bus.json` can drive the app, and only the app can answer.

```json
{ "v": 1, "nonce": "…", "auth": "…", "id": 1, "type": "render", "project": "/abs/cottage", "view": "iso", "size": 768, "highlight": [] }
{ "v": 1, "nonce": "…", "auth": "…", "id": 1, "type": "highlight", "project": "/abs/cottage", "ids": ["roof/1"], "from": "claude-code" }
{ "v": 1, "nonce": "…", "auth": "…", "id": 1, "type": "camera", "project": "/abs/cottage", "view": "front" }
{ "v": 1, "nonce": "…", "auth": "…", "id": 1, "type": "selection", "project": "/abs/cottage" }
{ "v": 1, "nonce": "…", "auth": "…", "id": 1, "type": "approve", "session": "c0123…", "request": { "tool_name": "Bash", "input": { "command": "ls" }, "tool_use_id": "…" } }
{ "v": 1, "nonce": "…", "auth": "…", "id": 1, "type": "ask", "session": "c0123…", "question": { "question": "How tall?", "header": "Height", "options": [{ "label": "6 bricks" }, { "label": "3 bricks" }], "multi": false } }
{ "v": 1, "nonce": "…", "auth": "…", "id": 1, "type": "notes", "session": "c0123…" }
```

```json
{ "id": 1, "delivered": true, "png": "<base64>", "camera": { "view": "iso", "position": [], "target": [], "fov": 35 }, "highlighted": [], "auth": "…" }
{ "id": 1, "delivered": true, "highlighted": ["roof/1"], "auth": "…" }
{ "id": 1, "delivered": true, "camera": { "view": "front" }, "auth": "…" }
{ "id": 1, "delivered": true, "camera": { "view": "iso" }, "selected": ["roof/1", "roof/2"], "auth": "…" }
{ "id": 1, "delivered": true, "answer": { "behavior": "deny", "message": "The user denied this in Emmet." }, "auth": "…" }
{ "id": 1, "delivered": true, "answer": { "answered": true, "choices": ["6 bricks"], "other": null }, "auth": "…" }
{ "id": 1, "delivered": true, "answer": [{ "id": "n1", "text": "Make it blue", "at": "2026-10-25T12:00:00.000Z" }], "auth": "…" }
{ "id": 1, "delivered": false, "reason": "that project isn't open in Emmet" }
```

- `render`, `highlight`, `camera`, and `selection` answer only for the project the app has open (paths compared after
  canonicalizing). `camera` with a `view` turns the open viewer to it, and without one reads it; `selection` reads what
  the user has selected and the view (4B). The app's own sessions and terminal agents use them alike, through MCP's
  `highlight`, `camera`, and `selection`.
  `highlight`'s optional `from` is the name the agent's MCP client gives ("claude-code"): in each request's `_meta`
  in MCP 2026-07-28, which Claude Code speaks, or in `initialize` before it. The viewer
  names the agent in the bar it shows over the highlight ("Claude is pointing at 10 parts", with Zoom to them and
  Clear). An agent's highlight replaces what the user picked, and the user's next pick replaces it.
  `render` renders the project's current `build/placements.json` in the open window, offscreen: the window's camera
  doesn't move.
- `approve` (from `emmet_host approve`, Claude Code's permission prompt) shows an approval card in the chat session
  and is held until the user decides; `ask` (from `emmet_host ask_user`) shows a question card and is held until the
  user answers or Stop cancels it (`answered: false, reason: "stopped"`). `notes` (from `emmet mcp --session`) takes
  the session's queued redirect notes. A session the app doesn't know is `delivered: false`.
- Messages are UI-only. They can't run code or write files; an approval only answers the CLI's own question.

## Never an error

`emmet_bus::send` always returns a reply. No `bus.json`, no socket, a wrong token, a refused nonce, another protocol
version, another project open, an answer without the app's proof, or no answer in time is `delivered: false` with a
`reason`. `emmet render` then falls back to render mode (ADR 0005); MCP `highlight`, `camera`, and `selection` report "Emmet app not open";
`approve` denies with the reason; `ask_user` returns `reason: "stopped"` (or `"timeout"` just before the hour the CLIs
allow a tool call).
