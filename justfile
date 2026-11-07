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

# Format, lint, and typecheck across Rust, TS, and Python, plus the SPDX headers, the app's tokens-only styles, and
# unsafe code only where D-50 allows it.
check:
    cargo fmt --all --check
    # Clippy compiles the desktop crate, which embeds apps/desktop/dist: a fresh checkout (a hosted runner) builds it.
    [[ -d apps/desktop/dist ]] || pnpm -r build
    cargo clippy --workspace --all-targets --locked -- -D warnings
    pnpm -r typecheck
    pnpm exec eslint .
    uv run --project {{ dsl }} --frozen ruff check {{ dsl }}
    uv run --project {{ dsl }} --frozen ruff format --check {{ dsl }}
    node scripts/spdx-check.mjs
    node scripts/style-lint.mjs
    node scripts/unsafe-check.mjs

# Build the JS packages, then the Rust workspace (the desktop crate embeds apps/desktop/dist).
build:
    pnpm -r build
    cargo build --workspace --locked

# A debug build of the desktop app with a release's custom protocol and embedded frontend, without bundling.
build-app:
    cd apps/desktop && pnpm tauri build --debug --no-bundle --ci

# Integration tests, fastest first. `just test --changed` runs only those affected by the paths changed since
# EMMET_CI_BASE or the merge-base with main (a toolchain or root config change runs them all); any other argument
# filters test names. It builds only when a suite needs the build. tests/it is one test binary, run in three passes:
# the suite; the technique library's 400-odd builds, only when a change can break them; and the three timing checks,
# alone and one at a time, so the suite's other processes don't slow them. Written for bash 3.2, macOS's /bin/bash.
test *args:
    #!/usr/bin/env bash
    set -euo pipefail
    args="{{ args }}"
    all=(scripts python viewer desktop rust)
    techniques=1
    if [[ "$args" == "--changed" ]]; then
      base=${EMMET_CI_BASE:-$(git merge-base HEAD origin/main 2>/dev/null || git rev-list --max-parents=0 HEAD)}
      # Separate steps, so a base that isn't a commit fails here instead of reading as "nothing changed".
      diff=$(git diff --name-only "$base")
      changed=$( { echo "$diff"; git ls-files --others --exclude-standard; } | sed '/^$/d' | sort -u)
      pick() { grep -qE "$1" <<<"$changed"; }
      if ! pick '^(justfile|\.node-version|rust-toolchain\.toml|rustfmt\.toml|package\.json|pnpm-(lock|workspace)\.yaml|tsconfig\.base\.json|eslint\.config\.js)$'; then
        all=()
        # tests/scripts/design.test.mjs also guards the app's tokens, glyphs, and icons (2B).
        if pick '^(scripts|tests/scripts|docs|\.githooks|\.github|deny\.toml)/?|^apps/desktop/(src/styles|src/assets/glyphs|src-tauri/icons)/|^apps/desktop/src-tauri/tauri\.conf\.json$'; then all+=(scripts); fi
        if pick '^python/'; then all+=(python); fi
        if pick '^packages/viewer/'; then all+=(viewer); fi
        if pick '^apps/desktop/'; then all+=(desktop); fi
        # The Rust integration tests also embed or read the DSL, data/, the fixtures, the fake agents and their
        # transcripts, and docs/spec/mcp/.
        if pick '^(crates/|tests/it/|tests/fixtures/|tests/fake-agents/|tests/replay/|python/emmet_dsl/src/|data/|docs/spec/mcp/|apps/desktop/src-tauri/|Cargo\.)'; then all+=(rust); fi
        # The techniques build and validate through the DSL and the engine, on ldraw-mini.
        pick '^(data/techniques/|python/emmet_dsl/src/|crates/emmet-core/|crates/emmet-cli/src/main\.rs$|tests/it/(src/|tests/techniques\.rs$)|tests/fixtures/ldraw-mini/)' || techniques=0
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
        rust)
          if [[ -n "$args" ]]; then
            cargo test --workspace --locked -- $args
          else
            timing="run::warm_run_is_fast search::search_is_fast library::verify_detects_a_changed_file_and_reindexes"
            skip=(--skip techniques::)
            for t in $timing; do skip+=(--skip "$t"); done
            cargo test --workspace --locked -- "${skip[@]}"
            # --workspace --test it, not -p emmet-it: the same features as the pass above, so nothing builds again (-p
            # picked other features for serde, tokio, and more, and rebuilt 60 crates: 5 minutes on hosted Windows).
            if [[ $techniques == 1 ]]; then cargo test --workspace --locked --test it -- techniques::; else echo "test: the techniques can't change here; skipped"; fi
            cargo test --workspace --locked --test it -- --test-threads=1 $timing
          fi ;;
      esac
    done

# App E2E (plan section 16, ADR 0002): a debug build with the embedded WebDriver server and the E2E hooks, driven by
# WebdriverIO. tests/e2e/tiers.mjs says which specs: `just e2e` runs core and every area, `just e2e all` adds the
# extra tier (by hand only), `just e2e viewer,chat` runs those, and `just e2e --changed` what the changes since main
# can break (as CI does). hello runs first on its own, and when it fails nothing else runs. The app is built inside
# the workspace build (it embeds the frontend by default), so after `just test` only the webdriver feature's crates
# and the app build again.
e2e *spec: e2e-build
    #!/usr/bin/env bash
    set -euo pipefail
    arg="{{ spec }}"
    if [[ "$arg" == --changed ]]; then
      base=${EMMET_CI_BASE:-$(git merge-base HEAD origin/main 2>/dev/null || git rev-list --max-parents=0 HEAD)}
      diff=$(git diff --name-only "$base")
      specs=$( { echo "$diff"; git ls-files --others --exclude-standard; } | node tests/e2e/tiers.mjs --changed)
    else
      specs=$(node tests/e2e/tiers.mjs "$arg")
    fi
    [[ "$arg" == all ]] && export EMMET_E2E_ALL=1
    if [[ -z "$specs" ]]; then echo "e2e: nothing to run"; exit 0; fi
    echo "e2e: $specs"
    # WebKit draws nothing on a locked screen or a sleeping display, and every spec would time out one by one: say so
    # now. Then keep the display awake for the run (caffeinate -u wakes it).
    if [[ "$(uname -s)" == Darwin ]]; then
      if ioreg -n Root -d1 -a | grep -A1 CGSSessionScreenIsLocked | grep -q '<true/>'; then
        echo "e2e: this Mac's screen is locked, and WebKit doesn't draw on a locked screen. Unlock it and run again." >&2
        exit 1
      fi
      caffeinate -u -t 1
      caffeinate -d -i -w $$ &
    fi
    run() { pnpm --filter @emmet/e2e exec wdio run wdio.conf.mjs "$@"; }
    shared=(); first_run=0
    for s in ${specs//,/ }; do
      case $s in
        hello) ;;
        first-run) first_run=1 ;;
        *) shared+=(--spec "specs/$s.e2e.mjs") ;;
      esac
    done
    EMMET_E2E_GATE=1 run --spec specs/hello.e2e.mjs || {
      echo "e2e: hello failed, so this app or machine can't run E2E (see its error); the other specs didn't run" >&2
      exit 1
    }
    if [[ ${#shared[@]} -gt 0 ]]; then run "${shared[@]}"; fi
    # First run starts the app on its own fresh data dir (wdio.conf.mjs), so it's a run of its own.
    if [[ $first_run == 1 ]]; then EMMET_E2E_FIRST_RUN=1 run; fi

# The E2E build: the frontend with EMMET_E2E=1, then the workspace with emmet-desktop's `webdriver` feature (never in
# a release build; main.rs refuses it without debug assertions).
e2e-build:
    pnpm --filter @emmet/viewer build
    cd apps/desktop && EMMET_E2E=1 pnpm build
    cargo build --workspace --locked --features emmet-desktop/webdriver

# What CI's Mac jobs run, scoped by scripts/ci-scope.sh to what the changes since main can break. `just ci checks` (the
# mac job): lint and secrets always, the affected tests, the audits when deny.toml changes, and the release-protocol
# build when the native side changes. `just ci e2e` (the mac-e2e job, beside it): the E2E specs tests/e2e/tiers.mjs
# picks. `just ci` runs both. CI runs it, so there is no need to run it locally before a push. EMMET_CI_FULL=1 runs
# everything a PR can, as CI does weekly. The bench and the extra E2E tier are by hand only.
ci part="":
    #!/usr/bin/env bash
    set -euo pipefail
    part="{{ part }}"
    case "$part" in "" | checks | e2e) ;; *) echo "ci: no part $part (checks or e2e)" >&2; exit 1 ;; esac
    eval "$(scripts/ci-scope.sh)"
    echo "ci scope: full=$full deps=$deps build_app=$build_app e2e=${e2e_specs:-none}"
    if [[ "$part" != e2e ]]; then
      just check
      gitleaks git --redact --no-banner .
      if [[ $deps == true ]]; then cargo deny check && pnpm audit --prod; fi
      if [[ $full == true ]]; then just test; else just test --changed; fi
      if [[ $build_app == true ]]; then just build-app; fi
    fi
    if [[ "$part" != checks && $e2e_mac == true ]]; then just e2e "$e2e_specs"; fi
    echo "ci${part:+ $part}: ok"

# A live eval (plan section 16.1): `just eval --rung 1 --agent claude` (or codex). One run through the real CLI on
# the eval API key from the keychain (H-21, D-37); it says what it bills and asks first (D-38), then writes
# docs/evals/<date>-rung1.md. Quit the app first: the eval answers on its bus. The build puts the app next to emmet for
# the render.
eval *args: build
    target/debug/emmet eval {{ args }}

# Not yet: light and dark screenshots of key screens, for PRs and the guide.
screenshots:
    @echo "just screenshots: not yet; session 6B adds it" && exit 1

# The benches: `just bench` runs both, `just bench viewer` or `just bench validator` one. The viewer bench (plan
# section 13) runs in the real app: first load, slider → viewer, and orbit frame time for 93, 500, and 2,000 parts,
# written to tests/e2e/out/bench-<os>.md. The validator bench (plan section 11) validates 500- and 2,000-part models,
# warm, in a release build, fails over its budget, and writes tests/e2e/out/bench-validator-<os>.md.
bench *what:
    #!/usr/bin/env bash
    set -euo pipefail
    what="{{ what }}"
    if [[ -z "$what" || "$what" == viewer ]]; then
      just e2e-build
      EMMET_BENCH=1 pnpm --filter @emmet/e2e exec wdio run wdio.conf.mjs
    fi
    if [[ -z "$what" || "$what" == validator ]]; then
      cargo build --release --locked -p emmet-cli
      cargo test --release --locked -p emmet-it --test it -- --ignored validator_bench:: --nocapture
    fi

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
