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
