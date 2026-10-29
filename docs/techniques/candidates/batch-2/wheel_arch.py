# SPDX-License-Identifier: Apache-2.0
"""Wheel arch: a mudguard over a pair of wheels, so the body clears them and the arch shows round over each wheel.

m.use("wheel_arch", x, y, z, color="Red", title="Wheel arches") puts a car mudguard 2 x 4 (3788) with its 2 x 2 middle
over a wheel axle whose plate's footprint is at (x, y + 1), at the z of that plate's top: it runs across the vehicle
(along y), 4 wide, its arches over the wheels each side. The body builds on its studs. Returns the z of its top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["3788"]


def build(m, x, y, z, color="Red", title="Wheel arches"):
    """The mudguard; returns the z of its top."""
    with m.step(title):
        m.add("3788", color, at=(x, y, z), rot=0)
    return z + 2


def _wheels(m, x, y, z):
    """A wheel axle as wheel_axle builds it (4870 and two 4624c01), for the example."""
    ox, oy, oz = 20 * (y + 1), -8 * (z + 1), -20 * (x + 1)
    m.add_raw("4870", "Light Bluish Gray", (ox, oy, oz, 1, 0, 0, 0, 1, 0, 0, 0, 1))
    m.add_raw(
        "4624c01", "White", (ox + 10, oy + 29, oz + 16, 0, 0, -1, 0, 1, 0, 1, 0, 0)
    )
    m.add_raw(
        "4624c01", "White", (ox - 10, oy + 29, oz + 16, 0, 0, 1, 0, 1, 0, -1, 0, 0)
    )


def example(m):
    """A wheel axle with its arches over it."""
    with m.step("Wheels"):
        _wheels(m, 0, 1, 4.875)
    m.use("wheel_arch", x=0, y=0, z=5.875)
