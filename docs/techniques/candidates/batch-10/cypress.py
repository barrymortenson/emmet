# SPDX-License-Identifier: Apache-2.0
"""Cypress: a slim, upright tree, a dark green flame, of small leaf plates stacked on a single stud, each turned a
quarter from the one below, on a short trunk; built apart and placed as a submodel, so a row of them along a border is
one sub-assembly built "x n".

m.use("cypress", x, y, z, layers=8, color="Dark Green", trunk="Reddish Brown", base="Green", title="Cypress") stands a
cypress on the studs at height z (a baseplate is z = 0.5) over the 2 x 2 from (x, y): a plate 2 x 2 in `base` and a
jumper 2 x 2 that brings a single stud to its middle, a round 1 x 1 brick of trunk, then `layers` Plates 1 x 1 Round
with 3 Leaves (32607), each on a round 1 x 1 plate of stem in the trunk's color and turned a quarter further, so the
leaves spiral round the stem, and a cone 1 x 1 in `base` for the tip (neither part was made in Dark Green). It is about
3 studs wide and 2 x layers + 8 plates tall. layers: 4 to 14. It's a submodel (one file for each size and colors),
placed without a step of its own, so place a row in your own step (m.repeat spaces them out); returns the placed copy's
Handle, whose .max[2] is the top.
"""

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 10315 and 31218. Not derived
from any model file."""

PARTS = ["3022", "87580", "3062b", "32607", "6141", "4589"]

LEAVES = [
    -10.0,
    -3.828,
    -25.8,
    27.6,
    8.0,
    10.0,
]  # Plate 1 x 1 Round with 3 Leaves, in LDU; its origin is its stud


def _leaves(m, color, z, rot):
    """A leaf plate on the stem's stud (the middle of the 2 x 2), its bottom at z."""
    turn = frame.to_ldraw_rotation(frame.quarter_turns(rot))
    lo, _ = frame.box_in_dsl(frame.transform_box(turn, (0, 0, 0), LEAVES))
    m.add("32607", color, at=(1 + lo[0], 1 + lo[1], z), rot=rot)


def _cypress(m, layers, color, trunk, base, title):
    """The cypress in its own frame, its base's corner at the origin."""
    with m.step(f"{title}, base and trunk"):
        m.add("3022", base, at=(0, 0, 0))
        m.add("87580", base, at=(0, 0, 1))
        m.add("3062b", trunk, at=(0.5, 0.5, 2))
    z = 5
    for s in range(0, layers, 3):
        with m.step(f"{title}, leaves {s + 1} to {min(s + 3, layers)}"):
            for k in range(s, min(s + 3, layers)):
                _leaves(m, color, z, k % 4)
                m.add("6141", trunk, at=(0.5, 0.5, z + 1))
                z += 2
    with m.step(f"{title}, tip"):
        m.add("4589", base, at=(0.5, 0.5, z))


def build(
    m,
    x,
    y,
    z,
    layers=8,
    color="Dark Green",
    trunk="Reddish Brown",
    base="Green",
    title="Cypress",
):
    """The cypress; returns the placed copy's Handle."""
    if not 4 <= layers <= 14:
        raise ValueError(f"cypress takes 4 to 14 layers, not {layers}")
    tree = m.submodel(
        "cypress",
        _cypress,
        layers=layers,
        color=color,
        trunk=trunk,
        base=base,
        title=title,
    )
    lo = tree.box[0]
    return m.place(tree, at=(x + lo[0], y + lo[1], z + lo[2]))


def example(m):
    """Three cypresses in a row along the back of a baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    with m.step("Cypresses"):
        m.repeat(lambda i: m.use("cypress", x=2, y=10, z=0.5, layers=8), 3, dx=5)
