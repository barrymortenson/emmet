# SPDX-License-Identifier: Apache-2.0
"""Timber frame panels: white plaster panels crossed by dark posts and diagonal braces, each brace the dark strip left
between two triangular tiles.

m.use("timber_frame_panels", x, y, z, bays=3, plaster="White", timber="Black", title="Timber frame") builds a band 5
plates tall along x from (x, y, z), facing front (-y): a post, then a 3-stud bay, then a post, `bays` times, on bricks
with studs on one side (32952, 22885). Each post is a 1 x 2 tile in the timber color on its brick's side studs. In each
bay, 1 x 1 tiles and a 1 x 2 tile in the timber color face the side studs, except under the square corners of two
2 x 2 triangular tiles (35787) in the plaster color, where 1 x 1 plates give them a stud. The triangles sit in opposite
corners of the bay, a stud apart, so between their long edges runs a diagonal strip of the timber color a stud wide: a
brace. The braces lean one way, then the other, bay to bay, and the plaster stands a plate proud of the timbers. The
band sits on a plate course (in the timber color, as a sill rail) and takes plates on top. Returns the z of its top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["32952", "22885", "35787", "3069b", "3070b", "3024"]

FACE = Rot(y=-90, z=90)  # a 1 x 1 tile or plate on its back, its face to -y
POST = Rot(x=90)  # a 1 x 2 tile on its back, its face to -y, its long side up
# A triangular tile on its back, its face to -y, its square corner at the top left, bottom right, bottom left, or top
# right as seen from the front; and where its 1.85-stud body sits in its 2 x 2 (studs along x, plates up).
TRI = {
    "tl": (Rot(x=90, y=180), 0, 0.375),
    "br": (Rot(x=90), 0.15, 0),
    "bl": (Rot(x=90, y=90), 0, 0),
    "tr": (Rot(x=90, y=-90), 0.15, 0.375),
}
# By lean ("/" or "\\"): the two triangles as (corner, the bay column they start at), and the bay cells (column, row)
# whose stud a triangle's square corner takes.
LEAN = {
    "/": ((("tl", 0), ("br", 1)), {(0, 1), (2, 0)}),
    "\\": ((("bl", 0), ("tr", 1)), {(0, 0), (2, 1)}),
}
OUT = 0.4  # a plate's or tile's thickness, in studs


def build(m, x, y, z, bays=3, plaster="White", timber="Black", title="Timber frame"):
    """The bricks, the posts, and the bays; returns the z of the top."""
    if bays < 1:
        raise ValueError(f"timber_frame_panels needs 1 bay or more, not {bays}")
    for b in range(bays + 1):
        px, bx = x + 4 * b, x + 4 * b + 1
        with m.step(
            f"{title}, post {b + 1}" if b == bays else f"{title}, bay {b + 1}: bricks"
        ):
            m.add("32952", plaster, at=(px, y, z), rot=3)
            if b == bays:
                m.add("3069b", timber, at=(px, y - OUT, z), rot=POST)
                break
            m.add("22885", plaster, at=(bx, y, z), rot=3)
            m.add("32952", plaster, at=(bx + 2, y, z), rot=3)
        triangles, studs = LEAN["/" if b % 2 == 0 else "\\"]
        with m.step(f"{title}, bay {b + 1}: timbers"):
            for c in (-1, 1):
                m.add("3069b", timber, at=(bx + c, y - OUT, z), rot=POST)
            for c in (0, 2):
                for r in (0, 1):
                    part = "3024" if (c, r) in studs else "3070b"
                    m.add(part, timber, at=(bx + c, y - OUT, z + 2.5 * r), rot=FACE)
        with m.step(f"{title}, bay {b + 1}: plaster"):
            for corner, c in triangles:
                rot, dx, dz = TRI[corner]
                m.add("35787", plaster, at=(bx + c + dx, y - 2 * OUT, z + dz), rot=rot)
    return z + 5


def example(m):
    """A band of three braced bays on a dark sill, under a dark head rail."""
    from emmet_dsl import Plate

    with m.step("Sill"):
        m.row(Plate, "Black", at=(0, 0, 0), length=13)
    top = m.use("timber_frame_panels", x=0, y=0, z=1)
    with m.step("Head rail"):
        m.row(Plate, "Black", at=(0, 0, top), length=13, first=3)
