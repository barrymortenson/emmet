# SPDX-License-Identifier: Apache-2.0
"""Armchair and sofa: an upholstered seat a figure sits in, with arms, a seat cushion, and a padded back, from an
armchair 4 studs wide to a sofa 8 wide, built apart and placed.

m.use("armchair_and_sofa", x, y, z, kind="armchair", length=6, facing="-y", color="Dark Red", cushion="Tan", title=None)
stands a seat on the studs at height z, its footprint's min corner at (x, y), its seat toward `facing`. kind="armchair"
is 4 studs wide; kind="sofa" is `length` studs wide (6 or 8). Either is 2 deep (so 4 x 2 or length x 2 facing "-y" or
"+y", turned for "+x" or "-x"): a 2-deep base plate, an arm at each end (a brick with a curved slope on top, low at the
front), a seat of plates topped with 1 x 2 cushion tiles whose top is 3 plates up, and a back two bricks tall whose
upper course is bricks with studs on one side carrying 1 x 2 tiles as padding, capped with tiles. It's a submodel, built
once in its own steps, "<title>, base and arms", "<title>, seat", and "<title>, back", placed in a step titled "<title>"
("Armchair" or "Sofa" by default); it returns the placed copy's Handle.
"""

from emmet_dsl import Brick, Plate, Rot, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43217, set 43268, set 10350.
Not derived from any model file."""

PARTS = ["3020", "3795", "3004", "11477", "3023b", "3069b", "11211", "3010", "2431"]

PANEL = Rot(y=-90, z=90)  # a 1 x 2 tile on its back, its face to -y, long along x

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


def _seat(m, width, color, cushion, title):
    """The seat in its own frame: `width` along x and 2 deep, its front at y = 0."""
    inner = width - 2
    with m.step(f"{title}, base and arms"):
        m.row(Plate, color, at=(0, 0, 0), length=width, width=2)
        for side, ax in (("left", 0), ("right", width - 1)):
            m.add("3004", color, at=(ax, 0, 1), id=f"arm/{side}")
            # Turned a quarter, the curved slope runs back along the arm and falls toward the front.
            m.add("11477", color, at=(ax, 0, 4), rot=1, id=f"arm/{side}/top")
    with m.step(f"{title}, seat"):
        m.row(Plate, color, at=(1, 0, 1), length=inner)
        for i in range(inner // 2):
            m.add("3069b", cushion, at=(1 + 2 * i, 0, 2), rot=1, id=f"cushion/{i + 1}")
        m.row(Brick, color, at=(1, 1, 1), length=inner)
    with m.step(f"{title}, back"):
        for i in range(inner // 2):
            m.add("11211", color, at=(1 + 2 * i, 1, 4), rot=3, id=f"back/{i + 1}")
            m.add(
                "3069b",
                cushion,
                at=(1 + 2 * i, 0.6, 4.5),
                rot=PANEL,
                id=f"back/{i + 1}/pad",
            )
        m.row(Tile, color, at=(1, 1, 7), length=inner)


def build(
    m,
    x,
    y,
    z,
    kind="armchair",
    length=6,
    facing="-y",
    color="Dark Red",
    cushion="Tan",
    title=None,
):
    """The armchair or sofa; returns the placed copy's Handle."""
    if kind not in ("armchair", "sofa"):
        raise ValueError(f'kind must be "armchair" or "sofa", not {kind!r}')
    width = 4 if kind == "armchair" else length
    if width not in (4, 6, 8):
        raise ValueError(f"a sofa is 6 or 8 studs long, not {length}")
    title = title or kind.capitalize()
    sub = m.submodel(
        kind, _seat, width=width, color=color, cushion=cushion, title=title
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, width, 2))


def example(m):
    """A sofa against the back and an armchair turned to face it from the side."""
    with m.step("Floor"):
        m.add("3036", "Medium Nougat", at=(0, 0, 0), rot=1)
    m.use("armchair_and_sofa", x=0, y=4, z=1, kind="sofa", facing="-y")
    m.use(
        "armchair_and_sofa",
        x=6,
        y=0,
        z=1,
        kind="armchair",
        facing="-x",
        color="Dark Blue",
        cushion="White",
    )
