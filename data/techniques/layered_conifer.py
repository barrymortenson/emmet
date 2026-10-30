# SPDX-License-Identifier: Apache-2.0
"""Layered conifer: a tall, narrow evergreen of soft tiers, each a pinwheel of leaf plates on a round trunk plate, every
tier turning the other way from the one below so its leaves fill the gaps; built apart and placed as a submodel.

m.use("layered_conifer", x, y, z, tiers=8, colors=("Dark Green", "Green"), trunk="Reddish Brown", base="Green",
title="Conifer") stands a conifer on the studs at height z (a baseplate is z = 0.5) over the 2 x 2 from (x, y): a 2 x 2
brick in `base` (it fits a lawn socket, or stands on the ground), then `tiers` tiers, each a Plate 2 x 2 Round (4032b)
in the trunk's color with a Plant Leaves 4 x 3 (2423) on each of its studs, each reaching out a different way, and the
next tier's plate on the leaves' own studs. The top two tiers have two leaves, not four, so the tree thins toward its
tip. The tiers take `colors` in turn, so the layers read apart. A round plate and a cone 2 x 2 x 2 make the tip. The
leaves reach about 3.5 studs from the trunk's middle each way, so keep that much clear. tiers: 3 to 9, two plates each.
It's a submodel (one file for each size and colors), with its own steps; returns the placed copy's Handle, whose .max[2]
is the top.
"""

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 10315, 31218, and 11387. Not
derived from any model file."""

PARTS = ["3003", "4032b", "2423", "3942c"]

# Plant Leaves 4 x 3's body box in LDU; its origin is the stud it hangs on, and it has a stud of its own on top there.
LEAVES = {"2423": [-26.0, 0.0, -68.0, 26.0, 8.0, 10.0]}
# A tier's four leaves, one on each stud of its round plate, each reaching out a different way, a pinwheel; the next
# tier's pinwheel turns the other way, so its leaves fill the gaps. A leaf reaches toward +x at rot=0, and a quarter
# on, counterclockwise, with each turn.
TIERS = [
    [((1, 0), 0), ((1, 1), 1), ((0, 1), 2), ((0, 0), 3)],
    [((1, 1), 0), ((0, 1), 1), ((0, 0), 2), ((1, 0), 3)],
]


def _leaf(m, part, color, sx, sy, z, rot):
    """A leaf on the stud whose corner is (sx, sy), its bottom at z, reaching out by rot."""
    turn = frame.to_ldraw_rotation(frame.quarter_turns(rot))
    lo, _ = frame.box_in_dsl(frame.transform_box(turn, (0, 0, 0), LEAVES[part]))
    m.add(part, color, at=(sx + 0.5 + lo[0], sy + 0.5 + lo[1], z), rot=rot)


def _conifer(m, tiers, colors, trunk, base, title):
    """The conifer in its own frame, its base's corner at the origin."""
    with m.step(f"{title}, base"):
        m.add("3003", base, at=(0, 0, 0))
    z = 3
    for t in range(tiers):
        # The top two tiers have two leaves, not four, so the tree thins toward its tip.
        leaves = TIERS[t % 2] if t < tiers - 2 else TIERS[t % 2][::2]
        with m.step(f"{title}, tier {t + 1}"):
            m.add("4032b", trunk, at=(0, 0, z))
            for (sx, sy), rot in leaves:
                _leaf(m, "2423", colors[t % len(colors)], sx, sy, z + 1, rot)
        z += 2
    with m.step(f"{title}, tip"):
        m.add("4032b", trunk, at=(0, 0, z))
        m.add("3942c", colors[0], at=(0, 0, z + 1))


def build(
    m,
    x,
    y,
    z,
    tiers=8,
    colors=("Dark Green", "Green"),
    trunk="Reddish Brown",
    base="Green",
    title="Conifer",
):
    """The conifer; returns the placed copy's Handle."""
    if not 3 <= tiers <= 9:
        raise ValueError(f"layered_conifer takes 3 to 9 tiers, not {tiers}")
    tree = m.submodel(
        "layered_conifer",
        _conifer,
        tiers=tiers,
        colors=tuple(colors),
        trunk=trunk,
        base=base,
        title=title,
    )
    lo = tree.box[0]
    with m.step(title):
        return m.place(tree, at=(x + lo[0], y + lo[1], z + lo[2]))


def example(m):
    """A conifer of eight tiers on a baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("layered_conifer", x=7, y=7, z=0.5, tiers=8)
