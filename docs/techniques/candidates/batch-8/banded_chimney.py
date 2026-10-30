# SPDX-License-Identifier: Apache-2.0
"""Banded chimney: a stout outside chimney against a gable wall, in brick-textured bricks with a darker band every
few courses, stepping in with slopes at the shoulder and rising past the ridge to a cap and two pots.

m.use("banded_chimney", x, y, z, rows=6, above=4, band_every=3, facing="+x", color="Medium Nougat",
band="Dark Orange", cap="Dark Bluish Gray", pots="Reddish Brown", title="Chimney") builds, on the studs at height
z, a chimney whose body is 4 studs along the wall and 2 out from it, its footprint's min corner at (x, y); facing is
the way out from the wall ("+x" or "-x" for a gable end of a roof whose ridge runs along x, "+y" or "-y" otherwise).
The body is `rows` courses of bricks 1 x 2 with embossed bricks (98283), each course turned a quarter from the one
below so they bond, and a plate course in `band` after every `band_every` of them; then the shoulder, four slopes
45 2 x 1 facing along the wall, which step it in to 2 x 2 in the middle; then `above` courses of the stack (put the
shoulder just under the eave, and enough above to clear the ridge by a brick or two), a plate 2 x 4 in `cap`, and two
cones 1 x 1 as pots. A chimney stands on its own footing, so the roof can lift off past it. It brings its own steps,
a course at a time, and returns the z of the pots' top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387, set 21335. Not derived
from any model file."""

PARTS = ["98283", "3020", "3040b", "4589"]

# For each facing: which world axis runs along the wall (u) and which out from it (v), and whether v runs backward.
AXES = {
    "+x": ("y", "x", False),
    "-x": ("y", "x", True),
    "+y": ("x", "y", False),
    "-y": ("x", "y", True),
}
DOWN = {"+x": 0, "+y": 1, "-x": 2, "-y": 3}


def build(
    m,
    x,
    y,
    z,
    rows=6,
    above=4,
    band_every=3,
    facing="+x",
    color="Medium Nougat",
    band="Dark Orange",
    cap="Dark Bluish Gray",
    pots="Reddish Brown",
    title="Chimney",
):
    """The body, the shoulder, the stack, the cap, and the pots; returns the z of the pots' top."""
    if facing not in AXES:
        raise ValueError(f'facing is "+x", "-x", "+y" or "-y", not {facing!r}')
    along, out, back = AXES[facing]

    def add(part, c, u, v, w, su, sv, turn=None):
        """A part covering su studs along the wall from u and sv out from v, at height w."""
        if back:
            v = 2 - v - sv
        if along == "y":
            at, long_on_y = (x + v, y + u, w), su >= sv
        else:
            at, long_on_y = (x + u, y + v, w), sv >= su
        m.add(part, c, at=at, rot=turn if turn is not None else (0 if long_on_y else 1))

    w, done = z, 0
    for r in range(rows):
        with m.step(f"{title}, course {r + 1}"):
            if r % 2 == 0:
                for u in (0, 2):
                    for v in (0, 1):
                        add("98283", color, u, v, w, 2, 1)
            else:
                for u in range(4):
                    add("98283", color, u, 0, w, 1, 2)
        w += 3
        done += 1
        if done % band_every == 0 and r < rows - 1:
            with m.step(f"{title}, band {done // band_every}"):
                add("3020", band, 0, 0, w, 4, 2)
            w += 1
    with m.step(f"{title}: shoulder"):
        # Each slope is 2 deep along the wall, its high stud row over the stack's side of the body.
        low, high = (f"-{along}", f"+{along}")
        for v in (0, 1):
            add("3040b", color, 0, v, w, 2, 1, turn=DOWN[low])
            add("3040b", color, 2, v, w, 2, 1, turn=DOWN[high])
    w += 3
    for r in range(above):
        with m.step(f"{title}, stack {r + 1}"):
            for k in (0, 1):
                if r % 2:
                    add("98283", color, 1 + k, 0, w, 1, 2)
                else:
                    add("98283", color, 1, k, w, 2, 1)
        w += 3
    with m.step(f"{title}: cap and pots"):
        add("3020", cap, 0, 0, w, 4, 2)
        add("4589", pots, 1, 1, w + 1, 1, 1, turn=0)
        add("4589", pots, 2, 1, w + 1, 1, 1, turn=0)
    return w + 4


def example(m):
    """A small house with a gable roof, and the chimney outside its right end wall."""
    from emmet_dsl import Plate

    with m.step("Base"):
        m.fill(Plate, "Dark Bluish Gray", at=(0, 0, 0), size=(8, 8))
    for r in range(3):
        with m.step(f"Walls, row {r + 1}"):
            m.wall(0, 1, 1 + 3 * r, length=6, depth=6, color="White")
    with m.step("Ceiling"):
        m.fill(Plate, "White", at=(0, 1, 10), size=(6, 6))
    m.use("gable_roof", x=0, y=1, z=11, length=6, depth=6, color="Dark Red")
    m.use("banded_chimney", x=6, y=2, z=1, rows=3, above=3, band_every=2, facing="+x")
