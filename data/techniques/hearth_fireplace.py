# SPDX-License-Identifier: Apache-2.0
"""Hearth fireplace: a brick fireplace with a fire in its firebox, a mantel shelf with candles, and a flue rising
behind, built apart and placed against a wall.

m.use("hearth_fireplace", x, y, z, facing="-y", arch=False, flue=2, candles=True, color="Light Bluish Gray",
mantel="Medium Nougat", arch_color="Reddish Brown", title="Fireplace", id=None) stands a fireplace 4 studs wide and 2
deep on the studs at height z, its footprint's min corner at (x, y) (4 x 2 for facing "-y" or "+y", 2 x 4 for "+x" or
"-x"), its opening toward `facing`, so its back goes against a wall. It's a hearth plate, a firebox 2 studs wide and 2
bricks tall between two columns of masonry bricks, a black back, a grate with a triple flame, a mantel plate across the
top, a candle on each end of the mantel, and `flue` courses of masonry rising behind it to the ceiling or a chimney.
arch=True closes the top of the opening with a round arch in arch_color instead of square columns (the medieval look).
It's a submodel, built once in its own steps, "<title>, hearth", "<title>, firebox", "<title>, mantel" and "<title>,
flue", placed in a step titled "<title>" with the id `id` (by default the submodel's own); it returns the placed copy's
Handle.
"""

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 11387, set 10354. Not derived
from any model file."""

PARTS = [
    "3020",
    "98283",
    "15533",
    "3659",
    "3004",
    "3023b",
    "64647",
    "3062b",
    "6141",
    "37775",
]

FACING = {
    "-y": 0,
    "+x": 1,
    "+y": 2,
    "-x": 3,
}  # quarter turns that point a front built toward -y the given way


def _turn(px, py, k):
    for _ in range(k):
        px, py = -py, px
    return px, py


def _put(m, sub, x, y, z, facing, foot, id=None):
    """Places sub so the rectangle foot = (x0, y0, x1, y1) of its own frame lands with its min corner at (x, y) and
    its own z = 0 at z, its front (built toward -y) toward `facing`."""
    if facing not in FACING:
        raise ValueError(f"facing must be one of {', '.join(FACING)}, not {facing!r}")
    k = FACING[facing]
    (bx0, by0, bz0), (bx1, by1, _) = sub.box
    box = [_turn(px, py, k) for px in (bx0, bx1) for py in (by0, by1)]
    feet = [_turn(px, py, k) for px in (foot[0], foot[2]) for py in (foot[1], foot[3])]
    lo = (min(p[0] for p in box), min(p[1] for p in box))
    fl = (min(p[0] for p in feet), min(p[1] for p in feet))
    return m.place(
        sub, at=(x - fl[0] + lo[0], y - fl[1] + lo[1], z + bz0), rot=k, id=id
    )


def _fireplace(m, arch, flue, candles, color, mantel, arch_color, title):
    """The fireplace in its own frame: 4 wide along x, its opening at y = 0."""
    with m.step(f"{title}, hearth"):
        m.add("3020", "Dark Bluish Gray", at=(0, 0, 0), rot=1, id="hearth")
        m.add("98283", color, at=(0, 0, 1), id="column/left/1")
        m.add("98283", color, at=(3, 0, 1), id="column/right/1")
        m.add("3004", "Black", at=(1, 1, 1), rot=1, id="back/1")
    with m.step(f"{title}, firebox"):
        m.add("3023b", "Black", at=(1, 0, 1), rot=1, id="grate")
        # The triple flame's pin is at one end: turned halfway around, it spreads along the grate from its left stud.
        m.add("64647", "Trans-Orange", at=(1.185, 0.3, 2), rot=2, id="flame")
        if arch:
            m.add("3659", arch_color, at=(0, 0, 4), rot=1, id="arch")
            m.add("15533", color, at=(0, 1, 4), rot=1, id="back/2")
        else:
            m.add("98283", color, at=(0, 0, 4), id="column/left/2")
            m.add("98283", color, at=(3, 0, 4), id="column/right/2")
            m.add("3004", "Black", at=(1, 1, 4), rot=1, id="back/2")
    with m.step(f"{title}, mantel"):
        m.add("3020", mantel, at=(0, 0, 7), rot=1, id="mantel")
        if candles:
            for side, cx in (("left", 0), ("right", 3)):
                m.add("6141", "Pearl Gold", at=(cx, 0, 8), id=f"candle/{side}/dish")
                m.add("3062b", "White", at=(cx, 0, 9), id=f"candle/{side}")
                m.add(
                    "37775",
                    "Trans-Orange",
                    at=(cx + 0.14, 0.3, 12),
                    id=f"candle/{side}/flame",
                )
    if flue:
        with m.step(f"{title}, flue"):
            for k in range(flue):
                m.add("15533", color, at=(0, 1, 8 + 3 * k), rot=1, id=f"flue/{k + 1}")


def build(
    m,
    x,
    y,
    z,
    facing="-y",
    arch=False,
    flue=2,
    candles=True,
    color="Light Bluish Gray",
    mantel="Medium Nougat",
    arch_color="Reddish Brown",
    title="Fireplace",
    id=None,
):
    """The fireplace; returns the placed copy's Handle."""
    sub = m.submodel(
        "fireplace",
        _fireplace,
        arch=arch,
        flue=flue,
        candles=candles,
        color=color,
        mantel=mantel,
        arch_color=arch_color,
        title=title,
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, 4, 2), id=id)


def example(m):
    """A square fireplace and an arched one on a floor."""
    with m.step("Floor"):
        m.add("3036", "Reddish Brown", at=(0, 0, 0), rot=1)
    m.use("hearth_fireplace", x=0, y=4, z=1)
    m.use("hearth_fireplace", x=4, y=4, z=1, arch=True, flue=1, color="Tan")
