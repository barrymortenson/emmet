# SPDX-License-Identifier: Apache-2.0
"""Clamshell pod: a dark round pod with a seat inside, open at the front, whose domed cap lifts on a post to let a
figure in.

m.use("clamshell_pod", x, y, z, lifted=False, color="Black", seat="Dark Bluish Gray", title="Pod") builds a pod 8 x 8
studs from (x, y) on the studs at z: a Plate 8 x 8 Round (74611) as its floor, a minifig seat (4079) at the front
middle facing out, and the back half of the shell, three courses of Brick 4 x 4 Corner Round (48092) under two Plate 4
x 4 Corner Round (30565), the front left open. The cap is a submodel built once: a round plate 8 x 8 with four Brick 4
x 4 x 2 2/3 Corner Round with Dome Top (49612) on it, a dome 8 plates tall. Closed, it rests on the shell, 9 plates
over the seat, so a seated figure fits under it (docs/spec/building-rules.md, rule 3); lifted, it sits 9 plates higher
on a post of three bricks 1 x 2 at the back middle, the seat in full view. The pod keeps clear the room the seated
figure needs: a stud at each end of the seat, 2 studs in front, and 3 bricks over it. Returns the z of the cap's top,
with .underside (the z of the cap's bottom) and .seat (the seat's (x, y) and the z of its top).
"""

from emmet_dsl.core import fit, seat_space

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 75419. Not derived from any
model file."""

PARTS = ["74611", "48092", "30565", "49612", "4079", "3004"]

QUARTERS = [
    ((4, 4), 0),
    ((0, 4), 1),
    ((0, 0), 2),
    ((4, 0), 3),
]  # min corner from (x, y), and turn: back right first
SHELL = 10  # the shell's height in plates: three courses and a plate, 9 over the seat
SEAT = 1  # the seat's top, in plates over its bottom


def _cap(m, color):
    """The domed cap: a round plate and four dome quarters, its corner at the origin."""
    with m.step("Plate"):
        m.add("74611", color, at=(0, 0, 0))
    with m.step("Dome"):
        for (dx, dy), turn in QUARTERS:
            m.add("49612", color, at=(dx, dy, 1), rot=turn)


def build(
    m, x, y, z, lifted=False, color="Black", seat="Dark Bluish Gray", title="Pod"
):
    """The floor, the seat, the shell, the post, and the cap; returns the z of the cap's top, with .underside and
    .seat."""
    with m.step(f"{title}, floor and seat"):
        m.add("74611", color, at=(x, y, z))
        m.add("4079", seat, at=(x + 3, y + 1, z + 1), rot=3)
    seat_space(m, x + 3, y + 1, 2, 2, SEAT, "a figure sitting in the pod", z=z + 1)
    for r in range(3):
        with m.step(f"{title}, shell course {r + 1}"):
            for (dx, dy), turn in QUARTERS[:2]:
                m.add("48092", color, at=(x + dx, y + dy, z + 1 + 3 * r), rot=turn)
    with m.step(f"{title}, shell top"):
        for (dx, dy), turn in QUARTERS[:2]:
            m.add("30565", color, at=(x + dx, y + dy, z + 1 + 9), rot=turn)
    zc = z + 1 + SHELL
    if lifted:
        with m.step(f"{title}, post"):
            m.stack("3004", color, at=(x + 3, y + 5, zc), count=3, rot=1)
        zc += 9
    cap = m.submodel("pod_cap", _cap, color=color)
    with m.step(f"{title}, cap"):
        top = m.place(cap, at=(x, y, zc))
    return fit(top.max[2], underside=zc, seat=(x + 3, y + 1, z + 1 + SEAT))


def example(m):
    """Two pods on a deck, one closed and one with its cap lifted."""
    from emmet_dsl import Plate

    with m.step("Deck"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(18, 8))
    m.use("clamshell_pod", x=0, y=0, z=1)
    m.use("clamshell_pod", x=10, y=0, z=1, lifted=True, title="Open pod")
