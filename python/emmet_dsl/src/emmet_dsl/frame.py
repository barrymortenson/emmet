# SPDX-License-Identifier: Apache-2.0
"""The DSL frame and footprint placement (plan section 08).

The DSL is Z-up: x runs right and y runs back, both in studs; z counts plates. LDraw is −Y-up in LDU (1 stud = 20,
1 plate = 8). A DSL point (x, y, z) is the LDraw point (20·y, −8·z, −20·x), so a 2 × 4 brick, whose long side is
LDraw's X, covers x 0–2 and y 0–4 at rot=0. The map is a proper rotation plus scale, so rotations stay rotations.
`rot=k` turns the part k quarter turns counterclockwise, seen from above; `Rot(x=…, y=…, z=…)` turns it by any angles.
"""

from __future__ import annotations

import math

STUD = 20
PLATE = 8

# Unit vectors: row i is LDraw axis i written in DSL axes (LDraw X = DSL y, Y = −z, Z = −x).
_A = ((0, 1, 0), (0, 0, -1), (-1, 0, 0))
_AT = tuple(tuple(_A[j][i] for j in range(3)) for i in range(3))

Mat3 = tuple[tuple[float, float, float], tuple[float, float, float], tuple[float, float, float]]


def _mul(a: Mat3, b: Mat3) -> Mat3:
    return tuple(tuple(sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)) for i in range(3))


def quarter_turns(k: int) -> Mat3:
    """Rotation by k × 90° about DSL z, counterclockwise from above."""
    c, s = ((1, 0), (0, 1), (-1, 0), (0, -1))[k % 4]
    return ((c, -s, 0), (s, c, 0), (0, 0, 1))


def to_ldraw_rotation(r_dsl: Mat3) -> Mat3:
    """The same rotation written in LDraw axes: A · R · Aᵀ."""
    return _mul(_mul(_A, r_dsl), _AT)


def box_in_dsl(body: list[float]) -> tuple[tuple[float, float, float], tuple[float, float, float]]:
    """A body box [min X, min Y, min Z, max X, max Y, max Z] in LDU as (min, max) in DSL units."""
    x0, y0, z0, x1, y1, z1 = body
    return ((-z1 / STUD, x0 / STUD, -y1 / PLATE), (-z0 / STUD, x1 / STUD, -y0 / PLATE))


def box_in_ldraw(lo, hi) -> list[float]:
    """The inverse of `box_in_dsl`: a DSL box as [min X, min Y, min Z, max X, max Y, max Z] in LDU."""
    return [STUD * lo[1], -PLATE * hi[2], -STUD * hi[0], STUD * hi[1], -PLATE * lo[2], -STUD * lo[0]]


def to_ldraw_point(p) -> tuple[float, float, float]:
    """A DSL point in LDU, LDraw's frame."""
    return (STUD * p[1], -PLATE * p[2], -STUD * p[0])


def transform_box(r: Mat3, pos, body: list[float]) -> list[float]:
    """The axis-aligned LDU box around `body` turned by r and moved by pos (both in LDraw's frame)."""
    corners = [(x, y, z) for x in (body[0], body[3]) for y in (body[1], body[4]) for z in (body[2], body[5])]
    turned = [tuple(sum(r[i][k] * c[k] for k in range(3)) + pos[i] for i in range(3)) for c in corners]
    return [min(c[i] for c in turned) for i in range(3)] + [max(c[i] for c in turned) for i in range(3)]


def compose(outer_r: Mat3, outer_pos, r: Mat3, pos):
    """The transform `outer` ∘ (r, pos): a part placed by (r, pos) inside a submodel placed by outer."""
    moved = tuple(sum(outer_r[i][k] * pos[k] for k in range(3)) + outer_pos[i] for i in range(3))
    return _mul(outer_r, r), moved


def rotation(x: float = 0, y: float = 0, z: float = 0) -> Mat3:
    """Turns of x, y, then z degrees about the DSL axes, counterclockwise looking down each axis."""

    def turn(deg, axis):
        c, s = _snap(math.cos(math.radians(deg))), _snap(math.sin(math.radians(deg)))
        if axis == 0:
            return ((1, 0, 0), (0, c, -s), (0, s, c))
        if axis == 1:
            return ((c, 0, s), (0, 1, 0), (-s, 0, c))
        return ((c, -s, 0), (s, c, 0), (0, 0, 1))

    return _mul(turn(z, 2), _mul(turn(y, 1), turn(x, 0)))


def _snap(v: float) -> float:
    """cos(90°) is 6e-17, not 0: values within float error of 0 or ±1 snap to them, so quarter turns stay exact."""
    for target in (0.0, 1.0, -1.0):
        if abs(v - target) < 1e-12:
            return target
    return v


def place(body: list[float], at: tuple[float, float, float], r_dsl: Mat3):
    """Footprint placement: the part's origin in LDU and its rotation in LDraw axes, such that the minimum corner of
    its rotated body box lands on `at`, and the DSL box it covers."""
    r = to_ldraw_rotation(r_dsl)
    lo, hi = box_in_dsl(transform_box(r, (0, 0, 0), body))
    origin = tuple(at[i] - lo[i] for i in range(3))
    covered = (tuple(at), tuple(at[i] + hi[i] - lo[i] for i in range(3)))
    return to_ldraw_point(origin), r, covered


def num(v: float) -> str:
    """A number for an LDraw file or JSON: integers bare, others with at most 6 decimals, never -0."""
    r = round(v, 6)
    if r == 0:
        return "0"
    if r == int(r):
        return str(int(r))
    return f"{r:.6f}".rstrip("0").rstrip(".")


def clean(v: float) -> int | float:
    """The same value as `num`, as a JSON number."""
    r = round(v, 6)
    if r == int(r):
        return int(r)
    return r
