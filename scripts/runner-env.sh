#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# H-02: writes the toolchain PATH the runner service needs into <runner dir>/.path. Run it from a shell where
# `just setup` passes, then restart the service.
# Usage: scripts/runner-env.sh [runner dir, default ~/actions-runner/emmet]
set -euo pipefail
dir=${1:-$HOME/actions-runner/emmet}
[[ -d "$dir" ]] || { echo "runner-env: $dir doesn't exist; register the runner first (H-02)" >&2; exit 1; }

# The real directory of a tool, skipping version-manager shims that need the user's shell setup.
bin_dir() {
  local p
  if [[ $1 == node ]]; then
    p=$(node -p 'process.execPath') || return 1
  else
    p=$(type -ap "$1" | grep -v '/shims/' | head -n 1) || true
    [[ -n "$p" ]] || return 1
  fi
  dirname "$p"
}

dirs=()
for tool in cargo node pnpm uv just gitleaks cargo-deny gh; do
  d=$(bin_dir "$tool") || { echo "runner-env: $tool isn't on PATH; see the README's setup" >&2; exit 1; }
  dirs+=("$d")
done
dirs+=(/opt/homebrew/bin /usr/local/bin /usr/bin /bin /usr/sbin /sbin)
path=$(printf '%s\n' "${dirs[@]}" | awk '!seen[$0]++' | paste -sd: -)
printf '%s\n' "$path" >"$dir/.path"
echo "runner-env: wrote $dir/.path:"
echo "$path"
echo "Now restart the service: cd $dir && ./svc.sh stop && ./svc.sh start"
