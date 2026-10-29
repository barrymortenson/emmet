# SPDX-License-Identifier: Apache-2.0
"""Technic brace: a diagonal beam pinned across the corner of a Technic brick frame, on a 3-4-5 triangle so its end
holes land on the frame's holes, which stops the frame racking.

m.use("technic_brace", x, y, z, color="Light Bluish Gray", beam_color="Light Bluish Gray", pin_color="Dark Bluish
Gray", title="Brace") builds an L of Technic bricks with holes running along y: a base, a Technic brick 1 x 6 (3894)
along x from (x, y, z), and on its left end a post of Technic bricks 1 x 2 with a hole (3700) with two plates under
each, so the post's holes stand 2 and 4 studs over the base's line of holes. A thin beam 6 x 0.5 (32063), its end
holes 5 studs apart, goes across the corner in front (on y - 0.5), pinned (3/4 pins, 32002) to the base's hole 3 studs
from the post and to the post's hole 4 studs up: 3, 4, and 5, so both ends meet holes exactly. It brings its own steps
and returns the z of the post's top (z + 13).
"""

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3894", "3700", "3023b", "32063", "32002"]


def _put(m, part, color, point, r_dsl):
    """Places a part whose origin is the point that matters (a hole's middle, a pin's collar) at `point` (DSL),
    turned by r_dsl (DSL axes)."""
    r = frame.to_ldraw_rotation(r_dsl)
    p = frame.to_ldraw_point(point)
    return m.add_raw(part, color, [*p, *r[0], *r[1], *r[2]])


def build(
    m,
    x,
    y,
    z,
    color="Light Bluish Gray",
    beam_color="Light Bluish Gray",
    pin_color="Dark Bluish Gray",
    title="Brace",
):
    """The base, the post, and the diagonal beam; returns the z of the post's top."""
    line = z + 1.75  # the base's holes
    low, high = (
        (x + 4, line),
        (x + 1, line + 10),
    )  # the brace's two holes: 3 along, 4 studs (10 plates) up
    with m.step(f"{title}: base"):
        m.add("3894", color, at=(x, y, z), rot=1)
    for level in (1, 2):
        with m.step(f"{title}: post, level {level}"):
            zl = z + 5 * level - 2
            m.add("3023b", color, at=(x, y, zl), rot=1)
            m.add("3023b", color, at=(x, y, zl + 1), rot=1)
            m.add("3700", color, at=(x, y, zl + 2), rot=1)
    # The pins: the long side in the frame's hole, the short side toward the front, through the beam.
    out = frame.rotation(0, 0, 180)
    with m.step(f"{title}: pins"):
        for hx, hz in (low, high):
            _put(m, "32002", pin_color, (hx, y, hz), out)
    # The beam: its holes along y (a quarter turn about x), then its length tipped up 53.13° about y, from the low
    # hole back and up to the high one.
    with m.step(f"{title}: brace"):
        mid = ((low[0] + high[0]) / 2, y - 0.25, (low[1] + high[1]) / 2)
        _put(m, "32063", beam_color, mid, frame.rotation(90, 53.13010235415598, 0))
    return z + 13


def example(m):
    """A braced corner on a base."""
    with m.step("Base plate"):
        m.add("3020", "Dark Bluish Gray", at=(0, 0, 0), rot=1)
        m.add("3020", "Dark Bluish Gray", at=(4, 0, 0), rot=1)
    m.use("technic_brace", x=1, y=1, z=1)
