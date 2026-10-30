# SPDX-License-Identifier: Apache-2.0
"""Block beast: a game animal made of boxes, a box body on four box legs with a box head and a face of flat squares,
built apart and placed as a submodel.

m.use("block_beast", x, y, z, facing=0, body="Bright Pink", snout="Dark Pink", eyes="Black", title="Beast") stands a
boxy animal (a pig, in the default colors) on the studs at height z, the box it covers (2 x 5.4 studs, 11 plates)
starting at (x, y). facing: quarter turns counterclockwise from facing front (-y). Its legs are four 1 x 1 bricks;
its body a 2 x 4 plate, a 2 x 3 brick and a 2 x 3 plate reaching a stud past the hind legs; its head a 2 x 2 cube
on the front of the plate, a stud ahead of the front legs: two courses of a brick with two studs on its face (11211)
and a 1 x 2 brick behind, capped by a 2 x 2 plate. On the face, two 1 x 1 tiles are the eyes and a 1 x 2 plate on its
side, studs out, is the snout and its nostrils. For a cow or a sheep, change the colors. It's a submodel with its own
steps, placed in a step titled "<title>"; it returns the placed copy's Handle.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21128. Not derived from any
model file."""

PARTS = ["3005", "3020", "3002", "3021", "11211", "3004", "3022", "3023b", "3070b"]

FACE = Rot(y=-90, z=90)  # a tile or plate on its back, its face to -y, long along x
OUT = 0.4  # a plate's thickness, in studs


def _beast(m, body, snout, eyes, title):
    """The beast facing -y, its face at y = 0, its front legs a stud behind it."""
    with m.step(f"{title}, legs"):
        for lx in (0, 1):
            for ly in (1, 3):
                m.add("3005", body, at=(lx, ly, 0))
    with m.step(f"{title}, body"):
        m.add("3020", body, at=(0, 0, 3))
        m.add("3002", body, at=(0, 2, 4))
        m.add("3021", body, at=(0, 2, 7))
    with m.step(f"{title}, head"):
        m.add("11211", body, at=(0, 0, 4), rot=3)
        m.add("3004", body, at=(0, 1, 4), rot=1)
        m.add("3023b", snout, at=(0, -OUT, 4.5), rot=FACE)
    with m.step(f"{title}, head, top"):
        m.add("11211", body, at=(0, 0, 7), rot=3)
        m.add("3004", body, at=(0, 1, 7), rot=1)
        for ex in (0, 1):
            m.add("3070b", eyes, at=(ex, -OUT, 7.5), rot=FACE)
        m.add("3022", body, at=(0, 0, 10))


def build(
    m,
    x,
    y,
    z,
    facing=0,
    body="Bright Pink",
    snout="Dark Pink",
    eyes="Black",
    title="Beast",
):
    """The beast; returns the placed copy's Handle."""
    if facing not in (0, 1, 2, 3):
        raise ValueError(
            f"block_beast takes a facing of 0 to 3 quarter turns, not {facing}"
        )
    beast = m.submodel(
        "block_beast", _beast, body=body, snout=snout, eyes=eyes, title=title
    )
    with m.step(title):
        return m.place(beast, at=(x, y, z), rot=facing)


def example(m):
    """A pig on a patch of grass, facing front."""
    with m.step("Grass"):
        m.add("3031", "Bright Green", at=(0, 0, 0))
        m.add("3020", "Bright Green", at=(0, 4, 0), rot=1)
    m.use("block_beast", x=1, y=0.6, z=1)
