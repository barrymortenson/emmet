# SPDX-License-Identifier: Apache-2.0
"""Wall banner: a cloth banner hung down a wall's face from a bar, the bar held in two clips on the wall's top.

m.use("wall_banner", x, y, z, at=0, color="Red", bar="Black", title="Banner") hangs a banner from the top of a wall
along x whose front face is at y and whose top is at z: two plates 1 x 1 with a clip (4081b) on the top course at x + at
and x + at + 3, their clips out front (-y); a bar 4L (30374) laid in both clips along the wall; and a flag 2 x 2 (2335)
turned on its side, so its two clips grip the bar between the plates' clips and the cloth hangs down the face, about two
bricks. The clip plates take a plate's height on the wall top and carry studs, so the next course goes on over them.
Returns the z of the clip plates' top.
"""

from emmet_dsl import Brick, Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["4081b", "30374", "2335"]

ALONG_X = Rot(y=90)  # the bar on its side, along the wall
# The flag turned so its clips are along its top edge and the cloth hangs below.
HANGING = Rot(y=-90)
AXIS_UP = 0.75  # the clips' axis, in plates above the wall top
FLAG_TALL = 6.75  # the flag on its side, in plates, and its clips' axis 0.5 plates below its top


def build(m, x, y, z, at=0, color="Red", bar="Black", title="Banner"):
    """The clips, the bar, and the flag; returns the z of the clip plates' top."""
    if at < 0:
        raise ValueError(f"wall_banner needs `at` of 0 or more, not {at}")
    u = x + at
    with m.step(f"{title}: clips and bar"):
        for k in (0, 3):
            m.add("4081b", bar, at=(u + k, y - 0.9, z - 0.25), rot=3)
        m.add("30374", bar, at=(u, y - 0.7, z + AXIS_UP - 0.5), rot=ALONG_X)
    with m.step(f"{title}: banner"):
        m.add(
            "2335",
            color,
            at=(u + 1, y - 0.83, z + AXIS_UP + 0.5 - FLAG_TALL),
            rot=HANGING,
        )
    return z + 1


def example(m):
    """A wall three bricks tall with a banner hung from its top."""
    for r in range(3):
        with m.step(f"Wall, row {r + 1}"):
            m.row(
                Brick,
                "Light Bluish Gray",
                at=(0, 0, 3 * r),
                length=6,
                first=2 if r % 2 else None,
            )
    m.use("wall_banner", x=0, y=0, z=9, at=1)
