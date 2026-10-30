# SPDX-License-Identifier: Apache-2.0
"""Sunken pond bank: a pond whose water lies below the grass, with a bank of curved slopes running down to it all around,
so the lawn ends in a smooth curve at the waterline instead of a step.

m.use("sunken_pond_bank", x, y, z, length=6, depth=4, bank="Tan", earth="Reddish Brown", grass="Green", bed="Blue",
water="Trans-Light Blue", title="Pond") builds, on the studs at height z (a baseplate is z = 0.5), a pond whose water
covers `length` x `depth` studs from (x + 3, y + 3), (length + 6) x (depth + 6) over all. A layer of plates under it all
is the bed (in `bed` under the water, `earth` elsewhere); on it, the water is tiles, with a 1 x 1 corner-round tile in
each corner so it reads round, and the bank is a ring two studs wide of curved slopes in `bank`, Slope Brick Curved 2 x
2 (15068) and 2 x 1 (11477), each with its low edge at the water and its high edge level with the grass, and a 2 x 2
plate and tile at each corner. Outside it, a ring a stud wide of earth plates under grass plates comes up to the bank's
top, so the grass meets the bank's curve. length and depth: 2 to 16. Returns the z of the grass's top.
"""

from emmet_dsl import Plate, Tile
from emmet_dsl.core import in_steps

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 10315. Not derived from any
model file."""

PARTS = [
    "15068",
    "11477",
    "25269",
    "3022",
    "3068b",
    "3069b",
    "2431",
    "3070b",
    "3024",
    "3023b",
    "3623",
    "3710",
    "3666",
    "3460",
    "3021",
    "3020",
    "3795",
    "3034",
    "3832",
]

SIZES = [
    (2, 10),
    (2, 8),
    (2, 6),
    (2, 4),
    (2, 3),
    (2, 2),
    (1, 8),
    (1, 6),
    (1, 4),
    (1, 3),
    (1, 2),
    (1, 1),
]


def _fill(cells):
    """Covers a set of (i, j) studs with plates, the largest that fit first, long along y where they can be."""
    free, out = set(cells), []
    for j0, i0 in sorted((j, i) for i, j in cells):
        if (i0, j0) not in free:
            continue
        for a, b in SIZES:
            for dx, dy in ((a, b), (b, a)):
                block = {(i0 + p, j0 + q) for p in range(dx) for q in range(dy)}
                if block <= free:
                    free -= block
                    out.append(
                        (
                            Plate(min(dx, dy), max(dx, dy)),
                            (i0, j0),
                            0 if dx <= dy else 1,
                        )
                    )
                    break
            else:
                continue
            break
    return out


def build(
    m,
    x,
    y,
    z,
    length=6,
    depth=4,
    bank="Tan",
    earth="Reddish Brown",
    grass="Green",
    bed="Blue",
    water="Trans-Light Blue",
    title="Pond",
):
    """The bed, the water, the bank, and the grass ring; returns the z of the grass's top."""
    if not 2 <= length <= 16 or not 2 <= depth <= 16:
        raise ValueError(
            f"sunken_pond_bank takes a length and depth of 2 to 16, not {length} x {depth}"
        )
    wx, wy = x + 3, y + 3  # the water's front-left corner
    big, small = length + 6, depth + 6
    pond = {(i, j) for i in range(wx, wx + length) for j in range(wy, wy + depth)}
    rest = {(i, j) for i in range(x, x + big) for j in range(y, y + small)} - pond
    in_steps(
        m,
        f"{title}, bed",
        [
            (1, lambda p=p, at=(i, j, z), r=r, c=c: m.add(p, c, at=at, rot=r))
            for cells, c in ((pond, bed), (rest, earth))
            for p, (i, j), r in _fill(cells)
        ],
    )
    z += 1
    with m.step(f"{title}, water"):
        # A corner-round tile rounds back right at rot=0; each turn moves its around a quarter on, counterclockwise.
        corners = [
            ((wx, wy), 2),
            ((wx + length - 1, wy), 3),
            ((wx + length - 1, wy + depth - 1), 0),
            ((wx, wy + depth - 1), 1),
        ]
        for (i, j), rot in corners:
            m.add("25269", water, at=(i, j, z), rot=rot)
        left = pond - {c for c, _ in corners}
        for j in range(wy, wy + depth):
            run = sorted(i for i, jj in left if jj == j)
            if run:
                m.row(Tile, water, at=(run[0], j, z), length=len(run))

    def slopes(a, n, place):
        """Slopes along a side of n studs from a: 2 x 2s, and a 2 x 1 for an odd stud."""
        out, k = [], 0
        while k < n:
            w = 2 if k + 2 <= n else 1
            out.append(
                (1, lambda k=k, w=w: place("15068" if w == 2 else "11477", a + k))
            )
            k += w
        return out

    # A curved slope faces +x at rot=0, its low edge there; each turn moves its face a quarter on, counterclockwise.
    in_steps(
        m,
        f"{title}, bank, front and back",
        slopes(wx, length, lambda p, i: m.add(p, bank, at=(i, y + 1, z), rot=1))
        + slopes(wx, length, lambda p, i: m.add(p, bank, at=(i, wy + depth, z), rot=3)),
    )
    in_steps(
        m,
        f"{title}, bank, sides",
        slopes(wy, depth, lambda p, j: m.add(p, bank, at=(x + 1, j, z), rot=0))
        + slopes(wy, depth, lambda p, j: m.add(p, bank, at=(wx + length, j, z), rot=2)),
    )
    with m.step(f"{title}, bank corners"):
        for i, j in (
            (x + 1, y + 1),
            (wx + length, y + 1),
            (wx + length, wy + depth),
            (x + 1, wy + depth),
        ):
            m.add("3022", bank, at=(i, j, z))
            m.add("3068b", bank, at=(i, j, z + 1))
    with m.step(f"{title}, earth ring"):
        m.row(Plate, earth, at=(x, y, z), length=big)
        m.row(Plate, earth, at=(x, y + small - 1, z), length=big)
        m.row(Plate, earth, at=(x, y + 1, z), length=small - 2, axis="y")
        m.row(Plate, earth, at=(x + big - 1, y + 1, z), length=small - 2, axis="y")
    with m.step(f"{title}, grass ring"):
        m.row(Plate, grass, at=(x, y, z + 1), length=small, axis="y")
        m.row(Plate, grass, at=(x + big - 1, y, z + 1), length=small, axis="y")
        m.row(Plate, grass, at=(x + 1, y, z + 1), length=big - 2)
        m.row(Plate, grass, at=(x + 1, y + small - 1, z + 1), length=big - 2)
    return z + 2


def example(m):
    """A sunken pond with 6 x 4 studs of water, on a green baseplate."""
    with m.step("Baseplate"):
        m.add("3867", "Green", at=(0, 0, 0))
    m.use("sunken_pond_bank", x=2, y=3, z=0.5, length=6, depth=4)
