# SPDX-License-Identifier: Apache-2.0
"""Contour slab: a smooth rounded outline (a cheek, the side of an egg, a shoe's toe) built flat as layers of plates cut
to the shape, then stood on its side and pressed onto the side studs of a core, so the silhouette is the plates' outline
instead of a stair of bricks.

m.use("contour_slab", x, y, z, width=6, height=6, facing="front", color="White", title="Slab") builds a slab `width`
studs across by `height` studs tall (each 2 to 12) with rounded corners, and stands it against a face of a core whose
side studs are on the stud grid from z (a snot_core, whose courses are 2 studs tall: its studs meet the slab's rows 1,
3, 5, ... counting from 0). The slab is three layers of plates and a layer of tiles: the first in rows across, the
second in columns over the same outline, so it ties every row, each with a round-cornered plate at every corner (a
quarter circle up to 4 x 4, half the slab's width: an 8 x 8 slab is round); the third, rows again, one stud in all around
with smaller round corners, so the edge steps in like a bevel, its cross reaching over the outer layers' corners to tie
them; and tiles over the third, all but its round corners, whose studs show. facing is the face it goes on and the way
it faces: "front" (-y) and "back" (+y) go on a face along x, with x its left end and y the face; "left" (-x) and "right"
(+x) on a face along y, with x the face and y its front end. It's a submodel (one file for each size and color) with its
own steps, placed in a step titled `title`; two with mirrored facings make the two halves of a head. It stands 4 plates
(1.6 studs) out from the face (2 plates when it's under 4 studs across, too small for the inner layers) and returns the
placed copy's Handle.
"""

from emmet_dsl import Plate, Rot, Tile
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21368, 43279, and 43290. Not
derived from any model file."""

PARTS = [
    "79491",
    "30357",
    "30565",
    "3023b",
    "3022",
    "3021",
    "3020",
    "3795",
    "3034",
    "3623",
    "3710",
    "3666",
    "3460",
    "3069b",
    "3068b",
    "63864",
    "2431",
    "87079",
]

# The slab is built lying flat, its rows along x and its height along y; this stands it on edge so its height runs up
# and its top faces out of the face it's on.
FACING = {
    "front": Rot(x=90),
    "back": Rot(x=90, z=180),
    "left": Rot(x=90, z=-90),
    "right": Rot(x=90, z=90),
}


# Round-cornered plates and tiles by the corner's size, and the quarter turn that points the round corner at each
# corner of the slab (a corner piece's round corner points to +x +y at rot 0).
CORNER_PLATES = {2: "79491", 3: "30357", 4: "30565"}
LAG = {
    "79491": 1
}  # the 2 x 2's round corner points a quarter turn behind the others' at the same turn
CORNERS = (((0, 0), 2), ((1, 0), 3), ((1, 1), 0), ((0, 1), 1))


def _bands(m, kind, color, x0, y0, x1, y1, z, axis):
    """Actions covering the rectangle x0..x1 by y0..y1 with strips 2 wide running along `axis`."""
    actions = []
    if x1 <= x0 or y1 <= y0:
        return actions
    across = (y0, y1) if axis == "x" else (x0, x1)
    a = across[0]
    while a < across[1]:
        w = min(2, across[1] - a)
        if axis == "x":
            actions += row_actions(
                m, kind, color, (x0, a, z), x1 - x0, axis="x", width=w
            )
        else:
            actions += row_actions(
                m, kind, color, (a, y0, z), y1 - y0, axis="y", width=w
            )
        a += w
    return actions


def _layer(m, kind, corners, color, x0, y0, x1, y1, z, r, axis):
    """A rounded rectangle: a corner piece r x r at each corner (none when corners is None, leaving them open), the
    cross between them in strips along `axis`, its long arm along it too, so a layer laid along x and one laid along y
    have their seams crossed."""
    actions = []
    if r >= 2 and corners:
        for (cx, cy), rot in CORNERS:
            at = (x1 - r if cx else x0, y1 - r if cy else y0, z)
            part = corners[r]
            rot = (rot + LAG.get(part, 0)) % 4
            actions.append(
                (
                    1,
                    lambda at=at, rot=rot, part=part: m.add(
                        part, color, at=at, rot=rot
                    ),
                )
            )
    elif r < 2:
        r = 0
    if axis == "x":
        arms = [
            (x0, y0 + r, x1, y1 - r),
            (x0 + r, y0, x1 - r, y0 + r),
            (x0 + r, y1 - r, x1 - r, y1),
        ]
    else:
        arms = [
            (x0 + r, y0, x1 - r, y1),
            (x0, y0 + r, x0 + r, y1 - r),
            (x1 - r, y0 + r, x1, y1 - r),
        ]
    for ax0, ay0, ax1, ay1 in arms:
        actions += _bands(m, kind, color, ax0, ay0, ax1, ay1, z, axis)
    return actions


def _slab(m, width, height, color):
    """The slab lying flat in its own frame, its back on z = 0: two layers of the full outline, their strips crossed,
    then one a stud in all around whose cross reaches over the outer layers' corners and ties them, then its tiles."""
    r = min(4, width // 2, height // 2)
    r = r if r >= 2 else 0
    in_steps(
        m,
        "Layer 1",
        _layer(m, Plate, CORNER_PLATES, color, 0, 0, width, height, 0, r, "x"),
    )
    in_steps(
        m,
        "Layer 2",
        _layer(m, Plate, CORNER_PLATES, color, 0, 0, width, height, 1, r, "y"),
    )
    if width >= 4 and height >= 4:
        # Its corners small enough that its cross reaches over the outer corners.
        ri = min(3, r - 2, (min(width, height) - 4) // 2)
        ri = ri if ri >= 2 else 0
        in_steps(
            m,
            "Layer 3",
            _layer(
                m, Plate, CORNER_PLATES, color, 1, 1, width - 1, height - 1, 2, ri, "x"
            ),
        )
        # Tiles over all but the corners, whose round plates keep their studs.
        in_steps(
            m,
            "Tiles",
            _layer(m, Tile, None, color, 1, 1, width - 1, height - 1, 3, ri, "x"),
        )


def build(m, x, y, z, width=6, height=6, facing="front", color="White", title="Slab"):
    """The slab, stood on edge against the face; returns the placed copy's Handle."""
    if facing not in FACING:
        raise ValueError(
            f"facing={facing!r} isn't a side: use front, back, left, or right"
        )
    if not 2 <= width <= 12 or not 2 <= height <= 12:
        raise ValueError(
            f"contour_slab takes a width and height of 2 to 12, not {width}, {height}"
        )
    sub = m.submodel("contour_slab", _slab, width=width, height=height, color=color)
    lo, hi = sub.box
    thick = (
        (hi[2] - lo[2]) * 0.4
    )  # its plates, in studs: four, or two for a slab too thin for the inner layers
    at = {
        "front": (x, y - thick, z),
        "back": (x, y, z),
        "left": (x - thick, y, z),
        "right": (x, y, z),
    }[facing]
    with m.step(title):
        return m.place(sub, at=at, rot=FACING[facing])


def example(m):
    """A head: a core 2 x 2 studs and four courses tall, its face an 8 x 8 slab and a 2 x 8 slab on each side."""
    with m.step("Base"):
        m.add("3031", "Dark Bluish Gray", at=(-1, -1, 0))
    for c in range(4):
        zc = 1 + 5 * c
        with m.step(f"Core, course {c + 1}"):
            m.add("3022", "Red", at=(0, 0, zc))
            m.add("3022", "Red", at=(0, 0, zc + 1))
            for (i, j), rot in {(0, 0): 2, (1, 0): 3, (1, 1): 0, (0, 1): 1}.items():
                m.add("26604", "Red", at=(i, j, zc + 2), rot=rot)
    m.use(
        "contour_slab", x=-3, y=0, z=1, width=8, height=8, facing="front", title="Face"
    )
    m.use(
        "contour_slab",
        x=0,
        y=0,
        z=1,
        width=2,
        height=8,
        facing="left",
        title="Left cheek",
    )
    m.use(
        "contour_slab",
        x=2,
        y=0,
        z=1,
        width=2,
        height=8,
        facing="right",
        title="Right cheek",
    )
