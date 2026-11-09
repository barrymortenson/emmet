# SPDX-License-Identifier: Apache-2.0
"""Technic by holes (plan section 08, session 4G): a part's holes, pins, and axles in the model's frame, a beam laid
from one hole to another on a triangle that lands on holes, and a pin or axle seated in a named hole.

emmet passes each part's Technic points in build/part-geometry.json ("holes": name, kind, middle, axis, and length in
the part's LDraw frame, LDU), the same points part_info names. Here they're turned into the DSL frame: x and y in
studs, z in plates, directions as unit vectors in space along the DSL's axes.
"""

from __future__ import annotations

import math

from . import frame
from .errors import EmmetError

STUD = frame.STUD
PLATE = frame.PLATE

HOLE_KINDS = ("pin_hole", "axle_hole")
MALE_KINDS = ("pin", "axle", "pin_axle")

# Whole-number triangles, by their long side: the run and rise that land a beam's far hole on a hole of the grid.
TRIANGLES = ((3, 4, 5), (6, 8, 10), (5, 12, 13), (9, 12, 15), (8, 15, 17))


class Hole:
    """A Technic point of a placed part, in the model's frame: what Handle.hole returns.

    name: its name in part_info ("1" to "n"). kind: "pin_hole", "axle_hole", "pin", "axle", or "pin_axle". at: its
    middle, (x, y, z) in studs, studs, plates. axis: a unit direction along it, in the DSL's axes. length: in studs (a
    beam's hole is 1, a thin liftarm's 0.5). part and id: the part it belongs to."""

    __slots__ = ("name", "kind", "at", "axis", "length", "part", "id", "pending")

    def __init__(self, name, kind, at, axis, length, part, id, pending=False):
        self.name, self.kind, self.at, self.axis, self.length = name, kind, at, axis, length
        self.part, self.id, self.pending = part, id, pending

    def __repr__(self) -> str:
        at = ", ".join(frame.num(v) for v in self.at)
        return f"<{self.kind.replace('_', ' ')} {self.name} of {self.id} at ({at}) along {axis_words(self.axis)}>"


# Frames: LDraw points and directions, the DSL frame, and space in LDU along the DSL's axes ("physical").


def _apply(r, v, t=(0, 0, 0)):
    return tuple(sum(r[i][k] * v[k] for k in range(3)) + t[i] for i in range(3))


def _dsl_point(p):
    return (-p[2] / STUD, p[0] / STUD, -p[1] / PLATE)


def _dsl_dir(d):
    return (-d[2], d[0], -d[1])


def _ldraw_dir(d):
    return (d[1], -d[2], -d[0])


def _phys(p):
    """A DSL point in LDU along the DSL's axes."""
    return (p[0] * STUD, p[1] * STUD, p[2] * PLATE)


def _from_phys(v):
    return (v[0] / STUD, v[1] / STUD, v[2] / PLATE)


def _add(a, b, k=1.0):
    return tuple(a[i] + k * b[i] for i in range(3))


def _dot(a, b):
    return sum(a[i] * b[i] for i in range(3))


def _cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def _unit(v):
    n = math.sqrt(_dot(v, v))
    return tuple(x / n for x in v) if n > 1e-12 else (0.0, 0.0, 0.0)


def _snap_m(r):
    return tuple(tuple(frame._snap(round(v, 12)) for v in row) for row in r)


def _outer_sum(pairs):
    """Σ a·bᵀ over (a, b) pairs: the rotation that takes each b to its a, for orthonormal triples."""
    return tuple(tuple(sum(a[i] * b[j] for a, b in pairs) for j in range(3)) for i in range(3))


def axis_words(a) -> str:
    for i, name in enumerate("xyz"):
        if abs(abs(a[i]) - 1) < 1e-6:
            return ("-" if a[i] < 0 else "+") + name
    return "(" + ", ".join(frame.num(v) for v in a) + ")"


# The points emmet sent


def points(ctx, part: str):
    """The part's points as emmet sent them, or None before emmet has the part's geometry (a pass that's only
    collecting parts)."""
    found = ctx.geometry.get(part)
    if part.startswith("?") or found is None:
        return None
    return found.get("holes", [])


def world_holes(ctx, placed) -> list[Hole]:
    """A placed part's points in the frame of the model it was placed in."""
    pts = points(ctx, placed.part)
    out = []
    for p in pts or []:
        mid = _apply(placed.rot, p["mid"], placed.pos)
        axis = _unit(_dsl_dir(_apply(placed.rot, p["axis"])))
        at = tuple(frame.clean(v) for v in _dsl_point(mid))
        out.append(
            Hole(p["name"], p["kind"], at, tuple(frame.clean(v) for v in axis), p["len"] / STUD, placed.part, placed.id)
        )
    return out


def _names(pts) -> str:
    names = [p["name"] for p in pts]
    if names == [str(i) for i in range(1, len(names) + 1)]:
        return f"1 to {len(names)}" if len(names) > 1 else "1"
    return ", ".join(names)


def find(pts, name, what: str, rel: str, line: int) -> dict:
    """The point called `name` (a number or a string) among `pts`."""
    key = str(name)
    for p in pts:
        if p["name"] == key:
            return p
    if not pts:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"{what} has no Technic holes, pins, or axles.",
            "part_info (emmet part <id>) lists a part's holes; pick a part that has them.",
            rel,
            line,
        )
    raise EmmetError(
        "MODEL_EXEC_FAILED",
        f"{what} has no hole {key!r}.",
        f"Its holes are {_names(pts)}; part_info (emmet part <id>) says where each one is.",
        rel,
        line,
    )


def hole_of(ctx, placed, name, rel: str, line: int) -> Hole:
    pts = points(ctx, placed.part)
    if pts is None:  # emmet hasn't sent the part yet: any point will do until the next pass
        return Hole(str(name), "pin_hole", (0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 1.0, placed.part, placed.id, pending=True)
    find(pts, name, f"{placed.id} ({placed.part})", rel, line)
    return next(h for h in world_holes(ctx, placed) if h.name == str(name))


def corner_for_hole(ctx, part: str, body, name, target, r_dsl, rel: str, line: int):
    """The footprint corner that puts the part's hole `name` at `target`, turned by r_dsl."""
    pts = points(ctx, part)
    if pts is None:
        return tuple(target)
    p = find(pts, name, part, rel, line)
    origin, r, _ = frame.place(body, (0, 0, 0), r_dsl)
    d = _dsl_point(_apply(r, p["mid"], origin))
    return tuple(target[i] - d[i] for i in range(3))


# Beams on triangles

_PLANES = {
    # axis: (the hole axis, the run's direction, the rise's direction), in the DSL's axes
    "x": ((1, 0, 0), (0, 1, 0), (0, 0, 1)),
    "y": ((0, 1, 0), (1, 0, 0), (0, 0, 1)),
    "z": ((0, 0, 1), (1, 0, 0), (0, 1, 0)),
}


def plane(axis, rel: str, line: int):
    if not isinstance(axis, str) or axis not in _PLANES:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"axis={axis!r} isn't an axis.",
            'Use "x" or "y" for a beam standing on edge with its holes along that axis, or "z" for a beam lying flat.',
            rel,
            line,
        )
    return _PLANES[axis]


def _triangles(reach: float) -> str:
    fits = [t for t in TRIANGLES if t[2] <= reach + 1e-6]
    if not fits:
        return "Lay it level (rise=0) or upright (run=0)"
    return "Triangles that land on holes: " + ", ".join(
        f"run {a}, rise {b} (or {b}, {a}) for holes {c} apart" for a, b, c in fits
    )


def beam_pose(ctx, part: str, at, direction, normal, holes, reach, said, rel: str, line: int):
    """The LDraw (pos, rotation) that puts the part's hole holes[0] at the DSL point `at` and its far hole `reach`
    studs along `direction` (a unit vector in space along the DSL's axes), hole holes[0] running along `normal`. `said`
    names what set the reach, for messages ("A run of 12 and a rise of 6"). None while emmet hasn't sent the part."""
    pts = points(ctx, part)
    if pts is None:
        return None
    first, far = holes
    a = find(pts, first, part, rel, line)
    on_line = [
        p for p in pts if p is not a and abs(abs(_dot(p["axis"], a["axis"])) - 1) < 1e-3 and p["kind"] in HOLE_KINDS
    ]
    gaps = {p["name"]: math.dist(p["mid"], a["mid"]) / STUD for p in on_line}
    if far is None:
        close = [n for n, g in gaps.items() if abs(g - reach) <= 0.025]
        if not close:
            have = sorted({round(g, 3) for g in gaps.values()})
            what = ", ".join(frame.num(g) for g in have) or "none"
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"{said} reach {frame.num(round(reach, 3))} studs, "
                f"but {part}'s holes are {what} studs from hole {a['name']}.",
                f"{_triangles(max(have, default=0))}; or use a longer beam.",
                rel,
                line,
            )
        far = close[0]
    b = find(pts, far, part, rel, line)
    gap = math.dist(b["mid"], a["mid"]) / STUD
    if abs(gap - reach) > 0.025:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"{part}'s holes {a['name']} and {b['name']} are {frame.num(round(gap, 3))} studs apart, but "
            f"{said[0].lower()}{said[1:]} reach {frame.num(round(reach, 3))}.",
            f"{_triangles(gap)}.",
            rel,
            line,
        )
    u = _unit(_add(b["mid"], a["mid"], -1))
    v = _unit(a["axis"])
    if abs(_dot(u, v)) > 1e-3:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"{part}'s hole {b['name']} isn't square to hole {a['name']}'s axis.",
            "Pick two holes side by side (a beam's), or a hole and one beside its axis (a cross block's).",
            rel,
            line,
        )
    U, V = _ldraw_dir(direction), _ldraw_dir(normal)
    r = _snap_m(_outer_sum([(U, u), (V, v), (_cross(U, V), _cross(u, v))]))
    target = frame.to_ldraw_point(at)
    pos = _add(target, _apply(r, a["mid"]), -1)
    return pos, r


# Pins and axles in holes


def _rotation_onto(a, b):
    """The smallest rotation taking unit vector a onto unit vector b (LDraw directions)."""
    c = _dot(a, b)
    if c < -1 + 1e-9:
        # Half a turn about a direction square to a.
        ref = (0, 1, 0) if abs(a[1]) < 0.9 else (1, 0, 0)
        p = _unit(_add(ref, a, -_dot(ref, a)))
        return _snap_m(tuple(tuple(2 * p[i] * p[j] - (1 if i == j else 0) for j in range(3)) for i in range(3)))
    k = _cross(a, b)
    kx = ((0, -k[2], k[1]), (k[2], 0, -k[0]), (-k[1], k[0], 0))
    kx2 = tuple(tuple(sum(kx[i][m] * kx[m][j] for m in range(3)) for j in range(3)) for i in range(3))
    f = 1 / (1 + c)
    return _snap_m(tuple(tuple((1 if i == j else 0) + kx[i][j] + f * kx2[i][j] for j in range(3)) for i in range(3)))


def out_direction(hole: Hole, out, rel: str, line: int):
    """The way a pin sticks out of `hole`, a unit vector along the hole's axis: "+y", "-x", …, "+" and "-" along the
    hole's own axis, or a direction (x, y, z) along it; by default the positive way along it."""
    axis = hole.axis
    # The hole's axis, its largest part positive: "+".
    big = max(range(3), key=lambda i: abs(axis[i]))
    plus = axis if axis[big] > 0 else tuple(-v for v in axis)
    if out in (None, "+"):
        return plus
    if isinstance(out, tuple | list) and len(out) == 3:
        v = _unit(tuple(float(x) for x in out))
        if abs(abs(_dot(v, axis)) - 1) > 1e-3:
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"out={tuple(out)!r} isn't along hole {hole.name} of {hole.id}, which runs along {axis_words(plus)}.",
                "Give a direction along the hole's axis (hole.axis, or its opposite).",
                rel,
                line,
            )
        return v
    if out == "-":
        return tuple(-v for v in plus)
    if isinstance(out, str) and len(out) == 2 and out[0] in "+-" and out[1] in "xyz":
        v = [0.0, 0.0, 0.0]
        v[" xyz".index(out[1]) - 1] = 1.0 if out[0] == "+" else -1.0
        if abs(abs(_dot(v, axis)) - 1) > 1e-3:
            raise EmmetError(
                "MODEL_EXEC_FAILED",
                f"out={out!r} isn't along hole {hole.name} of {hole.id}, which runs along {axis_words(plus)[1:]}.",
                f'Use out="{axis_words(plus)}" or out="{axis_words(tuple(-x for x in plus))}".',
                rel,
                line,
            )
        return tuple(v)
    raise EmmetError(
        "MODEL_EXEC_FAILED",
        f"out={out!r} isn't a direction.",
        'Use "+x", "-x", "+y", "-y", "+z", or "-z" along the hole, or "+" and "-" along its axis.',
        rel,
        line,
    )


def male(pts, name, part: str, rel: str, line: int) -> dict:
    """The pin or axle of the part to insert: `name`, or its first one."""
    if name is not None:
        p = find(pts, name, part, rel, line)
    else:
        p = next((p for p in pts if p["kind"] in MALE_KINDS), None)
    if p is None or p["kind"] not in MALE_KINDS:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"{part} has no pin or axle to put in a hole.",
            "Insert a pin (2780), an axle (4519 is 3 long), or an axle pin (3749); place other parts with m.add.",
            rel,
            line,
        )
    return p


def insert_pose(ctx, part: str, hole: Hole, o, out_by, point, rel: str, line: int):
    """The LDraw (pos, rotation) that seats the part's pin or axle in `hole`, sticking out `out_by` studs past the
    hole's face along `o` (a unit vector along the DSL's axes). None while emmet hasn't sent the part or the hole."""
    pts = points(ctx, part)
    if pts is None or hole.pending:
        return None
    p = male(pts, point, part, rel, line)
    if hole.kind not in HOLE_KINDS:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"{hole.id}'s point {hole.name} is a {hole.kind.replace('_', ' ')}, not a hole.",
            "Insert into a pin hole or an axle hole (Handle.hole names them).",
            rel,
            line,
        )
    if p["kind"] == "pin" and hole.kind == "axle_hole":
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"{part} is a round pin, which doesn't fit hole {hole.name} of {hole.id}, an axle hole.",
            "Use an axle, or an axle pin (3749) with its axle end in the hole.",
            rel,
            line,
        )
    length = p["len"] / STUD
    if out_by is None:
        out_by = max(0.0, length - hole.length)
    if not isinstance(out_by, int | float) or isinstance(out_by, bool) or not 0 <= out_by <= length:
        raise EmmetError(
            "MODEL_EXEC_FAILED",
            f"out_by={out_by!r} isn't between 0 and {frame.num(length)} studs, the length of {part}.",
            "Give how many studs it sticks out past the hole's face.",
            rel,
            line,
        )
    # It goes in by its start (a 3/4 pin's long end, an axle pin's pin end), unless its start is its part's body (an
    # axle stub on a hub, a pin with a stop bush), or it's an axle pin going into an axle hole: then by its far end.
    toward = o
    if bool(p.get("closed")) != (p["kind"] == "pin_axle" and hole.kind == "axle_hole"):
        toward = tuple(-v for v in o)
    r = _rotation_onto(_unit(p["axis"]), _ldraw_dir(toward))
    face = _add(_phys(hole.at), o, hole.length * STUD / 2)
    mid = _add(face, o, (out_by - length / 2) * STUD)
    target = frame.to_ldraw_point(_from_phys(mid))
    pos = _add(target, _apply(r, p["mid"]), -1)
    return pos, r


def reflect_point(p, k: int, about: float):
    q = list(p)
    q[k] = 2 * about - q[k]
    return tuple(q)


def reflect_dir(d, k: int):
    q = list(d)
    q[k] = -q[k]
    return tuple(q)
