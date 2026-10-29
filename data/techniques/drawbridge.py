# SPDX-License-Identifier: Apache-2.0
"""Drawbridge: a plank deck on locking hinges at a gateway's threshold, which lies across the moat or swings up to close
the gate.

m.use("drawbridge", x, y, z, width=4, length=6, color="Reddish Brown", hinge="Dark Bluish Gray", title="Drawbridge")
builds a deck `width` studs wide (even, 2 to 8) and `length` studs long out from a threshold along x from (x, y), its
floor's studs at z, lowered toward the front (-y). At each edge of the threshold a locking hinge plate with two fingers
(44302a) lies on the floor, its fingers out past the front edge, and a hinge plate with one finger (44301a) clicks into
it, lying flat a stud in front of the threshold (the knuckle fills the stud between); the deck's plates go on those, so
the whole deck turns about the threshold's edge. Let the deck's far end rest on tiles, a smooth bank level with its
underside, so it lifts free. Planks (tiles 1 x width) run across the deck. Returns the z of the deck's top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model file."""

PARTS = [
    "44302a",
    "44301a",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
    "2445",
    "91988",
    "4282",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "4477",
    "60479",
    "3069b",
    "2431",
    "6636",
    "4162",
]

PLANKS = {2: "3069b", 4: "2431", 6: "6636", 8: "4162"}  # Tile 1 x n
KNUCKLE = 0.8  # how far the two-finger plate's fingers reach past its body


def build(
    m,
    x,
    y,
    z,
    width=4,
    length=6,
    color="Reddish Brown",
    hinge="Dark Bluish Gray",
    title="Drawbridge",
):
    """The hinges, the deck, and its planks; returns the z of the deck's top."""
    if width not in PLANKS or length < 3:
        raise ValueError(
            f"drawbridge takes an even width from 2 to 8 and a length of 3 or more, not {width}, {length}"
        )
    edges = [0, width - 1]
    with m.step(f"{title}: hinges"):
        for u in edges:
            # The two-finger plate on the threshold, its fingers out front; the one-finger plate clicked into them.
            m.add("44302a", hinge, at=(x + u, y - KNUCKLE, z), rot=2)
            m.add("44301a", hinge, at=(x + u, y - 3, z))
    front = y - 1 - length
    with m.step(f"{title}: deck"):
        m.fill(Plate, color, at=(x, front, z + 1), size=(width, length))
    with m.step(f"{title}: planks"):
        for k in range(length):
            m.add(PLANKS[width], color, at=(x, front + k, z + 2), rot=1)
    return z + 3


def example(m):
    """A gateway's threshold and a drawbridge lowered across a moat onto the far bank."""
    with m.step("Threshold and far bank"):
        m.add("3031", "Dark Bluish Gray", at=(0, 0, 0))  # Plate 4 x 4
        m.add("3710", "Dark Bluish Gray", at=(0, -6, 0), rot=1)
        m.add("2431", "Dark Bluish Gray", at=(0, -6, 1), rot=1)
    m.use("drawbridge", x=0, y=0, z=1, width=4, length=6)
