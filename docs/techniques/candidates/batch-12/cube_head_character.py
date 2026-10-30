# SPDX-License-Identifier: Apache-2.0
"""Cube-head character: a chunky brick-built game character, 2 x 2 studs, with a head as big as its body, arms on side
studs, and a cap.

m.use("cube_head_character", x, y, z, facing=3, skin="Light Nougat", shirt="Green", pants="Blue", shoes="Reddish
Brown", cap="Red", title="Character") stands a character on the studs at height z, its feet over (x, y) to (x + 2, y +
2): two 1 x 2 plates as shoes, a 2 x 2 brick for the legs, a torso of two bricks 1 x 2 with studs on one side (11211)
whose side studs carry the arms (1 x 2 plates hanging down, studs out, with a round 1 x 1 tile hand), a head two
bricks tall whose front brick has its studs out for two printed eye tiles (98138p07), and a cap: a 2 x 3 plate whose
front row is the brim, under a curved slope 2 x 2. About 5 bricks tall. It's a submodel; facing is the quarter turns
from facing +x (3, the default, faces front, -y). The arms reach 0.4 studs past the feet each side and the brim a stud
forward. Returns the placed copy's Handle, whose .max[2] is the top of the cap.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 72036 and set 72040. Not
derived from any model file."""

PARTS = ["3023b", "3003", "11211", "3004", "98138", "98138p07", "3021", "15068"]

ARM = {
    "left": Rot(x=90, z=-90),
    "right": Rot(x=90, z=90),
}  # a 1 x 2 plate on its side, long side up, studs to -x / +x
HAND = {
    "left": Rot(y=-90),
    "right": Rot(y=90),
}  # a round 1 x 1 tile on its back, its face to -x / +x
EYE = Rot(x=90)  # a round 1 x 1 tile on its back, its face to -y
OUT = 0.4  # a plate's or tile's thickness, in studs


def _character(m, skin, shirt, pants, shoes, cap):
    """The character facing -y, its feet over (0, 0) to (2, 2)."""
    with m.step("Shoes and legs"):
        m.add("3023b", shoes, at=(0, 0, 0))
        m.add("3023b", shoes, at=(1, 0, 0))
        m.add("3003", pants, at=(0, 0, 1))
    with m.step("Torso"):
        m.add("11211", shirt, at=(0, 0, 4), rot=2)
        m.add("11211", shirt, at=(1, 0, 4), rot=0)
    for side, ax in (("left", -OUT), ("right", 2)):
        with m.step(f"{side.capitalize()} arm"):
            # The arm's top cell on the torso's front side stud, the rest hanging beside the legs.
            m.add("3023b", shirt, at=(ax, 0, 2), rot=ARM[side])
            hx = ax - OUT if side == "left" else ax + OUT
            m.add("98138", skin, at=(hx, 0, 2), rot=HAND[side])
    with m.step("Head"):
        m.add("3003", skin, at=(0, 0, 7))
        m.add("11211", skin, at=(0, 0, 10), rot=3)
        m.add("3004", skin, at=(0, 1, 10), rot=1)
    with m.step("Eyes"):
        m.add("98138p07", "White", at=(0, -OUT, 10.5), rot=EYE)
        m.add("98138p07", "White", at=(1, -OUT, 10.5), rot=EYE)
    with m.step("Cap"):
        m.add("3021", cap, at=(0, -1, 13), rot=0)
        m.add("15068", cap, at=(0, 0, 14), rot=3)


def build(
    m,
    x,
    y,
    z,
    facing=3,
    skin="Light Nougat",
    shirt="Green",
    pants="Blue",
    shoes="Reddish Brown",
    cap="Red",
    title="Character",
):
    """The character; returns the placed copy's Handle."""
    sub = m.submodel(
        title, _character, skin=skin, shirt=shirt, pants=pants, shoes=shoes, cap=cap
    )
    # The box's corner after turning the character about the middle of its feet, (x + 1, y + 1); its own front is -y.
    turn = (facing - 3) % 4
    (x0, y0, z0), (x1, y1, _) = sub.box
    corners = [(cx - 1, cy - 1) for cx in (x0, x1) for cy in (y0, y1)]
    for _ in range(turn):
        corners = [(-cy, cx) for cx, cy in corners]
    lo = (min(c[0] for c in corners) + 1, min(c[1] for c in corners) + 1)
    with m.step(title):
        return m.place(sub, at=(x + lo[0], y + lo[1], z + z0), rot=turn)


def example(m):
    """Two characters on a patch of ground, one facing front and one facing right."""
    from emmet_dsl import Plate

    with m.step("Ground"):
        m.row(Plate, "Bright Green", at=(0, 0, 0), length=8, width=6)
    m.use("cube_head_character", x=1, y=2, z=1)
    m.use(
        "cube_head_character",
        x=5,
        y=2,
        z=1,
        facing=0,
        shirt="Orange",
        pants="Dark Blue",
        cap="Medium Azure",
        title="Second character",
    )
