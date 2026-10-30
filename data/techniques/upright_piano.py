# SPDX-License-Identifier: Apache-2.0
"""Upright piano: a piano 4 studs wide against a wall, its keyboard on a ledge at a seated figure's hands, with sheet
music on its front and a candle on its lid, built apart and placed.

m.use("upright_piano", x, y, z, facing="-y", color="Black", candle=True, title="Piano") stands a piano 4 x 2 on the
studs at height z, its footprint's min corner at (x, y), its keys toward `facing`, so its back goes against a wall. Two
1 x 1 bricks stand as the front legs with room for knees between them; a 2 x 4 plate over them and the first course of
the 1 x 4 body behind ties the two; a tile printed with piano keys lies on the front row, 5 plates up; the body rises
two more courses, the top one a brick with studs on its side carrying a white 1 x 4 tile as sheet music, then a 1 x 4
plate as the lid, with a candle (a round plate in pearl gold and a flame) and a small cone as a metronome. A chair or
stool from table_and_chairs goes in front of it. It's a submodel, built once in its own steps, "<title>, legs and keys",
"<title>, body", and "<title>, lid", placed in a step titled "<title>"; it returns the placed copy's Handle.
"""

from emmet_dsl import Rot

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43302. Not derived from any
model file."""

PARTS = [
    "3005",
    "3010",
    "3020",
    "2431px0",
    "30414",
    "2431",
    "3710",
    "6141",
    "37775",
    "4589",
]

PANEL = Rot(y=-90, z=90)  # a 1 x 4 tile on its back, its face to -y, long along x

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


def _put(m, sub, x, y, z, facing, foot):
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
    return m.place(sub, at=(x - fl[0] + lo[0], y - fl[1] + lo[1], z + bz0), rot=k)


def _piano(m, color, candle, title):
    """The piano in its own frame: 4 along x, 2 deep, its keys at y = 0."""
    with m.step(f"{title}, legs and keys"):
        m.add("3005", color, at=(0, 0, 0), id="leg/left")
        m.add("3005", color, at=(3, 0, 0), id="leg/right")
        m.add("3010", color, at=(0, 1, 0), rot=1, id="body/1")
        m.add("3020", color, at=(0, 0, 3), rot=1, id="shelf")
        m.add("2431px0", "White", at=(0, 0, 4), rot=1, id="keys")
    with m.step(f"{title}, body"):
        m.add("3010", color, at=(0, 1, 4), rot=1, id="body/2")
        m.add("30414", color, at=(0, 1, 7), rot=3, id="body/3")
        m.add("2431", "White", at=(0, 0.6, 7.5), rot=PANEL, id="music")
    with m.step(f"{title}, lid"):
        m.add("3710", color, at=(0, 1, 10), rot=1, id="lid")
        if candle:
            m.add("6141", "Pearl Gold", at=(0, 1, 11), id="candle")
            m.add("37775", "Trans-Orange", at=(0.14, 1.3, 12), id="candle/flame")
            m.add("4589", "Reddish Brown", at=(3, 1, 11), id="metronome")


def build(m, x, y, z, facing="-y", color="Black", candle=True, title="Piano"):
    """The piano; returns the placed copy's Handle."""
    sub = m.submodel("piano", _piano, color=color, candle=candle, title=title)
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, 4, 2))


def example(m):
    """A black piano and a brown one on a floor."""
    with m.step("Floor"):
        m.add("3036", "Medium Nougat", at=(0, 0, 0), rot=1)
    m.use("upright_piano", x=0, y=4, z=1)
    m.use(
        "upright_piano", x=6, y=0, z=1, facing="+x", color="Reddish Brown", candle=False
    )
