# SPDX-License-Identifier: Apache-2.0
"""Curved slope skin: a block whose top edges are rounded off with curved slopes and whose top is tiled, so a stack of
bricks reads as a smooth, soft body instead of a box.

m.use("curved_slope_skin", x, y, z, width, length, courses=1, color="Orange", top=None, title="Body") builds, on the
studs at height z, a block `width` studs along x by `length` along y (each 4 to 16) from (x, y): `courses` of brick
walls around its edge, a layer of plates over them that ties the walls (round-cornered plates at the corners), and then
the skin, 2 plates thick: along every edge a curved slope 2 x 2 (or a curved slope 2 x 1 to finish an odd length)
falling outward, at each corner a quarter circle 2 x 2 with a curved top, and in the middle plates covered with tiles,
flush with the slopes' tops. No stud shows. top is the skin's color (the block's by default). It brings its own steps
and returns the z of the top. Build further pieces on the sides with studs-on-side bricks, or leave a few studs in the
middle by tiling less.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21350, 21368, 43290, and
43008. Not derived from any model file."""

PARTS = [
    "15068",
    "11477",
    "5852",
    "79491",
    "3001",
    "3010",
    "3009",
    "3004",
    "3005",
    "3003",
    "3622",
    "3020",
    "3068b",
    "3069b",
]

# Quarter turns: a curved slope falls toward +x at rot 0 (+y at 1, -x at 2, -y at 3); a quarter circle's round corner
# points to +x +y at rot 0, and a quarter turn further for each corner going round counterclockwise.
FALL = {"front": 3, "back": 1, "left": 2, "right": 0}
CORNER = {"back-right": 0, "back-left": 1, "front-left": 2, "front-right": 3}


def build(m, x, y, z, width, length, courses=1, color="Orange", top=None, title="Body"):
    """The walls, the tie plates, and the skin; returns the z of the top."""
    if not 4 <= width <= 16 or not 4 <= length <= 16:
        raise ValueError(
            f"curved_slope_skin takes a width and length of 4 to 16, not {width}, {length}"
        )
    if courses < 1:
        raise ValueError(f"curved_slope_skin takes 1 or more courses, not {courses}")
    top = top or color
    sides = ("front", "back", "left", "right")
    for side in sides:
        with m.step(f"{title}: walls, {side}"):
            m.wall(x, y, z, width, length, color=color, sides=side, rows=courses)
    zt = z + 3 * courses
    with m.step(f"{title}: tie plates"):
        # A cross of plates over the walls, and a round-cornered plate at each corner, under the quarter circles.
        if width > 4:
            m.fill(Plate, color, at=(x + 2, y, zt), size=(width - 4, length))
        if length > 4:
            m.fill(Plate, color, at=(x, y + 2, zt), size=(2, length - 4))
            m.fill(Plate, color, at=(x + width - 2, y + 2, zt), size=(2, length - 4))
    with m.step(f"{title}: tie plates, corners"):
        for (cx, cy), corner in zip(
            (
                (x, y),
                (x + width - 2, y),
                (x + width - 2, y + length - 2),
                (x, y + length - 2),
            ),
            ("front-left", "front-right", "back-right", "back-left"),
        ):
            # The round-cornered plate's corner points a quarter turn behind the quarter circle's.
            m.add("79491", color, at=(cx, cy, zt), rot=(CORNER[corner] + 1) % 4)
    zs = zt + 1
    with m.step(f"{title}: corners"):
        for (cx, cy), corner in zip(
            (
                (x, y),
                (x + width - 2, y),
                (x + width - 2, y + length - 2),
                (x, y + length - 2),
            ),
            ("front-left", "front-right", "back-right", "back-left"),
        ):
            m.add("5852", top, at=(cx, cy, zs), rot=CORNER[corner])
    edges = {
        "front": (x + 2, width - 4, "x", y),
        "back": (x + 2, width - 4, "x", y + length - 2),
        "left": (y + 2, length - 4, "y", x),
        "right": (y + 2, length - 4, "y", x + width - 2),
    }
    for side in sides:
        start, n, axis, across = edges[side]
        if n < 1:
            continue
        with m.step(f"{title}: edge, {side}"):
            a = start
            while a < start + n:
                k = 2 if start + n - a >= 2 else 1
                # A 2 x 1 falls the same way; it's 1 stud along the edge.
                xy = (a, across) if axis == "x" else (across, a)
                m.add("15068" if k == 2 else "11477", top, at=(*xy, zs), rot=FALL[side])
                a += k
    if width > 4 and length > 4:
        with m.step(f"{title}: middle"):
            m.fill(Plate, top, at=(x + 2, y + 2, zs), size=(width - 4, length - 4))
            m.fill(Tile, top, at=(x + 2, y + 2, zs + 1), size=(width - 4, length - 4))
    return zs + 2


def example(m):
    """A soft body 6 x 8 studs and two bricks tall on a base."""
    from emmet_dsl import Plate

    with m.step("Base"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(8, 10))
    m.use("curved_slope_skin", x=1, y=1, z=1, width=6, length=8, courses=2)
