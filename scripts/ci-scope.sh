#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Decides what CI runs from the paths that changed (plan section 16), so a change runs only the checks that can catch
# a bug in it. Prints key=true|false lines for $GITHUB_OUTPUT or eval: full, mac, windows, deps, e2e_mac, e2e_windows,
# build_app, bench. The changes are those since EMMET_CI_BASE (CI sets it to the last green commit of the PR, or the
# PR's base), else since the merge-base with origin/main, plus uncommitted and untracked files. EMMET_CI_FULL=1, or a
# change to the toolchain, the lockfiles, the justfile, or a workflow, runs everything. Written for bash 3.2.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

base=${EMMET_CI_BASE:-$(git merge-base HEAD origin/main 2>/dev/null || git rev-list --max-parents=0 HEAD)}
changed=$( { git diff --name-only "$base"; git ls-files --others --exclude-standard; } | sort -u)
any() { grep -qE "$1" <<<"$changed"; }
flag() { if [[ $full == true ]] || any "$2"; then echo "$1=true"; else echo "$1=false"; fi; }

full=false
if [[ "${EMMET_CI_FULL:-}" == 1 ]] ||
  any '^(justfile|\.node-version|rust-toolchain\.toml|package\.json|pnpm-(lock|workspace)\.yaml|Cargo\.(toml|lock)|python/emmet_dsl/(pyproject\.toml|uv\.lock)|\.github/workflows/)'; then
  full=true
fi
echo "full=$full"
# The Mac job: anything but prose. The plan job checks docs/ on its own; docs/spec/mcp/ is read by the Rust tests.
if [[ $full == true ]] || grep -vE '^(docs/|\.claude/)|\.md$' <<<"$changed" | grep -q . || any '^docs/spec/mcp/'; then
  echo "mac=true"
else
  echo "mac=false"
fi
# Windows: what behaves differently there (paths, processes, the sidecars, Python under uv, line endings in fixtures).
# The UI and the viewer run the same JS in WebView2, so the Mac covers them.
flag windows '^(crates/|apps/desktop/src-tauri/|tests/(it|fixtures|replay|e2e)/|python/emmet_dsl/src/|data/)'
flag deps '^deny\.toml$'
flag e2e_mac '^(apps/|crates/|packages/|tests/(e2e|fixtures)/|python/emmet_dsl/src/|data/)'
flag e2e_windows '^(apps/desktop/src-tauri/|tests/e2e/)'
flag build_app '^apps/desktop/src-tauri/'
# The bench asserts nothing (its numbers are a summary table), so it runs only with everything else.
echo "bench=$full"
