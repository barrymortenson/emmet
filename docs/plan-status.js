// SPDX-License-Identifier: AGPL-3.0-or-later
// Execution status for docs/plan.html (plan section 20). Agents edit this on their session branch;
// scripts/plan-check.mjs validates it in CI. Never record progress by editing plan.html.
// Session status: todo | in-progress | review | done | blocked | skipped (Barry only) | backlog (Phase 8).
// done needs pr and at least one evidence line; skipped needs Barry's note; blocked needs an open blocker.
// Human status:   todo | waiting | done | not-needed
// Gate status:    todo | go | no-go | pivot
// A session Barry let start before its "after" sessions were done carries override: "<Barry's note>", which turns
// plan-check's error into a warning.
window.EMMET_STATUS = {
  updated: "2026-10-29",
  sessions: {
    "0A": {
      status: "done", pr: 1, branch: "bmortenson/s0a-foundation", started: "2026-10-24",
      evidence: [
        "just ci on this Mac: ci: ok",
        "CI ci check green #1 (mac on self-hosted emmet-mac, windows, plan)",
        "tests/it/tests/cli.rs, tests/scripts/{plan-check,gates,spdx-dco,ci,runner,justfile}.test.mjs"
      ],
      note: "1A and 1C must use the test HOME and data dir the runner guard sets; CLAUDE.md is a symlink, a text file on Windows without core.symlinks"
    },
    "0B": {
      status: "done", pr: 3, branch: "bmortenson/s0b-research-adrs", started: "2026-10-24",
      evidence: [
        "just ci on this Mac: ci: ok",
        "CI ci check green #3 (run 40154710641: mac on emmet-mac, windows, plan)",
        "docs/adr/0001-0006, tests/replay/{claude,codex}/, tests/fixtures/ldraw-mini/, docs/spec/ladder.md",
        "tests/e2e/specs/hello.e2e.mjs, tests/scripts/{scrub-transcript,ldraw-mini-check}.test.mjs"
      ],
      note: "Windows part loading in the render spike is slow (about 9 s for 5 parts) and unmeasured; 1B's first-load budget must check it (ADR 0005)"
    },
    "1A": {
      status: "done", pr: 9, branch: "bmortenson/s1a-engine-skeleton", started: "2026-10-24",
      evidence: [
        "just ci on this Mac: ci: ok",
        "CI ci check green #9 (run 40164346894: mac on emmet-mac, windows, plan)",
        "tests/it/tests/{run,errors,search,mcp,library,frame,runner}.rs; python/emmet_dsl/tests/test_stdlib_only.py",
        "warm run 96 ms, ldraw-mini reindex 0.092 s, full library reindex 4.05 s, search 0.056 ms a query"
      ],
      note: "LDView and LeoCAD fail Gatekeeper, so Emmet's three.js render stood in for the screenshot (Barry); a fresh library install fails its checksum once LDraw publishes 2026-10 until Emmet pins it (ADR 0007, 6A)"
    },
    "1B": {
      status: "done", pr: 11, branch: "bmortenson/s1b-app-shell-viewer", started: "2026-10-25",
      evidence: [
        "just ci on this Mac: ci: ok",
        "CI ci check green #11 (run 40224311960: mac on emmet-mac, windows, plan); E2E 6/6 spec files on both OSes",
        "tests/e2e/specs/{viewer,leak,render,runs,visual}.e2e.mjs; apps/desktop/src-tauri/tests/{paths,runs,watcher,bindings}.rs; tests/it/tests/{render,bus}.rs",
        "bench (ADR 0008): cottage first load 691 ms, slider → viewer 501 ms, orbit 13.9 ms on macOS"
      ],
      note: "Windows first load 7-8 s on CI's software WebGL misses section 13; check on real hardware (H-04, 1C)"
    },
    "1C": {
      status: "done", pr: 13, branch: "bmortenson/s1c-agent-bridge", started: "2026-10-25",
      evidence: [
        "CI ci check green #13 (run 40259994869: mac, windows with E2E, plan)",
        "tests/it/tests/{agent_claude,agent_codex,agent_sentinel}.rs; tests/e2e/specs/chat.e2e.mjs (15 tests, both fake agents)",
        "docs/spec/sessions/1C/live-mac.mp4, badges-{plan,api-key}.png, windows-*.png (H-04)",
        "chat event → DOM median 1 ms, p95 5 ms on macOS"
      ],
      note: "Barry's design and feature notes from the live check go to docs/design/notes.md in a small PR, for 2A"
    },
    "1D": {
      status: "done", pr: 17, branch: "bmortenson/s1d-close-loop-signed-build", started: "2026-10-25",
      evidence: [
        "CI ci check green #17 (run 40393047071: mac with E2E and bench, windows with E2E, nsis, plan)",
        "tests/it/tests/{check,validator_corpus,eval}.rs; tests/e2e/specs/checks.e2e.mjs on both OSes",
        "docs/evals/2026-10-26-rung1.md: rung 1 passes on Claude Code and Codex, each after one rerun (D-38)",
        "H-05: notarized .dmg, spctl and stapler pass; docs/spec/sessions/1D/fresh-account.png"
      ],
      note: "G1 (H-06) is Barry's decision from the eval report and his own testing"
    },
    "1E": {
      status: "done", pr: 21, branch: "bmortenson/s1e-chatgpt-plan-signin", started: "2026-10-26",
      evidence: [
        "CI ci check green #21 (run 40416376710: mac, windows with E2E, nsis, plan)",
        "tests/it/tests/{chatgpt,agent_codex}.rs (20 and 31 passed); tests/e2e/specs/chatgpt.e2e.mjs (6 tests)",
        "docs/spec/sessions/1E.md: 39 OpenAI requirements, each mapped to a test or screenshot",
        "Barry signed in on his ChatGPT plan and built rung 1 by hand: docs/spec/sessions/1E/plan-rung1.png"
      ],
      note: "6B's diagnostics bundle must join the no-token tests; a token split across two streamed deltas isn't caught by per-event redaction (ADR 0011)"
    },
    "1F": {
      status: "done", pr: 22, branch: "bmortenson/s1f-agent-setup-no-terminal", started: "2026-10-26",
      evidence: [
        "CI ci check green #22 (run 40448695024: mac with the full E2E suite, 63 tests, and the bench; windows paused, D-43)",
        "tests/it/tests/{setup,agent_sentinel}.rs; tests/e2e/specs/setup.e2e.mjs",
        "Barry's macOS fresh-account run on the notarized build: docs/spec/sessions/1F/mac-1..7-*.png",
        "docs/adr/0012-agent-setup.md; docs/terms/quotes.json (terms-check: fetched pages ok)"
      ],
      note: "Barry 2026-10-26 (D-43): merged without the Windows fresh-account run and Windows CI, which come back in the Windows pass; Antigravity is out of the MVP"
    },
    "2A": {
      status: "done", pr: 23, branch: "bmortenson/s2a-ux-research-directions", started: "2026-10-26",
      evidence: [
        "CI ci check green #23 (run 40618957958: mac with the full E2E suite, plan with the contrast gate; windows paused, D-43)",
        "docs/design/{research,storyboards}.md, directions/{a,b,c}/ (36 screens and states each), review.html, brand/",
        "scripts/contrast-check.mjs (contrast-check: ok) and tests/scripts/contrast-check.test.mjs; each #audit 0 failures",
        "H-07: B (Canvas), as revised; docs/decisions.md"
      ],
      note: "The Windows cottage visual baseline is stale until the Windows pass regenerates it (D-43); 2B can raise A's Chat focus layout and C's ⌘K palette again"
    },
    "2B": {
      status: "done", pr: 24, branch: "bmortenson/s2b-design-system-frozen", started: "2026-10-27",
      evidence: [
        "CI ci check green #24 (run 40626685728: mac, plan; windows and nsis paused, D-43)",
        "apps/desktop/src/styles/tokens.css, docs/design/{components,keyboard,content,icons}.md, frozen/ (47 screens), SIGNOFF.md",
        "tests/scripts/design.test.mjs (7 pass); frozen audit 376 renders, 0 failures; contrast-check: ok",
        "H-08: signed as is; docs/decisions.md"
      ],
      note: "The Steps tab is the model's tree and there's no command palette (docs/decisions.md); the Steps tree's every-copy highlight isn't drawn, since emmet render --highlight drew none for window ids (4C)"
    },
    "3A": {
      status: "done", pr: 30, branch: "bmortenson/s3a-validator-connection-model", started: "2026-10-27",
      evidence: [
        "CI ci check green #30 (run 40729470945: mac, plan; windows and nsis paused, D-43); nightly run 40729470833: 33 models, 0 errors",
        "tests/it/tests/{check,validator_corpus,validator_mutations,validator_bench}.rs; replay in agent_{claude,codex}.rs",
        "mutation recall 98.2 % (PR gate), 96.0 % (nightly); bench 16 ms for 510 parts, 65 ms for 2,078",
        "docs/adr/0013-collision-proxies.md, docs/spec/validator.md, D-44 to D-47"
      ],
      note: "Open nightly gaps are #25 to #29; Allow.rule and [build] age as enums and copies consistency are left for 3B; floating recall by the rule itself is 88.3 % nightly"
    },
    "3B": {
      status: "done", pr: 32, branch: "bmortenson/s3b-dsl-steps-bom-data", started: "2026-10-28",
      evidence: [
        "CI ci check green #32 (run 40767714403: mac with the full E2E suite and the bench, plan; windows and nsis paused, D-43)",
        "tests/it/tests/{dsl,bom,techniques,ladder,parts_data}.rs; python/emmet_dsl/tests/test_write_default.py",
        "BrickLink verified all 29 items and Rebrickable imported 29 lines, 154 pieces: docs/spec/sessions/3B/",
        "data/ladder/rung2_reference.py: 188 lines, 9 params, no findings; 20 techniques in data/techniques/"
      ],
      note: "The macOS E2E app hung intermittently on CI in three runs, cause unknown; holds.rs's filler() matches \"Shutter\" in 60594 (validator follow-up); the LPub3D export hasn't been opened in LPub3D"
    },
    "3C": {
      status: "done", pr: 33, branch: "bmortenson/s3c-technique-mining", started: "2026-10-28",
      evidence: [
        "CI ci check green #33; Phase 3 exit check in the PR description",
        "tests/it/tests/techniques.rs (120 passed: one per technique, plus the doctrine and coverage tests); dsl.rs::walls_are_laid_in_bond",
        "98 of 100 candidates accepted (H-22), 118 techniques in all; miner tests/it/src/bin/mine-techniques.rs over 1,318 OMR sets",
        "docs/adr/0003-licenses-and-reuse.md, D-48, D-49"
      ],
      note: "Techniques aren't derived from OMR models (D-48); the click-hinge collision at nonzero angles is a likely validator false positive, unfiled pending Barry's OK; the inputs hash ignores DSL content between dev builds"
    },
    "4A": {
      status: "done", pr: 34, branch: "bmortenson/s4a-shell-first-run", started: "2026-10-28",
      evidence: [
        "CI ci check green #34 (run 40925871399: mac with the full E2E suite, plan; windows and nsis paused, D-43)",
        "tests/e2e/specs/{screens,first-run,trust,import,panel,versions,keyboard}.e2e.mjs; tests/it/tests/versions.rs",
        "52 macOS side-by-sides in docs/spec/sessions/4A/side-by-side/; design change note 001 approved 2026-10-29",
        "first run 15 s to the built Cottage on library.ldraw.org; warm start interactive at 420 ms; style-lint: ok"
      ],
      note: "Barry 2026-10-29 (D-43): merged with the 'both OSes' items met on macOS; the Windows pass checks first run, trust, import, panel, the side-by-sides, and the title-row menus. A version without build placements can't be restored from the app yet (a design question)"
    },
    "4B": { status: "todo" },
    "4C": { status: "todo" },
    "4D": { status: "todo" },
    "4E": { status: "todo" },
    "4F": { status: "skipped", note: "Barry 2026-10-26 (D-43): Antigravity, Cursor, and Grok are out of the MVP; they come after it (Phase 8)" },
    "5A": { status: "todo" },
    "5B": { status: "todo" },
    "6A": { status: "todo" },
    "6B": { status: "todo" },
    "7A": { status: "todo" },
    "7B": { status: "todo" },
    "8D": { status: "backlog" },
    "8E": { status: "backlog" },
    "8F": { status: "backlog" },
    "8G": { status: "backlog" },
    "8H": { status: "backlog" },
    "8I": { status: "backlog" }
  },
  human: {
    "H-01": { status: "done", date: "2026-10-24", note: "repo created public; settings applied and read back in 0A" },
    "H-02": { status: "done", date: "2026-10-24", note: "runner emmet-mac registered by the agent with approval; PR #1 mac job green on it" },
    "H-03": { status: "done", date: "2026-10-24", note: "Claude Code 2.1.310 on Barry's plan; Codex 0.160.1 logged in using ChatGPT" },
    "H-04": { status: "done", date: "2026-10-25", note: "Windows 11 ARM VM (Parallels); Claude Code (native installer, Max plan) and Codex 0.164.0 (npm, API key) both edited the model from Emmet; ADR 0001's Windows section" },
    "H-05": { status: "done", date: "2026-10-26", note: "Developer ID Application certificate and the emmet-notary notarytool profile in the login keychain; just release-dry --sign: notarization Accepted, spctl accepted (Notarized Developer ID), stapler validate worked; the .dmg opened, set up, and built the cottage in a fresh Standard macOS account" },
    "H-06": { status: "done", date: "2026-10-26", note: "Rung 1 passed on both agents, each after one rerun (docs/evals/2026-10-26-rung1.md); Barry called G1 a go. terms-check and name-check weren't run for this call" },
    "H-07": { status: "done", date: "2026-10-27", note: "B (Canvas), as revised with Barry in 2A; asked through AskUserQuestion; docs/decisions.md" },
    "H-08": { status: "done", date: "2026-10-27", note: "Barry signed docs/design/SIGNOFF.md as is (47 screens, 49 components, the keyboard map, the content guide, the icons); asked through AskUserQuestion" },
    "H-09": { status: "todo" },
    "H-10": { status: "todo" },
    "H-11": { status: "done", date: "2026-10-24", note: "6743-1 Street Speeder, from the 0B shortlist (docs/spec/ladder.md)" },
    "H-12": { status: "todo" },
    "H-13": { status: "todo" },
    "H-14": { status: "not-needed", note: "releases publish from the main public repo (D-21)" },
    "H-15": { status: "todo", note: "emmetbuilder.com registered at Cloudflare 2026-10-24; API token still to do" },
    "H-16": { status: "done", note: "AGPL-3.0-or-later; Apache-2.0 for the DSL (2026-10-24)" },
    "H-17": { status: "done", note: "best-judgment search 2026-10-24; keep Emmet (D-30)" },
    "H-18": { status: "not-needed", note: "public from day one (D-12)" },
    "H-19": { status: "todo" },
    "H-20": { status: "done", date: "2026-10-24", note: "Commercial Terms accepted through the Console account; legal page read 2026-10-24. No description to Anthropic (Barry 2026-10-25, D-37)" },
    "H-21": { status: "done", date: "2026-10-24", note: "keychain items emmet-eval-anthropic and emmet-eval-openai exist, with spend caps set" },
    "H-22": { status: "done", date: "2026-10-28", note: "98 of 100 candidates accepted on five review pages (docs/techniques/candidates/batch-1 to 5); rejected: wheel_axle, wheel_arch; docs/decisions.md" },
    "H-23": { status: "todo" },
    "H-24": { status: "not-needed", note: "no note to Rebrickable; Q-14 settled by reading the Terms (Barry 2026-10-24)" },
    "H-25": { status: "not-needed", note: "Antigravity is out of the MVP (Barry 2026-10-26, D-43)" }
  },
  blockers: [],
  gates: {
    "G1": { status: "go", note: "Barry 2026-10-26: \"mark g1 as a go.\"" },
    "G2": { status: "todo", note: "" }
  }
};
