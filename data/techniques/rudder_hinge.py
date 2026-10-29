# SPDX-License-Identifier: Apache-2.0
"""Rudder hinge: a rudder that swings on a vertical bar held by two clip bricks laid in the stern's courses.

m.use("rudder_hinge", x, y, z, fitting="Black", rudder="Reddish Brown", title="Rudder") puts a Brick 1 x 1 with Clip
Vertical (60475b), in the fitting color, at (x, y) in the stern's course at z and another in the course over it, their
clips aft (-x), for a stern wall whose face is at x: lay the rest of those two courses around them after. A bar 4L
(30374) runs down through both clips as the rudder post, and a flag 2 x 2 (2335) clipped to its lower end is the blade:
it reaches 4 plates below the course at z, 2.5 studs aft, and turns about the post to steer. Returns the z of the upper
course's top.
"""

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = ["60475b", "30374", "2335"]

CLIP_OUT = 0.7  # how far a clip brick's clip reaches past its body


def build(m, x, y, z, fitting="Black", rudder="Reddish Brown", title="Rudder"):
    """The two clip bricks, the post, and the blade; returns the z of the upper course's top."""
    with m.step(f"{title}: clips"):
        for k in range(2):
            m.add("60475b", fitting, at=(x - CLIP_OUT, y, z + 3 * k), rot=2)
    # The clips' axis, and the post's foot.
    post_x, post_y, low = x - 0.5, y + 0.5, z - 4
    with m.step(f"{title}: post and blade"):
        m.add("30374", rudder, at=(post_x - 0.2, post_y - 0.2, low))
        m.add("2335", rudder, at=(post_x - 2.5, post_y - 0.33, low))
    return z + 6


def example(m):
    """The stern of a hull, four courses tall, with a rudder."""
    for r in range(2):
        with m.step(f"Stern, row {r + 1}"):
            m.add("3009", "Dark Red", at=(0, -2, 3 * r))  # Brick 1 x 6, along y
    m.use("rudder_hinge", x=0, y=1, z=6)
    for r in range(2, 4):
        with m.step(f"Stern, row {r + 1}"):
            m.add("3622", "Dark Red", at=(0, -2, 3 * r))
            m.add("3004", "Dark Red", at=(0, 2, 3 * r))
