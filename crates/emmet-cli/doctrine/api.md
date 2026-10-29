# emmet_dsl API

Generated from emmet_dsl's docstrings (`python -m emmet_dsl.api`); don't edit.

A model.py imports from emmet_dsl, marks one function with `@model`, and returns a `Model`. The function's
parameters are the model's params: each needs a default, and its type annotation picks the control (`int` or
`float` with `Range` and `Step`, `bool`, `Literal[...]` for a choice, `Color`). The docstring's `name: help` lines
are each param's help text. Units: x runs right and y back, in studs; z runs up, in plates (3 plates make a brick).

## Brick(w: int, l: int, h: int = 1) -> str

A family of parts by size: Brick, Plate, or Tile. Call it with a size for one part's id (Brick(2, 4) is
"3001"); pass the family itself to Model.row, Model.fill, or Model.wall.

## class Color

A color parameter: a name in any spelling ("Light Bluish Gray", "LBG") or an LDraw code.

## class EmmetError

An error with a code from plan section 09, a hint, and the user's file and line.

## class Handle

What Model.add and Model.place return: the id and the box the part (or submodel copy) covers, in studs and
plates.

### Handle.id (property)

The part's Emmet id, as the viewer, highlight, and findings name it.

### Handle.min (property)

The (x, y, z) corner of the part's body with the lowest values.

### Handle.max (property)

The (x, y, z) corner of the part's body with the highest values; the next part can sit at max's z.

## class Model

A brick model built from LDraw parts in the DSL frame: x right and y back in studs, z up in plates.

### Model.step(title: str)

Everything added inside is one build step (LDraw `0 STEP`). A step opened inside another is titled
"<outer>: <inner>" ("<outer>, <inner>" when the outer title has a colon; just the inner title when it already
starts with the outer one), so a technique's own steps group under the step that uses it.

### Model.add(part: str, color, at=(0, 0, 0), rot: int | Rot = 0, id: str | None = None) -> Handle

Places one part so the minimum corner of its body (studs excluded), after rotation, is at `at`.

part: an LDraw id ("3001") or a helper (Brick(2, 4)). color: a name in any spelling, or an LDraw code. at:
(x, y, z) in studs, studs, plates. rot: quarter turns counterclockwise about z, seen from above, or a Rot. id:
a stable name; by default "<step>/<n>".

### Model.add_raw(part: str, color, matrix, id: str | None = None) -> Handle

The escape hatch: places a part by an LDraw transform, in LDU in LDraw's frame (−Y up), as the 12 numbers
of an LDraw line: x y z a b c d e f g h i. Use it for a part placed by its LDraw origin rather than its
footprint, such as track at 22.5°, whose ends have to meet exactly.

### Model.stack(part: str, color, at=(0, 0, 0), count: int = 1, rot: int | Rot = 0, id: str | None = None)

`count` copies of a part, each on top of the one before: a column, a chimney. Returns their Handles.
With `id`, they're "<id>/1", "<id>/2", …

### Model.grid(part: str, color, at=(0, 0, 0), nx: int = 1, ny: int = 1, rot: int | Rot = 0, id: str | None = None)

nx × ny copies of a part side by side, each next to the last along x, then along y. Returns their
Handles. With `id`, they're "<id>/1", "<id>/2", … row by row.

### Model.row(kind, color, at=(0, 0, 0), length: int = 1, axis: str = 'x', width: int = 1, first=None, sizes=None, id: str | None = None)

A run of `length` studs along `axis` ("x" or "y") from `at`, `width` studs across, in the longest pieces
of a family (Brick, Plate, or Tile) the library has, longest first. first: the length of the first piece
(a running bond starts every other row with a shorter one). sizes: the piece lengths to use. Returns their
Handles.

### Model.fill(kind, color, at=(0, 0, 0), size=(2, 2), id: str | None = None)

Covers a size=(x, y) rectangle with plates or tiles (Plate or Tile): strips 2 studs wide along its long
side, the longest pieces first, and a 1-wide strip when the short side is odd. Returns their Handles.

### Model.wall(x: float, y: float, z: float, length: int, depth: int = 0, color='Light Bluish Gray', sides='all', rows: int = 1, bond: str = 'running', openings=(), kind=None, sizes=None)

Brick walls, a row at a time from z up. With depth, the four walls of a length × depth rectangle whose
front-left corner is (x, y), with interlocking corners; without it, one straight wall along x. sides: "all",
or some of "front", "back", "left", "right", and "sides" (left and right), to build a wall at a time. bond:
"running" (no joint on a joint in the course below that a Model.wall laid, when the sizes allow it, so the
joints below are bridged and the course over a door or window spans its edges) or "stacked". openings: (x, y) cells left empty, for doors and
windows. kind: the piece family (Brick by default). sizes: the piece lengths to use. Returns the Handles.

### Model.repeat(fn, n: int, dx: float = 0, dy: float = 0, dz: float = 0) -> list

Calls fn(i) for i in 0 … n−1 with every position it places moved by (i·dx, i·dy, i·dz). Returns what each
call returned. Give parts placed with an id the index (id=f"lamp/{i}"), so each copy's ids differ.

### Model.mirror(axis: str, about: float)

Everything placed inside is placed twice: as written, and mirrored across the plane x = about (axis "x")
or y = about (axis "y"), moved with Model.repeat like any position. A mirrored part turns to face the other way;
its id gets
"-mirror-x" (or "-y"). Parts are turned, not reflected, so a left-handed part stays left-handed.

### Model.submodel(name: str, fn, **params) -> Submodel

Defines a unit built once by fn(m, **params) on its own Model, with its own steps: a window, a lamp post,
a tower. Model.place puts it in the model, as often as needed. Each name and parameter set is one file in the
.mpd; calling submodel again with the same ones returns the same Submodel.

### Model.place(sub: Submodel, at=(0, 0, 0), rot: int | Rot = 0, id: str | None = None) -> Handle

Places a copy of a submodel so the minimum corner of the box its parts cover, after rotation, is at `at`.
The copy's parts get ids "<copy id>/<part id>"; the copy's id is by default "<submodel name>/<n>".

### Model.use(technique: str, **kw)

Applies a technique from Emmet's library (emmet technique lists them): gable_roof, snot_panel, window_frame,
track_curve, … with its keyword arguments. Returns what the technique returns (track gives the pose where the
next piece goes).

### Model.allow(rule: str, ids, reason: str) -> None

Records a deliberate exception: findings of `rule` whose parts are all in `ids` show in Checks as
"allowed", with the reason, never hidden. ids: an id, a Handle, or a list of them.

### Model.ground(*ids) -> None

Marks parts the model stands on, besides its lowest level and any baseplate (a pier's footing on a raised
base, say). ids: ids or Handles.

## Plate(w: int, l: int, h: int = 1) -> str

A family of parts by size: Brick, Plate, or Tile. Call it with a size for one part's id (Brick(2, 4) is
"3001"); pass the family itself to Model.row, Model.fill, or Model.wall.

## class Range

Annotated[int, Range(6, 16)]: the lowest and highest value a slider allows.

## class Rot

rot=Rot(x=90, z=45): a turn by any angles, in degrees, about the DSL axes (x first, then y, then z), for when
quarter turns about z (rot=0 to 3) aren't enough.

## Slope(angle: int, w: int, l: int) -> str

The part id of a slope: Slope(45, 2, 1) is the Slope Brick 45 2 x 1, w deep down the slope and l wide.

## class Step

Annotated[float, Step(0.5)]: the slider's increment.

## class Submodel

What Model.submodel and import_ldraw return: a unit built once, which Model.place puts in a model as often as
needed. The .mpd holds it once, as its own file, and each placement references it.

### Submodel.size (property)

How many parts one copy places, counting the submodels inside it.

### Submodel.box (property)

(min, max) of the box its parts cover in its own frame, in studs and plates.

## Tile(w: int, l: int, h: int = 1) -> str

A family of parts by size: Brick, Plate, or Tile. Call it with a size for one part's id (Brick(2, 4) is
"3001"); pass the family itself to Model.row, Model.fill, or Model.wall.

## import_ldraw(path: str) -> Submodel

Brings an existing LDraw model (an .ldr or .mpd inside the project) in as fixed geometry: a submodel to place
with Model.place. Its parts' ids are their line numbers ("L12", or "L4/L12" for line 12 of the submodel that line
4 places), its `0 STEP` lines are its steps, and findings point at its lines.

## model(fn)

Marks the model's entry function. Its signature is the param schema; it returns a Model.
