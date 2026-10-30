# SPDX-License-Identifier: Apache-2.0
"""Micropolis block: the frame every micro city module starts from, a 16 x 16 base with a 2-stud road of dark tiles
along each side that faces a street and a 1-stud sidewalk inside it.

m.use("micropolis_block", x, y, z, streets=("front", "left"), sidewalk=True, base="Green", road="Dark Bluish Gray",
walk="Light Bluish Gray", title="Block") lays a 16 x 16 plate with its front-left corner at (x, y, z), then, on each
side named in `streets` ("front", "back", "left", "right"), a strip of tiles 2 studs wide along the whole edge, so the
roads of two modules side by side meet as a 4-stud street and a corner where two streets meet is tiled through. With
sidewalk, a 1-stud strip of tiles in the walk color runs inside each road, turning the corners. The rest of the plate is
left as bare studs for the buildings. Returns a dict: "lot", the (x, y, length, depth) rectangle of bare studs inside
the sidewalks; "z", the height of those studs; and "street_z", the top of the road.
"""

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from the community micro city module standard as the research notes give it
(16 x 16 modules, a 2-stud road on each street side) and the micro city methods in the instruction books of set
21066. Not derived from any model file."""

PARTS = ["91405", "87079", "3068b", "4162", "2431", "6636", "3069b", "3070b", "63864"]

SIDES = ("front", "back", "left", "right")
SIZE = 16  # a module is 16 x 16 studs
ROAD = 2  # the road strip is 2 studs wide


def build(
    m,
    x,
    y,
    z,
    streets=("front", "left"),
    sidewalk=True,
    base="Green",
    road="Dark Bluish Gray",
    walk="Light Bluish Gray",
    title="Block",
):
    """The base, the roads, and the sidewalks; returns the lot, its z, and the road's top."""
    bad = [s for s in streets if s not in SIDES]
    if bad:
        raise ValueError(
            f"micropolis_block: streets are some of {', '.join(SIDES)}, not {bad}"
        )
    with m.step(f"{title}: base"):
        m.add("91405", base, at=(x, y, z), id=f"{title.lower()}/base")
    top = z + 1
    # The front and back roads run the whole edge; the side roads fill between them.
    y0 = y + (ROAD if "front" in streets else 0)
    y1 = y + SIZE - (ROAD if "back" in streets else 0)
    with m.step(f"{title}: roads"):
        if "front" in streets:
            m.fill(Tile, road, at=(x, y, top), size=(SIZE, ROAD))
        if "back" in streets:
            m.fill(Tile, road, at=(x, y1, top), size=(SIZE, ROAD))
        if "left" in streets and y1 > y0:
            m.fill(Tile, road, at=(x, y0, top), size=(ROAD, y1 - y0))
        if "right" in streets and y1 > y0:
            m.fill(Tile, road, at=(x + SIZE - ROAD, y0, top), size=(ROAD, y1 - y0))
    x0 = x + (ROAD if "left" in streets else 0)
    x1 = x + SIZE - (ROAD if "right" in streets else 0)
    lot = [x0, y0, x1 - x0, y1 - y0]
    if sidewalk and streets:
        # The same again a stud further in: front and back walks across, side walks between them.
        wy0 = y0 + (1 if "front" in streets else 0)
        wy1 = y1 - (1 if "back" in streets else 0)
        with m.step(f"{title}: sidewalks"):
            if "front" in streets:
                m.row(Tile, walk, at=(x0, y0, top), length=x1 - x0)
            if "back" in streets:
                m.row(Tile, walk, at=(x0, y1 - 1, top), length=x1 - x0)
            if "left" in streets and wy1 > wy0:
                m.row(Tile, walk, at=(x0, wy0, top), length=wy1 - wy0, axis="y")
            if "right" in streets and wy1 > wy0:
                m.row(Tile, walk, at=(x1 - 1, wy0, top), length=wy1 - wy0, axis="y")
        lx0 = x0 + (1 if "left" in streets else 0)
        lx1 = x1 - (1 if "right" in streets else 0)
        lot = [lx0, wy0, lx1 - lx0, wy1 - wy0]
    return {"lot": tuple(lot), "z": top, "street_z": top + 1}


def example(m):
    """A corner block with a three-story shop, a tree, and a lamp in its lot."""
    from emmet_dsl import Brick, Plate, Tile

    block = m.use("micropolis_block", x=0, y=0, z=0, streets=("front", "left"))
    lx, ly, _, _ = block["lot"]
    z = block["z"]
    for r in range(3):
        with m.step(f"Shop, story {r + 1}"):
            m.wall(
                lx + 1,
                ly + 1,
                z + 3 * r,
                length=6,
                depth=4,
                color="Tan" if r else "Dark Red",
            )
    with m.step("Shop, roof"):
        m.fill(Plate, "Dark Bluish Gray", at=(lx + 1, ly + 1, z + 9), size=(6, 4))
    with m.step("Shop, roof tiles"):
        m.fill(Tile, "Light Bluish Gray", at=(lx + 1, ly + 1, z + 10), size=(6, 4))
    with m.step("Tree and lamp"):
        m.add(Brick(1, 1), "Reddish Brown", at=(lx + 9, ly + 1, z))
        m.add("4032b", "Green", at=(lx + 8.5, ly + 0.5, z + 3))
        m.add("85861", "Black", at=(lx + 9, ly + 4, z))
        m.add("87994", "Black", at=(lx + 9.3, ly + 4.3, z))
        m.add("85861", "Trans Clear", at=(lx + 9, ly + 4, z + 6))
