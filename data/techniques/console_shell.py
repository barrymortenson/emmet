# SPDX-License-Identifier: Apache-2.0
"""Console shell: a boxy retro game console in two grays, a smooth dark band across its front on side studs and a
tiled lid that lifts off, with a vent strip, a cartridge slot cover, and two buttons.

m.use("console_shell", x, y, z, length=10, depth=8, body="Light Bluish Gray", band="Dark Bluish Gray", buttons=("Red",
"Dark Bluish Gray"), title="Console") builds, on the studs at height z over (x, y) to (x + length, y + depth), a floor
of plates; a front wall faced with `band` tiles standing on the side studs of bricks with a stud on one side (the
library's snot_panel), 5 plates tall; side and back walls of a course of bricks and two of plates, the same height;
and a lid built apart and seated on the wall tops: a layer of plates under `body` tiles, with a column of grille tiles
(2412b) in `band` near the right as the vents, a 4-stud `band` cover over the cartridge slot at the back, and two 1 x
2 tiles in `buttons` near the front. length is even, 8 to 16; depth 6 to 10. Returns the z of the lid's top.
"""

from emmet_dsl import Brick, Plate
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 71374. Not derived from any
model file."""

PARTS = [
    "87087",
    "87079",
    "3068b",
    "3004",
    "3010",
    "3023b",
    "3710",
    "2412b",
    "3069b",
    "3070b",
    "4162",
]

TILES = [
    ((2, 4), "87079", 0),
    ((4, 2), "87079", 1),
    ((2, 2), "3068b", 0),
    ((1, 2), "3069b", 0),
    ((2, 1), "3069b", 1),
    ((1, 1), "3070b", 0),
]


def _lid(m, length, depth, body, band, buttons):
    """The lid lying on the walls, its corner at the origin: plates, then tiles with the vents, slot, and buttons."""
    with m.step("Plates"):
        m.fill(Plate, body, at=(0, 0, 0), size=(length, depth))
    cells = {(c, r): body for c in range(length) for r in range(depth)}
    special = []
    # The vents: grille tiles up a column two studs in from the right side.
    for r in range(1, depth - 1, 2):
        special.append(("2412b", band, (length - 3, r), 0))
        for rr in (r, r + 1):
            del cells[(length - 3, rr)]
    # The cartridge slot cover: a band-colored 2 x 4 in the middle at the back.
    sx = (length - 4) // 2 - 1
    special.append(("87079", band, (sx, depth - 2), 1))
    for c in range(sx, sx + 4):
        for r in (depth - 2, depth - 1):
            del cells[(c, r)]
    # Two buttons side by side at the front left.
    for k, color in enumerate(buttons[:2]):
        special.append(("3069b", color, (1 + 2 * k, 0), 1))
        for c in (1 + 2 * k, 2 + 2 * k):
            del cells[(c, 0)]
    tiles = []
    for c, r in sorted(cells, key=lambda p: (p[1], p[0])):
        if (c, r) not in cells:
            continue
        color = cells[(c, r)]
        for (tw, th), part, rot in TILES:
            patch = [(c + i, r + j) for i in range(tw) for j in range(th)]
            if all(cells.get(p) == color for p in patch):
                for p in patch:
                    del cells[p]
                tiles.append((part, color, (c, r), rot))
                break
    in_steps(
        m,
        "Tiles",
        [
            (1, lambda t=t: m.add(t[0], t[1], at=(t[2][0], t[2][1], 1), rot=t[3]))
            for t in special + tiles
        ],
    )


def build(
    m,
    x,
    y,
    z,
    length=10,
    depth=8,
    body="Light Bluish Gray",
    band="Dark Bluish Gray",
    buttons=("Red", "Dark Bluish Gray"),
    title="Console",
):
    """The floor, the walls, and the lid; returns the z of the lid's top."""
    if length % 2 or not 8 <= length <= 16 or not 6 <= depth <= 10:
        raise ValueError(
            f"console_shell takes an even length 8 to 16 and a depth 6 to 10, not {length}, {depth}"
        )
    with m.step(f"{title}: floor"):
        m.fill(Plate, body, at=(x, y, z), size=(length, depth))
    m.use(
        "snot_panel",
        x=x,
        y=y,
        z=z + 1,
        length=length,
        height=1,
        facing="front",
        color=body,
        face_color=band,
        title=f"{title}: front",
    )
    with m.step(f"{title}: side and back walls"):
        for wx in (x, x + length - 1):
            m.row(Brick, body, at=(wx, y + 1, z + 1), length=depth - 1, axis="y")
        m.row(Brick, body, at=(x + 1, y + depth - 1, z + 1), length=length - 2)
    for k in range(2):
        with m.step(f"{title}: wall plates {k + 1}"):
            for wx in (x, x + length - 1):
                m.row(
                    Plate,
                    body,
                    at=(wx, y + 1, z + 4 + k),
                    length=depth - 1,
                    axis="y",
                    first=2 if k else None,
                )
            m.row(
                Plate,
                body,
                at=(x + 1, y + depth - 1, z + 4 + k),
                length=length - 2,
                first=2 if k else None,
            )
    lid = m.submodel(
        f"{title}: lid",
        _lid,
        length=length,
        depth=depth,
        body=body,
        band=band,
        buttons=tuple(buttons),
    )
    with m.step(f"{title}: lid"):
        m.place(lid, at=(x, y, z + 6))
    return z + 8


def example(m):
    """A console on a mat."""
    with m.step("Mat"):
        m.fill(Plate, "Tan", at=(-1, -1, 0), size=(12, 10))
    m.use("console_shell", x=0, y=0, z=1)
