# SPDX-License-Identifier: Apache-2.0
"""Panel pilaster: a flat pilaster 2 studs wide with a raised base, a dark panel up its face, and a capital, framing a
shopfront bay.

m.use("panel_pilaster", x, y, z, panels=1, color="Dark Green", accent="Nougat", panel="Black", title="Pilaster")
builds a pilaster on the wall line y (the wall runs along x and faces front, -y), from x to x + 2: a 2 x 2 brick base
standing a stud proud, a 2 x 2 plate in the accent color over it, then a shaft of 1 x 1 x 1 2/3 bricks with studs on
one side (32952), 10 plates tall per panel. Jumper plates (15573) on the shaft's side studs bring a stud to its
middle, and a 1 x 4 tile in the panel color stands up on them, so a dark slot runs up the face with the shaft color
round it. A 2 x 2 plate in the accent color caps it, a stud proud, with a jumper on it for a lamp or a plant. Returns
the z of the capital's top, which a wall of bricks beside it meets every 5 courses (with panels=1).
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["3003", "3022", "32952", "15573", "2431"]

FACE = Rot(y=-90, z=90)  # a 1 x 2 jumper on its back, its stud to -y, long along x
UP = Rot(x=90)  # a 1 x 4 tile on its back, its face to -y, its long side up
OUT = 0.4  # a plate's or tile's thickness, in studs


def build(
    m,
    x,
    y,
    z,
    panels=1,
    color="Dark Green",
    accent="Nougat",
    panel="Black",
    title="Pilaster",
):
    """The base, the shaft with its panels, and the capital; returns the z of the capital's top."""
    if panels < 1:
        raise ValueError(f"panel_pilaster needs 1 panel or more, not {panels}")
    with m.step(f"{title}: base"):
        m.add("3003", color, at=(x, y - 1, z))
        m.add("3022", accent, at=(x, y - 1, z + 3))
    zs = z + 4
    for p in range(panels):
        zp = zs + 10 * p
        for half in range(2):
            with m.step(
                f"{title}, panel {p + 1}: shaft" + (" (2 of 2)" if half else "")
            ):
                for c in range(2):
                    m.add("32952", color, at=(x + c, y, zp + 5 * half), rot=3)
                for k in range(2):
                    m.add(
                        "15573",
                        color,
                        at=(x, y - OUT, zp + 5 * half + 2.5 * k),
                        rot=FACE,
                    )
        with m.step(f"{title}, panel {p + 1}: face"):
            m.add("2431", panel, at=(x + 0.5, y - 2 * OUT, zp), rot=UP)
    top = zs + 10 * panels
    with m.step(f"{title}: capital"):
        m.add("3022", accent, at=(x, y - 1, top))
        m.add("15573", color, at=(x, y - 1, top + 1), rot=1)
    return top + 1


def example(m):
    """Two pilasters framing a stretch of wall, on a plate, with a plate course over them all."""
    from emmet_dsl import Brick, Plate

    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, -1, 0), length=8, width=2)
    for i, px in enumerate((0, 6)):
        m.use("panel_pilaster", x=px, y=0, z=1, title=f"Pilaster {i + 1}")
    for r in range(5):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "Dark Green",
                at=(2, 0, 1 + 3 * r),
                length=4,
                first=1 if r % 2 else None,
            )
    with m.step("Fascia"):
        m.row(Plate, "Dark Green", at=(0, 0, 16), length=8, first=3)
