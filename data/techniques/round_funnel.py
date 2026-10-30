# SPDX-License-Identifier: Apache-2.0
"""Round funnel: a tall round ship's funnel or chimney of 4 x 4 round bricks, a dark band at the top, steam pipes up
its side.

m.use("round_funnel", x, y, z, height=5, pipes=2, color="Red", band="Black", pipe_color="Light Bluish Gray",
title="Funnel") stands, on the studs at z, a funnel 4 x 4 studs from (x, y): `height` round bricks 4 x 4 (87081), the
rest in `color` and the top one in `band` on a round plate 4 x 4 in `band`, so the funnel has a dark top 4 plates
deep, and a round plate 4 x 4 with a 2 x 2 hole (11833) as the open rim. `pipes` steam pipes (0 to 2) rise up its
right side, each a column of round bricks 1 x 1 at x + 4 (y + 1 and y + 2), one brick shorter than the funnel, capped
with a round tile. Keep x + 4 to x + 5 free for them. height: 2 to 8. Returns the z of the rim's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10335. Not derived from any
model file."""

PARTS = ["87081", "60474", "11833", "3062b", "98138"]


def build(
    m,
    x,
    y,
    z,
    height=5,
    pipes=2,
    color="Red",
    band="Black",
    pipe_color="Light Bluish Gray",
    title="Funnel",
):
    """The funnel a course at a time, with the pipes beside it; returns the z of the rim's top."""
    if not 2 <= height <= 8 or not 0 <= pipes <= 2:
        raise ValueError(
            f"round_funnel takes a height of 2 to 8 and 0 to 2 pipes, not {height}, {pipes}"
        )
    py = [y + 1, y + 2][:pipes]
    zl = z
    for r in range(height):
        top = r == height - 1
        with m.step(f"{title}, course {r + 1}"):
            if top:
                m.add("60474", band, at=(x, y, zl))
                zl += 1
            m.add("87081", band if top else color, at=(x, y, zl))
            if r < height - 1:
                for p in py:
                    m.add("3062b", pipe_color, at=(x + 4, p, z + 3 * r))
        zl += 3
    with m.step(f"{title}, rim and pipe caps"):
        m.add("11833", band, at=(x, y, zl))
        for p in py:
            m.add("98138", pipe_color, at=(x + 4, p, z + 3 * (height - 1)))
    return zl + 1


def example(m):
    """A funnel on a patch of deck."""
    with m.step("Deck"):
        m.add("3958", "Tan", at=(-1, -1, 0))
    m.use("round_funnel", x=0, y=0, z=1)
