# SPDX-License-Identifier: Apache-2.0
"""Technic brick ring chassis: a large car's floor frame, a closed ring of Technic bricks two courses high with its
corners lapped course over course, locked by plates across its ends, top and bottom, that axles and pins go through.

m.use("technic_brick_ring_chassis", x, y, z, length=16, width=8, color="Light Bluish Gray", plate="Dark Bluish Gray",
title="Chassis") lays a plate 2 x `width` across each end of a `length` x `width` rectangle whose front-left corner is
(x, y), at z, then two courses of Technic bricks with holes on them: in the first the two sides run the full length and
the ends fit between them, in the second the ends run the full width and the sides fit between, so every corner is
lapped and the joints along the sides fall a stud apart. Plates 2 x `width` across the top, at the ends and every 6
studs or so, lock it. The sides' holes run along y (for wheel pins and axles) on the stud lines, 2.75 and 5.75 plates
above z; the ends' holes run along x. length and width are even, length 8 to 32, width 6 to 16. Returns the z of the top
of the tie plates.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10300, and the instruction
book 6670152. Not derived from any model file."""

PARTS = [
    "3700",
    "3701",
    "3894",
    "3702",
    "2730",
    "3895",
    "32018",
    "3703",
    "3795",
    "3034",
    "3832",
    "2445",
    "4282",
]

BRICKS = {
    16: "3703",
    14: "32018",
    12: "3895",
    10: "2730",
    8: "3702",
    6: "3894",
    4: "3701",
    2: "3700",
}  # Technic brick 1 x n with holes


def _run(m, color, at, n, axis):
    """A run of Technic bricks `n` studs long from `at` along axis, the longest first."""
    x, y, z = at
    while n:
        k = max(b for b in BRICKS if b <= n)
        m.add(BRICKS[k], color, at=(x, y, z), rot=1 if axis == "x" else 0)
        if axis == "x":
            x += k
        else:
            y += k
        n -= k


def build(
    m,
    x,
    y,
    z,
    length=16,
    width=8,
    color="Light Bluish Gray",
    plate="Dark Bluish Gray",
    title="Chassis",
):
    """The bottom plates, the two courses, and the tie plates; returns the z of the top."""
    if length % 2 or not 8 <= length <= 32:
        raise ValueError(
            f"technic_brick_ring_chassis takes an even length from 8 to 32, not {length}"
        )
    if width % 2 or not 6 <= width <= 16:
        raise ValueError(
            f"technic_brick_ring_chassis takes an even width from 6 to 16, not {width}"
        )
    far, back = x + length - 1, y + width - 1
    with m.step(f"{title}: bottom plates"):
        for px in (x, x + length - 2):
            m.add(Plate(2, width), plate, at=(px, y, z))
    z1, z2 = z + 1, z + 4
    with m.step(f"{title}, course 1: sides"):
        for side in (y, back):
            _run(m, color, (x, side, z1), length, "x")
    with m.step(f"{title}, course 1: ends"):
        for end in (x, far):
            _run(m, color, (end, y + 1, z1), width - 2, "y")
    with m.step(f"{title}, course 2: ends"):
        for end in (x, far):
            _run(m, color, (end, y, z2), width, "y")
    with m.step(f"{title}, course 2: sides"):
        for side in (y, back):
            _run(m, color, (x + 1, side, z2), length - 2, "x")
    # Tie plates at both ends and evenly between, about 6 studs apart.
    gaps = max(1, round((length - 2) / 6))
    ties = sorted({x + round(k * (length - 2) / gaps) for k in range(gaps + 1)})
    top = z2 + 3
    with m.step(f"{title}: tie plates"):
        for px in ties:
            m.add(Plate(2, width), plate, at=(px, y, top))
    return top + 1


def example(m):
    """A ring 20 studs long and 10 wide, the frame of a large display car."""
    m.use("technic_brick_ring_chassis", x=0, y=0, z=0, length=20, width=10)
