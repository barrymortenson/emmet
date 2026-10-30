# SPDX-License-Identifier: Apache-2.0
"""Tapered hood wedges: a vintage car's long hood that narrows toward the grille, two wing plates laid side by side
with their slanted edges outside, louvers down the middle.

m.use("tapered_hood_wedges", x, y, z, length=6, color="Dark Red", louvers="Black", title="Hood") lays a left and a
right wing plate 3 x `length` (6: 54384 and 54383; 12: 47397 and 47398) side by side on the studs at z, from x back,
y to y + 6, their slanted edges outside, so the hood is the full 6 studs wide at the cowl (x + length) and narrows to
about 2 at the grille (x); on the two studs down the middle, a run of 1 x 2 grille tiles across as the louvers, from a
stud behind the front to the cowl. It sits on a narrow chassis: anything carrying the middle studs holds it.
Returns the z of the wings' top.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction book of set 43277. Not derived from any
model file."""

PARTS = ["54384", "54383", "47397", "47398", "2412b"]

# Wing 3 x n, left and right, and how far in from y the left one's corner goes to put its studs on the grid (the
# 3 x 6 left wing's body is 2.95 studs across).
WINGS = {6: ("54384", "54383", 0.05), 12: ("47397", "47398", 0)}


def build(m, x, y, z, length=6, color="Dark Red", louvers="Black", title="Hood"):
    """The two wings and the louvers; returns the z of the wings' top."""
    if length not in WINGS:
        raise ValueError(f"tapered_hood_wedges takes a length of 6 or 12, not {length}")
    left, right, shift = WINGS[length]
    with m.step(f"{title}: wings"):
        # Turned end for end, the left wing goes on the left (-y) with its slant outside.
        m.add(left, color, at=(x, y + shift, z), rot=2)
        m.add(right, color, at=(x, y + 3, z), rot=2)
    for k, s in enumerate(range(1, length, 6)):
        with m.step(f"{title}: louvers {k + 1}"):
            for a in range(s, min(s + 6, length)):
                m.add("2412b", louvers, at=(x + a, y + 2, z + 1), rot=0)
    return z + 1


def example(m):
    """A hood on a vintage car's narrow chassis, with the cowl behind it."""
    with m.step("Chassis"):
        m.add("3832", "Black", at=(0, 2, 0), rot=1)
    m.use("tapered_hood_wedges", x=0, y=0, z=1)
    with m.step("Cowl"):
        m.add("3795", "Dark Red", at=(6, 0, 1), rot=0)
        m.add("3009", "Dark Red", at=(7, 0, 2), rot=0)
