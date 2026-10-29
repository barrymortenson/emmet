# The project format

A project is a folder (plan section 07). Its sources are the truth; `build/` is derived, and emmet can always
rebuild it.

```text
cottage/
├─ emmet.toml        project metadata
├─ model.py          the entry model (one @model function)
├─ models/           optional modules model.py imports (*.py)
├─ params.json       parameter overrides: only values that differ from model.py's defaults
├─ CARD.md           What it is · Design notes · Don't · an AUTO block for emmet
├─ AGENTS.md         a short guide for terminal agents; CLAUDE.md links to it (a copy on Windows)
├─ .gitignore        build/ and .emmet/
├─ build/            derived: model.mpd, placements.json, steps.json, manifest.json, validation.json, bom.json,
│                    part-geometry.json, colors.json, export/, run.log
├─ history/          saved versions, one folder each (below)
└─ .emmet/           local only: trust, generation
```

`emmet new <dir> [--template cottage|empty]` makes one. The folder must be new or empty. `emmet new <dir> --import
<file>` makes one around an existing LDraw model (an `.ldr` or `.mpd`): the file is copied in, and `model.py` places
`import_ldraw("<file>")`, so it builds, checks, and has steps like any model, until the agent makes it parametric.

## emmet.toml

```toml
schema = 1
name = "Cottage"
created_by = "m:4f1c9a0b2d3e5f67"   # this install's machine hash
library = "2026-09"                 # the LDraw release the project was made with

[agent]
preferred = "claude"

[run]
timeout = 30                        # seconds a model may run, more than 0 and at most 3600; optional

[build]                             # the build settings, which steer the agent; each optional ("Any" until set)
base = "32 × 32"                    # "None", "16 × 16", "32 × 32", or "48 × 48"
scale = "Figure scale"              # "Micro", "Micropolis", "Midi", "Figure scale", "Large", or "Car 1:8"
age = "6+"                          # "4+", "6+", "9+", "13+", or "18+"
detail = "Low"                      # "Low", "Medium", or "High"
pieces = "Small"                    # "Small" (50–250), "Medium" (250–1,000), "Large" (1,000–2,500), "Huge" (2,500+)
style = "Town"                      # a suggestion ("Town", "Medieval", …) or the user's own words

[[check.allow]]                     # a finding the user has decided to allow; optional, any number
rule = "floating"
parts = ["flag/1"]
reason = "held by hand in photos"
```

The app writes `[build]` and `[[check.allow]]` (Options' build settings, and It's fine in Checks); a build setting
doesn't change the model on its own: Rebuild asks the agent to rework the model for it. The one thing a setting does
by itself is `[build] age`, which sets the step_size target: 4 parts a step for 4+, 12 for 18+, and 8 otherwise. Each `[[check.allow]]`
moves each finding of that rule whose parts are all in `parts` from the findings to `allowed`, as `m.allow` does from
the model (`docs/spec/validator.md`).

`schema` is the format version. emmet refuses a project with a newer schema rather than guess. `created_by` is
`m:` plus 16 hex digits of a hash of a random secret kept in the app data folder. It isn't personal data, and it
isn't used for trust (below).

## Sources and the inputs hash

The sources are `model.py`, `models/**/*.py`, `params.json`, and `emmet.toml`. `manifest.json`'s `inputs_hash` is
the sha256 of the sources (line ends normalized to LF), the run's `--params` overrides, and the Emmet, library, and
parts data versions. A run whose hash differs from the manifest's is out of date.

## Versions

`history/<n>-<slug>/` holds version `n` (from 1; the slug is its name in lowercase words, `3-roof-and-chimney`): a copy
of the sources (`model.py`, `models/`, `params.json`, `emmet.toml`, `CARD.md`, and the `.ldr` and `.mpd` files at the
top, which an imported model brings in), that build's `model.mpd` and
`placements.json` when the build was current (so the app can show the version to compare without running it), a
`thumbnail.png` when the app saved it, and `version.json`:

```json
{ "n": 3, "name": "Roof and chimney", "time": 1793232393, "auto": false, "parts": 147, "inputs_hash": "…",
  "dir": "3-roof-and-chimney", "thumbnail": true, "placements": true }
```

`emmet version save <name> [--note …]` saves one; `emmet version list` lists them, newest first; `emmet version restore
<n>` first saves the current state as Emmet's own version ("Before restoring version n", `auto`), then writes the
version's sources back atomically and removes the `models/` files and `CARD.md` it doesn't have (an LDraw file at the
top stays), so a restore never loses anything; `emmet version open <n> <dir>` copies one out as a new project, named after its folder and trusted on
this machine. Emmet keeps its last 20 automatic versions (a restore prunes after it has read the version it restores);
named ones stay. A save that can't finish leaves no folder behind. A version never reads or writes through a
link, and an untrusted project's versions can't be restored or copied out (they're as untrusted as its `model.py`).

## Who writes what

- The agent and the user edit `model.py`, `models/`, and `CARD.md`.
- `emmet params --set` (and the app) write `params.json`, always atomically.
- Only emmet writes `build/`, `history/`, and `.emmet/`. Every write goes to a temp file in the same folder, is synced, then
  renamed. emmet refuses to write through a `build/`, `history/`, or `.emmet/` that is a link or a file.

## Trust (D-27)

`model.py` is code, so emmet runs it only in a trusted project. `.emmet/trust` holds the sha256 of this install's
secret and the project's full path. emmet makes that file for a project it creates and when you run `emmet trust`.
Nothing a project carries can make it: not a copied marker, and not `created_by`. A project moved to a new folder
needs trusting again. An untrusted project's `emmet run` and `emmet params` fail with `PROJECT_UNTRUSTED` before any
of its code runs.

## The generation counter

Each run that reaches the model, successful or not, adds one to `.emmet/generation` and reports it in the envelope's
`meta.generation` and in `manifest.json`'s `run` block. The viewer uses it to ignore stale events.

## The app data folder

`--data-dir`, else `EMMET_DATA_DIR`, else `.dev-data/` in the repo for a debug build, else
`~/Library/Application Support/com.emmetbuilder.emmet/` (macOS) or `%APPDATA%\com.emmetbuilder.emmet\` (Windows).

| Path | Holds |
|---|---|
| `ldraw/` | the LDraw library, as `complete.zip` has it under `ldraw/` |
| `library.json` | the installed release, the zip's sha256, the file count |
| `ldraw.sha256` | each library file's sha256, for `emmet library verify` |
| `parts.sqlite` | the parts index (plan section 12), rebuilt by `emmet library verify --reindex` |
| `python/` | uv's Python 3.13, uv's cache, and the venv models run in, with emmet_dsl installed |
| `machine-id` | the install's random secret (trust and `created_by`) |
| `downloads/` | a library download in progress (`.part`), which `emmet library install` resumes |

## Exit codes

0 ok · 2 usage (`PROJECT_NOT_FOUND` included) · 3 runtime (model errors, `PROJECT_UNTRUSTED`, `IO_ERROR`) · 4
environment (`LIBRARY_MISSING`, `LIBRARY_CORRUPT`, `PYTHON_MISSING`). `emmet doctor` exits 4 when a check fails.
