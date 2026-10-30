# SPDX-License-Identifier: Apache-2.0
"""Clamshell pod: a dark round pod with a seat inside, open at the front, whose domed cap lifts on a post to let a
figure in.

m.use("clamshell_pod", x, y, z, lifted=False, color="Black", seat="Dark Bluish Gray", title="Pod") builds a pod 8 x 8
studs from (x, y) on the studs at z: a Plate 8 x 8 Round (74611) as its floor, a minifig seat (4079) at the front
middle facing out, and the back half of the shell, two courses of Brick 4 x 4 Corner Round (48092), the front left
open. A post rises at the back middle, inside the shell: two bricks 1 x 2 when closed, a brick 1 x 2 x 5 (2454b) when
lifted. The cap is a submodel built once: a round plate 8 x 8 with four Brick 4 x 4 x 2 2/3 Corner Round with Dome
Top (49612) on it, a dome 8 plates tall. Closed, it rests on the shell and the post; lifted, it sits on the tall post
9 plates higher, the seat in full view. Returns the z of the cap's top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 75419. Not derived from any
model file."""

PARTS = ["74611", "48092", "49612", "4079", "3004", "2454b"]

QUARTERS = [
    ((4, 4), 0),
    ((0, 4), 1),
    ((0, 0), 2),
    ((4, 0), 3),
]  # min corner from (x, y), and turn: back right first
SHELL = 6  # the shell's height in plates: two courses


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
    """The floor, the seat, the shell, the post, and the cap; returns the z of the cap's top."""
    with m.step(f"{title}, floor and seat"):
        m.add("74611", color, at=(x, y, z))
        m.add("4079", seat, at=(x + 3, y + 1, z + 1), rot=3)
    for r in range(2):
        with m.step(f"{title}, shell course {r + 1}"):
            for (dx, dy), turn in QUARTERS[:2]:
                m.add("48092", color, at=(x + dx, y + dy, z + 1 + 3 * r), rot=turn)
            if not lifted:
                m.add("3004", color, at=(x + 3, y + 5, z + 1 + 3 * r), rot=1)
    if lifted:
        with m.step(f"{title}, post"):
            m.add("2454b", color, at=(x + 3, y + 5, z + 1), rot=1)
    zc = z + 1 + (15 if lifted else SHELL)
    cap = m.submodel("pod_cap", _cap, color=color)
    with m.step(f"{title}, cap"):
        top = m.place(cap, at=(x, y, zc))
    return top.max[2]


def example(m):
    """Two pods on a deck, one closed and one with its cap lifted."""
    from emmet_dsl import Plate

    with m.step("Deck"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(18, 8))
    m.use("clamshell_pod", x=0, y=0, z=1)
    m.use("clamshell_pod", x=10, y=0, z=1, lifted=True, title="Open pod")
