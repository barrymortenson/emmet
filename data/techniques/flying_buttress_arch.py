# SPDX-License-Identifier: Apache-2.0
"""Flying buttress arch: a row of flying buttresses at midi scale, each an arch leaping from a slim outer pier to the
nave wall, its back sloped up toward the wall and a pinnacle on the pier.

m.use("flying_buttress_arch", x, y, z, count=3, spacing=3, height=3, stone="Tan", title="Buttresses") stands `count`
buttresses along x from x, every `spacing` studs, on the studs at height z, against a nave wall 1 stud thick whose front
face is at y. Each is a pier of 1 x 1 bricks `height` bricks tall 3 studs in front of the wall, a 1 x 4 arch on top
whose far leg is a cell of the wall's course (lay that course with an opening at (x, y) for it, and carry the wall on
over the leg's studs), a 45 degree slope on the arch rising toward the wall, and a pinnacle (a round plate and a cone)
over the pier. Returns the z of the arch's top, the course where the wall carries on.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21061. Not derived from any
model file. The book leans a bar in clips at about 45 degrees; this grid version springs an arch instead."""

PARTS = ["3005", "3659", "3040b", "6141", "4589"]


def build(m, x, y, z, count=3, spacing=3, height=3, stone="Tan", title="Buttresses"):
    """The piers, the arches, their slopes, and the pinnacles; returns the z of the arches' top."""
    if count < 1 or spacing < 2 or not 1 <= height <= 12:
        raise ValueError(
            f"flying_buttress_arch takes a count of 1 or more, a spacing of 2 or more, and a height of 1 to 12, not "
            f"{count}, {spacing}, {height}"
        )
    za = z + 3 * height
    for i in range(count):
        bx = x + i * spacing
        with m.step(f"{title}, pier {i + 1}"):
            m.stack("3005", stone, at=(bx, y - 3, z), count=height)
    for i in range(count):
        bx = x + i * spacing
        with m.step(f"{title}, arch {i + 1}"):
            m.add("3659", stone, at=(bx, y - 3, za))
            m.add("3040b", stone, at=(bx, y - 2, za + 3), rot=3)
            m.add("6141", stone, at=(bx, y - 3, za + 3))
            m.add("4589", stone, at=(bx, y - 3, za + 4))
    return za + 3


def example(m):
    """Three buttresses against a stretch of nave wall."""
    from emmet_dsl import Plate

    with m.step("Ground"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(8, 6))
    with m.step("Nave wall"):
        m.wall(0, 4, 1, 8, color="Tan", rows=3)
    top = m.use("flying_buttress_arch", x=1, y=4, z=1, count=3, height=3)
    with m.step("Nave wall, upper"):
        m.wall(0, 4, top - 3, 8, color="Tan", rows=1, openings=[(1, 4), (4, 4), (7, 4)])
    with m.step("Nave wall, top"):
        m.wall(0, 4, top, 8, color="Tan", rows=2)
