#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# First step of every self-hosted job (plan section 17). Refuses to run unless the event is trusted: this repo (no forks),
# Barry as the actor and PR author, and the job's data dir and test HOME inside the runner's temp dir, so tests never
# touch ~/.claude, ~/.codex, or real app data. The workflow passes the PR fields in EMMET_PR_HEAD_REPO and EMMET_PR_AUTHOR.
set -euo pipefail
repo=barrymortenson/emmet
owner=barrymortenson
fail() {
  echo "runner-guard: refusing to run: $*" >&2
  exit 1
}

[[ "${GITHUB_REPOSITORY:-}" == "$repo" ]] || fail "the repository is '${GITHUB_REPOSITORY:-unset}', not $repo"
[[ "${GITHUB_ACTOR:-}" == "$owner" ]] || fail "the actor is '${GITHUB_ACTOR:-unset}', not $owner"
case "${GITHUB_EVENT_NAME:-}" in
  push | schedule | workflow_dispatch) ;;
  pull_request)
    [[ "${EMMET_PR_HEAD_REPO:-}" == "$repo" ]] || fail "the PR comes from '${EMMET_PR_HEAD_REPO:-unset}', not $repo"
    [[ "${EMMET_PR_AUTHOR:-}" == "$owner" ]] || fail "the PR was opened by '${EMMET_PR_AUTHOR:-unset}', not $owner"
    ;;
  *) fail "the event '${GITHUB_EVENT_NAME:-unset}' isn't trusted" ;;
esac

temp=${RUNNER_TEMP:-}
[[ -n "$temp" && -d "$temp" ]] || fail "RUNNER_TEMP isn't a directory"
in_temp() { [[ "$1" == "$temp"/* && "$1" != *..* ]]; }
in_temp "${EMMET_DATA_DIR:-}" || fail "EMMET_DATA_DIR ('${EMMET_DATA_DIR:-unset}') isn't inside RUNNER_TEMP"
in_temp "${EMMET_TEST_HOME:-}" || fail "EMMET_TEST_HOME ('${EMMET_TEST_HOME:-unset}') isn't inside RUNNER_TEMP"

echo "runner-guard: ok ($GITHUB_EVENT_NAME by $GITHUB_ACTOR)"
