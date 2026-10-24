#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Fails when a commit in <base>..<head> lacks a Signed-off-by trailer that matches its author (the DCO, CONTRIBUTING.md).
# Usage: scripts/dco-check.sh <base> [head]
set -euo pipefail
base=${1:?usage: scripts/dco-check.sh <base> [head]}
head=${2:-HEAD}
bad=0
n=0
for c in $(git rev-list --no-merges "$base..$head"); do
  n=$((n + 1))
  author=$(git log -1 --format='%an <%ae>' "$c")
  if ! git log -1 --format='%(trailers:key=Signed-off-by,valueonly)' "$c" | sed 's/[[:space:]]*$//' | grep -qxF "$author"; then
    echo "error: $(git log -1 --format='%h %s' "$c") has no 'Signed-off-by: $author'" >&2
    bad=$((bad + 1))
  fi
done
if [[ $bad -gt 0 ]]; then
  echo "dco-check: $bad of $n commits need a sign-off (git rebase --signoff $base)" >&2
  exit 1
fi
echo "dco-check: ok ($n commits)"
