# SPDX-License-Identifier: Apache-2.0
"""Ribbed side panel: a big flat wall face cut by deep vertical grooves, built flat and stood on side studs, with girder
panels between as framed openings.

m.use("ribbed_side_panel", x, y, z, panels=2, width=5, height=6, girders=True, color="Light Bluish Gray",
backing="Dark Bluish Gray", title="Ribbed side") builds, on the studs at z, a wall along x from (x, y), facing front
(-y), `height` studs tall (4, 6, or 8; 2.5 plates a stud). Each of its `panels` panels is built flat as a submodel: a
plate `width` x `height` with ribs along it, a brick 1 x `height` capped with a tile on every other stud from the
first, a tile in the grooves between; stood up, the ribs run up the wall, 1.2 studs proud of the grooves. Each panel
hangs on a backing of bricks 1 2/3 tall with studs on one side (22885, 32952) a stud thick, its face 2 studs out from
the backing. With girders (and a height of 6), a girder panel 1 x 6 x 5 (64448) stands in the backing's line between
panels, an open frame set back from the ribs. A course of plates over the backing and girders ties them. width: 3 to
7. Returns the z of the top.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10341. Not derived from any
model file."""

PARTS = [
    "22885",
    "32952",
    "3010",
    "3009",
    "3008",
    "2431",
    "6636",
    "4162",
    "64448",
    "3666",
    "3460",
    "3710",
]

UPRIGHT = Rot(x=90)  # a part built flat, stood up with its studs to -y
RIBS = {
    4: ("3010", "2431"),
    6: ("3009", "6636"),
    8: ("3008", "4162"),
}  # brick and tile 1 x n
PROUD = 2  # the panel's depth: a plate, a brick, and a tile, in studs


def _panel(m, width, height, color):
    """The ribbed panel, flat: its plate's corner at the origin, the ribs along y."""
    brick, tile = RIBS[height]
    with m.step("Plate"):
        m.fill(Plate, color, at=(0, 0, 0), size=(width, height))
    with m.step("Ribs"):
        for i in range(0, width, 2):
            m.add(brick, color, at=(i, 0, 1))
    with m.step("Rib caps and grooves"):
        for i in range(width):
            m.add(tile, color, at=(i, 0, 4 if i % 2 == 0 else 1))


def _backing(m, x, y, z, width, rows, color, title):
    """A stud-thick backing of side-stud bricks 5 plates tall, its joints crossed row to row."""
    for r in range(rows):
        with m.step(f"{title}, row {r + 1}"):
            k = 0
            if r % 2:
                m.add("32952", color, at=(x, y, z + 5 * r), rot=3)
                k = 1
            while k + 2 <= width:
                m.add("22885", color, at=(x + k, y, z + 5 * r), rot=3)
                k += 2
            if k < width:
                m.add("32952", color, at=(x + k, y, z + 5 * r), rot=3)


def build(
    m,
    x,
    y,
    z,
    panels=2,
    width=5,
    height=6,
    girders=True,
    color="Light Bluish Gray",
    backing="Dark Bluish Gray",
    title="Ribbed side",
):
    """The backings, the panels, the girders, and the tie course; returns the z of the top."""
    if height not in RIBS or not 3 <= width <= 7 or not 1 <= panels <= 4:
        raise ValueError(
            f"ribbed_side_panel takes a height of 4, 6, or 8, a width of 3 to 7 and 1 to 4 panels, "
            f"not {height}, {width}, {panels}"
        )
    girders = girders and height == 6
    panel = m.submodel("ribbed_panel", _panel, width=width, height=height, color=color)
    at = x
    for p in range(panels):
        _backing(
            m, at, y, z, width, height // 2, backing, f"{title}, panel {p + 1} backing"
        )
        with m.step(f"{title}, panel {p + 1}"):
            m.place(panel, at=(at, y - PROUD, z), rot=UPRIGHT)
        at += width
        if girders and p < panels - 1:
            with m.step(f"{title}, girder {p + 1}"):
                m.add("64448", backing, at=(at, y, z), rot=1)
            at += 6
    zt = z + 5 * (height // 2)
    with m.step(f"{title}, tie course"):
        m.row(Plate, backing, at=(x, y, zt), length=at - x)
    return zt + 1


def example(m):
    """Two ribbed panels with a girder between, on a strip of floor."""
    with m.step("Floor"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, -2, 0), size=(16, 3))
    m.use("ribbed_side_panel", x=0, y=0, z=1)
