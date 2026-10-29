# SPDX-License-Identifier: Apache-2.0
"""Worm drive: a worm gear on one axle turning a 24-tooth gear on an axle across it, a 24 to 1 reduction that holds its
load when you let go, for winches, cranes, and slow turntables.

m.use("worm_drive", x, y, z, color="Light Bluish Gray", gear_color="Dark Bluish Gray", worm_color="Light Bluish Gray",
axle_color="Black", title="Worm drive") builds a frame on the 4 × 4 studs whose front-left corner is (x, y - 1), from z:
two Technic bricks 1 x 4 (3701) along y at x and x + 3, a plate 1 x 4 along each end row, on the lower bricks' end
studs, and two more Technic bricks 1 x 4 along x on those rows, a brick and two plates up. The input axle (6L, along x)
runs through the lower bricks' middle holes, 1.75 plates above z, with a bush on each end to turn it by and the worm
(4716) on it between the bricks. The output axle (5L, along y) runs through the upper bricks' middle holes, 5 plates
higher, sticking a stud out the back, with the 24-tooth gear on it over the worm, centered by a half bush each side. It
brings its own steps and returns the (x, y, z) of the output axle's back end, where a drum, an arm, or a gear goes on.
"""

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3701", "3710", "3706", "32073", "3713", "32123a", "4716", "3648b"]


def _raw(m, part, color, center, z=0):
    """A part whose origin is its center (axles, gears, bushes) at `center` (DSL), turned z degrees."""
    p = frame.to_ldraw_point(center)
    r = frame.to_ldraw_rotation(frame.rotation(0, 0, z))
    return m.add_raw(part, color, [*p, *r[0], *r[1], *r[2]])


def build(
    m,
    x,
    y,
    z,
    color="Light Bluish Gray",
    gear_color="Dark Bluish Gray",
    worm_color="Light Bluish Gray",
    axle_color="Black",
    title="Worm drive",
):
    """The frame, the worm on its axle, and the gear on the output axle; returns the output axle's back end."""
    low, high = z + 1.75, z + 6.75  # the two axles' heights
    with m.step(f"{title}: lower bricks"):
        m.add("3701", color, at=(x, y - 1, z))
        m.add("3701", color, at=(x + 3, y - 1, z))
    # Axles along x are LDraw's X turned a quarter about z; worms and gears face along their own Z, which is x.
    with m.step(f"{title}: input axle and worm"):
        _raw(m, "3706", axle_color, (x + 2, y + 1, low), z=90)
        _raw(m, "4716", worm_color, (x + 2, y + 1, low))
        _raw(m, "3713", axle_color, (x - 0.5, y + 1, low))
        _raw(m, "3713", axle_color, (x + 4.5, y + 1, low))
    with m.step(f"{title}: plates"):
        for row in (y - 1, y + 2):
            m.add("3710", color, at=(x, row, z + 3), rot=1)
            m.add("3710", color, at=(x, row, z + 4), rot=1)
    with m.step(f"{title}: upper bricks"):
        m.add("3701", color, at=(x, y - 1, z + 5), rot=1)
        m.add("3701", color, at=(x, y + 2, z + 5), rot=1)
    with m.step(f"{title}: output axle and gear"):
        _raw(m, "32073", axle_color, (x + 2, y + 1.5, high))
        _raw(m, "32123a", axle_color, (x + 2, y + 0.25, high), z=90)
        _raw(m, "3648b", gear_color, (x + 2, y + 1, high), z=90)
        _raw(m, "32123a", axle_color, (x + 2, y + 1.75, high), z=90)
    return (x + 2, y + 4, high)


def example(m):
    """A worm drive on a base."""
    with m.step("Base"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))
    m.use("worm_drive", x=0, y=1, z=1)
