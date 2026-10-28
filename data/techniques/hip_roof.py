# SPDX-License-Identifier: Apache-2.0
"""Hip roof: 45° slopes on all four sides, with corner slopes, rising to a short ridge (or a point on a square).

m.use("hip_roof", x, y, z, length, depth, color="Dark Bluish Gray", title="Roof") covers the length × depth rectangle
whose front-left corner is (x, y), starting at height z: the top of the walls, with a layer of plates (a ceiling)
under the whole rectangle for the first course to stand on. Each course is a ring a brick tall that steps in one
stud on every side: a double convex corner slope (2 x 2) at each corner, plain slopes between them, and, while the
roof is still wide, a band of bricks inside for the next ring to stand on. The top course is a ridge of double slopes
along the longer side, ended by triple slopes (1 x 2); on a square it's two triple slopes back to back. The shorter
side must be even (4 or more). It brings its own steps, a course at a time, titled "<title>, course 1" (split into
"(1 of 2)" and so on when a course is more than a step), then "<title>, ridge", and returns the z of the ridge's top.
"""

from emmet_dsl import Brick, Plate, Slope
from emmet_dsl.core import in_steps

PARTS = [
    "3045",
    "3037",
    "3038",
    "3039",
    "3040b",
    "3048b",
    "3041",
    "3043",
    "3044c",
    "3006",
    "3007",
    "2456",
    "3001",
    "3002",
    "3003",
    "3004",
    "3005",
    "3008",
    "3009",
    "3010",
    "3622",
]

# Double slopes for the ridge, by length (the library has no 3-long one).
RIDGES = {4: "3041", 2: "3043", 1: "3044c"}
# The quarter turn that points a slope down toward each side; a corner slope's turn for the corner it caps.
DOWN = {"+x": 0, "+y": 1, "-x": 2, "-y": 3}
CORNER = {"back-right": 0, "back-left": 1, "front-left": 2, "front-right": 3}
SLOPE_SIZES = (4, 3, 2, 1)
BRICK_SIZES = {2: (10, 8, 6, 4, 3, 2, 1), 1: (8, 6, 4, 3, 2, 1)}


def _lengths(n, sizes):
    out = []
    for size in sorted(sizes, reverse=True):
        while n >= size:
            out.append(size)
            n -= size
    return out


def _run(m, part_of, sizes, x, y, z, n, axis, color, rot_of):
    """A run of n studs along axis from (x, y), in the longest pieces first."""
    for size in _lengths(n, sizes):
        m.add(part_of(size), color, at=(x, y, z), rot=rot_of(size))
        x, y = (x + size, y) if axis == "x" else (x, y + size)


def _slopes(m, x, y, z, n, axis, down, color):
    _run(
        m,
        lambda s: Slope(45, 2, s),
        SLOPE_SIZES,
        x,
        y,
        z,
        n,
        axis,
        color,
        lambda s: DOWN[down],
    )


def _bricks(m, x, y, z, n, axis, width, color):
    """A run of bricks `width` studs across: a brick's long side runs along y at rot=0."""

    def turn(size):
        return 1 if (axis == "x") == (size > width) else 0

    _run(
        m, lambda s: Brick(width, s), BRICK_SIZES[width], x, y, z, n, axis, color, turn
    )


def _fill_actions(m, x, y, z, w, d, color):
    """A w × d block of bricks as in_steps actions: strips two wide along its long side, one wide for an odd rest."""
    axis = "x" if w >= d else "y"
    long_side, short_side = (w, d) if axis == "x" else (d, w)
    actions, done = [], 0
    while done < short_side:
        width = 2 if short_side - done >= 2 else 1
        sx, sy = (x, y + done) if axis == "x" else (x + done, y)
        n = len(_lengths(long_side, BRICK_SIZES[width]))
        actions.append(
            (
                n,
                lambda sx=sx, sy=sy, width=width: _bricks(
                    m, sx, sy, z, long_side, axis, width, color
                ),
            )
        )
        done += width
    return actions


def _ring(m, x0, y0, x1, y1, z, color):
    """One course's ring over the rectangle (x0, y0)–(x1, y1), as in_steps actions: the inside, the back, the ends,
    then the front."""
    lk, dk = x1 - x0, y1 - y0
    actions = []
    # Inside first, where it's easy to reach: the bricks the next ring stands on, two studs in from this one's slopes
    # and two wide, so each ring's band overlaps the one below it by a stud. No band under the ridge.
    if min(lk, dk) >= 6:
        ix0, iy0, iw, id_ = x0 + 2, y0 + 2, lk - 4, dk - 4
        if min(iw, id_) <= 4:
            actions += _fill_actions(m, ix0, iy0, z, iw, id_, color)
        else:
            for bx, by, bw, bd in (
                (ix0, iy0 + id_ - 2, iw, 2),  # back
                (ix0, iy0 + 2, 2, id_ - 4),  # left
                (ix0 + iw - 2, iy0 + 2, 2, id_ - 4),  # right
                (ix0, iy0, iw, 2),  # front
            ):
                actions += _fill_actions(m, bx, by, z, bw, bd, color)

    def side(sy, down, corners):
        def add():
            m.add("3045", color, at=(x0, sy, z), rot=CORNER[corners[0]])
            _slopes(m, x0 + 2, sy, z, lk - 4, "x", down, color)
            m.add("3045", color, at=(x1 - 2, sy, z), rot=CORNER[corners[1]])

        return (2 + len(_lengths(lk - 4, SLOPE_SIZES)), add)

    def ends():
        for ex, down in ((x0, "-x"), (x1 - 2, "+x")):
            _slopes(m, ex, y0 + 2, z, dk - 4, "y", down, color)

    actions.append(side(y1 - 2, "+y", ("back-left", "back-right")))
    if dk > 4:
        actions.append((2 * len(_lengths(dk - 4, SLOPE_SIZES)), ends))
    actions.append(side(y0, "-y", ("front-left", "front-right")))
    return actions


def build(m, x, y, z, length, depth, color="Dark Bluish Gray", title="Roof"):
    """The roof; returns the z of the ridge's top."""
    short = min(length, depth)
    if short < 4 or short % 2:
        raise ValueError(
            f"hip_roof needs an even shorter side of 4 or more, not {length} × {depth}"
        )
    rings = (short - 2) // 2
    for k in range(rings):
        zk = z + 3 * k
        x0, y0, x1, y1 = x + k, y + k, x + length - k, y + depth - k
        actions = _ring(m, x0, y0, x1, y1, zk, color)
        in_steps(m, f"{title}, course {k + 1}", actions)

    top = z + 3 * rings
    x0, y0, x1, y1 = x + rings, y + rings, x + length - rings, y + depth - rings
    with m.step(f"{title}, ridge"):
        if x1 - x0 >= y1 - y0:
            # Along x: triple slopes (1 x 2, across the ridge) at the ends, double slopes between.
            m.add("3048b", color, at=(x0, y0, top), rot=2)
            rx = x0 + 1
            for n in _lengths(x1 - x0 - 2, RIDGES):
                m.add(RIDGES[n], color, at=(rx, y0, top), rot=1)
                rx += n
            m.add("3048b", color, at=(x1 - 1, y0, top), rot=0)
        else:
            m.add("3048b", color, at=(x0, y0, top), rot=3)
            ry = y0 + 1
            for n in _lengths(y1 - y0 - 2, RIDGES):
                m.add(RIDGES[n], color, at=(x0, ry, top), rot=0)
                ry += n
            m.add("3048b", color, at=(x0, y1 - 1, top), rot=1)
    return top + 3


def example(m):
    """An 8 × 6 hip roof on a plate deck."""
    with m.step("Deck"):
        m.fill(Plate, "Light Bluish Gray", at=(0, 0, 0), size=(8, 6))
    m.use("hip_roof", x=0, y=0, z=1, length=8, depth=6)
