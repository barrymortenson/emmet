#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Runs after checkout in every self-hosted job (plan section 17). Refuses to run unless the event is one section 17
# trusts: Barry's push to main or a tag, the nightly schedule, or a PR Barry opened from this repo (no forks). It also checks that
# EMMET_DATA_DIR and EMMET_TEST_HOME sit inside the runner's temp dir; nothing reads them yet, and the sessions that
# touch app data or agent config (1A, 1C) must use them. The workflow passes the PR fields in EMMET_PR_HEAD_REPO and
# EMMET_PR_AUTHOR. A pull_request run uses the PR's own copy of this script, so it catches routing mistakes, not a
# hostile PR: that rests on approving fork runs only after reading them.
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
  push) [[ "${GITHUB_REF:-}" == refs/heads/main || "${GITHUB_REF:-}" == refs/tags/* ]] ||
    fail "a push to '${GITHUB_REF:-unset}', not main or a tag" ;;
  schedule) ;;
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
