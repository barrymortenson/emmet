# SPDX-License-Identifier: Apache-2.0
"""Ball joint arm: a jointed arm of plates with a ball on one end and a socket on the other, each snapped onto the
last, posed in a curve; a robot arm, a crane jib, a tentacle.

m.use("ball_joint_arm", x, y, z, segments=3, rise=20, bend=20, color="Light Bluish Gray", joint_color="Dark Bluish
Gray", tiles=True, title="Arm") puts a plate 1 x 2 with a ball on its end (22890) on the studs at (x, y, z), lying
along x with the ball past its right end (keep x + 2 to x + 3 clear: the ball reaches half a plate below z), and
chains `segments` plates 1 x 2 with a socket and a ball (14419) out from it, each socket on the ball before. The first
segment rises `rise` degrees from level and each one after bends `bend` degrees further up (negative bends down); a
ball joint holds any pose. Each segment is 3 studs from joint to joint, and with tiles each gets a 1 x 2 tile on top.
segments is 1 to 8. It brings its own steps and returns the (x, y, z) of the last ball's center, where a socket part
(14418, or another 14419) goes on.
"""

import math

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["22890", "14419", "3069b"]

SOCKET, BALL = (30, 4, 0), (-30, 4, 0)  # on a 14419, in its own frame (LDU)


def _mul(a, b):
    return tuple(
        tuple(sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3))
        for i in range(3)
    )


def _put(m, part, color, point, local, r_dsl):
    """Places a part turned by r_dsl (DSL axes) so the point `local` of its own frame (LDU) lands on `point` (DSL)."""
    r = frame.to_ldraw_rotation(r_dsl)
    p = frame.to_ldraw_point(point)
    pos = [p[i] - sum(r[i][k] * local[k] for k in range(3)) for i in range(3)]
    return m.add_raw(part, color, [*pos, *r[0], *r[1], *r[2]])


def build(
    m,
    x,
    y,
    z,
    segments=3,
    rise=20,
    bend=20,
    color="Light Bluish Gray",
    joint_color="Dark Bluish Gray",
    tiles=True,
    title="Arm",
):
    """The base ball plate and the chain of segments; returns the last ball's center."""
    if not 1 <= segments <= 8:
        raise ValueError(f"ball_joint_arm takes 1 to 8 segments, not {segments}")
    if not -90 <= rise <= 90 or not -60 <= bend <= 60:
        raise ValueError(
            f"ball_joint_arm takes a rise of -90 to 90 and a bend of -60 to 60, not {rise}, {bend}"
        )
    with m.step(f"{title}: base"):
        # Its footprint starts at the ball's underside, half a plate below the plate's.
        m.add("22890", joint_color, at=(x, y, z - 0.5), rot=3)
    joint = (x + 2.5, y + 0.5, z + 0.5)  # the base's ball
    for i in range(segments):
        a = rise + i * bend
        # Lying along x (a quarter turn), its socket toward -x, then tipped up `a` degrees about y.
        turn = _mul(frame.rotation(0, -a, 0), frame.rotation(0, 0, 90))
        with m.step(f"{title}: segment {i + 1}"):
            _put(m, "14419", joint_color, joint, SOCKET, turn)
            if tiles:
                # On its studs, a plate (8 LDU) above the segment's own origin, turned the same.
                _put(m, "3069b", color, joint, (SOCKET[0], SOCKET[1] + 8, 0), turn)
        t = math.radians(a)
        joint = (joint[0] + 3 * math.cos(t), joint[1], joint[2] + 7.5 * math.sin(t))
    return joint


def example(m):
    """A three-segment arm curling up from a deck."""
    with m.step("Deck"):
        m.add("3020", "Light Bluish Gray", at=(0, 0, 0), rot=1)
    m.use("ball_joint_arm", x=2, y=0, z=1, segments=3, rise=15, bend=25)
