# SPDX-License-Identifier: Apache-2.0
"""Cave mouth: the dark entrance of a cave in a rock face, an arch for its lintel standing on rock jambs, with black
bricks set back inside so the opening reads deep, and broken rock over it.

m.use("cave_mouth", x, y, z, color="Dark Bluish Gray", dark="Black", cap="Light Bluish Gray", title="Cave") builds, on
the studs at height z, a rock face 8 studs along x from x and 3 deep (y to y + 3), its face to the front (-y), about 6
bricks tall, with an opening 4 studs wide (x + 2 to x + 6) and a little over 3 bricks high. Two courses of jambs, 2 x 3
bricks then 2 x 2s, stand either side; an arch 1 x 6 x 2 spans them, its legs on the jambs; behind the opening, two
studs back, black bricks are the cave's dark inside, tied into the jambs by a black 1 x 6 brick and into the arch's
backing. A 75° slope leans on each jamb's front as a buttress (keep two studs clear in front of y), and a 2 x 4 slope
over the arch, 2 x 1 slopes beside it, and cheese slopes on top break the rock's top. Returns the z of the top.
"""

from emmet_dsl.core import in_steps

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
    color="Dark Bluish Gray",
    dark="Black",
    cap="Light Bluish Gray",
    title="Cave",
):
    """The cave mouth; returns the z of its top."""

    def add(part, at, rot=0, c=color):
        return (1, lambda: m.add(part, c, at=at, rot=rot))

    rows = [
        # The jambs, 2 x 3, and the dark back wall between them, two studs in.
        [
            add("3002", (x, y, z)),
            add("3002", (x + 6, y, z)),
            add("3010", (x + 2, y + 2, z), 1, dark),
        ],
        # 2 x 2 jambs, and a black 1 x 6 that reaches into them and ties them together.
        [
            add("3003", (x, y, z + 3)),
            add("3005", (x, y + 2, z + 3)),
            add("3003", (x + 6, y, z + 3)),
            add("3005", (x + 7, y + 2, z + 3)),
            add("3009", (x + 1, y + 2, z + 3), 1, dark),
        ],
    ]
    for r in (2, 3):
        zr = z + 3 * r
        row = [
            add("3622", (x, y, zr)),
            add("3622", (x + 7, y, zr)),
            add("3004", (x + 1, y + 1, zr)),
            add("3004", (x + 6, y + 1, zr)),
            add("3010", (x + 2, y + 2, zr), 1, dark),
        ]
        if r == 2:
            # The arch's legs stand on the jambs' inner studs.
            row.insert(0, add("15254", (x + 1, y, zr), 1))
        rows.append(row)
    zt = z + 12
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
    return zt + 3


def example(m):
    """A cave mouth on a gray plate."""
    with m.step("Ground"):
        m.add("3036", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("cave_mouth", x=0, y=2, z=1)
