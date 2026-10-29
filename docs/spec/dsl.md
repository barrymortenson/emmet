# emmet_dsl v1

The Python a model is written in (plan section 08). v0 (session 1A) was `@model`, `Model.add`, `Model.step`, and
parameters; v1 (3B) adds the helpers that place many parts, repetition and symmetry, submodels used many times, the
technique library, part helpers, any rotation, LDraw import, and the validator's `allow` and `ground`. emmet_dsl uses
the standard library only, so a model never needs `pip install`. `emmet api` prints the vocabulary from the
docstrings (`crates/emmet-cli/doctrine/api.md`).

```python
from typing import Annotated

from emmet_dsl import Brick, Color, Model, Range, Step, model


def lamp(m, color="Black"):
    """A lamp post, built once in its own steps."""
    with m.step("Post"):
        m.stack(Brick(1, 1), color, at=(0, 0, 0), count=3)
        m.add("4589", "Trans-Yellow", at=(0, 0, 9))


@model
def street(
    length: Annotated[int, Range(8, 32), Step(8)] = 16,
    wall_color: Color = "Tan",
):
    """A wall and a row of lamps on a baseplate.
    length: how many studs long the street is
    wall_color: the wall's color
    """
    m = Model()
    with m.step("Base"):
        m.add("3811", "Green", at=(0, 0, 0), id="base")
    for row in range(2):
        with m.step(f"Wall, row {row + 1}"):
            m.wall(x=2, y=8, z=0.5 + 3 * row, length=length, color=wall_color)
    post = m.submodel("Lamp", lamp)
    with m.step("Lamps"):
        m.repeat(lambda: m.place(post, at=(2, 4, 0.5)), length // 4, dx=4)
    return m
```

## Frame and units

- **Z is up.** `x` runs right and `y` runs back, in studs (1 stud = 20 LDU). `z` counts plates (1 plate = 8 LDU,
  3 plates make a brick).
- **To LDraw.** The DSL point (x, y, z) is the LDraw point (20·y, −8·z, −20·x) in LDU. The map is a proper rotation
  plus scale. `tests/fixtures/projects/frame/expected.ldr` pins it with lines worked out by hand.
- **Footprint placement.** `at=(x, y, z)` is the minimum corner of the part's body box after rotation. The body box
  is the part's geometry with its studs left out (every `stud*.dat` and `stug*.dat` primitive, at any depth). A
  Brick 2 × 4 (3001) at `(0, 0, 0)` with `rot=0` covers x 0–2, y 0–4, z 0–3. A part's long side runs along y at
  `rot=0`.
- **Baseplates are half a plate thick** (4 LDU), so a model on one starts at z = 0.5.
- **Rotation.** `rot=k` turns the part k quarter turns counterclockwise, seen from above. A part's LDraw front (−Z
  in its own file) faces +x at `rot=0`, −y at `rot=3`, −x at `rot=2`, and +y at `rot=1`. `rot=Rot(x=…, y=…, z=…)`
  turns it by any angles in degrees about the DSL axes, x first, then y, then z; footprint placement still applies.
- **The escape hatch.** `m.add_raw(part, color, matrix)` places a part by the 12 numbers of an LDraw line (x y z
  a b c d e f g h i, in LDU in LDraw's frame), for a part placed by its LDraw origin rather than its footprint, such as track at 22.5°, whose ends have to meet exactly.

## API

| Call | Does |
|---|---|
| `@model` | Marks the entry function in model.py. Its signature is the parameter schema; it returns a `Model`. `@model(knobs=["weathering", "seed"])` names the knobs: the two to five parameters the app's panel shows, the ones that change the model's character, never raw dimensions or colors. A knob that isn't a parameter is `PARAM_INVALID`. |
| `Model()` | An empty model. |
| `m.add(part, color, at=(0, 0, 0), rot=0, id=None)` | Places one part and returns a `Handle` (`.id`, `.min`, `.max`: the box it covers). `part` is an LDraw id (`"3001"`, `"3001.dat"`, any case) or a part helper. `color` is a name or an LDraw code. Without `id`, the id is `<step slug>/<n>`, numbered per slug across the model, so steps that share a title still get distinct ids. An `id` given twice is an error. The call's file and line are recorded. |
| `m.add_raw(part, color, matrix, id=None)` | The escape hatch above. |
| `with m.step(title):` | Everything added inside is one build step (`0 STEP`). A step opened inside another is titled "outer: inner" ("outer, inner" when the outer title already has a colon, and just the inner title when it already starts with the outer one), so a technique's own steps group under the step that uses it; parts added to the outer step after the inner one closes go in a step after it. Steps with no parts are left out. |
| `m.stack(part, color, at, count, rot=0, id=None)` | `count` copies straight up, each on the one before: a column, a chimney. |
| `m.grid(part, color, at, nx, ny, rot=0, id=None)` | nx × ny copies side by side. |
| `m.row(kind, color, at, length, axis="x", width=1, first=None, sizes=None, id=None)` | A run of `length` studs along `axis` in the longest pieces of a family (`Brick`, `Plate`, `Tile`), `width` studs across; `first` sets the first piece (for a running bond), `sizes` limits the lengths. |
| `m.fill(kind, color, at, size=(x, y), id=None)` | Covers a rectangle with plates or tiles: 2-wide strips along its long side, the longest pieces first. |
| `m.wall(x, y, z, length, depth=0, color, sides="all", rows=1, bond="running", openings=(), kind=Brick, sizes=None)` | Brick walls a course at a time. With `depth`, the four walls of a rectangle with interlocking corners (the front and back own the corners on even courses, the sides on odd ones); without it, one straight wall along x, its joints staggered row to row. `sides` builds a wall at a time ("front", "back", "left", "right", "sides"); `openings` are (x, y) cells left empty. `bond` applies to a straight wall; a rectangle's courses alternate at the corners whatever the bond. |
| `m.repeat(fn, n, dx=0, dy=0, dz=0)` | Calls `fn(i)` (or `fn()`) for i in 0 … n−1 with every position it places moved by (i·dx, i·dy, i·dz). |
| `with m.mirror(axis, about):` | Everything placed inside is placed again across the plane x = about (or y = about), moved with `m.repeat` like any position, and turned to face the other way, with "-mirror-x" (or "-y") on its id. Parts are turned, not reflected: a left-handed part stays left-handed. |
| `sub = m.submodel(name, fn, **params)` | A unit built once by `fn(m, **params)` on its own Model, with its own steps. The same name, function, and params return the same submodel. |
| `m.place(sub, at, rot=0, id=None)` | Places a copy by the footprint of its parts' box. The copy's id is by default `<name slug>/<n>`; its parts' ids are `<copy id>/<part id>` (towers/2/wall/3). |
| `m.use(technique, **kw)` | Applies a technique from the library and returns what it returns (below). |
| `m.allow(rule, ids, reason)` | Records a deliberate exception: the validator lists findings of `rule` whose parts are all in `ids` as allowed, with the reason, never hidden. Ids may be Handles or a copy's id (all its parts). |
| `m.ground(*ids)` | Parts the model stands on, besides its lowest level and any baseplate. |
| `import_ldraw(path)` | An .ldr or .mpd inside the project as fixed geometry: a submodel to `place`. Its parts' ids are their line numbers (`L12`, or `L4/L12` inside one of its own submodels), its `0 STEP` lines its steps, and their source lines point into the file. |
| `Brick(w, l, h=1)`, `Plate(w, l)`, `Tile(w, l)`, `Slope(angle, w, l)` | Part helpers: the plain part of that size (`Brick(2, 4)` is "3001"; `Slope(45, 2, 1)` the Slope Brick 45 2 x 1, w deep down the slope). They resolve through the parts index between passes, and a size the library lacks fails with the sizes it has. `Brick`, `Plate`, and `Tile` themselves are the families `row`, `fill`, and `wall` take. |

## Submodels

A submodel is built once and placed as often as needed (plan section 08). The .mpd holds one `0 FILE` per submodel
and parameter set (`window.ldr`; another parameter set of the same name is `window-2.ldr`), and each placement is a
reference line to it, so a change to the submodel changes every copy. In `placements.json` each of a copy's parts is
its own row, in world coordinates, with `submodel` (the file names from the main model down) and `copies` (each
enclosing copy's id and the parent step that places it). The BOM counts every copy's parts. `emmet_core`'s .mpd
reader gives the same rows from `model.mpd`.

## Steps

People build from the steps, so they follow section 08's rules: bottom up, a few parts a step, a sub-assembly built
apart. The model's `with m.step(...)` blocks are its steps. Parts added outside any step fall back to steps made a
layer at a time: whole layers (parts starting at one height) together while they fit the step target, bottom up, a
layer bigger than the target split, and parts in add order within each step. The step target is 8 parts, or 4 and
12 for the Age settings 4+ and 18+ (`[build] age` in emmet.toml). A submodel's steps are its own, numbered from 1,
and the step that places its copies shows it as one sub-assembly "× n".

`build/steps.json`, which `emmet steps` and the MCP `steps` tool read:

```json
{ "schema": 1, "model": "station",
  "steps": [ { "n": 1, "title": "Walls, row 1: back wall", "group": "Walls, row 1", "part_ids": ["…"],
               "parts": [ { "part": "3004", "color": 19, "count": 5 } ],
               "submodels": [ { "name": "Window", "file": "window.ldr", "count": 2, "ids": ["window/left", "…"] } ] } ],
  "submodels": [ { "name": "Window", "file": "window.ldr", "count": 5, "parts": 2, "steps": ["…"] } ] }
```

`parts` is the step's parts box: each new part and color with its count, larger parts first. A sub-assembly's
`part_ids` are its own ids, the same in every copy.

## Techniques

`m.use(name, **kw)` applies a technique from `data/techniques/` (installed with emmet_dsl as
`emmet_dsl.techniques`). Each is a module with a doc, its `PARTS`, `build(m, …)`, and `example(m)`, which builds it
on its own; `emmet technique` lists them, and `emmet technique <name>` gives the doc, the example, and the parts.
Each has a render (`data/techniques/<name>.png`, from `scripts/render-techniques`) and a test that its example
validates with no findings (`tests/it/tests/techniques.rs`). Most techniques bring their own small steps, titled
after their `title` argument; bench, lamp_post, and tree are sub-assemblies, placed in the step you call them in. The library (v0): gable_roof, hip_roof, hinge_roof, snot_panel, jumper_offset, window_frame,
door_frame, stairs, fence, tree, bench, lamp_post, platform_edge, canopy, track_straight_run, track_curve,
track_switch, baseplate_road, brick_built_arch, tiled_floor. The track techniques take a pose and return
one, `(x, y, z, heading)`: where the track's center line ends and the way it runs (track_switch returns one for each
way, `{"straight", "branch"}`), so `p = m.use("track_curve", at=p, …)` chains pieces and a loop closes exactly.

## Parameters

Every parameter of the `@model` function needs a default. The ones `@model(knobs=...)` names show in the app's panel,
each as a slider, a switch, a choice, a color picker, or Shuffle (an int named `seed`); the rest work from the CLI
and the agent's tools.

| Annotation | Type | Notes |
|---|---|---|
| `int` | int | `Annotated[int, Range(lo, hi)]` limits it; `Step(n)` sets the increment, counted from the minimum. A value or default off the steps is invalid. |
| `float` | float | Same as int; steps allow for float error, so 0.3 is on steps of 0.1. |
| `bool` | bool | |
| `Literal["a", "b"]` | choice | |
| `Color` | color | Any spelling the color table accepts, or an LDraw code. With the parts data pack installed, only a color some part was produced in. |

Help text comes from docstring lines of the form `name: help`. Values come from the defaults, then `params.json`,
then `emmet run --params k=v` (for one run). `emmet params --set k=v` writes `params.json`; a value equal to its
default is dropped from the file. `emmet params --write-default k` ("make this the default") rewrites that
parameter's default in model.py to its current value, through Python's ast positions, so the rest of the file stays
byte for byte, and drops it from `params.json`; a default that isn't a plain literal (`= WIDE`, `= 2 * 3`) is refused.

## Colors

A color is found by its LDraw code or by a name, ignoring case, spaces, underscores, and punctuation, with "grey"
read as "gray". So "Light Bluish Gray", "Light_Bluish_Grey", and "lightbluishgray" are all code 71. Abbreviations
in `data/synonyms.toml` work too ("LBG"). The table is the installed library's `LDConfig.ldr`, shown with US
spellings. With the parts data pack (plan section 12), Rebrickable's and BrickLink's names resolve too, and each
color in `build/colors.json` says whether any part was `produced` in it.

## Output

`emmet run` writes, in `build/`:

- `model.mpd`: the main model's `0 FILE <model>.ldr`, then one `0 FILE` per submodel. Each part or copy is preceded
  by `0 !EMMET ID <id> SRC <file>:<line>`, each titled step starts with `0 !EMMET STEP <title>`, and every step ends
  with `0 STEP`. Other LDraw tools ignore the `!EMMET` lines.
- `placements.json`: `{ schema, model, parts: [{ id, part, color, matrix, step, submodel, copies?, src }], allow?,
  ground? }`. `matrix` is the 4 × 4 LDraw transform in LDU, column-major (three.js `Matrix4.fromArray` order). `allow`
  and `ground` are what `m.allow` and `m.ground` recorded, with every copy's ids (`docs/spec/validator.md`).
- `steps.json`: the steps, above.
- `manifest.json`: the inputs hash, the Emmet, emmet_dsl, library, and parts data versions, the part, step, and
  submodel counts, the bounding box in DSL units, and a `run` block (generation and timing) that differs from run to
  run.

Everything but `run.log` and the manifest's `run` block is the same, byte for byte, after a clean rebuild.
`emmet export --format ldr|mpd|lpub3d` writes, to `build/export/`, a flat .ldr (every part in world coordinates, the
main model's steps kept), a clean .mpd, or an LPub3D-ready .mpd (each sub-assembly a callout), all without the
`!EMMET` lines.

## Errors

The DSL raises `EmmetError` with a code from plan section 09 and the user's own file and line, never a frame inside
emmet_dsl (techniques included: an error inside one points at the `m.use` line). The reply carries the source line as
`excerpt`, read only from files inside the project.

| Code | When |
|---|---|
| `MODEL_EXEC_FAILED` | Python raised (a syntax error, a NameError, …), the model broke a rule (no default, a duplicate id, a bad `rot`, an unknown technique or rule, an `allow` of an id it didn't place), model.py is missing, or Python stopped without finishing (then `file` is `build/run.log`). The hint carries Python's own did-you-mean. An error with no frame in the project points at the `@model` function. |
| `IO_ERROR` | emmet_dsl couldn't write `build/`. |
| `UNKNOWN_PART` | The id isn't in the library, the part has no geometry to place it by, or a part helper's size isn't in the library. The hint lists close ids, the best search matches, or the sizes there are. |
| `UNKNOWN_COLOR` | The name or code isn't in the color table. The hint lists the nearest names. |
| `PARAM_INVALID` | A value is the wrong type, out of range, off the steps, a color no part was produced in, or names no parameter; params.json isn't JSON; or `--write-default` met a default that isn't a literal. It points at the parameter's line, or at params.json. |
| `MODEL_TIMEOUT` | The model ran past `[run] timeout` in emmet.toml (30 s by default, at most 3600). It points at the line the model was on. |
| `MODEL_TOO_LARGE` | The model placed more than 20,000 parts, every submodel copy counted. It points at the call that went over. |

## How a run works

emmet runs `python -I -B -X utf8 -m emmet_dsl.run` in the project folder with an empty environment (Windows keeps
`SYSTEMROOT` and `WINDIR`), sends one JSON request on stdin, and reads one JSON reply line from stdout. The model's own
output goes to stderr, kept as `build/run.log` (up to 10 MB). The DSL never reads the library: emmet writes the body
boxes of the parts the model uses, and the part helpers it has resolved ("brick 2x4" → 3001, or none with the nearest
sizes), to `build/part-geometry.json`, and the color table to `build/colors.json`. When the model asks for a part or
a helper that isn't in the file, that pass writes nothing and reports them with their call sites; emmet looks them up
and runs again. Missing parts come before any error the pass raised, since an error can follow from a size the model
didn't have yet. After a clean delete that's two passes, and warm runs take one.
Python gets 2 s after it exits to finish its output; a process the model started and left running doesn't hold up the
run. `mode: write_default` rewrites one default without running the model.
