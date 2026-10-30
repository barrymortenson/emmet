# SPDX-License-Identifier: Apache-2.0
"""Layered plate cap: a gently domed cap of round plates, each 2 studs smaller than the one under it, topped with a
round tile; one on each face of a core makes a ball.

m.use("layered_plate_cap", x, y, z, size=6, facing="up", color="White", title="Cap") builds a cap whose base is a round
plate `size` studs across (4, 6, or 8), with a round plate 2 studs smaller on it, and so on down to a round plate 2 x 2,
closed by a round tile 2 x 2: 3 to 5 plates thick, its studs only in rings around each step (the pebbled look of a ball).
It's a submodel (one file per size and color) with its own steps, placed in a step titled `title`. facing is the way the
cap bulges: "up" sits on the studs at height z with its base's corner at (x, y); "down" hangs under a face whose
underside is at z, built smallest first (a round plate 2 x 2 at the bottom, no tile), so whatever goes on top presses
onto its base; "front" (-y), "back" (+y), "left" (-x), and "right" (+x) stand it on the side studs of a core (a
snot_core, or bricks 1 x 2 x 1 2/3 with studs on the side), its base's lowest edge at z: for "front" and "back", x is
its left end and y the face; for "left" and "right", x is the face and y its front end. The side studs must sit on the
base's grid, 1.25 plates above z and every 2.5 plates (a stud) up from there. Returns the placed copy's Handle.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43008. Not derived from any
model file."""

PARTS = ["74611", "11213", "60474", "4032a", "14769"]

ROUND = {8: "74611", 6: "11213", 4: "60474", 2: "4032a"}  # Plate N x N Round
# The cap is built bulging up; these stand it so it bulges out of a face.
FACING = {
    "up": Rot(),
    "down": Rot(),
    "front": Rot(x=90),
    "back": Rot(x=90, z=180),
    "left": Rot(x=90, z=-90),
    "right": Rot(x=90, z=90),
}


def _cap(m, size, color, down):
    """The cap in its own frame, its base's corner at the origin: bulging up, or built smallest first to hang down."""
    sizes = list(range(size, 0, -2))
    if down:
        with m.step("Plates, smallest first"):
            for k, n in enumerate(reversed(sizes)):
                off = (size - n) / 2
                m.add(ROUND[n], color, at=(off, off, k))
        return
    with m.step("Plates"):
        for k, n in enumerate(sizes):
            off = (size - n) / 2
            m.add(ROUND[n], color, at=(off, off, k))
    with m.step("Top"):
        off = (size - 2) / 2
        m.add("14769", color, at=(off, off, len(sizes)))


def build(m, x, y, z, size=6, facing="up", color="White", title="Cap"):
    """The cap, placed on its face; returns the placed copy's Handle."""
    if size not in (4, 6, 8):
        raise ValueError(f"layered_plate_cap takes a size of 4, 6, or 8, not {size}")
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a way: use up, down, front, back, left, or right"
        )
    sub = m.submodel(
        "layered_plate_cap", _cap, size=size, color=color, down=facing == "down"
    )
    lo, hi = sub.box
    thick = (
        hi[2] - lo[2]
    ) * 0.4  # the cap's height, in studs, when it lies on its side
    at = {
        "up": (x, y, z),
        "down": (x, y, z - (hi[2] - lo[2])),
        "front": (x, y - thick, z),
        "back": (x, y, z),
        "left": (x - thick, y, z),
        "right": (x, y, z),
    }[facing]
    with m.step(title):
        return m.place(sub, at=at, rot=FACING[facing])


def example(m):
    """A cap on a plate, and another on the side studs of a pillar beside it."""
    with m.step("Base"):
        m.add("3958", "Dark Bluish Gray", at=(0, 0, 0))
        m.add("3958", "Dark Bluish Gray", at=(6, 0, 0))
    m.use("layered_plate_cap", x=0, y=0, z=1, size=6, title="Cap on top")
    with m.step("Pillar"):
        m.add("3009", "Dark Bluish Gray", at=(6, 3, 1), rot=1)  # Brick 1 x 6, along x
        m.add("3710", "Dark Bluish Gray", at=(7, 3, 4), rot=1)  # Plate 1 x 4
        m.add("3710", "Dark Bluish Gray", at=(7, 3, 5), rot=1)
        m.add("22885", "Dark Bluish Gray", at=(7, 3, 6), rot=3)  # studs to the front
        m.add("22885", "Dark Bluish Gray", at=(9, 3, 6), rot=3)
        m.add("3009", "Dark Bluish Gray", at=(6, 3, 11), rot=1)
    m.use(
        "layered_plate_cap",
        x=7,
        y=3,
        z=1 + 5,
        size=4,
        facing="front",
        title="Cap on the side",
    )
