# ADR 0010: validate v0, the eval harness, and the release dry run as built

Status: accepted, 2026-10-25 (session 1D)

## Context

Session 1D closes the agent's loop (validate, BOM, the Checks drawer), adds the live eval, and makes the first signed
build. Plan section 11 names validate v0's rules but not its numbers; section 16.1 names the eval's scores but not how
a headless run answers for the app; section 17 names the release flow but leaves the dry run to this session. No
dependency was added.

## Decisions

### validate v0 (`crates/emmet-core/src/check/`)

- **Collision.** Each part's body box (studs left out, from the parts index) placed as an oriented box, shrunk 1 LDU
  on every side, tested with the separating axis theorem. A pair collides when the overlap along the separating axis
  of least overlap is more than 0.5 LDU vertically, or more than 5 LDU sideways. The sideways allowance takes in the
  pins, clips, and hinges body boxes include (a classic window's shutter pins reach 4 LDU into its neighbors); a part a
  half stud or more into its neighbor still collides, and a part sunk half a plate into the baseplate is caught. A
  window or door frame and the glass, pane, shutter, or door inside its box don't collide, and the frame holds them.
- **Floating.** Studs come from the primitive table (`ldraw::geometry::SYSTEM_STUDS`, a test per family in
  `check/connectors.rs`): a part's references to them at any depth, with each stud's reach (4 LDU, or more for a
  stretched one whose base is sunk into its part, as on an inverted slope). A stud connects its part to another when
  its base is at or below that part's underside (within 1.5 LDU) and its tip reaches at least 2 LDU into it, at least
  4 LDU inside its sides. The receiving side is the body's underside, not anti-stud primitives: 3A adds those, the
  LDCad shadow data, and clips, pins, and hinges. Every component with a part on the model's lowest level is grounded;
  any other component floats, and its hint names the part under it and the gap in plates.
- **Part rules.** `unknown_part` (error), `obsolete_part` (warning: a `~Moved to` or `(Obsolete)` part, with the
  current id), `alias_part` (info, with the canonical id).
- **Findings.** Ordered by severity, then the first source line, then the rule and parts, and numbered `F-0001`…, so
  the same inputs give the same ids. `[[check.allow]]` in emmet.toml (`rule`, `parts`, `reason`) moves a finding whose
  parts are all listed to `allowed`; only the user writes emmet.toml.
- **Speed.** The corpus runs in 1–30 ms a model warm; files are looked up in the library on demand
  (`LibFiles::probe`), not scanned.

### Two body-box fixes the validator found (the parts index is schema 2)

- Underside tubes and posts (`stud3`, `stud4*`, …) stay in body boxes: a 1 x 1 round plate's lower half is a
  `stud4`, so leaving every `stud*` out made it 3 LDU tall. Only top studs (of every system) and `stug*` groups are
  left out.
- Door and window frames draw their cut corner studs by hand, so their bodies were a stud taller than they stack (the
  DSL sized door frame 60596 at 18.5 plates). When a body's top is exactly a stud above the part's origin plane, and
  everything above the plane is stud-sized islands (at most 13 LDU across) no more than a stud high on a body at
  least a stud wide each way, the top is the plane. 103 parts change, all brick-system frames, panels, and the like.

### Fixtures

- `tests/fixtures/omr/`: the five smallest OMR models validate v0 can judge: built with studs only and with no moved
  or obsolete part ids (validate rightly warns about those, and most older OMR files use some). They're the 544-1 Car,
  Cow, and Airplane, 566-1 Seaplane, and 40013-1 Halloween Ghost, unmodified, CC BY 2.0, credited in
  `ATTRIBUTION.md` with their sha256s, which a test checks. Of about 55 small OMR models tried, the rest fail v0 only where 3A
  is needed (clips, pins, hinges, minifigures, the cavity under an inverted double slope).
- ldraw-mini gains two true alias parts (15570, 18860) and the wheels the OMR models use (3137c05).

### The eval (`crates/emmet-cli/src/eval.rs`)

- It starts the CLI through the app's own adapters and answers for the app: it runs a bus server and hub on the data
  dir, so it refuses while an app holds that bus (`emmet_bus::listening`). Questions get the rung's scripted answer;
  approvals allow Emmet's tools and edits to `<project>/model.py` and `<project>/models/*.py` only, by path from the
  project root.
- Keys (D-37): Claude gets `ANTHROPIC_API_KEY` from the keychain item `emmet-eval-anthropic` in its environment;
  Codex gets the `emmet-eval-openai` key through `SessionConfig.openai_key` (new; the app leaves it unset and uses its
  own keychain item). A run that reports a plan instead of a key is stopped at once.
- A turn is a tool call (questions are `ask_user` calls). Scoring follows section 16.1; availability is reported as
  not yet scored until 3B's parts data.
- `--dry-run` uses the fake agents and derived rung 1 transcripts; it skips the keychain and the question before a
  live run. The report keeps one section per agent in `docs/evals/<date>-rung1.md`.

### The release dry run (`scripts/release-dry.sh`)

- The sidecars (`emmet`, `uv` 0.12.22 with pinned sha256s) go in through `tauri.release.conf.json`, so dev and E2E
  builds don't need them. Tauri's universal build wants per-architecture copies as well as the `lipo`'d ones.
- With `--sign`, the script signs the sidecars itself (hardened runtime, secure timestamp) before Tauri signs the app,
  then signs, notarizes (`notarytool --keychain-profile emmet-notary`), and staples the `.dmg`. Tauri's own
  notarization takes credentials from the environment; the keychain profile keeps the App Store Connect key out of it.
- `CI=true` keeps the dmg step from scripting Finder, which times out without a GUI session.
- Windows: the unsigned NSIS installer, built by CI's `nsis` job when the release files change, weekly, and by hand.

### The LDraw library pin moves to 2026-10

LDraw replaced `complete.zip` with release 2026-10 on 2026-10-26 (`023083918da2…a803fa610f`, 147 MB, 37,448 files,
the same CAreadme), and serves no older complete archive, so every fresh install failed the 2026-09 sha256 check. H-05's
fresh-account check found it. The pin is now 2026-10. Installs already on 2026-09 keep working (only an install checks
the pin), and projects record the release they ran against. ADRs 0003 and 0004 keep 2026-09: it's what they measured.
`ldraw-mini` stays a 2026-09 subset; `scripts/make-ldraw-mini` still pins that zip, so adding parts to it needs a
2026-09 copy until 6A's mirror (plan section 12) hosts one.

## Consequences

- v0 reports clip-, pin-, and hinge-held parts as floating; 3A's connection model fixes that, and its corpus grows to
  ten OMR models and a nightly set.
- The eval's answer and approval rules have unit tests, and the dry run an integration test on both agents.
- 6A builds `release.yml` from the dry run's steps.
