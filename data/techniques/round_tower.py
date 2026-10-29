# SPDX-License-Identifier: Apache-2.0
"""Round tower: a hollow round tower 8 studs across, rings of four curved bricks tied by round plate floors, with
merlons over the seams at the top.

m.use("round_tower", x, y, z, rows=6, floor_every=3, color="Light Bluish Gray", floor_color=None, merlons=True,
title="Tower") builds an 8 x 8 tower whose front-left corner is (x, y), from z: each course a ring of four Brick 4 x 4
Corner Round (48092), whose seams fall in the same four places course after course. A ring stack on its own is four
loose columns, so every `floor_every` courses (and at the top) a Plate 8 x 8 Round (74611) lies across the ring and ties
the four quarters; it is also the tower's floor, and its edge shows as a thin band (in the walls' color unless
floor_color is given). With merlons, a 1 x 2 brick sits over each of the four seams on the top floor, capped with a
tile, the gaps between them the crenels. Returns the z of the top floor.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["48092", "74611", "3004", "3069b"]

QUARTERS = [
    ((4, 4), 0),
    ((0, 4), 1),
    ((0, 0), 2),
    ((4, 0), 3),
]  # min corner from (x, y), and turn
SEAMS = [
    ((3, 0), 1),
    ((7, 3), 0),
    ((3, 7), 1),
    ((0, 3), 0),
]  # where a 1 x 2 over a seam goes, and its turn


def build(
    m,
    x,
    y,
    z,
    rows=6,
    floor_every=3,
    color="Light Bluish Gray",
    floor_color=None,
    merlons=True,
    title="Tower",
):
    """The rings, the floors, and the merlons; returns the z of the top floor."""
    if rows < 1 or floor_every < 1:
        raise ValueError(
            f"round_tower needs rows and floor_every of 1 or more, not {rows}, {floor_every}"
        )
    floor_color = floor_color or color
    zr = z
    for r in range(rows):
        with m.step(f"{title}, ring {r + 1}"):
            for (dx, dy), turn in QUARTERS:
                m.add("48092", color, at=(x + dx, y + dy, zr), rot=turn)
        zr += 3
        if (r + 1) % floor_every == 0 or r == rows - 1:
            with m.step(f"{title}, floor over ring {r + 1}"):
                m.add("74611", floor_color, at=(x, y, zr))
            zr += 1
    if merlons:
        with m.step(f"{title}: merlons"):
            for (dx, dy), turn in SEAMS:
                m.add("3004", color, at=(x + dx, y + dy, zr), rot=turn)
            for (dx, dy), turn in SEAMS:
                m.add("3069b", color, at=(x + dx, y + dy, zr + 3), rot=turn)
    return zr


def example(m):
    """A tower four rings tall, floored halfway up and at the top."""
    m.use(
        "round_tower", x=0, y=0, z=0, rows=4, floor_every=2, floor_color="Reddish Brown"
    )
