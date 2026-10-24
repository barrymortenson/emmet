# Emmet Builder

Emmet Builder is a free desktop app for macOS and Windows where your own Claude Code or Codex agent builds brick models
from real LDraw parts. The agent writes a parametric Python model, and you watch it take shape in a 3D viewer with
sliders, buildability checks, a parts list, and build steps. It runs locally: no hosting, no accounts, and you bring
your own agent.

**Status: foundation only.** Nothing builds a model yet. The build plan is [`docs/plan.html`](docs/plan.html) (open it
in a browser), and [`docs/plan-status.js`](docs/plan-status.js) tracks progress.

## What works today

- `emmet --version [--json]`: the CLI prints its version, or the JSON envelope from plan section 09.
- `just dev`: opens the Emmet window, which shows the name and version.
- The crates (`emmet-core`, `emmet-agent`, `emmet-bus`), the viewer package (`createViewer`), and the Python package
  (`emmet_dsl.__version__`) are one-function stubs. Each builds and has an integration test.
- CI on macOS and Windows, the license, SPDX, DCO, secret, and supply-chain checks, and the plan tracker check.

The app icon is a placeholder until the design phase.

## Setup

You need these on your PATH. `just setup` checks them and installs nothing globally.

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
| `just test [filter]` | Integration tests, fastest first; `just test --changed` runs only those affected by your changes |
| `just ci` | Everything CI's mac job runs; it must pass before you push |
| `just plan` | Check the status file and open the plan |
| `just e2e`, `just eval`, `just screenshots`, `just bench`, `just release-dry`, `just release` | Not yet; each one names the session that adds it |

## License

Emmet is free software under the [GNU AGPL-3.0-or-later](LICENSE): you may use, change, and share it, and if you
distribute it or run it as a service, you must publish your full source under the same license. The Python DSL
(`python/emmet_dsl`), the techniques library, and the project templates are [Apache-2.0](python/emmet_dsl/LICENSE), so
the models you write are clearly yours, as is everything you make with Emmet. The names "Emmet Builder" and "Emmet"
aren't covered by either license ([TRADEMARKS.md](TRADEMARKS.md)). Contributions come in under the DCO
([CONTRIBUTING.md](CONTRIBUTING.md)). See [NOTICE](NOTICE) for attributions.

LEGO® is a trademark of the LEGO Group, which does not sponsor, authorize, or endorse this software.
