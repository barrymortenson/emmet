# Emmet

Emmet is a Tauri 2 desktop app (macOS + Windows) where the user's own Claude Code or Codex agent builds brick
models from real LDraw parts by writing a parametric Python model (emmet_dsl). The plan is docs/plan.html; progress
is docs/plan-status.js. Section 19 of the plan has the agent rules; they apply to every task here.

## Commands
just setup · just dev · just check · just test [--changed] · just e2e · just ci (must pass before the final push)

## Map
crates/emmet-core (LDraw, index, validator, BOM, steps) · crates/emmet-agent (Claude/Codex adapters) ·
crates/emmet-bus (local socket) · crates/emmet-cli (emmet + emmet mcp) · apps/desktop (Tauri host + React UI) ·
packages/viewer (three.js) · python/emmet_dsl · tests/it (integration) · tests/e2e (WebdriverIO) · docs/

## Invariants
- Project files are the source of truth; build/ is derived and reproducible.
- Never write to ~/.claude, ~/.codex or any user-level agent config.
- Integration tests first; unit tests only for dense logic; never weaken a test to get green.
- No new dependency or tool without approval (plan section 19, rule 6).
- Every IPC path is checked against the project root; all process spawning is in Rust.
- No secrets, real names, emails or personal paths in the repo, fixtures, transcripts or screenshots. The exceptions:
  Barry's name where the license needs it (copyright lines, TRADEMARKS.md) and in commits and sign-offs, which use his
  GitHub noreply address; and third-party authors' names that a license requires us to keep (LDraw parts, OMR models,
  and their attribution files).
- PR titles: `<session id>: <outcome>` for a session ("0B: Research spikes and ADRs"), `<area>: <change>` otherwise
  ("ci: Cache the E2E build"); 50 characters or fewer, plain words, no version numbers or paths.
- US English.
