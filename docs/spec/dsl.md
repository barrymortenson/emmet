# emmet_dsl v0

The Python a model is written in (plan section 08). v0 is what session 1A ships: `@model`, `Model.add`,
`Model.step`, and parameters. The helpers (`m.wall`, `m.stack`, …), `Rot`, `add_raw`, submodels, and techniques
arrive with v1 in 3B. emmet_dsl uses the standard library only, so a model never needs `pip install`.

```python
from typing import Annotated

from emmet_dsl import Color, Model, Range, Step, model


@model
def wall(
    width: Annotated[int, Range(2, 8), Step(2)] = 4,
    color: Color = "Light Bluish Gray",
):
    """A wall on a baseplate.
    width: how many studs wide
    color: the wall's color
    """
    m = Model()
    with m.step("Base"):
        m.add("3811", "Green", at=(0, 0, 0), id="base")
    with m.step("Walls"):
        for x in range(0, width, 2):
            m.add("3001", color, at=(10 + x, 10, 0.5))
    return m
```

## Frame and units

- **Z is up.** `x` runs right and `y` runs back, in studs (1 stud = 20 LDU). `z` counts plates (1 plate = 8 LDU,
  3 plates make a brick).
- **To LDraw.** The DSL point (x, y, z) is the LDraw point (20·y, −8·z, −20·x) in LDU. The map is a proper rotation
  plus scale. `tests/fixtures/projects/frame/expected.ldr` pins it with lines worked out by hand.
- **Footprint placement.** `at=(x, y, z)` is the minimum corner of the part's body box after rotation. The body box
  is the part's geometry with its studs left out (every `stud*.dat` and `stug*.dat` primitive, at any depth). A
  Brick 2 × 4 (3001) at `(0, 0, 0)` with `rot=0` covers x 0–2, y 0–4, z 0–3.
- **Baseplates are half a plate thick** (4 LDU), so a model on one starts at z = 0.5.
- **Rotation.** `rot=k` turns the part k quarter turns counterclockwise, seen from above. A part's LDraw front (−Z
  in its own file) faces +x at `rot=0`, −y at `rot=3`, −x at `rot=2`, and +y at `rot=1`.

## API

| Call | Does |
|---|---|
| `@model` | Marks the entry function in model.py. Its signature is the parameter schema; it returns a `Model`. |
| `Model()` | An empty model. |
| `m.add(part, color, at=(0, 0, 0), rot=0, id=None)` | Places one part and returns a `Handle` (`.id`, `.min`, `.max`: the box it covers). `part` is an LDraw id (`"3001"`, `"3001.dat"`, any case). `color` is a name or an LDraw code. Without `id`, the id is `<step slug>/<n>`, numbered per slug across the model, so steps that share a title still get distinct ids. An `id` given twice is an error. The call's file and line are recorded. |
| `with m.step(title):` | Everything added inside is one build step (`0 STEP`). Parts added outside any step go in a step of their own (untitled, ids `part/<n>`). Steps with no parts are left out. |

## Parameters

Every parameter of the `@model` function needs a default; each becomes a slider or a switch in the app.

| Annotation | Type | Notes |
|---|---|---|
| `int` | int | `Annotated[int, Range(lo, hi)]` limits it; `Step(n)` sets the increment, counted from the minimum. A value or default off the steps is invalid. |
| `float` | float | Same as int; steps allow for float error, so 0.3 is on steps of 0.1. |
| `bool` | bool | |
| `Literal["a", "b"]` | choice | |
| `Color` | color | Any spelling the color table accepts, or an LDraw code. |

Help text comes from docstring lines of the form `name: help`. Values come from the defaults, then `params.json`,
then `emmet run --params k=v` (for one run). `emmet params --set k=v` writes `params.json`; a value equal to its
default is dropped from the file.

## Colors

A color is found by its LDraw code or by a name, ignoring case, spaces, underscores, and punctuation, with "grey"
read as "gray". So "Light Bluish Gray", "Light_Bluish_Grey", and "lightbluishgray" are all code 71. Abbreviations
in `data/synonyms.toml` work too ("LBG"). The table is the installed library's `LDConfig.ldr`, shown with US
spellings. Rebrickable and BrickLink names join with the parts data pack (3B).

## Output

`emmet run` writes, in `build/`:

- `model.mpd`: one `0 FILE <model>.ldr`. Each part is preceded by `0 !EMMET ID <id> SRC <file>:<line>`, each titled
  step starts with `0 !EMMET STEP <title>`, and every step ends with `0 STEP`. Other LDraw tools ignore the `!EMMET` lines.
- `placements.json`: `{ schema, model, parts: [{ id, part, color, matrix, step, submodel, src }] }`. `matrix` is the
  4 × 4 LDraw transform in LDU, column-major (three.js `Matrix4.fromArray` order).
- `manifest.json`: the inputs hash, the Emmet, emmet_dsl, library, and parts data versions, the part and step counts,
  the bounding box in DSL units, and a `run` block (generation and timing) that differs from run to run.

Everything but `run.log` and the manifest's `run` block is the same, byte for byte, after a clean rebuild.

## Errors

The DSL raises `EmmetError` with a code from plan section 09 and the user's own file and line, never a frame inside
emmet_dsl. The reply carries the source line as `excerpt`, read only from files inside the project.

| Code | When |
|---|---|
| `MODEL_EXEC_FAILED` | Python raised (a syntax error, a NameError, …), the model broke a rule (no default, a duplicate id, a bad `rot`), model.py is missing, or Python stopped without finishing (then `file` is `build/run.log`). The hint carries Python's own did-you-mean. An error with no frame in the project points at the `@model` function. |
| `IO_ERROR` | emmet_dsl couldn't write `build/`. |
| `UNKNOWN_PART` | The id isn't in the library, or the part has no geometry to place it by. The hint lists close ids and the best search matches. |
| `UNKNOWN_COLOR` | The name or code isn't in the color table. The hint lists the nearest names. |
| `PARAM_INVALID` | A value is the wrong type, out of range, off the steps, or names no parameter; or params.json isn't JSON. It points at the parameter's line, or at params.json. |
| `MODEL_TIMEOUT` | The model ran past `[run] timeout` in emmet.toml (30 s by default, at most 3600). It points at the line the model was on. |
| `MODEL_TOO_LARGE` | The model placed more than 20,000 parts. It points at the call that went over. |

## How a run works

emmet runs `python -I -B -X utf8 -m emmet_dsl.run` in the project folder with an empty environment (Windows keeps
`SYSTEMROOT` and `WINDIR`), sends one JSON request on stdin, and reads one JSON reply line from stdout. The model's own output
goes to stderr, kept as `build/run.log` (up to 10 MB). The DSL never reads the library: emmet writes the body boxes of
the parts the model uses to `build/part-geometry.json` and the color table to `build/colors.json`. When the model
asks for a part that isn't in the file, that pass writes nothing and reports the missing ids with their call sites;
emmet looks them up and runs again. Missing parts come before any error the pass raised, since an error can
follow from a size the model didn't have yet. After a clean delete that's two passes, and warm runs take one. Python
gets 2 s after it exits to finish its output; a process the model started and left running doesn't hold up the run.
