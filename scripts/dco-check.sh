#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Fails when a commit in <base>..<head> lacks a Signed-off-by trailer that matches its author (the DCO, CONTRIBUTING.md),
# or when the range doesn't resolve. Merge commits are skipped. Dependabot signs off as <support@github.com> while
# authoring as its noreply address, so that one pairing is accepted.
# Usage: scripts/dco-check.sh <base> [head]
set -euo pipefail
base=${1:?usage: scripts/dco-check.sh <base> [head]}
head=${2:-HEAD}
dependabot='dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>'
# An assignment, unlike a for loop's word list, stops the script when git fails.
commits=$(git rev-list --no-merges "$base..$head") || { echo "dco-check: can't read $base..$head" >&2; exit 1; }
bad=0
n=0
for c in $commits; do
  n=$((n + 1))
  author=$(git log -1 --format='%an <%ae>' "$c")
  want=$author
  [[ "$author" == "$dependabot" ]] && want='dependabot[bot] <support@github.com>'
  if ! git log -1 --format='%(trailers:key=Signed-off-by,valueonly)' "$c" | sed 's/[[:space:]]*$//' | grep -qxF "$want"; then
    echo "error: $(git log -1 --format='%h %s' "$c") has no 'Signed-off-by: $want'" >&2
    bad=$((bad + 1))
  fi
done
if [[ $bad -gt 0 ]]; then
  echo "dco-check: $bad of $n commits need a sign-off (git rebase --signoff $base)" >&2
  exit 1
fi
echo "dco-check: ok ($n commits)"
