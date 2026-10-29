# SPDX-License-Identifier: Apache-2.0
"""@model, Model and its calls, submodels, and techniques (plan section 08, v1)."""

from __future__ import annotations

import contextlib
import difflib
import importlib
import inspect
import pkgutil
import re
from dataclasses import dataclass, field

from . import frame
from .errors import EmmetError, caller

MAX_PARTS = 20_000

# The validator's rules (docs/spec/validator.md), which m.allow can name.
RULES = (
    "unknown_part",
    "collision",
    "floating",
    "disconnected_submodel",
    "step_order",
    "step_size",
    "weak_connection",
    "off_grid",
    "obsolete_part",
    "alias_part",
    "part_unavailable",
    "color_unavailable",
)

# The functions marked with @model, in definition order. run.py takes the last one model.py defines.
registry: list = []

# What emmet run supplies (part geometry, colors, the submodels made so far); run.py sets it before loading the model.
active = None


def model(fn):
    """Marks the model's entry function. Its signature is the param schema; it returns a Model."""
    registry.append(fn)
    return fn


@dataclass
class Placed:
    id: str
    part: str  # an LDraw part id, or a submodel's file name for a placement of one
    color: int
    pos: tuple  # the origin, in LDU in LDraw's frame
    rot: frame.Mat3  # in LDraw's axes
    body: list  # the LDU box the transform applies to: the part's body, or the submodel's box in its own frame
    box: tuple  # (min, max) in DSL units, in the model's frame
    file: str
    line: int
    step: int = 0
    sub: Submodel | None = None


@dataclass
class _Step:
    title: str
    slug: str
    parts: list = field(default_factory=list)
    auto: bool = False  # parts added outside any step, which Model.steps splits into layers


class Handle:
    """What Model.add and Model.place return: the id and the box the part (or submodel copy) covers, in studs and
    plates."""

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


class Rot:
    """rot=Rot(x=90, z=45): a turn by any angles, in degrees, about the DSL axes (x first, then y, then z), for when
    quarter turns about z (rot=0 to 3) aren't enough."""

    def __init__(self, x: float = 0, y: float = 0, z: float = 0):
        self.x, self.y, self.z = x, y, z

    def _matrix(self) -> frame.Mat3:
        return frame.rotation(self.x, self.y, self.z)

    def __repr__(self) -> str:
        return f"Rot(x={self.x}, y={self.y}, z={self.z})"


class Submodel:
    """What Model.submodel and import_ldraw return: a unit built once, which Model.place puts in a model as often as
    needed. The .mpd holds it once, as its own file, and each placement references it."""

    def __init__(self, name: str, file: str, built: Model):
        self.name = name
        self.file = file
        self.model = built

    @property
    def size(self) -> int:
        """How many parts one copy places, counting the submodels inside it."""
        return sum(p.sub.size if p.sub else 1 for s in self.model._steps for p in s.parts)

    @property
    def box(self) -> tuple:
        """(min, max) of the box its parts cover in its own frame, in studs and plates."""
        boxes = [p.box for s in self.model._steps for p in s.parts]
        lo = tuple(min(b[0][i] for b in boxes) for i in range(3))
        hi = tuple(max(b[1][i] for b in boxes) for i in range(3))
        return (lo, hi)

    def __repr__(self) -> str:
        return f"<submodel {self.name} ({self.size} parts)>"


def _slug(title: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-") or "step"


def _ids(value) -> list[str]:
    """One id, a Handle, or a list of either, as id strings."""
    items = [value] if isinstance(value, str | Handle) else list(value)
    return [v.id if isinstance(v, Handle) else v for v in items]


class Model:
    """A brick model built from LDraw parts in the DSL frame: x right and y back in studs, z up in plates."""

    def __init__(self):
        self._steps: list[_Step] = []
        self._current: _Step | None = None
        self._loose: _Step | None = None  # the step for parts added outside any `with m.step(...)`
        self._ids: dict[str, Placed] = {}
        self._numbered: dict[str, int] = {}  # parts numbered so far under each slug, for default ids
        self._offset = (0.0, 0.0, 0.0)  # what Model.repeat adds to each position
        self._mirrors: list[tuple[str, float]] = []  # the open Model.mirror blocks
        self._allowed: list[tuple] = []  # (rule, ids, reason, file, line)
        self._grounded: list[tuple] = []  # (ids, file, line)
        # Where Model.wall's pieces meet, by (axis, the line across it, z), so the course above lays them in bond.
        self._joints: dict[tuple, set] = {}
        self._context = active

    # Steps

    @contextlib.contextmanager
    def step(self, title: str):
        """Everything added inside is one build step (LDraw `0 STEP`). A step opened inside another is titled
        "<outer>: <inner>" ("<outer>, <inner>" when the outer title has a colon; just the inner title when it already
        starts with the outer one), so a technique's own steps group under the step that uses it."""
        outer = self._current
        if outer is not None and outer.title and not title.startswith(outer.title):
            title = f"{outer.title}{', ' if ':' in outer.title else ': '}{title}"
        self._loose = None
        self._current = _Step(title, _slug(title))
        self._steps.append(self._current)
        try:
            yield self
        finally:
            if outer is not None:
                # Parts added to the outer step from here on come after the inner one, in a step of their own.
                self._current = _Step(outer.title, outer.slug)
                self._steps.append(self._current)
            else:
                self._current = None

    def _numbered_steps(self) -> list[_Step]:
        """The steps that have parts, numbered from 1 in the order they began. Parts added outside any step are
        split into steps a layer at a time, bottom up, at most the step target each."""
        out = []
        for s in self._steps:
            if not s.parts:
                continue
            out += _layers(s, _step_target(self._context)) if s.auto else [s]
        for n, s in enumerate(out, 1):
            for p in s.parts:
                p.step = n
        return out

    # Placing parts

    def add(self, part: str, color, at=(0, 0, 0), rot: int | Rot = 0, id: str | None = None) -> Handle:
        """Places one part so the minimum corner of its body (studs excluded), after rotation, is at `at`.

        part: an LDraw id ("3001") or a helper (Brick(2, 4)). color: a name in any spelling, or an LDraw code. at:
        (x, y, z) in studs, studs, plates. rot: quarter turns counterclockwise about z, seen from above, or a Rot. id:
        a stable name; by default "<step>/<n>".
        """
        file, line = caller()
        ctx = self._ctx("Model.add", file, line)
        rel = ctx.rel(file)
        _rotation(rot, rel, line)
        at = _point(at, rel, line)
        part_id = ctx.part_id(part, rel, line)
        code = ctx.color(color, rel, line)
        body = ctx.body(part_id, rel, line)
        _count(ctx, 1, rel, line)
        return self._put(part_id, code, body, at, rot, id, rel, line, None)

    def add_raw(self, part: str, color, matrix, id: str | None = None) -> Handle:
        """The escape hatch: places a part by an LDraw transform, in LDU in LDraw's frame (−Y up), as the 12 numbers
        of an LDraw line: x y z a b c d e f g h i. Use it for a part placed by its LDraw origin rather than its
        footprint, such as track at 22.5°, whose ends have to meet exactly."""
        file, line = caller()
        ctx = self._ctx("Model.add_raw", file, line)
        rel = ctx.rel(file)
        if self._mirrors:
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                "add_raw can't be mirrored.",
                "Place the mirrored part with its own add_raw outside the mirror block.",
                rel,
                line,
            )
        try:
            nums = [float(v) for v in matrix]
            if len(nums) != 12:
                raise ValueError
        except (TypeError, ValueError):
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"matrix={matrix!r} isn't 12 numbers.",
                "Pass the 12 numbers of an LDraw line after the color: x y z a b c d e f g h i, in LDU.",
                rel,
                line,
            ) from None
        part_id = ctx.part_id(part, rel, line)
        code = ctx.color(color, rel, line)
        body = ctx.body(part_id, rel, line)
        _count(ctx, 1, rel, line)
        dx, dy, dz = frame.to_ldraw_point(self._offset)
        pos = (nums[0] + dx, nums[1] + dy, nums[2] + dz)
        r = (tuple(nums[3:6]), tuple(nums[6:9]), tuple(nums[9:12]))
        box = frame.box_in_dsl(frame.transform_box(r, pos, body))
        return self._record(Placed("", part_id, code, pos, r, body, box, rel, line), id, rel, line)

    def _put(self, part, code, body, at, rot, id, rel, line, sub) -> Handle:
        """Places by footprint at `at` (plus the repeat offset), and its image under each open mirror."""
        at = tuple(at[i] + self._offset[i] for i in range(3))
        return self._put_all(part, code, body, at, rot, id, rel, line, sub, self._mirrors)

    def _put_all(self, part, code, body, at, rot, id, rel, line, sub, mirrors) -> Handle:
        """The part, then its image across the first mirror, each under the remaining mirrors: every combination."""
        if not mirrors:
            pos, r, box = frame.place(body, at, _rotation(rot, rel, line))
            return self._record(Placed("", part, code, pos, r, body, box, rel, line, sub=sub), id, rel, line)
        (axis, about), rest = mirrors[0], mirrors[1:]
        if id is None:
            id = self._default_id(sub)
        first = self._put_all(part, code, body, at, rot, id, rel, line, sub, rest)
        lo, hi = frame.place(body, at, _rotation(rot, rel, line))[2]
        at2 = (2 * about - hi[0], lo[1], lo[2]) if axis == "x" else (lo[0], 2 * about - hi[1], lo[2])
        turned = _mirror_rotation(rot, axis, rel, line)
        self._put_all(part, code, body, at2, turned, f"{id}-mirror-{axis}", rel, line, sub, rest)
        return first

    def _step_for_new(self) -> _Step:
        if self._current is not None:
            return self._current
        if self._loose is None:
            self._loose = _Step("", "part", auto=True)
            self._steps.append(self._loose)
        return self._loose

    def _default_id(self, sub: Submodel | None) -> str:
        # Counted per slug across the model, so two steps with one title, or loose parts before and after a step,
        # still get distinct ids. A submodel copy is numbered by the submodel's name.
        slug = _slug(sub.name) if sub else self._step_for_new().slug
        n = self._numbered.get(slug, 0) + 1
        self._numbered[slug] = n
        return f"{slug}/{n}"

    def _record(self, placed: Placed, id, rel: str, line: int) -> Handle:
        step = self._step_for_new()
        if id is None:
            id = self._default_id(placed.sub)
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
        placed.id = id
        step.parts.append(placed)
        self._ids[id] = placed
        return Handle(placed)

    # Helpers that place many parts

    def stack(self, part: str, color, at=(0, 0, 0), count: int = 1, rot: int | Rot = 0, id: str | None = None):
        """`count` copies of a part, each on top of the one before: a column, a chimney. Returns their Handles.
        With `id`, they're "<id>/1", "<id>/2", …"""
        handles = []
        x, y, z = at
        for k in range(count):
            h = self.add(part, color, at=(x, y, z), rot=rot, id=f"{id}/{k + 1}" if id else None)
            handles.append(h)
            z += h.max[2] - h.min[2]
        return handles

    def grid(self, part: str, color, at=(0, 0, 0), nx: int = 1, ny: int = 1, rot: int | Rot = 0, id: str | None = None):
        """nx × ny copies of a part side by side, each next to the last along x, then along y. Returns their
        Handles. With `id`, they're "<id>/1", "<id>/2", … row by row."""
        handles = []
        x0, y0, z = at
        y, k = y0, 0
        for _j in range(ny):
            x, depth = x0, 0
            for _i in range(nx):
                k += 1
                h = self.add(part, color, at=(x, y, z), rot=rot, id=f"{id}/{k}" if id else None)
                handles.append(h)
                x += h.max[0] - h.min[0]
                depth = h.max[1] - h.min[1]
            y += depth
        return handles

    def row(
        self,
        kind,
        color,
        at=(0, 0, 0),
        length: int = 1,
        axis: str = "x",
        width: int = 1,
        first=None,
        sizes=None,
        id: str | None = None,
    ):
        """A run of `length` studs along `axis` ("x" or "y") from `at`, `width` studs across, in the longest pieces
        of a family (Brick, Plate, or Tile) the library has, longest first. first: the length of the first piece
        (a running bond starts every other row with a shorter one). sizes: the piece lengths to use. Returns their
        Handles."""
        file, line = caller()
        ctx = self._ctx("Model.row", file, line)
        rel = ctx.rel(file)
        family = _family(kind, "row", rel, line)
        _axis(axis, rel, line)
        pieces = family.split(length, width, sizes, first, rel, line)
        handles = []
        x, y, z = at
        for n, part in pieces:
            h = self.add(
                part, color, at=(x, y, z), rot=_turn(axis, n, width), id=f"{id}/{len(handles) + 1}" if id else None
            )
            handles.append(h)
            x, y = (x + n, y) if axis == "x" else (x, y + n)
        return handles

    def fill(self, kind, color, at=(0, 0, 0), size=(2, 2), id: str | None = None):
        """Covers a size=(x, y) rectangle with plates or tiles (Plate or Tile): strips 2 studs wide along its long
        side, the longest pieces first, and a 1-wide strip when the short side is odd. Returns their Handles."""
        file, line = caller()
        ctx = self._ctx("Model.fill", file, line)
        rel = ctx.rel(file)
        family = _family(kind, "fill", rel, line)
        sx, sy = size
        along = "x" if sx >= sy else "y"
        long_side, short_side = (sx, sy) if along == "x" else (sy, sx)
        handles = []
        x, y, z = at
        done = 0
        while done < short_side:
            w = 2 if short_side - done >= 2 and family.has_width(2) else 1
            start = (x, y + done, z) if along == "x" else (x + done, y, z)
            for n, part in family.split(long_side, w, None, None, rel, line):
                k = len(handles) + 1
                h = self.add(part, color, at=start, rot=_turn(along, n, w), id=f"{id}/{k}" if id else None)
                handles.append(h)
                start = (start[0] + n, start[1], z) if along == "x" else (start[0], start[1] + n, z)
            done += w
        return handles

    def wall(
        self,
        x: float,
        y: float,
        z: float,
        length: int,
        depth: int = 0,
        color="Light Bluish Gray",
        sides="all",
        rows: int = 1,
        bond: str = "running",
        openings=(),
        kind=None,
        sizes=None,
    ):
        """Brick walls, a row at a time from z up. With depth, the four walls of a length × depth rectangle whose
        front-left corner is (x, y), with interlocking corners; without it, one straight wall along x. sides: "all",
        or some of "front", "back", "left", "right", and "sides" (left and right), to build a wall at a time. bond:
        "running" (no joint on a joint in the course below that a Model.wall laid, so each piece bridges one and the
        course over a door or window spans its edges) or "stacked". openings: (x, y) cells left empty, for doors and
        windows. kind: the piece family (Brick by default). sizes: the piece lengths to use. Returns the Handles."""
        file, line = caller()
        ctx = self._ctx("Model.wall", file, line)
        rel = ctx.rel(file)
        from .parts import Brick

        family = _family(kind or Brick, "wall", rel, line)
        if bond not in ("running", "stacked"):
            raise EmmetError(
                "MODEL_EXEC_FAILED", f"bond={bond!r} isn't a bond.", 'Use "running" or "stacked".', rel, line
            )
        wanted = _sides(sides, depth, rel, line)
        holes = {(round(c[0]), round(c[1])) for c in openings}
        handles = []
        for r in range(rows):
            zr = z + family.height * r
            course = round(zr / family.height)  # the course from the ground decides the corners and the stagger
            for name, (sx, sy), n, axis in _runs(x, y, length, depth, course % 2):
                if name not in wanted:
                    continue
                cells = [(sx + i, sy) if axis == "x" else (sx, sy + i) for i in range(n)]
                for (cx, cy), run in _gaps(cells, holes, axis):
                    # The joints of this line of wall, where Model.repeat has moved it, a course down and here.
                    ox, oy, oz = self._offset
                    across, start = (cy + oy, cx + ox) if axis == "x" else (cx + ox, cy + oy)
                    below = self._joints.get((axis, across, zr + oz - family.height), set())
                    here = self._joints.setdefault((axis, across, zr + oz), set())
                    # Over joints inside this run, lay the course in bond over them; over none (a run on one long
                    # piece, or the first course), a straight wall's odd course starts with a short piece instead.
                    if bond == "running" and any(start < j < start + run for j in below):
                        pieces = family.bond(start, run, below, sizes, rel, line)
                    else:
                        first = None
                        if bond == "running" and course % 2 and depth == 0:
                            first = max(1, family.longest(run, 1, sizes) // 2)
                        pieces = family.split(run, 1, sizes, first, rel, line)
                    here.add(start)
                    for size, part in pieces:
                        handles.append(self.add(part, color, at=(cx, cy, zr), rot=_turn(axis, size, 1)))
                        cx, cy = (cx + size, cy) if axis == "x" else (cx, cy + size)
                        start += size
                        here.add(start)
        return handles

    # Repetition and symmetry

    def repeat(self, fn, n: int, dx: float = 0, dy: float = 0, dz: float = 0) -> list:
        """Calls fn(i) for i in 0 … n−1 with every position it places moved by (i·dx, i·dy, i·dz). Returns what each
        call returned. Give parts placed with an id the index (id=f"lamp/{i}"), so each copy's ids differ."""
        takes_index = _takes_argument(fn)
        out = []
        saved = self._offset
        try:
            for i in range(n):
                self._offset = (saved[0] + i * dx, saved[1] + i * dy, saved[2] + i * dz)
                out.append(fn(i) if takes_index else fn())
        finally:
            self._offset = saved
        return out

    @contextlib.contextmanager
    def mirror(self, axis: str, about: float):
        """Everything placed inside is placed twice: as written, and mirrored across the plane x = about (axis "x")
        or y = about (axis "y"), moved with Model.repeat like any position. A mirrored part turns to face the other way;
        its id gets
        "-mirror-x" (or "-y"). Parts are turned, not reflected, so a left-handed part stays left-handed."""
        file, line = caller()
        _axis(axis, self._context.rel(file) if self._context else file, line)
        # In the frame Model.repeat has moved to, so a mirror inside a repeat mirrors each copy across its own plane.
        self._mirrors.append((axis, float(about) + self._offset[0 if axis == "x" else 1]))
        try:
            yield self
        finally:
            self._mirrors.pop()

    # Submodels

    def submodel(self, name: str, fn, **params) -> Submodel:
        """Defines a unit built once by fn(m, **params) on its own Model, with its own steps: a window, a lamp post,
        a tower. Model.place puts it in the model, as often as needed. Each name and parameter set is one file in the
        .mpd; calling submodel again with the same ones returns the same Submodel."""
        file, line = caller()
        ctx = self._ctx("Model.submodel", file, line)
        rel = ctx.rel(file)
        if not isinstance(name, str) or not name.strip():
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"name={name!r} isn't a name.",
                'Name the submodel in words, like "window".',
                rel,
                line,
            )
        key = (name, _function_key(fn), repr(sorted(params.items())))
        made = ctx.submodels.get(key)
        if made is not None:
            return made
        child = Model()
        # A copy's parts count when it's placed, not while it's built.
        counted = ctx.count
        result = fn(child, **params)
        ctx.count = counted
        built = result if isinstance(result, Model) else child
        if not any(s.parts for s in built._steps):
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"The submodel '{name}' has no parts.",
                f"{getattr(fn, '__name__', 'Its function')} must add parts to the Model it's given.",
                rel,
                line,
            )
        sub = Submodel(name, _file_name(ctx, name), built)
        ctx.submodels[key] = sub
        return sub

    def place(self, sub: Submodel, at=(0, 0, 0), rot: int | Rot = 0, id: str | None = None) -> Handle:
        """Places a copy of a submodel so the minimum corner of the box its parts cover, after rotation, is at `at`.
        The copy's parts get ids "<copy id>/<part id>"; the copy's id is by default "<submodel name>/<n>"."""
        file, line = caller()
        ctx = self._ctx("Model.place", file, line)
        rel = ctx.rel(file)
        if not isinstance(sub, Submodel):
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"{sub!r} isn't a submodel.",
                "Place what m.submodel(...) or import_ldraw(...) returned.",
                rel,
                line,
            )
        _rotation(rot, rel, line)
        at = _point(at, rel, line)
        _count(ctx, sub.size, rel, line)
        body = frame.box_in_ldraw(*sub.box)
        return self._put(sub.file, 16, body, at, rot, id, rel, line, sub)

    # Techniques

    def use(self, technique: str, **kw):
        """Applies a technique from Emmet's library (emmet technique lists them): gable_roof, snot_panel, window_frame,
        track_curve, … with its keyword arguments. Returns what the technique returns (track gives the pose where the
        next piece goes)."""
        file, line = caller()
        ctx = self._ctx("Model.use", file, line)
        module = _technique(technique, ctx.rel(file), line)
        return module.build(self, **kw)

    # What the validator should know

    def allow(self, rule: str, ids, reason: str) -> None:
        """Records a deliberate exception: findings of `rule` whose parts are all in `ids` show in Checks as
        "allowed", with the reason, never hidden. ids: an id, a Handle, or a list of them."""
        file, line = caller()
        rel = self._context.rel(file) if self._context else file
        if rule not in RULES:
            close = difflib.get_close_matches(str(rule), RULES, n=1)
            maybe = f"Did you mean '{close[0]}'? " if close else ""
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"There's no rule '{rule}'.",
                f"{maybe}The rules are: {', '.join(RULES)}.",
                rel,
                line,
            )
        if not isinstance(reason, str) or not reason.strip():
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                "m.allow needs a reason.",
                'Say why, in words the user will read: reason="held by hand in the photos".',
                rel,
                line,
            )
        self._allowed.append((rule, _ids(ids), reason.strip(), rel, line))

    def ground(self, *ids) -> None:
        """Marks parts the model stands on, besides its lowest level and any baseplate (a pier's footing on a raised
        base, say). ids: ids or Handles."""
        file, line = caller()
        rel = self._context.rel(file) if self._context else file
        found = [i for v in ids for i in _ids(v)]
        self._grounded.append((found, rel, line))

    def _ctx(self, what: str, file: str, line: int):
        ctx = self._context
        if ctx is None:
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"{what} ran outside emmet run.",
                "Run the model with emmet run; it supplies the parts and colors.",
                file,
                line,
            )
        return ctx


def in_steps(m: Model, title: str, actions) -> None:
    """For techniques and templates: runs actions, each (how many parts it adds, a function that adds them), in steps
    of at most the step target. A title that has to split is "title (1 of 2)", and so on; one action's parts (a frame
    and its glass) stay together."""
    target = _step_target(m._context)
    chunks, count = [[]], 0
    for n, add in actions:
        if chunks[-1] and count + n > target:
            chunks.append([])
            count = 0
        chunks[-1].append(add)
        count += n
    chunks = [c for c in chunks if c]
    for i, chunk in enumerate(chunks):
        with m.step(title if len(chunks) == 1 else f"{title} ({i + 1} of {len(chunks)})"):
            for add in chunk:
                add()


def row_actions(m: Model, kind, color, at, length: int, axis: str = "x", width: int = 1, sizes=None) -> list:
    """For techniques and templates: Model.row as in_steps actions, one a piece, so a long run splits across steps."""
    file, line = caller()
    rel = m._context.rel(file) if m._context else file
    family = _family(kind, "row", rel, line)
    _axis(axis, rel, line)
    actions = []
    x, y, z = at
    for n, part in family.split(length, width, sizes, None, rel, line):
        actions.append(
            (1, lambda part=part, x=x, y=y, n=n: m.add(part, color, at=(x, y, z), rot=_turn(axis, n, width)))
        )
        x, y = (x + n, y) if axis == "x" else (x, y + n)
    return actions


def suggest(name: str, choices) -> list[str]:
    return difflib.get_close_matches(name, list(choices), n=3, cutoff=0.5)


def _count(ctx, n: int, rel: str, line: int) -> None:
    ctx.count += n
    if ctx.count > MAX_PARTS:
        raise EmmetError(
            "MODEL_TOO_LARGE",
            f"The model places more than {MAX_PARTS:,} parts.",
            "Reduce repeats, or use larger parts.",
            rel,
            line,
        )


def _step_target(ctx) -> int:
    return getattr(ctx, "step_target", None) or 8


def _layers(step: _Step, target: int) -> list[_Step]:
    """Loose parts as steps, bottom up: whole layers (parts starting at one height) together while they fit the
    target, and a layer bigger than the target split into runs of the target."""
    by_z: dict[float, list] = {}
    for p in step.parts:
        by_z.setdefault(round(p.box[0][2], 6), []).append(p)
    out: list[_Step] = []
    current: list = []
    for z in sorted(by_z):
        layer = by_z[z]
        if current and len(current) + len(layer) > target:
            out.append(current)
            current = []
        while len(layer) > target:
            out.append(layer[:target])
            layer = layer[target:]
        current = current + layer
    if current:
        out.append(current)
    # Layers decide where steps break; within a step, parts stay in the order they were added.
    order = {id(p): i for i, p in enumerate(step.parts)}
    return [_Step(step.title, step.slug, sorted(parts, key=lambda p: order[id(p)]), auto=True) for parts in out]


def _point(at, rel: str, line: int) -> tuple:
    try:
        point = tuple(float(v) for v in at)
        if len(point) != 3:
            raise ValueError
        return point
    except (TypeError, ValueError):
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"at={at!r} isn't an (x, y, z) position.",
            "Pass at=(x, y, z): studs right, studs back, plates up.",
            rel,
            line,
        ) from None


def _rotation(rot, rel: str, line: int) -> frame.Mat3:
    if isinstance(rot, Rot):
        return rot._matrix()
    if not isinstance(rot, int) or isinstance(rot, bool):
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"rot={rot!r} isn't a whole number of quarter turns.",
            "Use rot=0, 1, 2, or 3 (quarter turns counterclockwise, seen from above), or Rot(z=22.5) for any angle.",
            rel,
            line,
        )
    return frame.quarter_turns(rot)


def _mirror_rotation(rot, axis: str, rel: str, line: int):
    """The turn that faces a part the other way across the mirror plane."""
    if isinstance(rot, Rot):
        if rot.x or rot.y:
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"{rot!r} can't be mirrored: only turns about z can.",
                "Mirror parts turned about z only, or place the other side yourself.",
                rel,
                line,
            )
        return Rot(z=180 - rot.z if axis == "x" else -rot.z)
    return (2 - rot) % 4 if axis == "x" else (-rot) % 4


def _turn(axis: str, along: int, across: int) -> int:
    """The quarter turn that lays a family piece `along` studs along the axis and `across` studs across it: a piece's
    long side runs along y at rot=0."""
    if along == across:
        return 0
    return 1 if (axis == "x") == (along > across) else 0


def _axis(axis: str, rel: str, line: int) -> None:
    if axis not in ("x", "y"):
        raise EmmetError("MODEL_EXEC_FAILED", f"axis={axis!r} isn't an axis.", 'Use "x" or "y".', rel, line)


def _takes_argument(fn) -> bool:
    try:
        params = [p for p in inspect.signature(fn).parameters.values() if p.default is inspect.Parameter.empty]
    except (TypeError, ValueError):
        return True
    return any(p.kind in (p.POSITIONAL_ONLY, p.POSITIONAL_OR_KEYWORD, p.VAR_POSITIONAL) for p in params)


def _function_key(fn) -> tuple:
    """What makes two builder functions the same: their code, their defaults, and what they close over. Lambdas
    share a qualname, so `lambda t, color=color: …` in a loop must differ by its default, not look alike."""
    cells = tuple(repr(c.cell_contents) for c in getattr(fn, "__closure__", None) or () if _filled(c))
    return (
        getattr(fn, "__code__", fn),
        repr(getattr(fn, "__defaults__", None)),
        repr(getattr(fn, "__kwdefaults__", None)),
        cells,
    )


def _filled(cell) -> bool:
    try:
        cell.cell_contents  # noqa: B018 - an empty cell raises
    except ValueError:
        return False
    return True


def _file_name(ctx, name: str) -> str:
    """The submodel's file in the .mpd: its name, then "-2", "-3", … for each other parameter set."""
    # The main model's file is taken too: a submodel named like the @model function would reference itself.
    taken = {s.file for s in ctx.submodels.values()} | {getattr(ctx, "main_file", "")}
    base = _slug(name)
    file, n = f"{base}.ldr", 1
    while file in taken:
        n += 1
        file = f"{base}-{n}.ldr"
    return file


def _family(kind, what: str, rel: str, line: int):
    from .parts import Family

    if isinstance(kind, Family):
        return kind
    raise EmmetError(
        "MODEL_EXEC_FAILED",
        f"{what} takes a family of parts, not {kind!r}.",
        f"Pass Brick, Plate, or Tile itself: m.{what}(Plate, ...).",
        rel,
        line,
    )


def _sides(sides, depth: int, rel: str, line: int) -> set[str]:
    if depth == 0:
        return {"front"}
    names = [sides] if isinstance(sides, str) else list(sides)
    out = set()
    for n in names:
        if n == "all":
            out |= {"front", "back", "left", "right"}
        elif n == "sides":
            out |= {"left", "right"}
        elif n in ("front", "back", "left", "right"):
            out.add(n)
        else:
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"sides={n!r} isn't a side.",
                'Use "all", "front", "back", "left", "right", or "sides".',
                rel,
                line,
            )
    return out


def _runs(x, y, length: int, depth: int, odd: int):
    """The walls of one course: (name, start, length, axis). The front and back own the corners on even courses, the
    sides on odd ones, so corners interlock. A straight wall (depth 0) is its front."""
    if depth == 0:
        return [("front", (x, y), length, "x")]
    x1, y1 = x + length - 1, y + depth - 1
    if not odd:
        return [
            ("back", (x, y1), length, "x"),
            ("left", (x, y + 1), depth - 2, "y"),
            ("right", (x1, y + 1), depth - 2, "y"),
            ("front", (x, y), length, "x"),
        ]
    return [
        ("back", (x + 1, y1), length - 2, "x"),
        ("left", (x, y), depth, "y"),
        ("right", (x1, y), depth, "y"),
        ("front", (x + 1, y), length - 2, "x"),
    ]


def _gaps(cells, holes, axis: str):
    """Unbroken runs of the cells not in holes: [(first cell, length), …]."""
    out = []
    for c in cells:
        if (round(c[0]), round(c[1])) in holes:
            continue
        if out:
            (lx, ly), n = out[-1]
            if c == ((lx + n, ly) if axis == "x" else (lx, ly + n)):
                out[-1] = ((lx, ly), n + 1)
                continue
        out.append((c, 1))
    return out


def _technique(name: str, rel: str, line: int):
    if not isinstance(name, str) or not name.isidentifier():
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"{name!r} isn't a technique name.",
            'Name one from emmet technique, like "gable_roof".',
            rel,
            line,
        )
    try:
        return importlib.import_module(f"emmet_dsl.techniques.{name}")
    except ModuleNotFoundError as e:
        if e.name != f"emmet_dsl.techniques.{name}" and e.name != "emmet_dsl.techniques":
            raise
        known = techniques()
        close = difflib.get_close_matches(name, known, n=3)
        maybe = f"Did you mean {', '.join(repr(c) for c in close)}? " if close else ""
        raise EmmetError(
            "MODEL_EXEC_FAILED", f"There's no technique '{name}'.", f"{maybe}emmet technique lists them all.", rel, line
        ) from None


def techniques() -> list[str]:
    """The techniques this emmet_dsl has, by name."""
    try:
        package = importlib.import_module("emmet_dsl.techniques")
    except ModuleNotFoundError:
        return []
    return sorted(m.name for m in pkgutil.iter_modules(package.__path__) if not m.name.startswith("_"))
