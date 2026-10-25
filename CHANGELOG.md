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

### Changed

- CI skips a mac or windows job's work when the same code already passed on that OS (`scripts/ci-inputs-hash`), so
  re-runs and status-only commits don't repeat an 11-minute Windows run.

- Agent permissions are sandbox first (D-40), section 10 quotes OpenAI's current wording, and the LDCad shadow
  library's connector data will ship as its own CC BY-SA 4.0 file (Q-01).
