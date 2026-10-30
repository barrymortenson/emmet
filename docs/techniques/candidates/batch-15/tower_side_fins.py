# SPDX-License-Identifier: Apache-2.0
"""Tower side fins: a slender micro tower whose faces are tall tiles hung on side studs, each column its own strip, so
the seams between them read as fins and a plate between sections as a band of floors.

m.use("tower_side_fins", x, y, z, width=2, sections=2, faces="front", core="Dark Bluish Gray", face="Light Bluish Gray",
band="Trans Light Blue", title="Tower") stands a tower `width` studs wide along x and a stud deep, on the studs at
height z from (x, y): a plate, then `sections` sections, each a column per stud of a brick with a side stud (87087, or
47905 with studs front and back when faces is "both"), a 1 x 1 x 3 brick (14716), a 1 x 1 brick, another side-stud brick
and a 1 x 1 brick, 21 plates in all, with a plate across the columns in the band color on top. A 1 x 8 tile stands on
each column's two side studs, six studs apart, covering the section but for half a plate at each end, so between
sections the band shows. A plate in the core color caps the tower. Returns the z of the cap's top.
"""

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = ["87087", "47905", "14716", "3005", "4162", "3024", "3023b", "3623", "3710"]

FRONT = Rot(x=90)  # a tile on its back, its face to -y, its long side up
BACK = Rot(x=-90)  # the same, its face to +y
OUT = 0.4  # a tile's thickness, in studs
SECTION = 21  # a section's bricks, in plates


def _row(m, color, x, y, z, width):
    if width == 1:
        return m.add(Plate(1, 1), color, at=(x, y, z))
    return m.add(Plate(1, width), color, at=(x, y, z), rot=1)


def build(
    m,
    x,
    y,
    z,
    width=2,
    sections=2,
    faces="front",
    core="Dark Bluish Gray",
    face="Light Bluish Gray",
    band="Trans Light Blue",
    title="Tower",
):
    """The columns, their tiles, the bands, and the cap; returns the z of the top."""
    if not 1 <= width <= 4 or sections < 1 or faces not in ("front", "both"):
        raise ValueError(
            "tower_side_fins takes a width of 1 to 4, 1 or more sections, and faces 'front' or 'both', "
            f"not {width}, {sections}, {faces!r}"
        )
    side = "87087" if faces == "front" else "47905"
    with m.step(f"{title}: base"):
        _row(m, core, x, y, z, width)
    zs = z + 1
    for s in range(sections):
        # (part, height) up the column; the side-stud bricks are 15 plates apart, six studs.
        column = [(side, 3), ("14716", 9), ("3005", 3), (side, 3), ("3005", 3)]
        for i in range(width):
            with m.step(f"{title}, section {s + 1}: column {i + 1}"):
                zc = zs
                for part, h in column:
                    m.add(part, core, at=(x + i, y, zc), rot=3)
                    zc += h
        with m.step(f"{title}, section {s + 1}: faces"):
            for i in range(width):
                m.add("4162", face, at=(x + i, y - OUT, zs + 0.5), rot=FRONT)
                if faces == "both":
                    m.add("4162", face, at=(x + i, y + 1, zs + 0.5), rot=BACK)
            _row(m, band if s < sections - 1 else core, x, y, zs + SECTION, width)
        zs += SECTION + 1
    return zs


def example(m):
    """A tower two sections tall, faced front and back, beside a lower one faced in front only."""
    with m.step("Ground"):
        m.add(Plate(4, 6), "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use(
        "tower_side_fins",
        x=1,
        y=1,
        z=1,
        width=2,
        sections=2,
        faces="both",
        title="Tall tower",
    )
    m.use(
        "tower_side_fins",
        x=4,
        y=2,
        z=1,
        width=1,
        sections=1,
        core="Black",
        face="White",
        title="Short tower",
    )
