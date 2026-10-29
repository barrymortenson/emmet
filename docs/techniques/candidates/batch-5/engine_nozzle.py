# SPDX-License-Identifier: Apache-2.0
"""Engine nozzle: a round engine pointing out of a ship's stern, a stack of round parts on a bracket's side studs,
ending in a dish with a glowing center.

m.use("engine_nozzle", x, y, z, length=1, facing="back", color="Dark Bluish Gray", bell="Light Bluish Gray",
glow="Trans-Orange", title="Engine") puts a bracket 1 x 2 - 2 x 2 (99207) on the two edge studs at z from (x, y), its
upright face just past the edge with four studs facing out, and builds the engine out from them: a 2 x 2 round plate,
`length` 2 x 2 round bricks (3941), a 2 x 2 jumper plate (87580) to bring one stud to the middle, a 2 x 2 dish (4740)
as the nozzle's bell, and a 1 x 1 round plate in `glow` on its center stud as the flame. facing is the way the engine
points: "back" (+y) and "front" (-y) engines sit on a row along x, "left" (-x) and "right" (+x) ones on a row along y;
(x, y) is the first of the two studs. The engine is 2 studs wide and 5 plates tall, from z, and reaches 1.8 + 1.2 ×
length studs past the edge. It brings its own steps and returns how far past the edge it reaches, in studs.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3C), written from general building knowledge. Not derived from any model
file."""

PARTS = ["99207", "4032a", "3941", "87580", "4740", "6141"]

# By facing: the way the row runs, the bracket's quarter turn, and the turn that points a round part's top out.
FACING = {
    "back": ("x", 1, Rot(x=-90)),
    "front": ("x", 3, Rot(x=90)),
    "left": ("y", 2, Rot(y=-90)),
    "right": ("y", 0, Rot(y=90)),
}
LIP = 0.2  # the bracket's upright stands this far past the stud row, in studs


def build(
    m,
    x,
    y,
    z,
    length=1,
    facing="back",
    color="Dark Bluish Gray",
    bell="Light Bluish Gray",
    glow="Trans-Orange",
    title="Engine",
):
    """The bracket and the engine on it; returns how far past the edge the engine reaches, in studs."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use back, front, left, or right"
        )
    if not 0 <= length <= 6:
        raise ValueError(f"engine_nozzle takes a length from 0 to 6, not {length}")
    _, turn, out = FACING[facing]

    def at(d, depth, size):
        """The min corner of a part `size` studs across, starting `d` studs out from the edge and `depth` deep."""
        side = (2 - size) / 2  # centered on the two studs
        zz = z + 2.5 * (2 - size) / 2
        if facing == "back":
            return (x + side, y + 1 + d, zz)
        if facing == "front":
            return (x + side, y - d - depth, zz)
        if facing == "left":
            return (x - d - depth, y + side, zz)
        return (x + 1 + d, y + side, zz)

    stack = [("4032a", color, 0.4, 2)] + [("3941", color, 1.2, 2)] * length
    stack += [("87580", color, 0.4, 2), ("4740", bell, 0.4, 2), ("6141", glow, 0.4, 1)]
    with m.step(f"{title}: bracket"):
        m.add("99207", color, at=at(-1, 1.2, 2)[:2] + (z,), rot=turn)
    d = LIP
    for i in range(0, len(stack), 4):
        with m.step(
            f"{title}: engine" + (f" ({i // 4 + 1})" if len(stack) > 4 else "")
        ):
            for part, c, depth, size in stack[i : i + 4]:
                m.add(part, c, at=at(d, depth, size), rot=out)
                d += depth
    return d


def example(m):
    """Two engines on the back of a small hull."""
    with m.step("Hull"):
        m.add("3020", "Light Bluish Gray", at=(0, 0, 0), rot=1)
        m.add("3020", "Light Bluish Gray", at=(0, 2, 0), rot=1)
    m.use("engine_nozzle", x=0, y=3, z=1, title="Left engine")
    m.use("engine_nozzle", x=2, y=3, z=1, length=2, title="Right engine")
