# SPDX-License-Identifier: Apache-2.0
"""Hangar chevron floor: a smooth black hangar deck, fully tiled, with white chevrons down its middle and a framed lift
pad.

m.use("hangar_chevron_floor", x, y, z, length=12, depth=10, chevrons=2, lift=True, color="Black", marking="White",
pad="Dark Bluish Gray", title="Hangar floor") builds a deck `length` x `depth` studs from (x, y) on the studs at z: a
course of plates, then tiles over all of it. Down the middle (right of the lift pad, if there is one), `chevrons`
chevrons point back (+y), 4 studs apart, each two 2 x 2 squares split on the diagonal, a triangular tile (35787) in
`marking` and one in `color`, mirrored so the white halves meet in a point. With lift, a 4 x 4 lift pad a stud in from
the front-left corner: a 2 x 2 tile in `pad` framed by four 1 x 3 tiles in `marking`, laid as a pinwheel. Large tiles (2
x 4 across the plates' seams, then 2 x 2, 1 x 2, 1 x 1) cover the rest. length: 8 to 16 (12 or more with the lift);
depth: 6 to 16; both even. Returns the z of the deck's top.
"""

from emmet_dsl import Plate

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 75419. Not derived from any
model file."""

PARTS = [
    "35787",
    "63864",
    "87079",
    "3068b",
    "3069b",
    "3070b",
    "3034",
    "3832",
    "3460",
    "3666",
]

# A triangular tile's body is 1.85 studs a side; its right angle sits in the cell's corner. The offset of its body from
# the cell's corner, by quarter turn: the right angle at front-right (0), back-right (1), back-left (2), front-left (3).
TRI = {0: (0.15, 0), 1: (0.15, 0.15), 2: (0, 0.15), 3: (0, 0)}
TILES = [
    ("87079", 2, 4, 0),
    ("87079", 4, 2, 1),
    ("3068b", 2, 2, 0),
    ("3069b", 1, 2, 0),
    ("3069b", 2, 1, 1),
    ("3070b", 1, 1, 0),
]  # (part, x size, y size, turn), largest first


def build(
    m,
    x,
    y,
    z,
    length=12,
    depth=10,
    chevrons=2,
    lift=True,
    color="Black",
    marking="White",
    pad="Dark Bluish Gray",
    title="Hangar floor",
):
    """The plates, the markings, and the tiles; returns the z of the top."""
    if not (8 <= length <= 16 and 6 <= depth <= 16) or length % 2 or depth % 2:
        raise ValueError(
            f"hangar_chevron_floor takes an even length of 8 to 16 and depth of 6 to 16, not {length}, {depth}"
        )
    if lift and length < 12:
        raise ValueError(
            "hangar_chevron_floor needs a length of 12 or more for the lift pad"
        )
    if chevrons < 0 or 4 * chevrons > depth:
        raise ValueError(
            f"hangar_chevron_floor fits up to {depth // 4} chevrons in a depth of {depth}, not {chevrons}"
        )
    with m.step(f"{title}, plates"):
        m.fill(Plate, color, at=(x, y, z), size=(length, depth))
    zt = z + 1
    used = set()
    # The chevrons' left cell, from x: in the middle, or right of the lift pad when there is one.
    cx = max(length // 2 - 2, 6) if lift else length // 2 - 2
    y0 = (
        depth - 4 * chevrons
    ) // 2 + 1  # the first chevron, from y, centered along the deck
    for c in range(chevrons):
        cy = y0 + 4 * c
        with m.step(f"{title}, chevron {c + 1}"):
            # Left square: the marking's right angle at front-right; right square: at front-left. The deck's color
            # takes the other half of each.
            for dx, white, black in ((0, 0, 2), (2, 3, 1)):
                for turn, col in ((white, marking), (black, color)):
                    ox, oy = TRI[turn]
                    m.add(
                        "35787", col, at=(x + cx + dx + ox, y + cy + oy, zt), rot=turn
                    )
        used |= {(cx + i, cy + j) for i in range(4) for j in range(2)}
    if lift:
        with m.step(f"{title}, lift pad"):
            lx, ly = 1, 1
            m.add("3068b", pad, at=(x + lx + 1, y + ly + 1, zt))
            # A pinwheel of 1 x 3 tiles round the pad.
            m.add("63864", marking, at=(x + lx, y + ly, zt), rot=1)
            m.add("63864", marking, at=(x + lx + 3, y + ly, zt), rot=0)
            m.add("63864", marking, at=(x + lx + 1, y + ly + 3, zt), rot=1)
            m.add("63864", marking, at=(x + lx, y + ly + 1, zt), rot=0)
        used |= {(lx + i, ly + j) for i in range(4) for j in range(4)}
    tiles = []
    for j in range(depth):
        for i in range(length):
            if (i, j) in used:
                continue
            for part, w, d, turn in TILES:
                cells = {(i + a, j + b) for a in range(w) for b in range(d)}
                if i + w <= length and j + d <= depth and not cells & used:
                    used |= cells
                    tiles.append((part, i, j, turn))
                    break
    for k in range(0, len(tiles), 8):
        with m.step(f"{title}, tiles {k // 8 + 1}"):
            for part, i, j, turn in tiles[k : k + 8]:
                m.add(part, color, at=(x + i, y + j, zt), rot=turn)
    return zt + 1


def example(m):
    """A hangar deck 12 x 10 studs with two chevrons and a lift pad."""
    m.use("hangar_chevron_floor", x=0, y=0, z=0)
