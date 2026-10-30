# SPDX-License-Identifier: Apache-2.0
"""Cross gable: a 45° gable roof with a small gable across its front face, over the door, that breaks the long roof
line and can carry a nameplate.

m.use("cross_gable", x, y, z, length, depth, at, project=2, color="Dark Red", gable="White", nameplate="Tan",
title="Roof") lays a roof course for course like gable_roof (slopes along x facing front and back, a stud in each
course, bricks behind, double slopes along the ridge) over the length x depth rectangle from (x, y) at z, but leaves
its first two front courses open from `at` to at + 4, where the cross gable goes: slopes along y facing left and
right, running from `project` studs in front of the roof back into it, double slopes along its own ridge, and a row
of bricks behind it that carries the main roof's third course over the valley. Projecting 2, the slopes' front studs
are `gable` color, so the gable's face reads as a white triangle, over a `gable` plate 2 x 4 under the projection
(the soffit, which ties them on). With `nameplate` (a color, or None for none), a brick 1 x 4 with studs on its side
hangs under the cross gable's front, faced with a tile 1 x 4: the sign. depth must be even, 6 or more;
project 1 or 2; at from x + 1 to x + length - 5. It brings its own steps and returns the z of the ridge's top.
"""

from emmet_dsl import Brick, Rot, Slope
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21335, set 11387. Not derived
from any model file."""

PARTS = [
    "3037",
    "3038",
    "3039",
    "3040b",
    "3043",
    "3044c",
    "3010",
    "3622",
    "3004",
    "3005",
    "3020",
    "30414",
    "2431",
]

RIDGES = {2: "3043", 1: "3044c"}
DOWN = {"+x": 0, "+y": 1, "-x": 2, "-y": 3}
SIGN = Rot(y=-90, z=90)  # a tile on its back, its face to -y, long along x


def _lengths(n, sizes):
    out = []
    for size in sorted(sizes, reverse=True):
        while n >= size:
            out.append(size)
            n -= size
    return out


def _run(m, part_of, color, x0, x1, at_y, z, turn):
    """Pieces from x0 to x1 along x, the longest first, as in_steps actions."""
    actions, sx = [], x0
    for n in _lengths(x1 - x0, part_of):
        actions.append(
            (1, lambda n=n, sx=sx: m.add(part_of[n], color, at=(sx, at_y, z), rot=turn))
        )
        sx += n
    return actions


def build(
    m,
    x,
    y,
    z,
    length,
    depth,
    at,
    project=2,
    color="Dark Red",
    gable="White",
    nameplate="Tan",
    title="Roof",
):
    """The main roof with the cross gable in its front; returns the z of the ridge's top."""
    if depth < 6 or depth % 2:
        raise ValueError(f"cross_gable needs an even depth of 6 or more, not {depth}")
    if project not in (1, 2) or not x + 1 <= at <= x + length - 5:
        raise ValueError(
            f"cross_gable takes project 1 or 2 and at from {x + 1} to {x + length - 5}"
        )
    slopes = {n: Slope(45, 2, n) for n in (4, 3, 2, 1)}
    bricks = {n: Brick(1, n) for n in (4, 3, 2, 1)}
    courses = (depth - 4) // 2 + 1
    y1 = y + depth - 1
    gap = (at, at + 4)
    for k in range(courses):
        zk = z + 3 * k
        inner = list(range(y + k + 2, y1 - 1 - k))
        half = (len(inner) + 1) // 2
        for side, sy, turn, rows in (
            ("front", y + k, 3, inner[:half]),
            ("back", y1 - 1 - k, 1, inner[half:]),
        ):
            actions = []
            if side == "front" and k < 2:
                actions += _run(m, slopes, color, x, gap[0], sy, zk, turn)
                actions += _run(m, slopes, color, gap[1], x + length, sy, zk, turn)
            else:
                actions += _run(m, slopes, color, x, x + length, sy, zk, turn)
            for ry in rows:
                actions += _run(m, bricks, color, x, x + length, ry, zk, 1)
            in_steps(m, f"{title}, course {k + 1}: {side}", actions)
        if k == 0:
            # The cross gable's slopes. Projecting 2, a `gable` stud at their front ends and a soffit plate under the
            # projection, which ties those studs to the rest.
            y0 = y - project
            with m.step(f"{title}, cross gable"):
                if nameplate:
                    below = zk - (1 if project == 2 else 0)
                    m.add("30414", gable, at=(at, y0, below - 3), rot=3)
                    m.add("2431", nameplate, at=(at, y0 - 0.4, below - 2.5), rot=SIGN)
                if project == 2:
                    m.add("3020", gable, at=(at, y0, zk - 1), rot=1)
                for gx, face in ((at, "-x"), (at + 2, "+x")):
                    if project == 2:
                        m.add(Slope(45, 2, 1), gable, at=(gx, y0, zk), rot=DOWN[face])
                        m.add(
                            Slope(45, 2, 3), color, at=(gx, y0 + 1, zk), rot=DOWN[face]
                        )
                    else:
                        m.add(Slope(45, 2, 3), color, at=(gx, y0, zk), rot=DOWN[face])
        if k == 1:
            with m.step(f"{title}, cross gable: ridge"):
                ry = y - project
                for n in _lengths(project + 2, RIDGES):
                    m.add(RIDGES[n], color, at=(at + 1, ry, zk), rot=0)
                    ry += n
                m.add(bricks[4], color, at=(at, y + 2, zk), rot=1)
    top = z + 3 * courses
    in_steps(
        m, f"{title}, ridge", _run(m, RIDGES, color, x, x + length, y + courses, top, 1)
    )
    return top + 3


def example(m):
    """Walls 10 x 6 with a door, a ceiling, and the roof with a cross gable over the door."""
    from emmet_dsl import Plate

    with m.step("Base"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, -2, 0), size=(10, 8))
    for r in range(3):
        with m.step(f"Walls, row {r + 1}"):
            m.wall(0, 0, 1 + 3 * r, length=10, depth=6, color="Tan")
    with m.step("Ceiling"):
        m.fill(Plate, "Tan", at=(0, 0, 10), size=(10, 6))
    m.use("cross_gable", x=0, y=0, z=11, length=10, depth=6, at=3)
