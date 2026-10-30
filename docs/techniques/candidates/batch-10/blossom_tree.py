# SPDX-License-Identifier: Apache-2.0
"""Blossom tree: a small flowering tree whose trunk leans as it rises, on jumper plates that step it half a stud at a
time, under a fluffy crown of pink and white leaves in pinwheels, dotted with petal plates; built apart and placed.

m.use("blossom_tree", x, y, z, lean=2, layers=3, blossom=("Bright Pink", "White"), flowers=("Dark Pink", "White"),
trunk="Reddish Brown", title="Blossom tree") stands a tree on the studs at height z (a baseplate is z = 0.5) over the 2
x 2 from (x, y): a round brick 2 x 2 and a jumper 2 x 2 for the foot, then a round 1 x 1 brick, and `lean` times a
jumper 1 x 2 that moves the trunk half a stud toward +x and another round brick on it, so the trunk leans in a curve. On
its top stud, a 2 x 2 plate carries the crown: `layers` pinwheels of four Plant Leaves 4 x 3 (2423), each reaching out a
different way, alternate layers turning the other way, each layer on a round 2 x 2 plate (in the trunk's color, like a
branch) on the studs of the one below, the layers taking `blossom` in turn. On the top plate's four studs, plates 1 x 1
with 5 petals (24866) in `flowers`. The crown spreads about 4 studs round the trunk's top. lean: 0 to 3; layers: 1 to 3.
It's a submodel, with its own steps; returns the placed copy's Handle, whose .max[2] is the top.
"""

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 10315 and 31218. Not derived
from any model file."""

PARTS = ["3941", "87580", "3062b", "3794b", "3022", "2423", "4032b", "24866"]

BODIES = {
    "2423": [-26.0, 0.0, -68.0, 26.0, 8.0, 10.0],
    "24866": [-9.926, 0.0, -9.5, 9.926, 8.0, 10.043],
}
# A layer's four leaves, one on each stud of the plate under it, each reaching out a different way; the next layer
# turns the other way. A leaf reaches toward +x at rot=0, and a quarter on, counterclockwise, with each turn.
LAYERS = [
    [((1, 0), 0), ((1, 1), 1), ((0, 1), 2), ((0, 0), 3)],
    [((1, 1), 0), ((0, 1), 1), ((0, 0), 2), ((1, 0), 3)],
]


def _on_stud(m, part, color, sx, sy, z, rot=0):
    """A part on the stud whose corner is (sx, sy), its bottom at z."""
    turn = frame.to_ldraw_rotation(frame.quarter_turns(rot))
    lo, _ = frame.box_in_dsl(frame.transform_box(turn, (0, 0, 0), BODIES[part]))
    m.add(part, color, at=(sx + 0.5 + lo[0], sy + 0.5 + lo[1], z), rot=rot)


def _tree(m, lean, layers, blossom, flowers, trunk, title):
    """The tree in its own frame, its foot's corner at the origin."""
    with m.step(f"{title}, foot"):
        m.add("3941", trunk, at=(0, 0, 0))
        m.add("87580", trunk, at=(0, 0, 3))
        m.add("3062b", trunk, at=(0.5, 0.5, 4))
    tx, z = 0.5, 7
    with m.step(f"{title}, trunk"):
        for _ in range(lean):
            # A jumper with its left half on the trunk's stud: its own stud, half a stud on, takes the next brick.
            m.add("3794b", trunk, at=(tx, 0.5, z), rot=1)
            tx += 0.5
            m.add("3062b", trunk, at=(tx, 0.5, z + 1))
            z += 4
    # The crown's 2 x 2 plate sits centered on the trunk's top stud.
    cx, cy = tx - 0.5, 0
    with m.step(f"{title}, crown plate"):
        m.add("3022", trunk, at=(cx, cy, z))
    z += 1
    for k in range(layers):
        with m.step(f"{title}, crown layer {k + 1}"):
            if k:
                m.add("4032b", trunk, at=(cx, cy, z))
                z += 1
            for (sx, sy), rot in LAYERS[k % 2]:
                _on_stud(m, "2423", blossom[k % len(blossom)], cx + sx, cy + sy, z, rot)
        z += 1
    with m.step(f"{title}, flowers"):
        m.add("4032b", trunk, at=(cx, cy, z))
        for n, (sx, sy) in enumerate(((0, 0), (1, 0), (1, 1), (0, 1))):
            _on_stud(
                m, "24866", flowers[n % len(flowers)], cx + sx, cy + sy, z + 1, rot=n
            )


def build(
    m,
    x,
    y,
    z,
    lean=2,
    layers=3,
    blossom=("Bright Pink", "White"),
    flowers=("Dark Pink", "White"),
    trunk="Reddish Brown",
    title="Blossom tree",
):
    """The tree; returns the placed copy's Handle."""
    if not 0 <= lean <= 3 or not 1 <= layers <= 3:
        raise ValueError(
            f"blossom_tree takes a lean of 0 to 3 and 1 to 3 layers, not {lean}, {layers}"
        )
    tree = m.submodel(
        "blossom_tree",
        _tree,
        lean=lean,
        layers=layers,
        blossom=tuple(blossom),
        flowers=tuple(flowers),
        trunk=trunk,
        title=title,
    )
    lo = tree.box[0]
    with m.step(title):
        return m.place(tree, at=(x + lo[0], y + lo[1], z + lo[2]))


def example(m):
    """A blossom tree leaning over a baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("blossom_tree", x=5, y=7, z=0.5, lean=2, layers=3)
