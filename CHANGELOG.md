# Changelog

Notable changes to Emmet. Versions follow semver; the minor version bumps per phase, and 1.0.0 is the public launch.

## Unreleased

### Added

- The repo foundation (session 0A): the AGPL-3.0-or-later license (Apache-2.0 for `python/emmet_dsl`), NOTICE,
  TRADEMARKS.md, CONTRIBUTING.md (DCO), SECURITY.md, and the code of conduct.
- The skeleton: four Rust crates, the Tauri 2 desktop app (one window showing the name and version), the viewer package,
  the Python DSL package, and an integration test for each.
- `emmet --version [--json]`.
- The `justfile` (every command in plan section 18), git hooks, and CI on the self-hosted Mac and hosted Windows, with
  SPDX, DCO, gitleaks, cargo-deny, and plan-tracker checks.
- Claude Code commands for plan section 19: `/create-pr` opens the session's PR, sets the session to review, and runs
  the PR review toolkit on it; `/merge` evaluates the session against the plan, sets it to done on the PR branch,
  waits for the required checks, and merges.
- Research spikes and ADRs 0001–0006 (session 0B): how Claude Code and Codex really behave (recorded, scrubbed
  transcripts in `tests/replay/`, the Codex app-server schema in `crates/emmet-agent/schema/codex/0.160.1/`), E2E with
  WebdriverIO and the embedded WebDriver server (`just e2e`), licenses and reuse, LDraw ↔ Rebrickable ↔ BrickLink
  mapping, headless rendering in a hidden window, and two executables on Windows.
- `tests/fixtures/ldraw-mini/` (141 LDraw parts in 177 part files, with aliases, and their attribution, checked in CI) and
  `tests/fixtures/rebrickable-mini/`, with `scripts/make-ldraw-mini` and `scripts/make-rebrickable-mini`.
- `scripts/scrub-transcript`, `scripts/terms-check` (with the provider quotes in `docs/terms/`), and the model ladder
  spec (`docs/spec/ladder.md`), with rung 3 rebuilding 6743-1 Street Speeder.

- The engine walking skeleton (session 1A): an LDraw parser and writer with body boxes (studs left out); the
  library manager (`emmet library status|install|verify`: the pinned 2026-09 `complete.zip`, sha256-checked,
  resumable, with a zip-slip guard); the parts index in SQLite FTS5 with dimension parsing, synonyms
  (`data/synonyms.toml`), and ranking (`emmet search`, `emmet part`, `emmet colors`); emmet_dsl v0 (`@model`,
  `Model.add`, `Model.step`, parameters from the signature, the Z-up stud and plate frame with footprint placement,
  ids with file:line, color names, error codes); the Python bootstrap through uv; `emmet new` (cottage and empty
  templates), `emmet run`, `emmet params`, `emmet trust`, `emmet doctor`; and `emmet mcp` with `search_parts`,
  `part_info`, `run_model`, `get_params`, `set_params`, and `list_colors`, whose schemas are in `docs/spec/mcp/`.
  New dependencies: ureq and toml (ADR 0007).

- The app shell and live viewer (session 1B): `packages/viewer` builds its own three.js scene from placements, each
  unique part loaded once through LDrawLoader and instanced per (part, color) with instanced edges, diff updates by
  Emmet id, highlight, views, frame, offscreen PNG renders, and stats (ADR 0008 confirms D-23). The Tauri host opens,
  creates, and reopens projects, watches sources (150 ms debounce), runs the model through the emmet sidecar (latest
  wins, stale runs canceled, older generations dropped), serves the library over `emmet-lib://` in one request per
  model, and types every command and event through tauri-specta. A plain UI: project tree, viewer, params drawer,
  run panel with the traceback, status line. `emmet render` and MCP `render` and `highlight` go through the open app
  over emmet-bus v0 (`docs/spec/bus.md`), or the app's hidden render mode. `just dev [project]`, `just bench`, and
  E2E specs with visual baselines and a leak test. New dependencies, all named in plan section 06 but
  `@types/three` (approved): three, tauri-specta, notify, interprocess, tauri-plugin-dialog, pixelmatch.

### Changed

- Agent permissions are sandbox first (D-40), section 10 quotes OpenAI's current wording, and the LDCad shadow
  library's connector data will ship as its own CC BY-SA 4.0 file (Q-01).
