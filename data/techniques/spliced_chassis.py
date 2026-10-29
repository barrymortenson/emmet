# SPDX-License-Identifier: Apache-2.0
"""Spliced chassis: a long chassis from two plates butted end to end, locked by plates across the seam.

m.use("spliced_chassis", x, y, z, length=24, width=6, color="Dark Bluish Gray", title="Chassis") lays two plates
`width` wide end to end along x from (x, y, z), `length` studs in all, and across the seam a second layer of plates
4 studs long (2 each side of it), so the joint can't bend. Returns the z of the top of the bottom layer (the splice
plates sit on it at the seam).
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 60440. Not derived from any model file."""

PARTS = ["3027", "3028", "3029", "3033", "3036", "3958", "3020"]

LONG = {
    (6, 12): "3028",
    (6, 10): "3033",
    (6, 16): "3027",
    (6, 8): "3036",
    (6, 6): "3958",
    (4, 12): "3029",
}


def build(m, x, y, z, length=24, width=6, color="Dark Bluish Gray", title="Chassis"):
    """The two long plates and the splice; returns the z of the bottom layer's top."""
    half = length // 2
    if (width, half) not in LONG or length % 2:
        raise ValueError(
            f"spliced_chassis takes 6 x 12, 16, 20, 24, or 32, or 4 x 24; not {width} x {length}"
        )
    with m.step(f"{title}: plates"):
        m.add(LONG[(width, half)], color, at=(x, y, z), rot=1)
        m.add(LONG[(width, half)], color, at=(x + half, y, z), rot=1)
    with m.step(f"{title}: splice"):
        for b in range(0, width, 2):
            m.add("3020", color, at=(x + half - 2, y + b, z + 1), rot=1)
    return z + 1


def example(m):
    """A 6 x 24 chassis."""
    m.use("spliced_chassis", x=0, y=0, z=0)
