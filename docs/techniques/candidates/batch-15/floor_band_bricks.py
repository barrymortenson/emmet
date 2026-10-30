# SPDX-License-Identifier: Apache-2.0
"""Floor band bricks: a micro glass tower built of bricks with grooved faces, whose grooves read as many fine floor
lines, with an angled roofline on top.

m.use("floor_band_bricks", x, y, z, depth=2, floors=8, color="Sand Blue", roof="slant", title="Tower") stands a tower 2
studs wide along x and `depth` deep (1 or 2), on the studs at height z from (x, y): a plate, then `floors` courses of 1
x 2 bricks with embossed courses (98283), every one along x so the grooves face front and back (each brick shows three
bands, so 8 courses read as a tower of about 24 stories), and a plate over them that ties the rows. roof "slant": a
cheese slope (85984) over each row, facing front, each row a plate higher than the one in front, for an angled roofline;
"flat": tiles. Returns the z of the top.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = ["98283", "3023b", "3022", "85984", "3069b", "3068b"]


def build(
    m, x, y, z, depth=2, floors=8, color="Sand Blue", roof="slant", title="Tower"
):
    """The plates, the grooved courses, and the roof; returns the z of the top."""
    if depth not in (1, 2) or floors < 1 or roof not in ("slant", "flat"):
        raise ValueError(
            "floor_band_bricks takes a depth of 1 or 2, 1 or more floors, and a roof of 'slant' or 'flat', "
            f"not {depth}, {floors}, {roof!r}"
        )
    with m.step(f"{title}: base"):
        m.add(Plate(depth, 2), color, at=(x, y, z), rot=1)
    for f in range(floors):
        with m.step(f"{title}: course {f + 1}"):
            for j in range(depth):
                m.add("98283", color, at=(x, y + j, z + 1 + 3 * f), rot=1)
    zt = z + 1 + 3 * floors
    with m.step(f"{title}: roof"):
        m.add(Plate(depth, 2), color, at=(x, y, zt), rot=1)
        zt += 1
        if roof == "flat":
            m.add(Tile(depth, 2), color, at=(x, y, zt), rot=1)
            return zt + 1
        top = zt
        for j in range(depth):
            if j:
                m.add(Plate(1, 2), color, at=(x, y + j, zt), rot=1)
            m.add("85984", color, at=(x, y + j, zt + j), rot=3)
            top = zt + j + 2
    return top


def example(m):
    """Two grooved towers of different heights on a dark plate."""
    with m.step("Ground"):
        m.add(Plate(4, 6), "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("floor_band_bricks", x=0, y=1, z=1, depth=2, floors=10, title="Tall tower")
    m.use(
        "floor_band_bricks",
        x=3,
        y=1,
        z=1,
        depth=1,
        floors=6,
        color="White",
        roof="flat",
        title="Low tower",
    )
