# SPDX-License-Identifier: Apache-2.0
"""Wall lamp bracket: small lanterns hung from short arms that stick out of one of a wall's plate courses, either side
of a door.

m.use("wall_lamp_bracket", x, y, z, length, at=None, color="Tan", arm="Black", glass="Trans-Yellow", title="Lamps") lays
one plate course of a wall one stud thick and `length` studs long along x from (x, y), at z, with a Plate 1 x 2 in the
arm color at each of `at` (studs from the wall's start; by default one near each end), its back stud in the wall and its
front stud a stud out; plates in the wall color fill the rest. Under each arm's front stud hangs a lantern: a round 1 x
1 plate as the cap, a round 1 x 1 brick in the glass color, and a round 1 x 1 plate and a cone under it, each held by
the stud of the one below in its underside. Lay the courses below and above as usual; a lamp hangs 6 plates below the
course, a stud in front of the wall. Returns the z of the course's top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350, set 43302. Not derived
from any model file."""

PARTS = ["3023b", "6141", "3062b", "4589", "3024", "3623", "3710", "3666"]


def build(
    m,
    x,
    y,
    z,
    length,
    at=None,
    color="Tan",
    arm="Black",
    glass="Trans-Yellow",
    title="Lamps",
):
    """The course with its arms, then the lanterns under them; returns the z of the course's top."""
    spots = [0, length - 1] if at is None else sorted(at)
    if (
        not spots
        or spots[0] < 0
        or spots[-1] > length - 1
        or len(set(spots)) < len(spots)
    ):
        raise ValueError(
            f"wall_lamp_bracket needs its lamps at different studs along the wall, not {at}"
        )
    with m.step(f"{title}: course and arms"):
        u = 0
        for s in spots + [length]:
            if s > u:
                m.row(Plate, color, at=(x + u, y, z), length=s - u)
            if s < length:
                m.add("3023b", arm, at=(x + s, y - 1, z))
            u = s + 1
    with m.step(f"{title}: lanterns"):
        for s in spots:
            m.add("6141", arm, at=(x + s, y - 1, z - 1))
            m.add("3062b", glass, at=(x + s, y - 1, z - 4))
            m.add("6141", arm, at=(x + s, y - 1, z - 5))
            m.add("4589", arm, at=(x + s, y - 1, z - 7))
    return z + 1


def example(m):
    """A door with a lamp each side, in a short wall."""
    from emmet_dsl import Brick

    with m.step("Door"):
        frame = m.add("60596", "White", at=(1, 0, 0), rot=3)
        (x0, y0, z0), (x1, y1, _) = frame.min, frame.max
        m.add(
            "60623",
            "Dark Green",
            at=((x0 + x1 - 3.55) / 2, (y0 + y1 - 0.3) / 2, z0 + 0.875),
            rot=3,
        )
    for r in range(6):
        with m.step(f"Wall, row {r + 1}"):
            for cx in (0, 5):
                m.add("3005", "Tan", at=(cx, 0, 3 * r))
    z = m.use("wall_lamp_bracket", x=0, y=0, z=18, length=6, at=[0, 5])
    with m.step("Wall over the door"):
        m.row(Brick, "Tan", at=(0, 0, z), length=6)
