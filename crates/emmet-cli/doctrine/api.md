# emmet_dsl API

Generated from emmet_dsl's docstrings (`python -m emmet_dsl.api`); don't edit.

A model.py imports from emmet_dsl, marks one function with `@model`, and returns a `Model`. The function's
parameters are the model's params: each needs a default, and its type annotation picks the control (`int` or
`float` with `Range` and `Step`, `bool`, `Literal[...]` for a choice, `Color`). The docstring's `name: help` lines
are each param's help text. Units: x runs right and y back, in studs; z runs up, in plates (3 plates make a brick).

## class Color

A color parameter: a name in any spelling ("Light Bluish Gray", "LBG") or an LDraw code.

## class EmmetError

An error with a code from plan section 09, a hint, and the user's file and line.

## class Handle

What Model.add returns: the placed part's id and the box it covers, in studs and plates.

### Handle.id (property)

The part's Emmet id, as the viewer, highlight, and findings name it.

### Handle.min (property)

The (x, y, z) corner of the part's body with the lowest values.

### Handle.max (property)

The (x, y, z) corner of the part's body with the highest values; the next part can sit at max's z.

## class Model

A brick model built from LDraw parts in the DSL frame: x right and y back in studs, z up in plates.

### Model.step(title: str)

Everything added inside is one build step (LDraw `0 STEP`).

### Model.add(part: str, color, at=(0, 0, 0), rot: int = 0, id: str | None = None) -> Handle

Places one part so the minimum corner of its body (studs excluded), after rotation, is at `at`.

part: an LDraw id ("3001"). color: a name in any spelling, or an LDraw code. at: (x, y, z) in studs, studs,
plates. rot: quarter turns counterclockwise about z, seen from above. id: a stable name; by default
"<step>/<n>".

### Model.steps() -> list[_Step]

The steps that have parts, numbered from 1 in the order they began.

## class Range

Annotated[int, Range(6, 16)]: the lowest and highest value a slider allows.

## class Step

Annotated[float, Step(0.5)]: the slider's increment.

## model(fn)

Marks the model's entry function. Its signature is the param schema; it returns a Model.
