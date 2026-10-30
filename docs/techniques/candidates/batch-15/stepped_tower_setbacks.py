# SPDX-License-Identifier: Apache-2.0
"""Stepped tower setbacks: a tower that narrows in tiers, each a stud in from the one below on every side with a tiled
ledge round it, up to a needle spire.

m.use("stepped_tower_setbacks", x, y, z, size=6, heights=(4, 3, 3), color="White", ledge="Light Bluish Gray",
spire=True, title="Tower") stands, on the studs at height z with its front-left corner at (x, y), a square tower in
tiers: the first `size` x `size` studs (an even 4 to 8), each next one 2 studs narrower and centered, as many tiers as
`heights` has entries (each the tier's height in bricks), down to 2 x 2 at most. Each tier is walls of bricks with
interlocking corners (a 2 x 2 tier, a stack of 2 x 2 bricks), a layer of plates over them, and a ring of tiles in the
ledge color round the studs the next tier leaves bare. With spire, the top tier takes a jumper plate (87580), a round 1
x 1 brick, and a bar 4L standing in its hollow stud (when the top tier is 2 x 2). Returns the z of the top: the spire's
tip, or the top tier's plates.
"""

from emmet_dsl import Brick, Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = [
    "3003",
    "3004",
    "3010",
    "3009",
    "3008",
    "3022",
    "3020",
    "3958",
    "3069b",
    "2431",
    "6636",
    "87580",
    "3062b",
    "30374",
]


def build(
    m,
    x,
    y,
    z,
    size=6,
    heights=(4, 3, 3),
    color="White",
    ledge="Light Bluish Gray",
    spire=True,
    title="Tower",
):
    """The tiers, their ledges, and the spire; returns the z of the top."""
    if (
        size % 2
        or not 4 <= size <= 8
        or not heights
        or size - 2 * (len(heights) - 1) < 2
        or min(heights) < 1
    ):
        raise ValueError(
            "stepped_tower_setbacks takes an even size of 4 to 8 and a height of 1 or more for each tier, no more "
            f"tiers than narrow it to 2 x 2, not {size}, {heights}"
        )
    zt = z
    for t, rows in enumerate(heights):
        s, ox, oy = size - 2 * t, x + t, y + t
        for r in range(rows):
            zr = zt + 3 * r
            if s == 2:
                with m.step(f"{title}, tier {t + 1}: row {r + 1}"):
                    m.add(Brick(2, 2), color, at=(ox, oy, zr))
            elif s < 8:
                with m.step(f"{title}, tier {t + 1}: row {r + 1}"):
                    m.wall(ox, oy, zr, length=s, depth=s, color=color)
            else:
                with m.step(f"{title}, tier {t + 1}: row {r + 1}, front and back"):
                    m.wall(
                        ox,
                        oy,
                        zr,
                        length=s,
                        depth=s,
                        color=color,
                        sides=("front", "back"),
                    )
                with m.step(f"{title}, tier {t + 1}: row {r + 1}, sides"):
                    m.wall(ox, oy, zr, length=s, depth=s, color=color, sides="sides")
        zt += 3 * rows
        with m.step(f"{title}, tier {t + 1}: roof"):
            m.fill(Plate, color, at=(ox, oy, zt), size=(s, s))
        zt += 1
        if t < len(heights) - 1:
            with m.step(f"{title}, tier {t + 1}: ledge"):
                m.row(Tile, ledge, at=(ox, oy, zt), length=s)
                m.row(Tile, ledge, at=(ox, oy + s - 1, zt), length=s)
                m.row(Tile, ledge, at=(ox, oy + 1, zt), length=s - 2, axis="y")
                m.row(Tile, ledge, at=(ox + s - 1, oy + 1, zt), length=s - 2, axis="y")
    s = size - 2 * (len(heights) - 1)
    if spire and s == 2:
        ox, oy = x + len(heights) - 1, y + len(heights) - 1
        with m.step(f"{title}: spire"):
            m.add("87580", color, at=(ox, oy, zt))
            m.add("3062b", color, at=(ox + 0.5, oy + 0.5, zt + 1))
            # A bar 4L is 10 plates long: 2 plates down inside the round brick, 8 above it.
            m.add("30374", ledge, at=(ox + 0.8, oy + 0.8, zt + 2))
        return zt + 12
    return zt


def example(m):
    """A three-tier tower with its spire on a paved plate."""
    with m.step("Ground"):
        m.add(Plate(8, 8), "Dark Bluish Gray", at=(0, 0, 0))
    m.use("stepped_tower_setbacks", x=1, y=1, z=1, size=6, heights=(4, 3, 3))
