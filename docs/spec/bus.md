# emmet-bus v0

How `emmet` and `emmet mcp` reach the running app for ephemeral UI events (plan sections 04 and 09). The crate is
`crates/emmet-bus`; its tests are `tests/it/tests/bus.rs` and `tests/e2e/specs/render.e2e.mjs`. Session 1C extends it
(redirect notes, `ask_user`) and hardens it (section 15: the Windows pipe ACL, the socket's tests).

## Finding the app

While the app runs, it keeps `<data dir>/bus.json`, created readable by this user only:

```json
{ "v": 1, "name": "emmet-0123456789abcdef.sock", "token": "<64 hex digits, new each launch>", "pid": 4242 }
```

`name` is a local socket: a named pipe on Windows, `/tmp/<name>` (0600) on macOS. It's derived from the data dir, so
the dev, test, and installed apps never meet. The app removes `bus.json` when it quits. A stale file (the app crashed)
just fails to connect.

## Messages

One JSON object per line (NDJSON), one reply per request, on one connection:

```json
{ "v": 1, "token": "…", "id": 1, "type": "render", "project": "/abs/cottage", "view": "iso", "size": 768, "highlight": [] }
{ "v": 1, "token": "…", "id": 2, "type": "highlight", "project": "/abs/cottage", "ids": ["roof/1"] }
```

```json
{ "id": 1, "delivered": true, "png": "<base64>", "camera": { "view": "iso", "position": [], "target": [], "fov": 35 }, "highlighted": [] }
{ "id": 2, "delivered": true, "highlighted": ["roof/1"] }
{ "id": 3, "delivered": false, "reason": "that project isn't open in Emmet" }
{ "id": 4, "delivered": true, "error": "can't render: 3001 (the library is missing …)" }
```

- The app answers only for the project it has open (paths compared after canonicalizing).
- `render` renders the project's current `build/placements.json` in the open window (showing it first if it's newer
  than the scene on screen), with the window's own viewer, offscreen: the window's camera doesn't move.
- Messages are UI-only. They can't run code or write files; the PNG comes back in the reply, and `emmet` writes it.

## Never an error

`emmet_bus::send` always returns a reply. No `bus.json`, no socket, a wrong token, another protocol version, or no
answer in time is `delivered: false` with a `reason`. `emmet render` then falls back to render mode
(`Emmet --render`, a hidden window, ADR 0005); MCP `highlight` reports "Emmet app not open".
