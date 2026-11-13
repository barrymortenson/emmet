#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Decides what CI runs from the paths that changed (plan section 16), so a change runs only the checks that can catch
# a bug in it. Prints key=value lines for $GITHUB_OUTPUT or eval: full, mac, windows, deps, e2e_mac, e2e_specs,
# e2e_windows, build_app, nsis. e2e_specs is the comma-separated list tests/e2e/tiers.mjs picks. The changes are those
# since EMMET_CI_BASE (CI sets it to the last green commit of the PR, or the PR's base), else since the merge-base with
# origin/main, plus uncommitted and untracked files. EMMET_CI_FULL=1, or a change to the toolchain or a lockfile, runs
# everything a PR can; the bench and the by-hand E2E tier run only by hand (just bench, just e2e all). A change to the
# justfile or a workflow runs the checks and core E2E.
# Written for bash 3.2.
set -euo pipefail
# The tiers come from this checkout, whichever repo is being scoped (tests/scripts/ci.test.mjs scopes temp repos).
tiers="$(cd "$(dirname "$0")/.." && pwd)/tests/e2e/tiers.mjs"
cd "$(git rev-parse --show-toplevel)"

base=${EMMET_CI_BASE:-$(git merge-base HEAD origin/main 2>/dev/null || git rev-list --max-parents=0 HEAD)}
# Separate steps, so a base that isn't a commit fails here instead of reading as "nothing changed".
diff=$(git diff --name-only "$base")
changed=$( { echo "$diff"; git ls-files --others --exclude-standard; } | sed '/^$/d' | sort -u)
any() { grep -qE "$1" <<<"$changed"; }
flag() { if [[ $full == true ]] || any "$2"; then echo "$1=true"; else echo "$1=false"; fi; }

full=false
if [[ "${EMMET_CI_FULL:-}" == 1 ]] ||
  any '^(\.node-version|rust-toolchain\.toml|package\.json|pnpm-(lock|workspace)\.yaml|Cargo\.(toml|lock)|python/emmet_dsl/(pyproject\.toml|uv\.lock))$'; then
  full=true
fi
echo "full=$full"
# How CI itself runs: the justfile and the workflows. Their own tests are in tests/scripts, and core E2E shows the E2E
# steps still work.
infra=false
if any '^(justfile|\.github/workflows/.*)$'; then infra=true; fi
# The Mac job: anything but prose. The plan job checks docs/ on its own; docs/spec/mcp/ is read by the Rust tests, and
# docs/design/ by tests/scripts/design.test.mjs (the frozen design, its inventory, and its icons; 2B), which needs the
# installed packages the plan job doesn't have.
if [[ $full == true ]] || grep -vE '^(docs/|\.claude/)|\.md$' <<<"$changed" | grep -q . || any '^docs/(spec/mcp|design)/'; then
  echo "mac=true"
else
  echo "mac=false"
fi
# Windows: what behaves differently there (paths, processes, the sidecars, Python under uv, line endings in fixtures).
# The UI and the viewer run the same JS in WebView2, so the Mac covers them.
flag windows '^(crates/|apps/desktop/src-tauri/|tests/(it|fixtures|replay|e2e|fake-agents)/|python/emmet_dsl/src/|data/)'
flag deps '^deny\.toml$'
# The E2E specs (tests/e2e/tiers.mjs): core for anything that reaches the app, each area that changed, and a spec whose
# own file changed. Everything but the by-hand tier on a full run.
if [[ $full == true ]]; then
  specs=$(node "$tiers")
else
  specs=$(node "$tiers" --changed <<<"$changed")
  if [[ -z "$specs" && $infra == true ]]; then specs=$(node "$tiers" core); fi
fi
if [[ -n "$specs" ]]; then echo "e2e_mac=true"; else echo "e2e_mac=false"; fi
echo "e2e_specs=$specs"
# Windows E2E: the specs that cover paths and processes (tests/e2e/tiers.mjs windows), when the native side, the agent
# bridge, or the specs change.
flag e2e_windows '^(apps/desktop/src-tauri/|tests/e2e/|crates/emmet-agent/|tests/fake-agents/)'
# The release-protocol build (the custom protocol and the embedded frontend), when the native side changes.
flag build_app '^apps/desktop/src-tauri/'
# The unsigned NSIS installer (release-dry on Windows): a release build, so only when the release files change.
flag nsis '^(scripts/release-dry\.sh|apps/desktop/src-tauri/tauri\.release\.conf\.json)$'
