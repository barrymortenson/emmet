# SPDX-License-Identifier: Apache-2.0
"""Wheel axle: a pair of wheels on a plate with two wheel pins, built into the bottom layer of a chassis.

m.use("wheel_axle", x, y, z, color="Light Bluish Gray", wheel="White", title="Wheels") puts a plate 2 x 2 with two
wheel pins (4870) with its plate's footprint at (x, y) and its plate's bottom at z, the pins out to the front and back
(along y, across a vehicle that runs along x), and a wheel with its tyre (4624c01) on each pin. The holder and the
wheels hang below the plate: stand the vehicle on the ground with z = 4.875 (or on a baseplate, 0.5 more). The chassis
plate goes on top at z + 1. Returns the z of the plate's top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge; the method is in the instruction
book of set 60440. Not derived from any model file."""

PARTS = ["4870", "4624c01"]

# The wheels on the plate's pins, in the plate's LDraw frame: where each goes and its turn, so its hub's hole (along
# the rim's own z) runs onto the pin (along LDraw x, 29 LDU below the plate's top, 16 LDU off center).
WHEELS = (
    ((10, 29, 16), (0, 0, -1, 0, 1, 0, 1, 0, 0)),
    ((-10, 29, 16), (0, 0, 1, 0, 1, 0, -1, 0, 0)),
)
GROUND = (
    4.875  # the plate's bottom above the ground when the wheels stand on it, in plates
)


def build(m, x, y, z, color="Light Bluish Gray", wheel="White", title="Wheels"):
    """The axle plate and its two wheels; returns the z of the plate's top."""
    # The plate's LDraw origin is the middle of its top face.
    ox, oy, oz = 20 * (y + 1), -8 * (z + 1), -20 * (x + 1)
    with m.step(title):
        m.add_raw("4870", color, (ox, oy, oz, 1, 0, 0, 0, 1, 0, 0, 0, 1))
        for (dx, dy, dz), turn in WHEELS:
            m.add_raw("4624c01", wheel, (ox + dx, oy + dy, oz + dz, *turn))
    return z + 1


def example(m):
    """A small car's chassis: two axles under a 4 x 6 plate, standing on the ground."""
    for k, x in enumerate((0, 4)):
        m.use("wheel_axle", x=x, y=1, z=GROUND, title=f"Wheels {k + 1}")
    with m.step("Chassis"):
        m.add("3032", "Red", at=(0, 0, GROUND + 1), rot=1)
