# SPDX-License-Identifier: Apache-2.0
"""Bar counter: a pub's wooden bar, its front a row of window frames with dark panes standing in for raised panels,
under a tiled top with a beer tap and glasses, built apart and placed.

m.use("bar_counter", x, y, z, panels=3, facing="-y", wood="Reddish Brown", panes="Dark Brown", top="Reddish Brown",
title="Bar") stands a bar 2 deep and 2 × panels studs long on the studs at height z, its footprint's min corner at (x,
y), its paneled front toward `facing` (the barkeep's side is behind it). On a 2-deep plate, each panel is a window 1 x 2
x 2 in `wood` with its glass in `panes` (opaque, so it reads as a sunk wooden panel), and two stacks of 1 x 1 bricks
hold up the back corners. A course of 2-deep plates in `top` ties the frames, and tiles finish it, 8 plates up, a
figure's chest height, except the end 2 x 2: two glasses on the far side and a tap with a drip tile on the near. Put a
wall_shelf of bottles on the wall behind. It's a submodel, built once in its own steps, "<title>, panels", "<title>,
back", "<title>, top", and "<title>, tap and glasses", placed in a step titled "<title>"; it returns the placed copy's
Handle.
"""

from emmet_dsl import Plate, Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10350. Not derived from any
model file."""

PARTS = ["60592", "60601", "3005", "3795", "87079", "2343", "4599b", "3070b"]

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


def _bar(m, panels, wood, panes, top, title):
    """The bar in its own frame: 2 a panel along x, 2 deep, its front at y = 0."""
    length = 2 * panels
    with m.step(f"{title}, panels"):
        m.row(Plate, wood, at=(0, 0, 0), length=length, width=2)
        for i in range(panels):
            f = m.add("60592", wood, at=(2 * i, 0, 1), rot=1, id=f"panel/{i + 1}")
            (x0, y0, z0), (x1, y1, _) = f.min, f.max
            m.add(
                "60601",
                panes,
                at=((x0 + x1 - 1.65) / 2, (y0 + y1 - 0.2) / 2, z0 + 0.875),
                rot=1,
                id=f"panel/{i + 1}/pane",
            )
    with m.step(f"{title}, back"):
        for side, bx in (("left", 0), ("right", length - 1)):
            m.stack("3005", wood, at=(bx, 1, 1), count=2, id=f"back/{side}")
    with m.step(f"{title}, top"):
        m.row(Plate, top, at=(0, 0, 7), length=length, width=2)
        m.fill(Tile, top, at=(0, 0, 8), size=(length - 2, 2))
    with m.step(f"{title}, tap and glasses"):
        m.add("2343", "Trans-Clear", at=(length - 2, 0, 8), id="glass/1")
        m.add("2343", "Trans-Clear", at=(length - 2, 1, 8), id="glass/2")
        # The tap's spout reaches a stud to its left: turned a quarter, it reaches forward over the drip tile.
        m.add("4599b", "Pearl Gold", at=(length - 0.9, 0.5, 8), rot=1, id="tap")
        m.add("3070b", "Dark Bluish Gray", at=(length - 1, 0, 8), id="drip")


def build(
    m,
    x,
    y,
    z,
    panels=3,
    facing="-y",
    wood="Reddish Brown",
    panes="Dark Brown",
    top="Reddish Brown",
    title="Bar",
):
    """The bar; returns the placed copy's Handle."""
    if panels < 2:
        raise ValueError(f"a bar has 2 or more panels, not {panels}")
    sub = m.submodel(
        "bar", _bar, panels=panels, wood=wood, panes=panes, top=top, title=title
    )
    with m.step(title):
        return _put(m, sub, x, y, z, facing, (0, 0, 2 * panels, 2))


def example(m):
    """A bar of three panels across a pub floor."""
    with m.step("Floor"):
        m.add("3036", "Medium Nougat", at=(0, 0, 0), rot=1)
    m.use("bar_counter", x=1, y=2, z=1)
