# SPDX-License-Identifier: Apache-2.0
"""Porch posts: posts of a round brick under a tall 1 x 1 brick, a beam across them, and a low balustrade between.

m.use("porch_posts", x, y, z, length=6, height=6, gap=None, color="White", title="Porch") stands a post at each end of
a run `length` studs along x from (x, y, z), each a round 1 x 1 brick under tall 1 x 1 bricks, `height` bricks in all,
lays a plate across their tops as the beam, and between them a balustrade of round plates stacked two high under a
rail of tiles. length is 5 or more, so the posts stand 3 studs clear apart. gap=(offset, width) leaves the balustrade
out for `width` studs (3 or more) from x + offset: the way in, in line with the door and the steps, which it keeps
clear 3 studs out in front, up to the beam. height is the clear room under the beam: 6 (the default) clears a 1 x
4 x 6 door behind the porch, and less than 5 can't be walked under, so it's 5 to 12 (docs/spec/building-rules.md, rule
1). Returns the z of the beam's top, with .posts (the (x, y) of each post) and .underside (the z of the beam's bottom).
"""

from emmet_dsl import Plate, Tile
from emmet_dsl.core import APPROACH, fit
from emmet_dsl.techniques.canopy import column

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 43217. Not derived
from any model file."""

PARTS = [
    "3062b",
    "2453b",
    "14716",
    "3005",
    "6141",
    "3666",
    "3460",
    "3710",
    "2431",
    "3069b",
    "6636",
    "3070b",
]


def build(m, x, y, z, length=6, height=6, gap=None, color="White", title="Porch"):
    """The posts, the beam, and the balustrade; returns the z of the beam's top, with .posts and .underside."""
    if length < 5:
        raise ValueError(
            f"porch_posts needs a length of 5 or more (its posts 3 studs clear apart), not {length}"
        )
    if not 5 <= height <= 12:
        raise ValueError(
            f"porch_posts height is 5 to 12 bricks (6 clears a door behind it), not {height}"
        )
    posts = [(x, y), (x + length - 1, y)]
    open_ = set(range(gap[0], gap[0] + gap[1])) if gap else set()
    if open_ and (min(open_) < 1 or max(open_) > length - 2):
        raise ValueError(
            f"porch_posts' gap {gap} must lie between its posts (offsets 1 to {length - 2})"
        )
    if open_ and len(open_) < 3:
        raise ValueError(
            f"porch_posts' gap is 3 studs or more, so a figure fits through, not {len(open_)}"
        )
    rail = [a for a in range(1, length - 1) if a not in open_]
    top = z + 3 * height
    with m.step(f"{title}: posts"):
        for px, py in posts:
            m.add("3062b", color, at=(px, py, z))
            h = z + 3
            for part, n in column(height - 1):
                m.add(part, color, at=(px, py, h))
                h += 3 * n
    with m.step(f"{title}: balustrade"):
        for a in rail:
            m.add("6141", color, at=(x + a, y, z))
            m.add("6141", color, at=(x + a, y, z + 1))
    with m.step(f"{title}: rail and beam"):
        for a, n in _runs(rail):
            m.row(Tile, color, at=(x + a, y, z + 2), length=n)
        m.row(Plate, color, at=(x, y, top), length=length)
    if gap:
        # The way in from the front; behind the posts is the porch floor, as deep as the caller made it, and the
        # house door there keeps its own space.
        m.keep_clear(
            (
                (x + gap[0], y - APPROACH, z + 1),
                (x + gap[0] + gap[1], y, z + 3 * height),
            ),
            f"a figure walking into {title}",
        )
    return fit(top + 1, posts=posts, underside=top)


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
    """A porch front on its floor."""
    with m.step("Floor"):
        m.add("3795", "Reddish Brown", at=(0, 0, 0), rot=1)
    m.use("porch_posts", x=0, y=0, z=1)
