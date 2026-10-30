# SPDX-License-Identifier: Apache-2.0
"""Tower setback plinth: a podium a stud wider than the tower on chosen sides, its edge covered with cheese slopes, so a
micro tower flares out at street level.

m.use("tower_setback_plinth", x, y, z, width=2, depth=2, sides=("front",), height=1, color="Light Bluish Gray",
slope_color=None, title="Plinth") builds, on the studs at height z from (x, y), a podium `height` bricks tall whose
footprint is the tower's width x depth plus a stud on each side named in `sides` ("front", "back", "left", "right"):
courses of bricks with their joints crossed, then a layer of plates over them that ties the courses together. On the
plates, a cheese slope (54200, or 85984 where two cells of an edge pair up) on each stud of the ledge, its low edge
facing out, and a double convex one (7826) where two ledges meet at a corner; slope_color defaults to the podium's. The
tower stands on the studs left in the middle. Returns (tower x, tower y, z): where the tower's front-left corner goes
and the z of the studs it stands on.
"""

from emmet_dsl import Brick, Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = [
    "54200",
    "85984",
    "7826",
    "3005",
    "3004",
    "3622",
    "3010",
    "3003",
    "3001",
    "3023b",
    "3022",
    "3020",
]

FACING = {
    "front": 3,
    "back": 1,
    "left": 2,
    "right": 0,
}  # the quarter turn that points a slope's low edge out
# A double convex slope's two low faces point +x and -y at rot=0; each corner's turn.
CORNER = {
    ("front", "right"): 0,
    ("back", "right"): 1,
    ("back", "left"): 2,
    ("front", "left"): 3,
}


def build(
    m,
    x,
    y,
    z,
    width=2,
    depth=2,
    sides=("front",),
    height=1,
    color="Light Bluish Gray",
    slope_color=None,
    title="Plinth",
):
    """The podium, its tie plates, and the slopes; returns where the tower stands."""
    bad = [s for s in sides if s not in FACING]
    if bad or not sides or width < 1 or depth < 1 or height < 1:
        raise ValueError(
            "tower_setback_plinth needs some of front, back, left, right as sides, and a width, depth and height of "
            f"1 or more, not {sides}, {width}, {depth}, {height}"
        )
    slope_color = slope_color or color
    left, right = int("left" in sides), int("right" in sides)
    front, back = int("front" in sides), int("back" in sides)
    pw, pd = width + left + right, depth + front + back
    for r in range(height):
        zr = z + 3 * r
        with m.step(f"{title}: podium, row {r + 1}"):
            # Bricks 2 deep along x; every other course runs along y instead, so the joints cross.
            if r % 2 == 0 or pw < 2:
                for j in range(0, pd, 2):
                    m.row(
                        Brick, color, at=(x, y + j, zr), length=pw, width=min(2, pd - j)
                    )
            else:
                for i in range(0, pw, 2):
                    m.row(
                        Brick,
                        color,
                        at=(x + i, y, zr),
                        length=pd,
                        axis="y",
                        width=min(2, pw - i),
                    )
    zt = z + 3 * height
    with m.step(f"{title}: tie plates"):
        m.fill(Plate, color, at=(x, y, zt), size=(pw, pd))
    zs = zt + 1
    tx, ty = x + left, y + front
    with m.step(f"{title}: slopes"):
        # Each ledge's cells along its edge, between the corners; a pair of cells takes one 1 x 2 slope.
        edges = {
            "front": [(tx + i, y) for i in range(width)],
            "back": [(tx + i, y + pd - 1) for i in range(width)],
            "left": [(x, ty + j) for j in range(depth)],
            "right": [(x + pw - 1, ty + j) for j in range(depth)],
        }
        for side in sides:
            cells, rot = edges[side], FACING[side]
            k = 0
            while k < len(cells):
                cx, cy = cells[k]
                if k + 1 < len(cells):
                    # 85984 runs along y at rot 0 and 2, along x at 1 and 3.
                    m.add("85984", slope_color, at=(cx, cy, zs), rot=rot)
                    k += 2
                else:
                    m.add("54200", slope_color, at=(cx, cy, zs), rot=rot)
                    k += 1
        for (a, b), rot in CORNER.items():
            if a in sides and b in sides:
                cx = x if b == "left" else x + pw - 1
                cy = y if a == "front" else y + pd - 1
                m.add("7826", slope_color, at=(cx, cy, zs), rot=rot)
    return (tx, ty, zs)


def example(m):
    """A tower on a plinth that flares to the front and right, on a paved corner."""
    from emmet_dsl import Tile

    with m.step("Ground"):
        m.add(Plate(6, 6), "Dark Bluish Gray", at=(0, 0, 0))
    tx, ty, tz = m.use(
        "tower_setback_plinth",
        x=1,
        y=1,
        z=1,
        width=2,
        depth=2,
        sides=("front", "right"),
        color="White",
    )
    for r in range(4):
        with m.step(f"Tower, row {r + 1}"):
            m.add(Brick(2, 2), "Light Bluish Gray", at=(tx, ty, tz + 3 * r))
    with m.step("Tower, roof"):
        m.add(Tile(2, 2), "White", at=(tx, ty, tz + 12))
