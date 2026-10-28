# Emmet Builder

Emmet Builder is a free desktop app for macOS and Windows where your own Claude Code or Codex agent builds brick models
from real LDraw parts. The agent writes a parametric Python model, and you watch it take shape in a 3D viewer with
sliders, buildability checks, a parts list, and build steps. It runs locally: no hosting, no accounts, and you bring
your own agent.

**Status: the engine works from a terminal, and the app opens a project and shows it live.** The build plan is [`docs/plan.html`](docs/plan.html) (open it
in a browser), and [`docs/plan-status.js`](docs/plan-status.js) tracks progress.

## What works today

- The `emmet` CLI (session 1A) turns a Python model into an LDraw `.mpd` from the real LDraw library:
  ```sh
  cargo build -p emmet-cli
  target/debug/emmet library install        # the pinned LDraw release, about 150 MB, into .dev-data/ in a debug build
  target/debug/emmet new demo && target/debug/emmet run --project demo   # → demo/build/model.mpd
  ```
  Also `emmet search <words>` (try `brick 2x4`), `emmet part <id>`, `emmet colors`, `emmet params [--set k=v]`,
  `emmet trust`, `emmet doctor [--fix]`, `emmet library status|install|verify`, and `emmet mcp`, a stdio MCP server
  with the same operations for an agent. `--json` prints the JSON envelope from plan section 09. Python 3.13 and the
  DSL install themselves through uv on first use. The DSL is in [`docs/spec/dsl.md`](docs/spec/dsl.md), the project
  folder in [`docs/spec/project-format.md`](docs/spec/project-format.md).
- The app (session 1B): `just dev [project]` opens a project folder and shows its model in the 3D viewer
  ([`packages/viewer`](packages/viewer/README.md)). Edit `model.py` in any editor, or drag a slider in the params
  drawer, and the viewer updates in place without moving the camera; a broken model shows the error and its line
  while the last good model stays. Plain UI for now; Phase 2 designs it.
- `emmet render [--view iso] [--size 1024] [--highlight ids] [--out file.png]`: a PNG through the open app over
  [emmet-bus](docs/spec/bus.md), or the app's hidden render mode when it isn't open. `emmet mcp` has `render` (image
  content) and `highlight` too.
- `just bench`: the viewer's load, update, and orbit timings in the real app (ADR 0008).
- The chat pane (session 1C): pick Claude Code or Codex, then ask it to build. The agent edits `model.py` with Emmet's
  tools and the viewer updates as it works; approvals and questions appear as cards, and you can redirect it mid-turn
  or stop it. Claude Code uses your own sign-in; Codex runs on your OpenAI API key (Settings → Agents, kept in the
  keychain). `emmet rules` and `emmet api` print what the agent reads.
- Checks (session 1D): `emmet check` finds collisions, floating parts, and unknown or obsolete parts, each with the
  `model.py` line that placed it; `emmet bom` lists the parts. In the app, the Checks drawer highlights a finding's
  parts and shows its line, and "Ask agent to fix" sends it to the chat; agent runs build up part by part.
- CI on macOS and Windows, the license, SPDX, DCO, secret, and supply-chain checks, and the plan tracker check.
- `just e2e`: end-to-end specs drive the real app through WebdriverIO (`tests/e2e/`): the live viewer, rendering with
  the app open, a killed run, a leak test, and visual baselines.
- Research results in `docs/adr/` (agent protocols, E2E, licenses, data mapping, headless render, binaries), test
  fixtures (`tests/fixtures/ldraw-mini/`, `tests/fixtures/rebrickable-mini/`, `tests/replay/`), and the model ladder
  the evals measure against (`docs/spec/ladder.md`). Spike code lives in `tests/spikes/`.

The design is frozen in `docs/design/frozen/` (open `index.html` from the folder): every screen and state, built from
the tokens in `apps/desktop/src/styles/tokens.css` and the components in `docs/design/components.md`.
`node scripts/app-icon.mjs` makes the app icons from `docs/design/icon/`.

## Setup

You need these on your PATH. `just setup` checks for them; beyond the repo, it installs only the pinned Rust
toolchain (through rustup) and Python 3.13 (through uv's own cache).

| Tool | Version | Install |
|------|---------|---------|
| rustup | any; the toolchain comes from `rust-toolchain.toml` | [rustup.rs](https://rustup.rs) |
| Node | `.node-version` (22.21.1) | your version manager, or [nodejs.org](https://nodejs.org) |
| pnpm | `packageManager` in `package.json` (11.22.0) | [pnpm.io](https://pnpm.io/installation) |
| uv | 0.12 or later | `brew install uv`, or [docs.astral.sh/uv](https://docs.astral.sh/uv/) |
| just | 1.58 or later | `brew install just` |
| gitleaks | 8.30 or later | `brew install gitleaks` |
| cargo-deny | 0.20 or later | `cargo install --locked cargo-deny` |
| Xcode command line tools (macOS) | | `xcode-select --install` |

Then:

```sh
just setup
```

That installs the pinned Rust toolchain through rustup, the JS packages into `node_modules/`, and a uv-managed
Python 3.13 with the DSL's dev tools into `python/emmet_dsl/.venv/` (uv keeps its Python builds in its own cache), and
points git at the hooks in `.githooks/`. On Windows, run `just` from Git Bash.

## Commands

| Command | Does |
|---------|------|
| `just setup` | Check the toolchain, install dependencies into the repo, install the git hooks |
| `just dev` | The app with hot reload, under the dev bundle id `com.emmetbuilder.emmet.dev` |
| `just check` | Format, lint, typecheck, and SPDX headers (seconds) |
| `just test [filter]` | Integration tests, fastest first; `just test --changed` runs only those affected by your changes; run it before you push |
| `just ci` | What CI's mac job runs, scoped to what your changes can break (CI runs it for you) |
| `just plan` | Check the status file and open the plan |
| `just e2e [spec]` | Build the debug app with the embedded WebDriver server and run the E2E specs (`just e2e hello` runs one) |
| `just bench` | The viewer's load, update, and orbit timings in the real app |
| `just eval --rung 1 --agent claude` | One live eval run on the eval API key (bills it; asks first). Quit the app first |
| `just release-dry [--sign]` | A universal macOS `.dmg` (signed and notarized with `--sign`); on Windows, the NSIS installer |
| `just screenshots`, `just release` | Not yet; each one names the session that adds it |

Scripts run by hand: `scripts/scrub-transcript` (scrub an agent transcript before committing it),
`scripts/make-ldraw-mini` and `scripts/make-rebrickable-mini` (rebuild the fixtures from the pinned downloads),
`scripts/terms-check` (re-read the provider terms at the gates), and `scripts/name-check` (re-run the trademark
search for the name at the gates).

## License

Emmet is free software under the [GNU AGPL-3.0-or-later](LICENSE): you may use, change, and share it, and if you
distribute it or run it as a service, you must publish your full source under the same license. The Python DSL
(`python/emmet_dsl`), the techniques library, and the project templates are [Apache-2.0](python/emmet_dsl/LICENSE), so
the models you write are clearly yours, as is everything you make with Emmet. The names "Emmet Builder" and "Emmet"
aren't covered by either license ([TRADEMARKS.md](TRADEMARKS.md)). Contributions come in under the DCO
([CONTRIBUTING.md](CONTRIBUTING.md)). See [NOTICE](NOTICE) for attributions.

LEGO® is a trademark of the LEGO Group, which does not sponsor, authorize, or endorse this software.
