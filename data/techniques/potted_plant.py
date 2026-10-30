# SPDX-License-Identifier: Apache-2.0
"""Potted plant: a pot with a plant in it, for a doorstep, a path's edge, or a windowsill: a real flower pot with a
leafy plant or a flower on a stem, or a tiny pot of a round 1 x 1 brick with one bloom.

m.use("potted_plant", x, y, z, kind="fern", pot="Dark Orange", leaves="Green", bloom="Red", title="Pot") stands a pot on
the studs at height z (a baseplate is z = 0.5, a sill's top, a step). kind: "fern", a Flower Pot 2 x 2 x 1.333 (7077)
over the 2 x 2 from (x, y), with a plant with three large leaves (6255) on the stud in its floor; "flower", the same pot
with a stem of two round 1 x 1 bricks in `leaves` on that stud, a Plant Flower 2 x 2 Leaves (4727) on top, and a round
plate with tabs in `bloom` as the flower; or "small", a round 1 x 1 brick in `pot` on the stud at (x, y), a round 1 x 1
plate in `leaves`, and a plate 1 x 1 with 5 petals in `bloom`, a windowsill's pot. The flower pot was made only in
Black, Dark Orange, and one other color; the small pot comes in almost any. Returns the z of the plant's top.
"""

from emmet_dsl import frame

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 43217 and 43268. Not derived
from any model file."""

PARTS = ["7077", "6255", "3062b", "4727", "33291", "6141", "24866"]

# Each part's body box in LDU; its origin is the middle of the stud it goes on.
BODIES = {
    "6255": [-46.061, -27.45, -33.194, 45.979, 16.0, 54.95],
    "4727": [-20.0, 0.0, -20.0, 20.0, 24.0, 20.0],
    "33291": [-12.0, 0.0, -12.0, 12.0, 8.0, 12.0],
    "24866": [-9.926, 0.0, -9.5, 9.926, 8.0, 10.043],
    "3062b": [-10.0, 0.0, -10.0, 10.0, 24.0, 10.0],
}
KINDS = ("fern", "flower", "small")


def _on_stud(m, part, color, cx, cy, z):
    """A part on the stud whose middle is (cx, cy), its bottom at z; returns its Handle."""
    lo, _ = frame.box_in_dsl(
        frame.transform_box(
            frame.to_ldraw_rotation(frame.quarter_turns(0)), (0, 0, 0), BODIES[part]
        )
    )
    return m.add(part, color, at=(cx + lo[0], cy + lo[1], z))


def build(
    m, x, y, z, kind="fern", pot="Dark Orange", leaves="Green", bloom="Red", title="Pot"
):
    """The pot and its plant; returns the z of the plant's top."""
    if kind not in KINDS:
        raise ValueError(
            f"potted_plant: kind must be one of {', '.join(KINDS)}, not {kind!r}"
        )
    if kind == "small":
        with m.step(title):
            m.add("3062b", pot, at=(x, y, z))
            m.add("6141", leaves, at=(x, y, z + 3))
            return _on_stud(m, "24866", bloom, x + 0.5, y + 0.5, z + 4).max[2]
    # The pot's one stud is in the middle of its floor, a plate up.
    cx, cy, floor = x + 1, y + 1, z + 1
    with m.step(f"{title}, pot"):
        m.add("7077", pot, at=(x, y, z))
    with m.step(f"{title}, plant"):
        if kind == "fern":
            return _on_stud(m, "6255", leaves, cx, cy, floor).max[2]
        _on_stud(m, "3062b", leaves, cx, cy, floor)
        _on_stud(m, "3062b", leaves, cx, cy, floor + 3)
        _on_stud(m, "4727", leaves, cx, cy, floor + 6)
        return _on_stud(m, "33291", bloom, cx, cy, floor + 9).max[2]


def example(m):
    """The three kinds of pot in a row on a doorstep."""
    with m.step("Step"):
        m.add("3029", "Light Bluish Gray", at=(0, 0, 0), rot=1)
    for k, kind in enumerate(KINDS):
        m.use(
            "potted_plant",
            x=1 + 4 * k,
            y=1,
            z=1,
            kind=kind,
            bloom=("Red", "Yellow", "Bright Pink")[k],
            title=f"Pot {k + 1}",
        )
