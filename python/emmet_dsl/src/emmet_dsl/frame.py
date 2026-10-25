# SPDX-License-Identifier: Apache-2.0
"""The DSL frame and footprint placement (plan section 08).

The DSL is Z-up: x runs right and y runs back, both in studs; z counts plates. LDraw is −Y-up in LDU (1 stud = 20,
1 plate = 8). A DSL point (x, y, z) is the LDraw point (20·y, −8·z, −20·x), so a 2 × 4 brick, whose long side is
LDraw's X, covers x 0–2 and y 0–4 at rot=0. The map is a proper rotation plus scale, so rotations stay rotations.
`rot=k` turns the part k quarter turns counterclockwise, seen from above.
"""

from __future__ import annotations

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


def rotated_box(lo, hi, r: Mat3):
    """The axis-aligned box around box (lo, hi) after rotation r about the origin."""
    corners = [(x, y, z) for x in (lo[0], hi[0]) for y in (lo[1], hi[1]) for z in (lo[2], hi[2])]
    turned = [tuple(sum(r[i][k] * c[k] for k in range(3)) for i in range(3)) for c in corners]
    return (
        tuple(min(c[i] for c in turned) for i in range(3)),
        tuple(max(c[i] for c in turned) for i in range(3)),
    )


def place(body: list[float], at: tuple[float, float, float], rot: int):
    """Footprint placement: the part's origin in LDU and its rotation in LDraw axes, such that the minimum corner of
    its rotated body box lands on `at`, and the DSL box it covers."""
    r = quarter_turns(rot)
    lo, hi = rotated_box(*box_in_dsl(body), r)
    origin = tuple(at[i] - lo[i] for i in range(3))
    pos = (STUD * origin[1], -PLATE * origin[2], -STUD * origin[0])
    covered = (tuple(at), tuple(at[i] + hi[i] - lo[i] for i in range(3)))
    return pos, to_ldraw_rotation(r), covered


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
