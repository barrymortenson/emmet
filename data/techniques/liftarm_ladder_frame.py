# SPDX-License-Identifier: Apache-2.0
"""Liftarm ladder frame: a Technic car's main frame, two long beam rails tied by open frames at the ends and a beam
across the middle, pinned where they cross, laid flat so the suspension, engine, and seats pin into it.

m.use("liftarm_ladder_frame", x, y, z, length=15, color="Black", cross="Light Bluish Gray", pin="Black", title="Frame")
lays, from z, a Technic frame 7 x 5 with an open center (64179) across each end and a beam 7 (32524) across the middle,
all flat with their holes upright and running along y from y to y + 7, then pins friction pins (2780) upright into the
frames' corner holes and the beam's end holes, under the rails, and presses two flat beams `length` holes long (32525,
41239, or 32278 for 11, 13, or 15) on them as the rails, along x from x at y and y + 6. The rails' holes are on x + 0.5,
x + 1.5, …, at y + 0.5 and y + 6.5; each layer is 2.5 plates thick. Returns the z of the rails' top (z + 5).
"""

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 42213, set 42212, set 42202.
Not derived from any model file."""

PARTS = ["64179", "32524", "32525", "41239", "32278", "2780"]

RAILS = {11: "32525", 13: "41239", 15: "32278"}  # Technic beam n
ACROSS = frame.rotation(z=90)  # a flat beam or frame turned a quarter: it runs along y
UPRIGHT = frame.rotation(x=90)  # a pin turned a quarter about x: it stands upright
SPAN = 6  # holes between the rails, the frame 7 x 5's length less one


def _put(m, part, color, point, r=None):
    """Places a part whose origin is its center (a beam, a frame, a pin) at `point` (DSL), turned by r (DSL axes)."""
    r = frame.to_ldraw_rotation(r or frame.rotation())
    return m.add_raw(part, color, [*frame.to_ldraw_point(point), *r[0], *r[1], *r[2]])


def build(
    m,
    x,
    y,
    z,
    length=15,
    color="Black",
    cross="Light Bluish Gray",
    pin="Black",
    title="Frame",
):
    """The cross-members, the pins, and the rails; returns the z of the top."""
    if length not in RAILS:
        raise ValueError(
            f"liftarm_ladder_frame takes rails 11, 13, or 15 long, not {length}"
        )

    def hole(i):
        """The x of a rail's hole i."""
        return x + 0.5 + i

    sides = (y + 0.5, y + 0.5 + SPAN)
    mid_y = y + 0.5 + SPAN / 2
    low, high = z + 1.25, z + 3.75  # the layers' center lines
    # The end frames' centers, in holes; a pin goes in each corner, under the rails.
    ends = (2, length - 3)
    middle = (length - 1) // 2
    with m.step(f"{title}: cross-members"):
        for c in ends:
            _put(m, "64179", cross, (hole(c), mid_y, low), ACROSS)
        _put(m, "32524", cross, (hole(middle), mid_y, low), ACROSS)
    pins = [i for c in ends for i in (c - 2, c + 2)] + [middle]
    for k, side in enumerate(sides):
        with m.step(f"{title}: pins, {'left' if k == 0 else 'right'}"):
            for i in pins:
                _put(m, "2780", pin, (hole(i), side, z + 2.5), UPRIGHT)
    with m.step(f"{title}: rails"):
        for side in sides:
            _put(m, RAILS[length], color, (hole((length - 1) / 2), side, high))
    return z + 5


def example(m):
    """A frame with rails 15 long."""
    m.use("liftarm_ladder_frame", x=0, y=0, z=0)
