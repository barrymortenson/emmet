# SPDX-License-Identifier: Apache-2.0
"""Hanging planter: planters held out from a facade on short black arms, a pot of leaves with a bloom on each.

m.use("hanging_planter", x, y, z, length, at=None, color="Tan", arm="Black", pot="Reddish Brown", leaves="Green",
flowers=("Dark Pink", "White"), title="Planters") lays one plate course of a wall one stud thick and `length` studs long
along x from (x, y), at z, with a Plate 1 x 3 in the arm color at each of `at` (studs from the wall's start; by default
one in the middle), its back stud in the wall and two studs out; plates in the wall color fill the rest. On each arm's
end stud stands a planter: a round 1 x 1 brick in the pot color with a round plate with three leaves (32607) on it and a
bloom (a round plate with tabs) on top, each one of `flowers` in turn. Lay the courses below and above as usual. Returns
the z of the course's top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["3623", "3062b", "32607", "33291", "3024", "3023b", "3710", "3666"]


def build(
    m,
    x,
    y,
    z,
    length,
    at=None,
    color="Tan",
    arm="Black",
    pot="Reddish Brown",
    leaves="Green",
    flowers=("Dark Pink", "White"),
    title="Planters",
):
    """The course with its arms, then the planters on them; returns the z of the course's top."""
    spots = [length // 2] if at is None else sorted(at)
    if (
        not spots
        or spots[0] < 0
        or spots[-1] > length - 1
        or len(set(spots)) < len(spots)
    ):
        raise ValueError(
            f"hanging_planter needs its planters at different studs along the wall, not {at}"
        )
    with m.step(f"{title}: course and arms"):
        u = 0
        for s in spots + [length]:
            if s > u:
                m.row(Plate, color, at=(x + u, y, z), length=s - u)
            if s < length:
                m.add("3623", arm, at=(x + s, y - 2, z))
            u = s + 1
    with m.step(f"{title}: planters"):
        for k, s in enumerate(spots):
            m.add("3062b", pot, at=(x + s, y - 2, z + 1))
            m.add("32607", leaves, at=(x + s, y - 2, z + 4))
            m.add("33291", flowers[k % len(flowers)], at=(x + s - 0.1, y - 2.1, z + 5))
    return z + 1


def example(m):
    """A planter each side of a window, in a short wall."""
    from emmet_dsl import Brick

    with m.step("Wall, row 1"):
        m.row(Brick, "Tan", at=(0, 0, 0), length=6)
    with m.step("Window"):
        f = m.add("60592", "White", at=(2, 0, 3), rot=1)
        (x0, y0, z0), (x1, y1, _) = f.min, f.max
        m.add(
            "60601",
            "Trans-Clear",
            at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
            rot=1,
        )
    for r in range(2):
        with m.step(f"Wall, row {r + 2}"):
            m.add("3004", "Tan", at=(0, 0, 3 + 3 * r), rot=1)
            m.add("3004", "Tan", at=(4, 0, 3 + 3 * r), rot=1)
    z = m.use("hanging_planter", x=0, y=0, z=9, length=6, at=[0, 5])
    with m.step("Wall, row 4"):
        m.row(Brick, "Tan", at=(0, 0, z), length=6)
