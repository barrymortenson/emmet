# SPDX-License-Identifier: Apache-2.0
"""Stepped curve hull: a hull side that curves in as it rises, a contour stack of plate layers each set in from the one
below, every layer's edge finished with a curved slope so the steps read as one smooth sweep.

m.use("stepped_curve_hull", x, y, z, length=6, steps=(3, 2, 1), facing="front", color="Light Bluish Gray", title="Hull")
builds, on the studs at height z, a hull side `length` studs along x whose outer edge is at y and whose inner side is a
1-stud spine at y + sum(steps). Each layer is a plate course reaching from its outer edge to the spine, with a row of
curved slopes on its outer `step` studs, falling to the front (-y): a 6, 4 or 3 deep step takes a curved slope that deep
and a brick tall (42022, 61678, 50950), a 2 deep step the 2 x 1 curved slope (11477), and a 1 deep step a 1 x 1 cheese
slope (54200). Wide steps low and narrow ones high curve the side over like the shoulder of a hull. The next layer
stands on the spine (a brick, or two plates, the height of the slopes), so the stack is a hollow shell; a row of tiles
finishes the spine's top. facing "back" builds the far side, its edge at y + sum(steps) + 1 and its spine at y, curving
to +y. Returns the z of the top.
"""

from emmet_dsl import Brick, Plate, Tile
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10356. Not derived from any
model file."""

PARTS = [
    "42022",
    "61678",
    "50950",
    "11477",
    "54200",
    "3023",
    "3021",
    "3795",
    "3034",
    "3832",
    "3666",
    "3710",
    "3010",
    "3009",
    "3069b",
    "6636",
]

# A step's depth: the curved slope that finishes it, and its height in plates.
EDGE = {
    6: ("42022", 3),
    4: ("61678", 3),
    3: ("50950", 3),
    2: ("11477", 2),
    1: ("54200", 2),
}
PLATES = (1, 2, 3, 4, 6, 8, 10, 12)  # the lengths of 2-wide plates


def build(
    m,
    x,
    y,
    z,
    length=6,
    steps=(3, 2, 1),
    facing="front",
    color="Light Bluish Gray",
    title="Hull",
):
    """The layers, their curved edges, and the spine; returns the z of the top."""
    if (
        not 2 <= length <= 16
        or not steps
        or any(s not in EDGE for s in steps)
        or facing not in ("front", "back")
    ):
        raise ValueError(
            f"stepped_curve_hull takes a length of 2 to 16, steps of {sorted(EDGE)}, and facing front or back, not "
            f"{length}, {steps}, {facing!r}"
        )
    depth = sum(steps) + 1
    front = facing == "front"
    spine = y + depth - 1 if front else y
    inset, zl = 0, z
    for i, step in enumerate(steps):
        part, h = EDGE[step]
        w = depth - inset
        # Plates across the layer, so each reaches the spine; a short piece at the edge sits under the slopes and is
        # tied by them.
        short = next(k for k in range(step) if w - k in PLATES)
        cols = [(c, min(2, length - c)) for c in range(0, length, 2)]
        actions = []
        for c, cw in cols:
            for lo, n in ((0, short), (short, w - short)) if short else ((0, w),):
                if n:
                    yy = y + inset + lo if front else y + depth - inset - lo - n
                    # A plate's long side runs along y unturned.
                    part_id, turn = (Plate(cw, n), 0) if n >= cw else (Plate(n, cw), 1)
                    actions.append(
                        (
                            1,
                            lambda c=c, yy=yy, p=part_id, t=turn: m.add(
                                p, color, at=(x + c, yy, zl), rot=t
                            ),
                        )
                    )
        in_steps(m, f"{title}, layer {i + 1}: plates", actions)
        edge = y + inset if front else y + depth - inset - step
        in_steps(
            m,
            f"{title}, layer {i + 1}: curve",
            [
                (
                    1,
                    lambda k=k: m.add(
                        part, color, at=(x + k, edge, zl + 1), rot=3 if front else 1
                    ),
                )
                for k in range(length)
            ],
        )
        with m.step(f"{title}, layer {i + 1}: spine"):
            if h == 3:
                m.row(Brick, color, at=(x, spine, zl + 1), length=length)
            else:
                for p in range(h):
                    m.row(Plate, color, at=(x, spine, zl + 1 + p), length=length)
        inset += step
        zl += 1 + h
    with m.step(f"{title}, spine cap"):
        m.row(Tile, color, at=(x, spine, zl), length=length)
    return zl + 1


def example(m):
    """A hull section 6 studs long, both sides curving in over a keel plate."""
    with m.step("Keel"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(6, 14))
    m.use(
        "stepped_curve_hull", x=0, y=7, z=1, length=6, facing="back", title="Far side"
    )
    m.use("stepped_curve_hull", x=0, y=0, z=1, length=6, title="Near side")
