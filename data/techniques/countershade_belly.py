# SPDX-License-Identifier: Apache-2.0
"""Countershade belly: an animal's body with a dark back and a pale belly, built in layers of plates with wedge plates
at the nose, the color changing on a layer whose white edge stands out under the dark ones as a clean pale line.

m.use("countershade_belly", x, y, z, length=10, back="Dark Bluish Gray", belly="White", title="Body") builds, on the
studs at height z, a body 6 studs wide and `length` long (8 to 16) from (x, y), its pointed nose at the front (-y), in
six layers of plates: from the bottom, a belly layer 2 studs wide, one 4 wide, and one 6 wide, then three in the back
color 4, 4, and 2 wide, each centered. A layer 4 or 6 wide has a pair of wing plates (2 x 4 or 3 x 6, left and right)
meeting in a point at the nose and plates behind them; a 2-wide layer starts 2 studs back. The widest belly layer stands
a stud out from the dark layer over it on each side, so a white line runs along the flank under the dark back, slanting
in along the wing plates' edges to the nose; it is what makes the body read as a fish or a shark. The back's top (2
wide) keeps its studs for a dorsal fin. It brings its own steps, a layer at a time, and returns the z of the top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21350 and 75936. Not derived
from any model file."""

PARTS = [
    "41769a",
    "41770a",
    "54383",
    "54384",
    "3020",
    "3795",
    "3034",
    "3832",
    "3021",
    "3022",
    "3023b",
    "3710",
]

# The layers, bottom up: (width, which color).
LAYERS = [
    (2, "belly"),
    (4, "belly"),
    (6, "belly"),
    (4, "back"),
    (4, "back"),
    (2, "back"),
]
# By width: the wing plates for the nose (the one on the left, the one on the right, each cut on its outer side when
# it points to -y) and how long they are; a quarter turn of 3 points them to -y.
# The 3 x 6 is 2.95 studs across, its studs lined up with its square side: it sits 0.05 in from the left.
NOSE = {4: ("41769a", "41770a", 4, 0), 6: ("54383", "54384", 6, 0.05)}


def build(m, x, y, z, length=10, back="Dark Bluish Gray", belly="White", title="Body"):
    """The six layers; returns the z of the top."""
    if not 8 <= length <= 16:
        raise ValueError(f"countershade_belly takes a length of 8 to 16, not {length}")
    for k, (w, part) in enumerate(LAYERS):
        color = belly if part == "belly" else back
        x0 = x + (6 - w) / 2
        zk = z + k
        with m.step(f"{title}: layer {k + 1}, {part}"):
            if w in NOSE:
                left, right, n, nudge = NOSE[w]
                half = w // 2
                m.add(left, color, at=(x0 + nudge, y, zk), rot=3)
                m.add(right, color, at=(x0 + half, y, zk), rot=3)
                m.fill(Plate, color, at=(x0, y + n, zk), size=(w, length - n))
            else:
                m.fill(Plate, color, at=(x0, y + 2, zk), size=(w, length - 2))
    return z + len(LAYERS)


def example(m):
    """A fish body 10 studs long on a post."""
    with m.step("Stand"):
        m.add("3031", "Dark Bluish Gray", at=(1, 3, 0))  # Plate 4 x 4
        m.stack("3062b", "Dark Bluish Gray", at=(2, 4, 1), count=2)  # Round brick 1 x 1
        m.stack("3062b", "Dark Bluish Gray", at=(3, 4, 1), count=2)
    m.use("countershade_belly", x=0, y=0, z=7, length=10)
