# SPDX-License-Identifier: Apache-2.0
"""Disc brake: a display motorbike's or e-bike's brake discs, on the front axle between the hub and each fork slider,
which stands in for the caliper, and one on the rear axle inside the left swingarm arm.

m.use("disc_brake", fork=fork, swingarm=None, disc_color="Light Bluish Gray", both=True, title="Brakes") takes what
`suspension_fork` (and `swingarm_shock`, for the rear disc) returned. A brake disc 6 x 6 (65416) goes on the front axle
against the inside of each slider (the right one only when both is False); its 3-stud radius is inside the wheel's rim,
so it clears the tire. With the swingarm, another goes on the rear axle against the inside of the left arm, across the
wheel from the chain's sprocket. The discs go on after the axles here; in a set's instructions they go on as each axle
goes in. It brings its own step and returns nothing.
"""

PROVENANCE = """Emmet's own (session 4G), written from general building knowledge of Technic motorbikes. Not derived
from any model file."""

PARTS = ["65416"]


def build(
    m, fork, swingarm=None, disc_color="Light Bluish Gray", both=True, title="Brakes"
):
    """The discs, front and rear."""
    x, y, z = fork["front_axle"]
    half = fork["half"]
    with m.step(title):
        for s in (-1, 1)[: 2 if both else 1]:
            m.add("65416", disc_color, at=(x, y + s * (half - 1), z), rot=1, hole=1)
        if swingarm is not None:
            rx, ry, rz = swingarm["rear_axle"]
            m.add("65416", disc_color, at=(rx, ry + 3.5, rz), rot=1, hole=1)


def example(m):
    """Brakes on a bike's front and rear axles."""
    frame = m.use("motorbike_frame", x=12, y=8, z=2.5 * 9.6)
    swingarm = m.use("swingarm_shock", frame=frame)
    fork = m.use("suspension_fork", frame=frame, swingarm=swingarm)
    m.use("disc_brake", fork=fork, swingarm=swingarm)
