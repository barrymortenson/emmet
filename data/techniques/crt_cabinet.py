# SPDX-License-Identifier: Apache-2.0
"""CRT cabinet: an old wooden television on four tapered legs, its gray front a studs-out tiled panel with a
round-cornered screen and a strip of knobs, and two antennas on top.

m.use("crt_cabinet", x, y, z, screen=(8, 6), wood="Reddish Brown", bezel="Light Bluish Gray", glass="Black",
knobs="Dark Bluish Gray", legs="Tan", picture=None, palette=None, title="TV") builds, on the studs at height z, a TV
whose cabinet covers (x, y) to (x + w + 7, y + 6), where (w, h) = screen in studs (h even, 2 to 12): four slope bricks
75 2 x 2 x 2 quadruple convex (3688) as tapered legs, their single top studs under the cabinet's corners; a floor of
two plate layers; side and back walls of bricks; a top of two crossed plate layers with two antennas (3957a). The
front is a panel w + 5 studs wide and h + 2 tall, built lying down and stood up studs-out on brackets (99780) inside
the walls: two crossed layers of plates in `glass`, then tiles, the largest that fit first: a 1-stud bezel round the
screen, a quarter-round 1 x 1 tile (25269) in each screen corner so the glass shows round there, and a 3-stud control
strip right of the screen with two round 2 x 2 knobs (14769) and two round 1 x 1 buttons. The screen is `glass`, or
`picture` (h rows of w letters, top first, keyed by `palette` like pixel_mosaic_panel). Returns the z of the top.
"""

from emmet_dsl import Brick, Plate, Rot
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 71374. Not derived from any
model file."""

PARTS = [
    "3688",
    "99780",
    "25269",
    "14769",
    "98138",
    "3957a",
    "3070b",
    "3069b",
    "3068b",
    "87079",
    "2431",
    "63864",
]

TILES = [
    ((2, 4), "87079", 0),
    ((4, 2), "87079", 1),
    ((2, 2), "3068b", 0),
    ((1, 4), "2431", 0),
    ((4, 1), "2431", 1),
    ((1, 3), "63864", 0),
    ((3, 1), "63864", 1),
    ((1, 2), "3069b", 0),
    ((2, 1), "3069b", 1),
    ((1, 1), "3070b", 0),
]
STAND = Rot(
    x=90
)  # the panel lying studs up, turned up about x: its studs face front (-y), its top row up
PANEL = 1.2  # the panel's thickness: two plates and a tile, in studs
LIP = 0.2  # how far a bracket's side face stands in front of its plate, in studs
DEPTH = 6  # the cabinet's depth, in studs
LEG = 6  # a leg's height, in plates


def _panel(m, w, h, bezel, glass, knobs, picture, palette):
    """The front panel lying studs up, its bottom-left corner at the origin: x across, y up the TV."""
    pw, ph = w + 5, h + 2
    with m.step("Backing, up"):
        for c in range(0, pw, 2):
            m.row(Plate, glass, at=(c, 0, 0), length=ph, axis="y", width=min(2, pw - c))
    with m.step("Backing, across"):
        for r in range(0, ph, 2):
            m.row(Plate, glass, at=(0, r, 1), length=pw, axis="x", width=min(2, ph - r))
    cells = {(c, r): bezel for c in range(pw) for r in range(ph)}
    pal = dict(palette or ())
    for r in range(h):
        for c in range(w):
            ch = picture[h - 1 - r][c] if picture else None
            cells[(1 + c, 1 + r)] = pal[ch] if ch and ch not in ". " else glass
    special = []
    # The screen's corners: a quarter-round tile whose square corner is the screen's corner (turns 0 to 3 put the
    # square corner at -x-y, +x-y, +x+y, -x+y).
    for (c, r), turn in (((1, 1), 0), ((w, 1), 1), ((w, h), 2), ((1, h), 3)):
        special.append(("25269", bezel, (c, r), turn))
        del cells[(c, r)]
    # The control strip: two knobs, each with a button beside it.
    for k, top in enumerate((ph - 2, ph - 6)):
        if top < 1:
            break
        for c in (w + 2, w + 3):
            for r in (top - 1, top):
                del cells[(c, r)]
        special.append(("14769", knobs, (w + 2, top - 1), 0))
        special.append(("98138", "Red" if k else "Yellow", (w + 4, top), 0))
        del cells[(w + 4, top)]
    tiles = []
    for c, r in sorted(cells, key=lambda p: (-p[1], p[0])):
        if (c, r) not in cells:
            continue
        color = cells[(c, r)]
        for (tw, th), part, rot in TILES:
            patch = [(c + i, r - j) for i in range(tw) for j in range(th)]
            if all(cells.get(p) == color for p in patch):
                for p in patch:
                    del cells[p]
                tiles.append((part, color, (c, r - th + 1), rot))
                break
    actions = [
        (1, lambda t=t: m.add(t[0], t[1], at=(t[2][0], t[2][1], 2), rot=t[3]))
        for t in tiles + special
    ]
    in_steps(m, "Tiles", actions)


def build(
    m,
    x,
    y,
    z,
    screen=(8, 6),
    wood="Reddish Brown",
    bezel="Light Bluish Gray",
    glass="Black",
    knobs="Dark Bluish Gray",
    legs="Tan",
    picture=None,
    palette=None,
    title="TV",
):
    """The legs, the cabinet, and the front panel; returns the z of the top."""
    w, h = screen
    if h % 2 or not 2 <= h <= 12 or not 2 <= w <= 16:
        raise ValueError(
            f"crt_cabinet takes a screen 2 to 16 wide and an even 2 to 12 tall, not {w} x {h}"
        )
    if picture and (len(picture) != h or any(len(row) != w for row in picture)):
        raise ValueError(f"crt_cabinet: picture must be {h} rows of {w} letters")
    pw, ph = w + 5, h + 2
    width = pw + 2
    with m.step(f"{title}: legs"):
        # A leg's one stud is in the middle of its top, under the corner cell of the cabinet's floor.
        for lx in (x - 0.5, x + width - 1.5):
            for ly in (y - 0.5, y + DEPTH - 1.5):
                m.add("3688", legs, at=(lx, ly, z))
    zf = z + LEG
    with m.step(f"{title}: floor"):
        m.fill(Plate, wood, at=(x, y, zf), size=(width, DEPTH))
    with m.step(f"{title}: floor, second layer"):
        m.fill(Plate, wood, at=(x, y + 2, zf + 1), size=(width, DEPTH - 2))
        m.row(Plate, wood, at=(x, y, zf + 1), length=2, axis="y")
        m.row(Plate, wood, at=(x + width - 1, y, zf + 1), length=2, axis="y")
    with m.step(f"{title}: floor, front tiles"):
        # Smooth under the front panel, which stands on them.
        for c in range(1, width - 1, 2):
            m.add(
                "3068b" if c + 2 <= width - 1 else "3069b", bezel, at=(x + c, y, zf + 1)
            )
    zp = zf + 2
    with m.step(f"{title}: brackets"):
        for c in range(0, pw - 1, 2):
            m.add("99780", glass, at=(x + 1 + c, y + 2 - LIP, zp), rot=3)
    wall = int(2.5 * ph)
    for k in range(wall // 3):
        with m.step(f"{title}: walls, row {k + 1}"):
            # Full-depth sides, every other course starting with a shorter brick; the back between them.
            for wx in (x, x + width - 1):
                m.row(
                    Brick,
                    wood,
                    at=(wx, y, zp + 3 * k),
                    length=DEPTH,
                    axis="y",
                    first=2 if k % 2 else None,
                )
            m.row(
                Brick,
                wood,
                at=(x + 1, y + DEPTH - 1, zp + 3 * k),
                length=width - 2,
                first=2 if k % 2 else None,
            )
    zt = zp + 3 * (wall // 3)
    for k in range(wall % 3):
        with m.step(f"{title}: wall tops" + (f" {k + 1}" if wall % 3 > 1 else "")):
            for wx in (x, x + width - 1):
                m.row(Plate, wood, at=(wx, y, zt + k), length=DEPTH, axis="y")
            m.row(Plate, wood, at=(x + 1, y + DEPTH - 1, zt + k), length=width - 2)
    sub = m.submodel(
        f"{title}: front",
        _panel,
        w=w,
        h=h,
        bezel=bezel,
        glass=glass,
        knobs=knobs,
        picture=tuple(picture) if picture else None,
        palette=tuple(sorted(palette.items())) if palette else None,
    )
    with m.step(f"{title}: front"):
        m.place(sub, at=(x + 1, y + 2 - LIP - PANEL, zp), rot=STAND)
    ztop = zp + wall
    with m.step(f"{title}: top"):
        m.fill(Plate, wood, at=(x, y, ztop), size=(width, DEPTH))
    with m.step(f"{title}: top, second layer"):
        # Strips front to back, crossing the first layer's, so the top is one piece.
        for c in range(0, width, 2):
            m.row(
                Plate,
                wood,
                at=(x + c, y, ztop + 1),
                length=DEPTH,
                axis="y",
                width=min(2, width - c),
            )
    with m.step(f"{title}: antennas"):
        m.add("3957a", "Black", at=(x + 2.1, y + DEPTH - 1.9, ztop + 2))
        m.add("3957a", "Black", at=(x + width - 2.9, y + DEPTH - 1.9, ztop + 2))
    return ztop + 2


def example(m):
    """A small TV showing a pixel landscape, on a rug."""
    with m.step("Rug"):
        m.fill(Plate, "Dark Red", at=(-1, -1, 0), size=(15, 8))
    m.use(
        "crt_cabinet",
        x=0,
        y=0,
        z=1,
        screen=(6, 4),
        picture=[
            "BWWBYB",
            "BBBBBB",
            "GGGGGG",
            "OOOOOO",
        ],
        palette={
            "B": "Medium Blue",
            "W": "White",
            "Y": "Yellow",
            "G": "Bright Green",
            "O": "Dark Orange",
        },
    )
