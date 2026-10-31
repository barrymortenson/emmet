# SPDX-License-Identifier: Apache-2.0
"""Cave mouth: the dark entrance of a cave in a rock face, an arch for its lintel standing on rock jambs, with black
bricks set back inside so the opening reads deep, and broken rock over it.

m.use("cave_mouth", x, y, z, jambs=5, color="Dark Bluish Gray", dark="Black", cap="Light Bluish Gray", title="Cave")
builds, on the studs at height z, a rock face 8 studs along x from x and 3 deep (y to y + 3), its face to the front
(-y), jambs + 3 bricks tall, with an opening 4 studs wide (x + 2 to x + 6). `jambs` courses of jambs, 2 x 3 bricks and 2
x 2s by turns, stand either side, the top one a 2 x 2 tied to the other jamb by a black 1 x 6 brick; an arch 1 x 6 x 2
spans them, its legs on the jambs, so the opening is `jambs` bricks high at its sides and higher under the arch's curve:
5 (the default) is 15 plates, room for a figure with a hat (docs/spec/building-rules.md, rule 1), and fewer make a mouth
too low to walk into. Behind the opening, two studs back, black bricks are the cave's dark inside, tied into the jambs
and into the arch's backing. A 75° slope leans on each jamb's front as a buttress (keep two studs clear in front of y),
and a 2 x 4 slope over the arch, 2 x 1 slopes beside it, and cheese slopes on top break the rock's top. A mouth 5
courses high or more keeps clear the way in (3 studs out in front, from a plate above the ground up to where the arch
springs; rule 2), so a rock or a barrel in it is a figure_clearance finding. Returns the z of the top, with .door_top
(the z the arch springs from) and .door ((x0, y0), (x1, y1), the mouth's footprint, 2 studs deep to the dark inside).
"""

from emmet_dsl.core import APPROACH, HEADROOM, fit, in_steps

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "3002",
    "3003",
    "3005",
    "3010",
    "3009",
    "15254",
    "3622",
    "3004",
    "3037",
    "3040b",
    "54200",
    "4460b",
]


def build(
    m,
    x,
    y,
    z,
    jambs=5,
    color="Dark Bluish Gray",
    dark="Black",
    cap="Light Bluish Gray",
    title="Cave",
):
    """The cave mouth; returns the z of its top, with .door_top and .door."""
    if jambs < 2:
        raise ValueError(f"cave_mouth takes 2 or more courses of jambs, not {jambs}")

    def add(part, at, rot=0, c=color):
        return (1, lambda: m.add(part, c, at=at, rot=rot))

    rows = []
    for r in range(jambs):
        zr = z + 3 * r
        if (jambs - r) % 2:
            # 2 x 2 jambs, and a black 1 x 6 that reaches into them and ties them together: the course under the arch.
            rows.append(
                [
                    add("3003", (x, y, zr)),
                    add("3005", (x, y + 2, zr)),
                    add("3003", (x + 6, y, zr)),
                    add("3005", (x + 7, y + 2, zr)),
                    add("3009", (x + 1, y + 2, zr), 1, dark),
                ]
            )
        else:
            # The jambs, 2 x 3, and the dark back wall between them, two studs in.
            rows.append(
                [
                    add("3002", (x, y, zr)),
                    add("3002", (x + 6, y, zr)),
                    add("3010", (x + 2, y + 2, zr), 1, dark),
                ]
            )
    springs = z + 3 * jambs
    for r in (jambs, jambs + 1):
        zr = z + 3 * r
        row = [
            add("3622", (x, y, zr)),
            add("3622", (x + 7, y, zr)),
            add("3004", (x + 1, y + 1, zr)),
            add("3004", (x + 6, y + 1, zr)),
            add("3010", (x + 2, y + 2, zr), 1, dark),
        ]
        if r == jambs:
            # The arch's legs stand on the jambs' inner studs.
            row.insert(0, add("15254", (x + 1, y, zr), 1))
        rows.append(row)
    zt = springs + 6
    rows.append(
        [
            add("3622", (x, y, zt)),
            add("3622", (x + 7, y, zt)),
            add("3040b", (x + 1, y, zt), 3),
            add("3037", (x + 2, y, zt), 3),
            add("3040b", (x + 6, y, zt), 3),
            add("3009", (x + 1, y + 2, zt), 1),
        ]
    )
    for k, row in enumerate(rows):
        in_steps(m, f"{title}, row {k + 1}", row)
        if k == 0:
            # 75° slopes lean on the jambs' fronts as buttresses, before the arch closes the way in over them.
            in_steps(
                m,
                f"{title}, buttresses",
                [
                    add("4460b", (x, y - 2, z), 3),
                    add("4460b", (x + 7, y - 2, z), 3, cap),
                ],
            )
    in_steps(
        m,
        f"{title}, top",
        [
            add("54200", (x, y, zt + 3), 2, cap),
            add("54200", (x + 3, y + 2, zt + 3), 1, cap),
            add("54200", (x + 7, y + 1, zt + 3), 0, cap),
        ],
    )
    if springs - z >= HEADROOM:
        m.keep_clear(
            ((x + 2, y - APPROACH, z + 1), (x + 6, y, springs)),
            f"a figure walking into {title}",
        )
    return fit(zt + 3, door_top=springs, door=((x + 2, y), (x + 6, y + 2)))


def example(m):
    """A cave mouth on a gray plate."""
    with m.step("Ground"):
        m.add("3036", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("cave_mouth", x=0, y=2, z=1)
