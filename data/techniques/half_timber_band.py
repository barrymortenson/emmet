# SPDX-License-Identifier: Apache-2.0
"""Half-timber band: a white plastered wall with dark timber posts, built with its faces sideways so the timbers are
flush plate edges.

m.use("half_timber_band", x, y, z, bays=3, plaster="White", timber="Black", title="Timber band") builds a band 5
plates tall along x from (x, y, z), facing front (-y): a timber post (a 1 x 2 tile standing on the two side studs of
a 1 x 1 x 1 2/3 brick), then a 2-stud bay of plaster (two 1 x 2 tiles on a 1 x 2 x 1 2/3 brick with studs on one side),
then a post, `bays` times. The band sits on a plate course and takes plates on top. Returns the z of its top.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3C), from a method in the instruction books of set 10350. Not derived from
any model file."""

PARTS = ["32952", "22885", "3069b"]

POST = Rot(x=90)  # a 1 x 2 tile on its back, its face to -y, its long side up
PANEL = Rot(y=-90, z=90)  # a 1 x 2 tile on its back, its face to -y, long along x


def build(m, x, y, z, bays=3, plaster="White", timber="Black", title="Timber band"):
    """The posts and the bays between them; returns the z of the top."""
    at = x
    for b in range(bays + 1):
        with m.step(f"{title}, post {b + 1}"):
            m.add("32952", plaster, at=(at, y, z), rot=3)
            m.add("3069b", timber, at=(at, y - 0.4, z), rot=POST)
        at += 1
        if b == bays:
            break
        with m.step(f"{title}, bay {b + 1}"):
            m.add("22885", plaster, at=(at, y, z), rot=3)
            for k in range(2):
                m.add("3069b", plaster, at=(at, y - 0.4, z + 2.5 * k), rot=PANEL)
        at += 2
    return z + 5


def example(m):
    """A band of three bays on a plate."""
    from emmet_dsl import Plate

    with m.step("Sill"):
        m.row(Plate, "Black", at=(0, 0, 0), length=10)
    m.use("half_timber_band", x=0, y=0, z=1)
