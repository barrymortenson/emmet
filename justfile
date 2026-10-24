# SPDX-License-Identifier: AGPL-3.0-or-later
# Every command in plan section 18. On Windows, run just from Git Bash.

set shell := ["bash", "-euo", "pipefail", "-c"]
set windows-shell := ["bash", "-euo", "pipefail", "-c"]

dsl := "python/emmet_dsl"

# List the recipes.
default:
    @just --list

# Check the pinned toolchain, install the git hooks, and install dependencies into the repo.
setup: tools deps
    git config core.hooksPath .githooks
    @echo "setup: ok"

# Fail with a hint for each missing tool, or a Node that doesn't match .node-version. The only installs outside the
# repo are the pinned Rust toolchain (through rustup) and, at deps, uv's Python.
tools:
    #!/usr/bin/env bash
    set -euo pipefail
    bad=0
    need() { command -v "$1" >/dev/null || { echo "missing: $1 ($2)"; bad=1; }; }
    need rustup "https://rustup.rs"
    need node "Node $(cat .node-version) from your version manager or nodejs.org"
    need pnpm "https://pnpm.io/installation; the version comes from package.json"
    need uv "brew install uv, or https://docs.astral.sh/uv/"
    need gitleaks "brew install gitleaks"
    need cargo-deny "cargo install --locked cargo-deny"
    [[ $bad == 0 ]] || exit 1
    want="v$(cat .node-version)"
    [[ "$(node --version)" == "$want" ]] || { echo "node is $(node --version), needs $want (.node-version)"; exit 1; }
    rustup toolchain install >/dev/null
    echo "tools: ok (rust $(rustc --version | cut -d' ' -f2), node $want, pnpm $(pnpm --version), $(uv --version | cut -d' ' -f1-2))"

# Install the JS and Python dependencies from the lockfiles.
deps:
    pnpm install --frozen-lockfile
    uv sync --project {{ dsl }} --frozen

# Tauri dev with Vite HMR, under the dev bundle id so it never touches real app data.
dev:
    cd apps/desktop && pnpm tauri dev --config src-tauri/tauri.dev.conf.json

# Format, lint, and typecheck across Rust, TS, and Python, plus the SPDX headers.
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    pnpm -r typecheck
    pnpm exec eslint .
    uv run --project {{ dsl }} --frozen ruff check {{ dsl }}
    uv run --project {{ dsl }} --frozen ruff format --check {{ dsl }}
    node scripts/spdx-check.mjs

# Build the JS packages, then the Rust workspace (the desktop crate embeds apps/desktop/dist).
build:
    pnpm -r build
    cargo build --workspace --locked

# A debug build of the desktop app with a release's custom protocol and embedded frontend, without bundling.
build-app:
    cd apps/desktop && pnpm tauri build --debug --no-bundle --ci

# Integration tests, fastest first. `just test --changed` runs only those affected by changed paths (a toolchain or
# root config change runs them all); any other argument filters test names. Written for bash 3.2, macOS's /bin/bash.
test *args: build
    #!/usr/bin/env bash
    set -euo pipefail
    args="{{ args }}"
    all=(scripts python viewer desktop rust)
    if [[ "$args" == "--changed" ]]; then
      base=$(git merge-base HEAD origin/main 2>/dev/null || git rev-list --max-parents=0 HEAD)
      changed=$( { git diff --name-only "$base"; git ls-files --others --exclude-standard; } | sort -u)
      pick() { grep -qE "$1" <<<"$changed"; }
      if ! pick '^(justfile|\.node-version|rust-toolchain\.toml|rustfmt\.toml|package\.json|pnpm-(lock|workspace)\.yaml|tsconfig\.base\.json|eslint\.config\.js)$'; then
        all=()
        if pick '^(scripts|tests/scripts|docs|\.githooks|\.github|deny\.toml)/?'; then all+=(scripts); fi
        if pick '^python/'; then all+=(python); fi
        if pick '^packages/viewer/'; then all+=(viewer); fi
        if pick '^apps/desktop/'; then all+=(desktop); fi
        if pick '^(crates/|tests/it/|apps/desktop/src-tauri/|Cargo\.)'; then all+=(rust); fi
      fi
      args=""
      echo "test --changed: ${all[*]:-nothing}"
    fi
    # Empty arrays expand with ${a[@]+...}: bash 3.2 calls a bare "${a[@]}" unbound under set -u.
    node_filter=(); py_filter=(); [[ -n "$args" ]] && node_filter=(--test-name-pattern "$args") && py_filter=(-k "$args")
    # node --test passes with 0 tests when its glob matches nothing, so a moved test dir fails here instead.
    tests_in() { compgen -G "$1" >/dev/null || { echo "test: no tests match $1" >&2; exit 1; }; }
    for suite in ${all[@]+"${all[@]}"}; do
      case $suite in
        # The script and gate tests need bash, gitleaks, and cargo-deny. The Mac job runs them all; the Linux plan
        # job runs the ones that don't (ci.yml).
        scripts)
          case "$(uname -s)" in
            MINGW* | MSYS* | CYGWIN*) echo "test: skipping tests/scripts on Windows" ;;
            *) tests_in "tests/scripts/*.test.mjs" && node --test ${node_filter[@]+"${node_filter[@]}"} "tests/scripts/*.test.mjs" ;;
          esac ;;
        python) uv run --project {{ dsl }} --frozen python -m unittest discover -s {{ dsl }}/tests ${py_filter[@]+"${py_filter[@]}"} ;;
        # --fail-if-no-match: without it a renamed package is a silent pass.
        viewer) tests_in "packages/viewer/test/*.test.mjs" && pnpm --filter @emmet/viewer --fail-if-no-match exec node --test ${node_filter[@]+"${node_filter[@]}"} "test/*.test.mjs" ;;
        desktop) tests_in "apps/desktop/test/*.test.mjs" && pnpm --filter @emmet/desktop --fail-if-no-match exec node --test ${node_filter[@]+"${node_filter[@]}"} "test/*.test.mjs" ;;
        rust) cargo test --workspace --locked -- $args ;;
      esac
    done

# Not yet: 1B sets up the WebdriverIO harness and its first specs.
e2e *spec:
    @echo "just e2e: not yet; session 1B adds it" && exit 1

# Exactly what CI's mac job runs. It must pass before a session's final push.
ci: check
    cargo deny check
    gitleaks git --redact --no-banner .
    pnpm audit --prod
    just test
    just build-app
    @echo "ci: ok"

# Not yet: live evals (section 16.1) arrive in 1D; they bill Barry's API keys and ask first.
eval *args:
    @echo "just eval: not yet; session 1D adds it (D-38)" && exit 1

# Not yet: light and dark screenshots of key screens, for PRs and the guide.
screenshots:
    @echo "just screenshots: not yet; session 6B adds it" && exit 1

# Not yet: the viewer bench comes in 1B, the validator bench in 3A.
bench:
    @echo "just bench: not yet; sessions 1B and 3A add it" && exit 1

# Not yet: the signed demo build (release-dry --sign) comes in 1D.
release-dry *args:
    @echo "just release-dry: not yet; session 1D adds it" && exit 1

# Not yet: the version-bump PR helper arrives in 6A.
release version:
    @echo "just release {{ version }}: not yet; session 6A adds it" && exit 1

# Open the plan and check its status file.
plan:
    node scripts/plan-check.mjs
    open docs/plan.html 2>/dev/null || start docs/plan.html
