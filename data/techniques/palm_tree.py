# SPDX-License-Identifier: Apache-2.0
"""Palm tree: a tall trunk of round bricks that leans as it rises, on jumper plates, under a head of drooping fronds of
curved slopes in two pinwheels, with coconuts; built apart and placed as a submodel.

m.use("palm_tree", x, y, z, height=6, lean=2, fronds="Green", trunk="Reddish Brown", coconuts="Reddish Brown",
title="Palm") stands a palm on the studs at height z (a baseplate is z = 0.5, or sand) over the 2 x 2 from (x, y): a
round brick 2 x 2 and a jumper 2 x 2 for the foot, then `height` round 1 x 1 bricks, the last `lean` of them each on a
jumper 1 x 2 that moves the trunk half a stud toward +x, so the top of the trunk bends over. On the top stud, a plate 6
x 6 is the lower head: four Slope Brick Curved 3 x 1 (50950), high end in, each reaching out a different way, a pinwheel
of fronds that droop a stud past its edge; four round 1 x 1 plates between them are coconuts, and a round brick 2 x 2 in
the middle carries a plate 4 x 4 with a second, smaller pinwheel. The fronds reach 4 studs from the trunk's top. height:
2 to 8; lean: 0 to 3 and no more than height - 1. It's a submodel, with its own steps; returns the placed copy's Handle,
whose .max[2] is the top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 31109 and 43268. Not derived
from any model file."""

PARTS = ["3941", "87580", "3062b", "3794b", "3958", "3031", "50950", "6141"]

# A curved slope faces +x at rot=0, its low end there, and a quarter on, counterclockwise, with each turn. It holds by
# its lower studs' worth (its high end has none under it), so each frond's high end and middle sit on the head's
# plate and its low end hangs a stud past the edge: (the high end's stud, rot). The lower head is a 6 x 6 plate, its
# middle 2 x 2 left for the column that carries the upper head, a 4 x 4 plate.
LOWER = [((4, 2), 0), ((3, 4), 1), ((1, 3), 2), ((2, 1), 3)]
UPPER = [((2, 1), 0), ((2, 2), 1), ((1, 2), 2), ((1, 1), 3)]
COCONUTS = [(1, 1), (4, 1), (4, 4), (1, 4)]


def _frond(m, color, hx, hy, z, rot):
    """A Slope Brick Curved 3 x 1 whose high end is on the stud at (hx, hy), facing by rot."""
    back = {0: (0, 0), 1: (0, 0), 2: (-2, 0), 3: (0, -2)}[rot]
    m.add("50950", color, at=(hx + back[0], hy + back[1], z), rot=rot)


def _palm(m, height, lean, fronds, trunk, coconuts, title):
    """The palm in its own frame, its foot's corner at the origin."""
    with m.step(f"{title}, foot"):
        m.add("3941", trunk, at=(0, 0, 0))
        m.add("87580", trunk, at=(0, 0, 3))
    tx, z = 0.5, 4
    with m.step(f"{title}, trunk"):
        for k in range(height):
            if k >= height - lean:
                # A jumper with its left half on the trunk's stud: its own stud, half a stud on, takes the next brick.
                m.add("3794b", trunk, at=(tx, 0.5, z), rot=1)
                tx, z = tx + 0.5, z + 1
            m.add("3062b", trunk, at=(tx, 0.5, z))
            z += 3
    # The lower head's 6 x 6 plate sits centered on the trunk's top stud.
    hx, hy = tx - 2.5, -2
    with m.step(f"{title}, lower fronds"):
        m.add("3958", fronds, at=(hx, hy, z))
        for (sx, sy), rot in LOWER:
            _frond(m, fronds, hx + sx, hy + sy, z + 1, rot)
    with m.step(f"{title}, coconuts"):
        for sx, sy in COCONUTS:
            m.add("6141", coconuts, at=(hx + sx, hy + sy, z + 1))
        m.add("3941", trunk, at=(hx + 2, hy + 2, z + 1))
    with m.step(f"{title}, upper fronds"):
        m.add("3031", fronds, at=(hx + 1, hy + 1, z + 4))
        for (sx, sy), rot in UPPER:
            _frond(m, fronds, hx + 1 + sx, hy + 1 + sy, z + 5, rot)


def build(
    m,
    x,
    y,
    z,
    height=6,
    lean=2,
    fronds="Green",
    trunk="Reddish Brown",
    coconuts="Reddish Brown",
    title="Palm",
):
    """The palm; returns the placed copy's Handle."""
    if not 2 <= height <= 8 or not 0 <= lean <= min(3, height - 1):
        raise ValueError(
            f"palm_tree takes a height of 2 to 8 and a lean of 0 to 3 (under the height), not {height}, "
            f"{lean}"
        )
    palm = m.submodel(
        "palm_tree",
        _palm,
        height=height,
        lean=lean,
        fronds=fronds,
        trunk=trunk,
        coconuts=coconuts,
        title=title,
    )
    lo = palm.box[0]
    with m.step(title):
        return m.place(palm, at=(x + lo[0], y + lo[1], z + lo[2]))


def example(m):
    """A leaning palm on a patch of sand on a baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    with m.step("Sand"):
        m.add("3031", "Tan", at=(4, 6, 0.5))
    m.use("palm_tree", x=5, y=7, z=1.5, height=6, lean=2)
