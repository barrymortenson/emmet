# SPDX-License-Identifier: Apache-2.0
"""Recessed glow panel: a glowing light panel set between two curved lips on a hull wall, all on side studs.

m.use("recessed_glow_panel", x, y, z, width=4, color="Dark Bluish Gray", lip="Light Bluish Gray", glow="Trans-Light
Blue", title="Glow panel") builds a wall one stud thick and `width` studs long (2, 4, or 6) along x from (x, y), 15
plates tall from z, facing front (-y), in three rows of bricks 1 2/3 tall with studs on one side (22885, and 32952 at
the ends of the middle row, so the joints cross). On the middle row's studs hangs the panel, 2 x 2 tiles in `glow`
standing on edge; on the rows over and under it, curved slopes 2 x 1 (11477) hang on the side studs, a column of them
per stud, their deep ends against the panel and their curves running back into the wall, so the panel sits in a rounded
slot: it stands 0.4 studs out from the wall, the lips 0.8. Returns the z of the top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10356. Not derived from any
model file."""

PARTS = ["22885", "32952", "11477", "3068b"]

OVER = Rot(x=90, y=-90)  # a curved slope on its back, its face to -y, its deep end down
UNDER = Rot(x=90, y=90)  # the same, its deep end up
PANEL = Rot(y=-90, z=90)  # a tile on its back, its face to -y, long along x


def build(
    m,
    x,
    y,
    z,
    width=4,
    color="Dark Bluish Gray",
    lip="Light Bluish Gray",
    glow="Trans-Light Blue",
    title="Glow panel",
):
    """The three rows, the lips, and the panel; returns the z of the top."""
    if width not in (2, 4, 6):
        raise ValueError(
            f"recessed_glow_panel takes a width of 2, 4, or 6, not {width}"
        )
    for r, name in enumerate(("lower lip", "panel", "upper lip")):
        zr = z + 5 * r
        with m.step(f"{title}, {name} row: bricks"):
            # Every other row starts with a 1 x 1, so the joints cross.
            k = 0
            if r % 2:
                m.add("32952", color, at=(x, y, zr), rot=3)
                m.add("32952", color, at=(x + width - 1, y, zr), rot=3)
                k = 1
            while k + 2 <= width - (r % 2):
                m.add("22885", color, at=(x + k, y, zr), rot=3)
                k += 2
        with m.step(f"{title}, {name}"):
            if name == "panel":
                for k in range(0, width, 2):
                    m.add("3068b", glow, at=(x + k, y - 0.4, zr), rot=PANEL)
            else:
                for k in range(width):
                    m.add(
                        "11477",
                        lip,
                        at=(x + k, y - 0.8, zr),
                        rot=UNDER if r == 0 else OVER,
                    )
    return z + 15


def example(m):
    """The panel in a hull wall, a column of bricks each side and a plate over it."""
    with m.step("Base"):
        m.add("3666", "Dark Bluish Gray", at=(-1, 0, 0), rot=1)
    for side, sx in (("left", -1), ("right", 4)):
        with m.step(f"Side, {side}"):
            m.stack("3005", "Dark Bluish Gray", at=(sx, 0, 1), count=5)
    m.use("recessed_glow_panel", x=0, y=0, z=1)
    with m.step("Top"):
        m.add("3666", "Dark Bluish Gray", at=(-1, 0, 16), rot=1)
