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

# Tauri dev with Vite HMR, under the dev bundle id so it never touches real app data. `just dev <project>` opens it.
dev *project:
    cd apps/desktop && pnpm tauri dev --config src-tauri/tauri.dev.conf.json -- --no-default-features -- {{ project }}

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

# Integration tests, fastest first. `just test --changed` runs only those affected by the paths changed since
# EMMET_CI_BASE or the merge-base with main (a toolchain or root config change runs them all); any other argument
# filters test names. It builds only when a suite needs the build. Written for bash 3.2, macOS's /bin/bash.
test *args:
    #!/usr/bin/env bash
    set -euo pipefail
    args="{{ args }}"
    all=(scripts python viewer desktop rust)
    if [[ "$args" == "--changed" ]]; then
      base=${EMMET_CI_BASE:-$(git merge-base HEAD origin/main 2>/dev/null || git rev-list --max-parents=0 HEAD)}
      # Separate steps, so a base that isn't a commit fails here instead of reading as "nothing changed".
      diff=$(git diff --name-only "$base")
      changed=$( { echo "$diff"; git ls-files --others --exclude-standard; } | sed '/^$/d' | sort -u)
      pick() { grep -qE "$1" <<<"$changed"; }
      if ! pick '^(justfile|\.node-version|rust-toolchain\.toml|rustfmt\.toml|package\.json|pnpm-(lock|workspace)\.yaml|tsconfig\.base\.json|eslint\.config\.js)$'; then
        all=()
        if pick '^(scripts|tests/scripts|docs|\.githooks|\.github|deny\.toml)/?'; then all+=(scripts); fi
        if pick '^python/'; then all+=(python); fi
        if pick '^packages/viewer/'; then all+=(viewer); fi
        if pick '^apps/desktop/'; then all+=(desktop); fi
        # The Rust integration tests also embed or read the DSL, data/, the fixtures, the fake agents and their
        # transcripts, and docs/spec/mcp/.
        if pick '^(crates/|tests/it/|tests/fixtures/|tests/fake-agents/|tests/replay/|python/emmet_dsl/src/|data/|docs/spec/mcp/|apps/desktop/src-tauri/|Cargo\.)'; then all+=(rust); fi
      fi
      args=""
      echo "test --changed: ${all[*]:-nothing}"
    fi
    # The same steps as `just build`: the desktop crate embeds apps/desktop/dist, and the JS tests read the builds.
    if grep -qE 'viewer|desktop|rust' <<<"${all[*]:-}"; then
      pnpm -r build
      cargo build --workspace --locked
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

# App E2E (plan section 16, ADR 0002): a debug build with the embedded WebDriver server and the E2E hooks, driven by
# WebdriverIO. `just e2e viewer` runs one spec file from tests/e2e/specs/. The app is built inside the workspace build
# (it embeds the frontend by default), so after `just test` only the webdriver feature's crates and the app build again.
e2e *spec: e2e-build
    pnpm --filter @emmet/e2e exec wdio run wdio.conf.mjs {{ if spec == "" { "" } else { "--spec specs/" + spec + ".e2e.mjs" } }}

# The E2E build: the frontend with EMMET_E2E=1, then the workspace with emmet-desktop's `webdriver` feature (never in
# a release build; main.rs refuses it without debug assertions).
e2e-build:
    pnpm --filter @emmet/viewer build
    cd apps/desktop && EMMET_E2E=1 pnpm build
    cargo build --workspace --locked --features emmet-desktop/webdriver

# What CI's mac job runs, scoped by scripts/ci-scope.sh to what the changes since main can break: lint and secrets
# always; the affected tests; E2E, the release-protocol build, and the audits only when their paths changed. CI runs it,
# so there is no need to run it locally before a push. EMMET_CI_FULL=1 runs everything, as CI does weekly.
ci:
    #!/usr/bin/env bash
    set -euo pipefail
    eval "$(scripts/ci-scope.sh)"
    echo "ci scope: full=$full deps=$deps e2e=$e2e_mac build_app=$build_app bench=$bench"
    just check
    gitleaks git --redact --no-banner .
    if [[ $deps == true ]]; then cargo deny check && pnpm audit --prod; fi
    if [[ $full == true ]]; then just test; else just test --changed; fi
    if [[ $build_app == true ]]; then just build-app; fi
    if [[ $e2e_mac == true ]]; then just e2e; fi
    if [[ $bench == true ]]; then just bench; fi
    echo "ci: ok"

# A live eval (plan section 16.1): `just eval --rung 1 --agent claude` (or codex). One run through the real CLI on
# the eval API key from the keychain (H-21, D-37); it says what it bills and asks first (D-38), then writes
# docs/evals/<date>-rung1.md. Quit the app first: the eval answers on its bus. The build puts the app next to emmet for
# the render.
eval *args: build
    target/debug/emmet eval {{ args }}

# Not yet: light and dark screenshots of key screens, for PRs and the guide.
screenshots:
    @echo "just screenshots: not yet; session 6B adds it" && exit 1

# The viewer bench (plan section 13) in the real app: first load, slider → viewer, and orbit frame time for 93,
# 500, and 2,000 parts. Prints the table and writes tests/e2e/out/bench-<os>.md. The validator bench arrives in 3A.
bench: e2e-build
    EMMET_BENCH=1 pnpm --filter @emmet/e2e exec wdio run wdio.conf.mjs

# A release build without a release (scripts/release-dry.sh): a universal .dmg on macOS (--sign: Developer ID,
# notarized and stapled, H-05), the unsigned NSIS installer on Windows.
release-dry *args:
    scripts/release-dry.sh {{ args }}

# Not yet: the version-bump PR helper arrives in 6A.
release version:
    @echo "just release {{ version }}: not yet; session 6A adds it" && exit 1

# Open the plan and check its status file.
plan:
    node scripts/plan-check.mjs
    open docs/plan.html 2>/dev/null || start docs/plan.html
