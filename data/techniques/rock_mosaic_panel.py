# SPDX-License-Identifier: Apache-2.0
"""Rock mosaic panel: a sheer cliff face of interlocking wedge facets with tan strata, built flat and hung upright on
side studs.

m.use("rock_mosaic_panel", x, y, z, width=8, height=6, seed=1, color="Light Bluish Gray", shadow="Dark Bluish Gray",
strata="Tan", title="Cliff face") builds, on the studs at z, a cliff face `width` studs along x from (x, y), facing
front (-y), `height` studs tall (2.5 plates a stud). The mosaic is a submodel built flat: a plate in `shadow`; over it,
packed edge to edge where seed says, wedge plates 2 x 4 and 2 x 3 (41769a/41770a, 43722a/43723a), right and left,
pointing every way in quarter turns, mostly in `color` and some in `shadow`, with 1 x 2 and 1 x 3 plates in `strata`
as veins between; the wedges' cut corners show the dark plate under them; and on some of the 2 x 4 wedges a 2 x 3
wedge of the same hand, square ends together, for a second layer of relief. Stood up, it hangs on a backing a stud
thick of bricks 1 2/3 tall with studs on one side (22885, 32952, their joints crossed row to row). width: 4 to 12 and
height: 2 to 8, both even. Returns the z of the top.
"""

import random

from emmet_dsl import Plate, Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11377. Not derived from any
model file."""

PARTS = [
    "41769a",
    "41770a",
    "43722a",
    "43723a",
    "3023b",
    "3623",
    "22885",
    "32952",
    "3958",
    "3036",
    "3034",
    "3832",
    "3020",
    "3795",
    "3460",
    "3666",
]

UPRIGHT = Rot(x=90)  # a part built flat, stood up with its studs to -y
WEDGES = {4: ("41769a", "41770a"), 3: ("43722a", "43723a")}  # by length: right, left
STRATA = {2: "3023b", 3: "3623"}  # Plate 1 x n
# Where a 2 x 3 wedge goes on a 2 x 4 so their square ends meet, by the turn: the 2 x 4's square end is at -x, -y, +x,
# +y for turns 0 to 3.
ON_TOP = {0: (0, 0), 1: (0, 0), 2: (1, 0), 3: (0, 1)}


def _mosaic(m, width, height, seed, color, shadow, strata):
    """The panel flat: the plate, the facets, and the second layer, its corner at the origin."""
    rng = random.Random(seed)
    with m.step("Plate"):
        m.fill(Plate, shadow, at=(0, 0, 0), size=(width, height))
    used, pieces = set(), []
    for j in range(height):
        for i in range(width):
            if (i, j) in used:
                continue
            options = []
            for n in (4, 3):
                for turn in range(4):
                    w, d = (n, 2) if turn % 2 == 0 else (2, n)
                    options.append(("wedge", n, turn, w, d))
            for n in (3, 2):
                options.append(("strata", n, 1, n, 1))
                options.append(("strata", n, 0, 1, n))
            rng.shuffle(options)
            # Strata now and then, wedges where they fit; a lone stud stays bare.
            veins = rng.random() < 0.15
            options.sort(key=lambda o: (o[0] == "strata") != veins)
            for kind, n, turn, w, d in options:
                cells = {(i + a, j + b) for a in range(w) for b in range(d)}
                if i + w <= width and j + d <= height and not cells & used:
                    used |= cells
                    pieces.append((kind, n, turn, i, j))
                    break
    for k in range(0, len(pieces), 6):
        with m.step(f"Facets {k // 6 + 1}"):
            for kind, n, turn, i, j in pieces[k : k + 6]:
                if kind == "strata":
                    m.add(STRATA[n], strata, at=(i, j, 1), rot=turn)
                else:
                    hand = rng.randrange(2)
                    col = shadow if rng.random() < 0.25 else color
                    m.add(WEDGES[n][hand], col, at=(i, j, 1), rot=turn)
                    if n == 4 and rng.random() < 0.35:
                        dx, dy = ON_TOP[turn]
                        m.add(WEDGES[3][hand], color, at=(i + dx, j + dy, 2), rot=turn)


def build(
    m,
    x,
    y,
    z,
    width=8,
    height=6,
    seed=1,
    color="Light Bluish Gray",
    shadow="Dark Bluish Gray",
    strata="Tan",
    title="Cliff face",
):
    """The backing and the mosaic; returns the z of the top."""
    if not (4 <= width <= 12 and 2 <= height <= 8) or width % 2 or height % 2:
        raise ValueError(
            f"rock_mosaic_panel takes an even width of 4 to 12 and height of 2 to 8, not {width}, {height}"
        )
    for r in range(height // 2):
        with m.step(f"{title}, backing row {r + 1}"):
            k = 0
            if r % 2:
                m.add("32952", shadow, at=(x, y, z + 5 * r), rot=3)
                m.add("32952", shadow, at=(x + width - 1, y, z + 5 * r), rot=3)
                k = 1
            while k + 2 <= width - (r % 2):
                m.add("22885", shadow, at=(x + k, y, z + 5 * r), rot=3)
                k += 2
    panel = m.submodel(
        "rock_mosaic",
        _mosaic,
        width=width,
        height=height,
        seed=seed,
        color=color,
        shadow=shadow,
        strata=strata,
    )
    lo, hi = panel.box
    depth = (
        hi[2] - lo[2]
    ) * 0.4  # the panel's thickness, plates to studs, once it stands
    with m.step(f"{title}, face"):
        m.place(panel, at=(x, y - depth, z), rot=UPRIGHT)
    return z + 5 * (height // 2)


def example(m):
    """A cliff face 8 studs wide and 6 tall on a strip of rock."""
    with m.step("Footing"):
        m.row(Plate, "Dark Bluish Gray", at=(0, -1, 0), length=8, width=2)
    m.use("rock_mosaic_panel", x=0, y=0, z=1)
