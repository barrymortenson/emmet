# SPDX-License-Identifier: Apache-2.0
"""Brick letters: a word in big, chunky capital letters built studs up, their fronts the smooth sides of bricks, the
diagonals and round corners filled with 45-degree slopes, on an optional contrasting outline wall behind.

m.use("brick_letters", x, y, z, text, height=9, depth=2, color="Blue", color2="Medium Blue", outline=None,
title="Letters") builds `text` (A to Z, 0 to 9, space, and a period; lower case is read as upper) on the studs at height
z, reading left to right along x from (x, y), facing front (-y). Each letter is `height` bricks tall (9 by default; up
to 15, the middle rows stretched) and `depth` studs deep (1 or 2), its strokes 2 studs wide, most letters 6 studs wide
(I is 2, 1 is 4, M and W are 8, N and Q are 7), with a 1-stud gap between letters and 3 studs for a space. The letters
come from a table of shapes on a grid of 1 stud by 1 brick: a cell is a brick, empty, or a 45-degree slope (Slope 2 x 2
for a 2-deep letter, 2 x 1 for a 1-deep one) or an inverted one that fills the corner of a step, so a diagonal is a
stair of bricks whose steps are filled to a slant. Rows alternate between 2-wide runs and 1-wide runs front and back, so
every row ties the one below. Letters alternate between `color` and `color2`, so they read apart. outline (a color, or
None) adds a wall 1 stud deep behind the letters that stands 1 stud past their outline at the sides and top, the border
the books put round big lettering. It brings its own steps, a letter at a time, bottom up, and returns (x_end, z_top):
the x just past the last letter and the z of the letters' top.
"""

from emmet_dsl import Brick
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 43008. Not derived from any
model file."""

PARTS = [
    "3001",
    "3003",
    "3004",
    "3005",
    "3009",
    "3010",
    "3622",
    "2456",
    "3002",
    "3039",
    "3660b",
    "3040b",
    "3665b",
]

# The letters, top row first, on a grid of 1 stud (across) by 1 brick (up). "#" a brick, "." empty; the four slope
# cells each share a slope with the brick cell beside them, which is the slope's full side:
#   "/" lower right filled (a slope rising to the right; its full side is the cell to its right)
#   "\" lower left filled (a slope rising to the left; its full side is the cell to its left)
#   "p" upper right filled (an inverted slope; its full side is the cell to its right)
#   "q" upper left filled (an inverted slope; its full side is the cell to its left)
FONT = {
    "A": [
        "./##\\.",
        "/####\\",
        "##..##",
        "##..##",
        "##..##",
        "######",
        "##..##",
        "##..##",
        "##..##",
    ],
    "B": [
        "#####\\",
        "######",
        "##..##",
        "##..##",
        "#####.",
        "##..##",
        "##..##",
        "######",
        "#####q",
    ],
    "C": [
        "/####\\",
        "######",
        "##....",
        "##....",
        "##....",
        "##....",
        "##....",
        "######",
        "p####q",
    ],
    "D": [
        "#####\\",
        "######",
        "##..##",
        "##..##",
        "##..##",
        "##..##",
        "##..##",
        "######",
        "#####q",
    ],
    "E": [
        "######",
        "######",
        "##....",
        "##....",
        "####..",
        "##....",
        "##....",
        "######",
        "######",
    ],
    "F": [
        "######",
        "######",
        "##....",
        "##....",
        "####..",
        "##....",
        "##....",
        "##....",
        "##....",
    ],
    "G": [
        "/####\\",
        "######",
        "##....",
        "##....",
        "##.###",
        "##..##",
        "##..##",
        "######",
        "p####q",
    ],
    "H": [
        "##..##",
        "##..##",
        "##..##",
        "##..##",
        "######",
        "##..##",
        "##..##",
        "##..##",
        "##..##",
    ],
    "I": ["##"] * 9,
    "J": [
        "....##",
        "....##",
        "....##",
        "....##",
        "....##",
        "....##",
        "##..##",
        "######",
        "p####q",
    ],
    "K": [
        "##../#",
        "##./##",
        "##/##q",
        "####q.",
        "####..",
        "####\\.",
        "##p##\\",
        "##.p##",
        "##..p#",
    ],
    "L": ["##...."] * 7 + ["######", "######"],
    "M": ["##\\../##", "###\\/###", "##.##.##"] + ["##....##"] * 6,
    "N": [
        "##\\..##",
        "###\\.##",
        "##.#\\##",
        "##..###",
        "##..###",
        "##...##",
        "##...##",
        "##...##",
        "##...##",
    ],
    "O": [
        "/####\\",
        "######",
        "##..##",
        "##..##",
        "##..##",
        "##..##",
        "##..##",
        "######",
        "p####q",
    ],
    "P": [
        "#####\\",
        "######",
        "##..##",
        "##..##",
        "######",
        "#####q",
        "##....",
        "##....",
        "##....",
    ],
    "Q": [
        "/####\\.",
        "######.",
        "##..##.",
        "##..##.",
        "##..##.",
        "##..##.",
        "##.###.",
        "#####\\.",
        "p######",
    ],
    "R": [
        "#####\\",
        "######",
        "##..##",
        "##..##",
        "#####q",
        "####\\.",
        "##p##\\",
        "##.p##",
        "##..p#",
    ],
    "S": [
        "/####\\",
        "######",
        "##....",
        "##....",
        "######",
        "....##",
        "....##",
        "######",
        "p####q",
    ],
    "T": ["######", "######"] + ["..##.."] * 7,
    "U": ["##..##"] * 7 + ["######", "p####q"],
    "V": ["##..##"] * 6 + ["p#..#q", ".####.", ".p##q."],
    "W": ["##....##"] * 3 + ["##.##.##"] * 4 + ["########", "p######q"],
    "X": [
        "##..##",
        "##..##",
        "p#..#q",
        ".####.",
        "..##..",
        ".####.",
        "/#..#\\",
        "##..##",
        "##..##",
    ],
    "Y": [
        "##..##",
        "##..##",
        "##..##",
        "p#..#q",
        ".####.",
        ".p##q.",
        "..##..",
        "..##..",
        "..##..",
    ],
    "Z": [
        "######",
        "######",
        "..../#",
        "../##q",
        "./##q.",
        "/##q..",
        "##q...",
        "######",
        "######",
    ],
    "0": [
        "/####\\",
        "######",
        "##..##",
        "##..##",
        "##..##",
        "##..##",
        "##..##",
        "######",
        "p####q",
    ],
    "1": ["./##", ".###"] + ["..##"] * 5 + ["####", "####"],
    "2": [
        "/####\\",
        "######",
        "....##",
        "....##",
        "######",
        "##....",
        "##....",
        "######",
        "######",
    ],
    "3": [
        "/####\\",
        "######",
        "....##",
        "....##",
        "..####",
        "....##",
        "....##",
        "######",
        "p####q",
    ],
    "4": [
        "##..##",
        "##..##",
        "##..##",
        "##..##",
        "######",
        "....##",
        "....##",
        "....##",
        "....##",
    ],
    "5": [
        "######",
        "######",
        "##....",
        "##....",
        "######",
        "....##",
        "....##",
        "######",
        "p####q",
    ],
    "6": [
        "/####\\",
        "######",
        "##....",
        "##....",
        "#####\\",
        "##..##",
        "##..##",
        "######",
        "p####q",
    ],
    "7": [
        "######",
        "######",
        ".../##",
        "../##q",
        "..##q.",
        "..##..",
        "..##..",
        "..##..",
        "..##..",
    ],
    "8": [
        "/####\\",
        "######",
        "##..##",
        "##..##",
        "######",
        "##..##",
        "##..##",
        "######",
        "p####q",
    ],
    "9": [
        "/####\\",
        "######",
        "##..##",
        "##..##",
        "######",
        "....##",
        "....##",
        "######",
        "p####q",
    ],
    ".": [".."] * 7 + ["##", "##"],
    " ": ["..."] * 9,
}
# Each slope cell: which way its full side is (+1: the cell to its right), whether it's inverted, and its quarter turn
# (a slope at rot 0 falls toward +x, an inverted one's cut faces +x).
SLOPES = {
    "/": (1, False, 2),
    "\\": (-1, False, 0),
    "p": (1, True, 2),
    "q": (-1, True, 0),
}
SLOPE_PARTS = {2: ("3039", "3660b"), 1: ("3040b", "3665b")}
# Rows a taller letter repeats, the nearest plain row to each, in order: the middles of the upper and lower halves.
STRETCH = (3, 6, 2, 5, 4, 1, 7)


def _glyph(ch, height):
    """The letter's rows, bottom row first, stretched to `height` by repeating plain rows."""
    if ch not in FONT:
        raise ValueError(
            f"brick_letters has no letter {ch!r}: use A to Z, 0 to 9, a space, or a period"
        )
    rows = list(FONT[ch])
    for k in range(height - 9):
        target = STRETCH[k % 2]
        plain = [i for i in range(len(rows)) if set(rows[i]) <= {"#", "."}]
        i = min(plain, key=lambda i: (abs(i - target), i))
        rows.insert(i, rows[i])
    return rows[::-1]


def _cells(rows):
    """For each row: the slope pieces, (first cell, slope char), and the cells that are plain brick."""
    out = []
    for r, row in enumerate(rows):
        taken, slopes = set(), []
        for c, ch in enumerate(row):
            if ch in SLOPES:
                side = SLOPES[ch][0]
                full = c + side
                if not 0 <= full < len(row) or row[full] != "#" or full in taken:
                    raise ValueError(
                        f"brick_letters: slope {ch!r} at row {r}, cell {c} has no brick cell beside it"
                    )
                taken |= {c, full}
                slopes.append((min(c, full), ch))
        bricks = [c for c, ch in enumerate(row) if ch == "#" and c not in taken]
        out.append((slopes, bricks))
    return out


def _runs(cells):
    """Consecutive cells as (start, length)."""
    runs = []
    for c in sorted(cells):
        if runs and runs[-1][0] + runs[-1][1] == c:
            runs[-1] = (runs[-1][0], runs[-1][1] + 1)
        else:
            runs.append((c, 1))
    return runs


def _pieces(rows, depth):
    """Every piece of a letter, bottom row first: (row, first cell, cells across, part kind, the (cell, k) cells its
    underside holds by, the cells its top holds by), kind being a brick's length or a slope char."""
    out = []
    for r, (slopes, bricks) in enumerate(_cells(rows)):
        ks = range(depth)
        for start, length in _runs(bricks):
            # The odd rows split front and back, so the seam between runs of the row below is bridged.
            bands = [tuple(ks)] if r % 2 == 0 or depth == 1 else [(k,) for k in ks]
            for band in bands:
                c = start
                for n, _ in Brick.split(length, len(band), None, None, "", 0):
                    cells = {(c + i, k) for i in range(n) for k in band}
                    out.append((r, c, band, n, cells, cells))
                    c += n
        for start, ch in slopes:
            side, inv, _ = SLOPES[ch]
            full = start + (1 if side == 1 else 0)
            both = {(start + i, k) for i in range(2) for k in ks}
            solid = {(full, k) for k in ks}
            out.append(
                (
                    r,
                    start,
                    tuple(ks),
                    ch,
                    solid if inv else both,
                    both if inv else solid,
                )
            )
    return out


def _batches(pieces):
    """The pieces in batches, bottom up, each held by the ones before it: a piece that rests on nothing yet (an
    overhang that hangs from the row above) waits for the piece that ties it to the rest."""
    placed, waiting, batches = set(), [], []
    for i, (r, *_, under, _top) in enumerate(pieces):
        below = [j for j, p in enumerate(pieces[:i]) if p[0] == r - 1 and under & p[5]]
        group = [g for g in waiting if any(j in g for j in below)]
        for g in group:
            waiting.remove(g)
        joined = sorted({j for g in group for j in g} | {i})
        if r == 0 or any(j in placed for j in below):
            placed.update(joined)
            batches.append(joined)
        else:
            waiting.append(joined)
    if waiting:
        raise ValueError("brick_letters: a letter has a piece nothing holds")
    return batches


def build(
    m,
    x,
    y,
    z,
    text,
    height=9,
    depth=2,
    color="Blue",
    color2="Medium Blue",
    outline=None,
    title="Letters",
):
    """The letters, then the outline wall; returns (x_end, z_top)."""
    if not 9 <= height <= 15:
        raise ValueError(
            f"brick_letters takes a height of 9 to 15 bricks, not {height}"
        )
    if depth not in SLOPE_PARTS:
        raise ValueError(f"brick_letters takes a depth of 1 or 2, not {depth}")
    text = text.upper()
    if not text.strip():
        raise ValueError("brick_letters needs some text")
    slope, inverted = SLOPE_PARTS[depth]
    at, filled, n = x, set(), 0
    for ch in text:
        rows = _glyph(ch, height)
        width = len(rows[0])
        if ch != " ":
            paint = (color, color2)[n % 2]
            n += 1
            pieces = _pieces(rows, depth)

            def add(piece, at=at, paint=paint):
                r, c, band, kind, *_ = piece
                where = (at + c, y + band[0], z + 3 * r)
                if kind in SLOPES:
                    part = inverted if SLOPES[kind][1] else slope
                    m.add(part, paint, at=where, rot=SLOPES[kind][2])
                else:
                    w = len(band)
                    m.add(
                        Brick(min(w, kind), max(w, kind)),
                        paint,
                        at=where,
                        rot=1 if kind > w else 0,
                    )

            actions = [
                (len(b), lambda b=b, pieces=pieces: [add(pieces[j]) for j in b])
                for b in _batches(pieces)
            ]
            in_steps(m, f"{title}: {ch}", actions)
            filled |= {
                (at + c, r)
                for r in range(height)
                for c, s in enumerate(rows[r])
                if s != "."
            }
        at += width + 1
    x_end = at - 1
    if outline is not None:
        # Every cell next to a letter (sideways, up, or corner to corner), and the letters' own cells.
        wall = {
            (cx + dx, r + dr) for cx, r in filled for dx in (-1, 0, 1) for dr in (0, 1)
        }
        for r in range(height + 1):
            cells = sorted(cx for cx, rr in wall if rr == r)
            actions = []
            for start, length in _runs(cells):
                zr = z + 3 * r
                if r % 2 and length > 2:
                    # Every other row starts a stud along, so its joints miss the ones below.
                    actions.append(
                        (
                            1,
                            lambda at=(start, y + depth, zr): m.add(
                                "3005", outline, at=at
                            ),
                        )
                    )
                    start, length = start + 1, length - 1
                actions += row_actions(
                    m, Brick, outline, (start, y + depth, zr), length
                )
            in_steps(m, f"{title}: outline, row {r + 1}", actions)
    return (x_end, z + 3 * height)


def example(m):
    """The word OK on a base, with a white outline behind it."""
    from emmet_dsl import Plate

    with m.step("Base"):
        m.row(Plate, "Dark Bluish Gray", at=(-1, 0, 0), length=15, width=3)
    m.use("brick_letters", x=0, y=0, z=1, text="OK", outline="White")
