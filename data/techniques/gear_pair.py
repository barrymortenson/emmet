# SPDX-License-Identifier: Apache-2.0
"""Gear pair: two Technic gears that mesh, each on an axle through a pair of Technic bricks, so turning one turns the
other faster, slower, or the other way.

m.use("gear_pair", x, y, z, ratio="24:8", color="Light Bluish Gray", gear_color="Light Bluish Gray",
axle_color="Black", title="Gears") builds, from z, a frame of two Technic bricks with holes (3701 1 x 4, or 3894 1 x 6
for 24:24) along x from (x, y), one stud apart (at y and y + 2), and runs two axles 4L (3705) along y through their
holes, a stud out the front of the frame, with a bush (3713) on each front end to turn it by. The gears sit between
the bricks: the first (the driver, "a" of "a:b") on the hole at x + 1 and the second the right distance along to mesh
with it, turned half a tooth so their teeth interlock. ratio is "24:8" (the second turns 3 times as fast), "8:24",
"16:16", "24:24", or "8:8". The axles are 1.75 plates above z; a 24-tooth gear reaches 2.3 plates below z and a
16-tooth one 0.9, so stand the frame on something that leaves them room. It brings its own steps and returns the z of
the frame's top (z + 3).
"""

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3701", "3894", "3705", "3713", "3648b", "3647", "94925"]

GEARS = {24: "3648b", 16: "94925", 8: "3647"}
# By ratio: the two gears' teeth, and how far apart their axles are, in studs (the sum of their radii).
RATIOS = {
    "24:8": (24, 8, 2),
    "8:24": (8, 24, 2),
    "16:16": (16, 16, 2),
    "24:24": (24, 24, 3),
    "8:8": (8, 8, 1),
}


def _raw(m, part, color, center, x=0, z=0):
    """A part whose origin is its center (gears, axles, bushes) at `center` (DSL), turned x then z degrees."""
    p = frame.to_ldraw_point(center)
    r = frame.to_ldraw_rotation(frame.rotation(x, 0, z))
    return m.add_raw(part, color, [*p, *r[0], *r[1], *r[2]])


def build(
    m,
    x,
    y,
    z,
    ratio="24:8",
    color="Light Bluish Gray",
    gear_color="Light Bluish Gray",
    axle_color="Black",
    title="Gears",
):
    """The frame, axles, gears, and bushes; returns the z of the frame's top."""
    if ratio not in RATIOS:
        raise ValueError(
            f"gear_pair takes a ratio of {', '.join(RATIOS)}, not {ratio!r}"
        )
    a, b, gap = RATIOS[ratio]
    brick = "3894" if gap > 2 else "3701"
    zc = z + 1.75  # the holes' center line
    holes = (x + 1, x + 1 + gap)
    with m.step(f"{title}: frame"):
        for row in (y, y + 2):
            m.add(brick, color, at=(x, row, z), rot=1)
    with m.step(f"{title}: axles and gears"):
        for hx in holes:
            _raw(m, "3705", axle_color, (hx, y + 1, zc))
        # Gears face along y (turned a quarter about z); the second turns half a tooth on its axle to mesh.
        _raw(m, GEARS[a], gear_color, (holes[0], y + 1.5, zc), z=90)
        _raw(m, GEARS[b], gear_color, (holes[1], y + 1.5, zc), x=180 / b, z=90)
    with m.step(f"{title}: bushes"):
        for hx in holes:
            _raw(m, "3713", axle_color, (hx, y - 0.5, zc), z=90)
    return z + 3


def example(m):
    """A 24-tooth gear driving an 8-tooth one, on a frame a brick up."""
    with m.step("Base"):
        m.add("3035", "Dark Bluish Gray", at=(0, 0, 0))
        m.add("3010", "Light Bluish Gray", at=(0, 1, 1), rot=1)
        m.add("3010", "Light Bluish Gray", at=(0, 3, 1), rot=1)
    m.use("gear_pair", x=0, y=1, z=4)
