# SPDX-License-Identifier: Apache-2.0
"""Off-road fender flare: a chunky arch over each wheel that stands half a stud proud of a boxy body's side, set out
on jumper plates.

m.use("offroad_fender_flare", x, y, z, width=8, color="Black", title="Fender flares") puts a flare over a wheel on
each side of a body `width` studs wide (y to y + width), on its top at z, x to x + 4 along the wheel: at each end a
1 x 2 jumper plate laid across the body's edge, so it reaches a stud past the side, and on the two jumpers' middle
studs an arch 1 x 4 (3659) with a 1 x 4 tile on top, so the arch stands half a stud outside the body's side, over
the wheel's top. Put it over the wheel's center (x + 2 over the axle). Returns the z of the tiles' top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 77264. Not derived from any
model file."""

PARTS = ["3794b", "3659", "2431"]


def build(m, x, y, z, width=8, color="Black", title="Fender flares"):
    """The jumpers, the arches, and their tiles; returns the z of the tiles' top."""
    sides = (
        (y - 1, y - 0.5),
        (y + width - 1, y + width - 0.5),
    )  # each side's jumpers and arch
    with m.step(f"{title}: jumpers"):
        for jy, _ in sides:
            for jx in (x, x + 3):
                m.add("3794b", color, at=(jx, jy, z), rot=0)
    with m.step(f"{title}: arches"):
        for _, ay in sides:
            m.add("3659", color, at=(x, ay, z + 1), rot=1)
            m.add("2431", color, at=(x, ay, z + 4), rot=1)
    return z + 5


def example(m):
    """An off-roader's front axle: balloon tires on an axle 8 through Technic bricks, risers to a high body, and the
    flares on the body over the wheels."""
    with m.step("Spine"):
        m.add("3035", "Dark Bluish Gray", at=(-3, 2, 2), rot=1)
    with m.step("Axle"):
        m.add("32064b", "Dark Bluish Gray", at=(0, 3, 3), rot=1)
        m.add("32064b", "Dark Bluish Gray", at=(0, 4, 3), rot=1)
        m.add("3707", "Black", at=(0.7, 0.025, 4), rot=0)
    with m.step("Wheels"):
        for cy, turn in (
            (0.85, (0, 0, 1, 0, 1, 0, -1, 0, 0)),
            (7.15, (0, 0, -1, 0, 1, 0, 1, 0, 0)),
        ):
            pos = (20 * cy, -8 * 4.75, -20 * 1)
            m.add_raw("55982", "Light Bluish Gray", (*pos, *turn))
            m.add_raw("58090", "Black", (*pos, *turn))
    with m.step("Risers"):
        m.add("3003", "Dark Bluish Gray", at=(0, 3, 6))
        m.add("3022", "Dark Bluish Gray", at=(0, 3, 9))
    with m.step("Body"):
        m.add("41539", "Sand Green", at=(-3, 0, 10))
    m.use("offroad_fender_flare", x=-1, y=0, z=11)
