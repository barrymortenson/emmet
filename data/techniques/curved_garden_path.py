# SPDX-License-Identifier: Apache-2.0
"""Curved garden path: a smooth path two studs wide whose bends are true quarter circles of curved tiles, so it sweeps
around a lawn's corner instead of turning in steps.

m.use("curved_garden_path", x, y, z, legs=(4, 4), turns="L", color="Tan", title="Path") lays a path on the studs at
height z (a baseplate is z = 0.5), starting at (x, y) and heading along +x, two studs wide from y to y + 2. It runs
`legs[0]` studs straight, then bends a quarter turn left ("L", toward +y) or right ("R", toward -y) as `turns` says,
runs `legs[1]` studs, and so on: len(turns) is len(legs) - 1. Each straight is tiles 2 wide; each bend is two
quarter-ring tiles, a Tile 3 x 3 Corner Round (79393) inside a Tile 4 x 4 Corner Round (3477), so the bend's inside edge
is a circle of radius 2 around the corner it wraps and its outside a circle of radius 4. The ground shows in the corners
the rings leave. A leg can be 0 (two bends back to back make an S). Returns the z of the path's top.
"""

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 43217 and 21335. Not derived
from any model file."""

PARTS = ["79393", "3477", "3068b", "87079", "6636", "4162", "3069b", "2431", "3070b"]

# The ring tiles' arcs curve around their front-left corner: at rot=0 a ring bulges toward back right (+x, +y), and
# each quarter turn moves the bulge a quarter on, counterclockwise.
QUARTER = {(1, 1): 0, (-1, 1): 1, (-1, -1): 2, (1, -1): 3}
RINGS = (("79393", 3), ("3477", 4))  # (part, size): radius 2 to 3, and 3 to 4


def build(m, x, y, z, legs=(4, 4), turns="L", color="Tan", title="Path"):
    """The straights and the bends; returns the z of the path's top."""
    if len(turns) != len(legs) - 1 or any(t not in "LR" for t in turns):
        raise ValueError(
            f"curved_garden_path needs one turn, L or R, between each two legs, not {turns!r}"
        )
    if any(n < 0 for n in legs):
        raise ValueError(
            f"curved_garden_path takes legs of 0 or more studs, not {legs}"
        )
    # The path's right-hand edge where the next piece starts, the way it heads, and its left (across the path).
    rx, ry, f, left = x, y, (1, 0), (0, 1)
    for k, n in enumerate(legs):
        if n:
            ex, ey = rx + n * f[0] + 2 * left[0], ry + n * f[1] + 2 * left[1]
            with m.step(f"{title}, straight {k + 1}"):
                m.fill(
                    Tile,
                    color,
                    at=(min(rx, ex), min(ry, ey), z),
                    size=(abs(ex - rx), abs(ey - ry)),
                )
            rx, ry = rx + n * f[0], ry + n * f[1]
        if k == len(turns):
            break
        if turns[k] == "L":
            # The corner it wraps is 4 studs to the left of the right edge; the bend bulges forward and right.
            cx, cy = rx + 4 * left[0], ry + 4 * left[1]
            bulge = (f[0] - left[0], f[1] - left[1])
            f, left = left, (-f[0], -f[1])
            rx, ry = cx - 4 * left[0], cy - 4 * left[1]
        else:
            cx, cy = rx - 2 * left[0], ry - 2 * left[1]
            bulge = (f[0] + left[0], f[1] + left[1])
            f, left = (-left[0], -left[1]), f
            rx, ry = cx + 2 * left[0], cy + 2 * left[1]
        with m.step(f"{title}, bend {k + 1}"):
            for part, size in RINGS:
                at = (
                    cx if bulge[0] > 0 else cx - size,
                    cy if bulge[1] > 0 else cy - size,
                    z,
                )
                m.add(part, color, at=at, rot=QUARTER[bulge])
    return z + 1


def example(m):
    """A path that bends left around a lawn corner and right again, on a green baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("curved_garden_path", x=0, y=1, z=0.5, legs=(4, 4, 2), turns="LR")
