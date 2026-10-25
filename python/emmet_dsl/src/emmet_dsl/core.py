# SPDX-License-Identifier: Apache-2.0
"""@model, Model, Model.add, and Model.step (plan section 08, v0)."""

from __future__ import annotations

import contextlib
import difflib
import re
from dataclasses import dataclass, field

from . import frame
from .errors import EmmetError, caller

MAX_PARTS = 20_000

# The functions marked with @model, in definition order. run.py takes the last one model.py defines.
registry: list = []

# What emmet run supplies (part geometry, colors); run.py sets it before calling the model.
active = None


def model(fn):
    """Marks the model's entry function. Its signature is the param schema; it returns a Model."""
    registry.append(fn)
    return fn


@dataclass
class Placed:
    id: str
    part: str
    color: int
    pos: tuple
    rot: frame.Mat3
    box: tuple  # (min, max) in DSL units
    file: str
    line: int
    step: int


@dataclass
class _Step:
    title: str
    slug: str
    parts: list = field(default_factory=list)


class Handle:
    """What Model.add returns: the placed part's id and the box it covers, in studs and plates."""

    def __init__(self, placed: Placed):
        self._p = placed

    @property
    def id(self) -> str:
        """The part's Emmet id, as the viewer, highlight, and findings name it."""
        return self._p.id

    @property
    def min(self) -> tuple:
        """The (x, y, z) corner of the part's body with the lowest values."""
        return self._p.box[0]

    @property
    def max(self) -> tuple:
        """The (x, y, z) corner of the part's body with the highest values; the next part can sit at max's z."""
        return self._p.box[1]

    def __repr__(self) -> str:
        return f"<{self._p.part} {self._p.id} at {self.min}>"


def _slug(title: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-") or "step"


class Model:
    """A brick model built from LDraw parts in the DSL frame: x right and y back in studs, z up in plates."""

    def __init__(self):
        self._steps: list[_Step] = []
        self._current: _Step | None = None
        self._loose: _Step | None = None  # the step for parts added outside any `with m.step(...)`
        self._ids: dict[str, Placed] = {}
        self._count = 0
        self._numbered: dict[str, int] = {}  # parts numbered so far under each step slug, for default ids
        self._context = active

    @contextlib.contextmanager
    def step(self, title: str):
        """Everything added inside is one build step (LDraw `0 STEP`)."""
        previous = self._current
        self._loose = None
        self._current = _Step(title, _slug(title))
        self._steps.append(self._current)
        try:
            yield self
        finally:
            self._current = previous

    def add(self, part: str, color, at=(0, 0, 0), rot: int = 0, id: str | None = None) -> Handle:
        """Places one part so the minimum corner of its body (studs excluded), after rotation, is at `at`.

        part: an LDraw id ("3001"). color: a name in any spelling, or an LDraw code. at: (x, y, z) in studs, studs,
        plates. rot: quarter turns counterclockwise about z, seen from above. id: a stable name; by default
        "<step>/<n>".
        """
        file, line = caller()
        ctx = self._context
        if ctx is None:
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                "Model.add ran outside emmet run.",
                "Run the model with emmet run; it supplies the parts and colors.",
                file,
                line,
            )
        rel = ctx.rel(file)
        if not isinstance(rot, int) or isinstance(rot, bool):
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"rot={rot!r} isn't a whole number of quarter turns.",
                "Use rot=0, 1, 2, or 3 (quarter turns counterclockwise, seen from above).",
                rel,
                line,
            )
        try:
            at = tuple(float(v) for v in at)
            if len(at) != 3:
                raise ValueError
        except (TypeError, ValueError):
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"at={at!r} isn't an (x, y, z) position.",
                "Pass at=(x, y, z): studs right, studs back, plates up.",
                rel,
                line,
            ) from None
        self._count += 1
        if self._count > MAX_PARTS:
            raise EmmetError(
                "MODEL_TOO_LARGE",
                f"The model places more than {MAX_PARTS:,} parts.",
                "Reduce repeats, or use larger parts.",
                rel,
                line,
            )
        part_id = ctx.part_id(part, rel, line)
        code = ctx.color(color, rel, line)
        body = ctx.body(part_id, rel, line)
        step = self._current
        if step is None:
            if self._loose is None:
                self._loose = _Step("", "part")
                self._steps.append(self._loose)
            step = self._loose
        if id is None:
            # Counted per slug across the model, so two steps with one title, or loose parts before and after a
            # step, still get distinct ids.
            n = self._numbered.get(step.slug, 0) + 1
            self._numbered[step.slug] = n
            id = f"{step.slug}/{n}"
        elif not isinstance(id, str) or not id or any(c.isspace() for c in id):
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"id={id!r} isn't a usable id.",
                'Use a short string without spaces, like id="roof/ridge".',
                rel,
                line,
            )
        if id in self._ids:
            first = self._ids[id]
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"Two parts have the id '{id}' (the first is at {first.file}:{first.line}).",
                "Give each part its own id, or leave id out to get one from the step.",
                rel,
                line,
            )
        pos, r, box = frame.place(body, at, rot)
        placed = Placed(id, part_id, code, pos, r, box, rel, line, 0)
        step.parts.append(placed)
        self._ids[id] = placed
        return Handle(placed)

    def steps(self) -> list[_Step]:
        """The steps that have parts, numbered from 1 in the order they began."""
        out = [s for s in self._steps if s.parts]
        for n, s in enumerate(out, 1):
            for p in s.parts:
                p.step = n
        return out


def suggest(name: str, choices) -> list[str]:
    return difflib.get_close_matches(name, list(choices), n=3, cutoff=0.5)
