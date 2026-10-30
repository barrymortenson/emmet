# SPDX-License-Identifier: Apache-2.0
"""Round corner bay: a house corner rounded off by curved corner windows stacked on a round-cornered plate, the plate
over them tying the bay to both walls.

m.use("round_corner_bay", x, y, z, tiers=3, corner="left", frame="Black", trim="White", title="Corner bay") rounds the
front-left (corner="left") or front-right ("right") corner of a building whose walls are one stud thick, the front one
along x facing front (-y), and whose outer corner is (x, y). A Plate 6 x 6 with a 4 x 4 Round Corner (6003) at z reaches
a stud out round the curve and two studs back under both walls; on it stand `tiers` Windows Round Corner 3 x 3 x 2 with
their glass (73878c01), one on another, in the 3 x 3 corner the walls leave open; a Plate 4 x 4 with Corner Round
(30565) caps them with the same stud of overhang, tiled flush with a Tile 4 x 4 Corner Round round its curve and a Tile
3 x 3 Corner Round inside it. The bay takes the corner's 3 x 3 cells (the front wall's first three and the side wall's
first three) from z to z + 2 + 6 * tiers; the walls stand on the bottom plate's back studs, and the course that ties the
wall tops (a plate course or the roof) reaches over the cap's back corner. Returns the z of the cap's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["73878c01", "6003", "30565", "27507", "79393"]

TURN = {
    "left": 2,
    "right": 3,
}  # the quarter turn that puts each part's round corner to the front-left or front-right


def build(
    m, x, y, z, tiers=3, corner="left", frame="Black", trim="White", title="Corner bay"
):
    """The bottom plate, the windows, the top plate and its tile; returns the z of the top plate's top."""
    if corner not in TURN:
        raise ValueError(f'corner is "left" or "right", not {corner!r}')
    turn = TURN[corner]
    wx, px = (x, x - 1) if corner == "left" else (x - 3, x - 5)
    with m.step(f"{title}: base"):
        m.add("6003", trim, at=(px, y - 1, z), rot=turn)
    for t in range(tiers):
        with m.step(f"{title}: windows, tier {t + 1}"):
            m.add("73878c01", frame, at=(wx, y, z + 1 + 6 * t), rot=turn)
    top = z + 1 + 6 * tiers
    with m.step(f"{title}: cap"):
        cx = x - 1 if corner == "left" else x - 3
        m.add("30565", trim, at=(cx, y - 1, top), rot=turn)
        m.add("27507", trim, at=(cx, y - 1, top + 1), rot=turn)
        m.add("79393", trim, at=(wx, y, top + 1), rot=turn)
    return top + 2


def example(m):
    """A front-right corner: the bay between the stubs of the front and side walls."""
    from emmet_dsl import Brick

    m.use("round_corner_bay", x=0, y=0, z=0, tiers=2, corner="right")
    for r in range(4):
        with m.step(f"Walls, row {r + 1}"):
            first = 1 if r % 2 else None
            m.row(Brick, "Sand Blue", at=(-7, 0, 1 + 3 * r), length=4, first=first)
            m.row(
                Brick,
                "Sand Blue",
                at=(-1, 3, 1 + 3 * r),
                length=4,
                axis="y",
                first=first,
            )
