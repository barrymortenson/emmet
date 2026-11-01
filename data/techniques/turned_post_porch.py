# SPDX-License-Identifier: Apache-2.0
"""Turned post porch: porch posts turned from round bricks and round plates, under a fascia that curves around the front
corners and returns to the wall.

m.use("turned_post_porch", x, y, z, length=6, depth=3, tiers=5, gap=None, color="Medium Nougat",
fascia="Reddish Brown", rail="White", title="Porch") stands, on a porch floor whose studs are at z, a post at each front corner of the length x
depth rectangle whose front-left corner is (x, y) (the house wall runs along its back, at y + depth): each post is
`tiers` round 1 x 1 bricks with a round 1 x 1 plate on each, so it reads as turned. Over them goes the fascia: a Plate 2
x 2 Quarter Circle (7904) at each front corner, so the edge curves round, plates along the front between them, and
plates back along each side to the wall; a second layer of plates, set in from the round corners, bridges their joints.
Between the posts at the front, a low rail of round plates under tiles; gap=(offset, width) leaves it out for `width`
studs (3 or more) from x + offset, the way in, in line with the door, which it keeps clear 3 studs out in front and
back over the porch floor to the wall, up to the fascia. length 5 to 12, depth 3 to 8. Each tier is 4
plates, so the fascia's underside is 4 x tiers plates up: 5 tiers (the default, 20 plates) clears a 1 x 4 x 6 door
behind the porch, and 4 (16) is the least a figure walks under, so tiers is 4 to 8 (docs/spec/building-rules.md, rule
1). Returns the z of the fascia's top, with .posts (the (x, y) of each post) and .underside (the z of the fascia's
bottom).
"""

from emmet_dsl import Plate, Tile
from emmet_dsl.core import APPROACH, fit

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387, set 43217. Not derived
from any model file."""

PARTS = [
    "3062b",
    "6141",
    "7904",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3069b",
    "2431",
    "6636",
    "3070b",
]

CORNER = {
    "left": 2,
    "right": 3,
}  # the quarter turn that puts a corner plate's round corner front-left or front-right


def build(
    m,
    x,
    y,
    z,
    length=6,
    depth=3,
    tiers=5,
    gap=None,
    color="Medium Nougat",
    fascia="Reddish Brown",
    rail="White",
    title="Porch",
):
    """The posts a tier at a time, the rail, and the fascia; returns the z of the fascia's top."""
    if not 5 <= length <= 12 or not 3 <= depth <= 8:
        raise ValueError(
            f"turned_post_porch takes a length of 5 to 12 and a depth of 3 to 8, not {length} x {depth}"
        )
    if not 4 <= tiers <= 8:
        raise ValueError(
            f"turned_post_porch takes 4 to 8 tiers (5 clears a door behind it), not {tiers}"
        )
    right = x + length - 1
    open_ = set(range(gap[0], gap[0] + gap[1])) if gap else set()
    if open_ and (min(open_) < 1 or max(open_) > length - 2):
        raise ValueError(
            f"turned_post_porch's gap {gap} must lie between its posts (offsets 1 to {length - 2})"
        )
    if open_ and len(open_) < 3:
        raise ValueError(
            f"turned_post_porch's gap is 3 studs or more, so a figure fits through, not {len(open_)}"
        )
    rail_at = [a for a in range(1, length - 1) if a not in open_]
    for t in range(tiers):
        with m.step(f"{title}: posts, tier {t + 1}"):
            for px in (x, right):
                m.add("3062b", color, at=(px, y, z + 4 * t))
                m.add("6141", color, at=(px, y, z + 4 * t + 3))
            if t == 0:
                for a in rail_at:
                    m.add("6141", rail, at=(x + a, y, z))
    with m.step(f"{title}: rail"):
        for a, n in _runs(rail_at):
            m.row(Tile, rail, at=(x + a, y, z + 1), length=n)
    top = z + 4 * tiers
    with m.step(f"{title}: fascia, corners"):
        m.add("7904", fascia, at=(x, y, top), rot=CORNER["left"])
        m.add("7904", fascia, at=(right - 1, y, top), rot=CORNER["right"])
        m.row(Plate, fascia, at=(x + 2, y, top), length=length - 4)
        for px in (x, right):
            m.row(Plate, fascia, at=(px, y + 2, top), length=depth - 2, axis="y")
    with m.step(f"{title}: fascia, top"):
        # A second layer, set in from the round corners, bridges every joint of the first.
        m.row(Plate, fascia, at=(x + 1, y, top + 1), length=length - 2)
        for px in (x, right):
            m.row(Plate, fascia, at=(px, y + 1, top + 1), length=depth - 1, axis="y")
    if gap:
        # The way in: 3 studs out in front, and over the porch floor back to the house wall.
        a, b = x + gap[0], x + gap[0] + gap[1]
        m.keep_clear(
            [
                ((a, y - APPROACH, z + 1), (b, y, top)),
                ((a, y + 1, z + 1), (b, y + depth, top)),
            ],
            f"a figure walking into {title}",
        )
    return fit(top + 2, posts=[(x, y), (right, y)], underside=top)


def _runs(cells):
    """Consecutive offsets as (first, how many)."""
    out = []
    for a in cells:
        if out and out[-1][0] + out[-1][1] == a:
            out[-1] = (out[-1][0], out[-1][1] + 1)
        else:
            out.append((a, 1))
    return out


def example(m):
    """A porch on its floor against a stub of the house wall."""
    from emmet_dsl import Brick

    with m.step("Floor"):
        m.add("3795", "Reddish Brown", at=(0, 0, 0), rot=1)
        m.add("3666", "Reddish Brown", at=(0, 2, 0), rot=1)
    for r in range(7):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick, "White", at=(0, 3, 3 * r), length=6, first=2 if r % 2 else None
            )
    m.use("turned_post_porch", x=0, y=0, z=1)
