# SPDX-License-Identifier: Apache-2.0
"""Porthole band: a clean, smooth band along a ship's side, with a row of round portholes set flush in it.

m.use("porthole_band", x, y, z, length, spacing=3, facing="front", color="White", glass="Black", title="Porthole band")
lays one course `length` studs long from (x, y), from z, of bricks with studs on one side (1 x 4 30414, 1 x 2 11211,
1 x 1 87087), their studs to `facing`. Every `spacing` studs (from the middle of the first gap) a round 1 x 1 tile in
`glass` goes on a side stud as a porthole, and flat tiles in the band color cover the side studs between, so the band is
smooth and the portholes sit flush in it, a plate proud of the course. facing: "front" (-y) and "back" (+y) bands run
along x; "left" (-x) and "right" (+x) ones along y. Lay it over the dark hull courses and close it with tiles or a rail
on top. It's the round-tile version of portholes, which sets Technic bricks in the course. Returns the z of the course's
top.
"""

from emmet_dsl import Rot
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10335. Not derived from any
model file."""

PARTS = ["30414", "11211", "87087", "98138", "2431", "63864", "3069b", "3070b"]

SNOT = {4: "30414", 2: "11211", 1: "87087"}
TILE = {4: "2431", 3: "63864", 2: "3069b", 1: "3070b"}
# By facing: the way the band runs, the quarter turn that points a side-stud brick's studs out, and the extra turn
# about z that takes a tile made for the front (-y) round to face that side.
FACING = {
    "front": ("x", 3, 0),
    "back": ("x", 1, 180),
    "left": ("y", 2, 270),
    "right": ("y", 0, 90),
}
OUT = 0.4  # a tile on its back stands out this far, in studs


def build(
    m,
    x,
    y,
    z,
    length,
    spacing=3,
    facing="front",
    color="White",
    glass="Black",
    title="Porthole band",
):
    """The course of side-stud bricks, the portholes, and the tiles between; returns the z of the course's top."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if spacing < 2 or length < spacing:
        raise ValueError(
            f"porthole_band needs a spacing of 2 or more and a length of at least the spacing, not {spacing}, {length}"
        )
    axis, turn, spin = FACING[facing]
    flat = Rot(y=-90, z=90 + spin)

    def cell(a):
        return (x + a, y) if axis == "x" else (x, y + a)

    def face_at(a):
        cx, cy = cell(a)
        if facing == "front":
            cy -= OUT
        elif facing == "back":
            cy += 1
        elif facing == "left":
            cx -= OUT
        else:
            cx += 1
        return (cx, cy, z + 0.5)

    with m.step(f"{title}, course"):
        a, n = 0, length
        for size in (4, 2, 1):
            while n >= size:
                m.add(SNOT[size], color, at=(*cell(a), z), rot=turn)
                a, n = a + size, n - size
    holes = set(range(spacing // 2, length, spacing))
    actions, a = [], 0
    while a < length:
        if a in holes:
            actions.append(
                (1, lambda a=a: m.add("98138", glass, at=face_at(a), rot=flat))
            )
            a += 1
            continue
        run = next((k for k in range(a, length) if k in holes), length) - a
        size = max(s for s in TILE if s <= run)
        actions.append(
            (
                1,
                lambda a=a, size=size: m.add(
                    TILE[size], color, at=face_at(a), rot=flat
                ),
            )
        )
        a += size
    in_steps(m, f"{title}, portholes", actions)
    return z + 3


def example(m):
    """A black hull side two courses tall with a white porthole band and a tiled top."""
    from emmet_dsl import Brick, Tile

    for r in range(2):
        with m.step(f"Hull, row {r + 1}"):
            m.row(
                Brick, "Black", at=(0, 0, 3 * r), length=10, first=2 if r % 2 else None
            )
    m.use("porthole_band", x=0, y=0, z=6, length=10)
    with m.step("Rail"):
        m.row(Tile, "White", at=(0, 0, 9), length=10)
