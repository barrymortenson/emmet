# SPDX-License-Identifier: Apache-2.0
"""Skyline depth rows: a micro city skyline in three rows by depth, dark towers at the back, gray ones in the middle
with one white landmark far taller than the rest, and low light blocks in front, so a shallow base reads deep.

m.use("skyline_depth_rows", x, y, z, length=10, seed=1, front=("White", "Light Bluish Gray"), middle=("Light Bluish
Gray", "Dark Bluish Gray"), back=("Dark Blue", "Dark Bluish Gray"), landmark="White", title="Skyline") fills, on the
studs at height z, a strip `length` studs long along x and 5 deep from (x, y), the front at y: the back row (y + 4, a
stud deep) with towers 1 or 2 studs wide, 5 to 9 bricks tall, of tall bricks (2453b, 2454b) in the back colors, topped
with curved-top bricks or left flat; the middle row (y + 2, 2 deep) with towers 1 or 2 studs wide, 3 to 7 bricks tall,
in the middle colors, and in it, near the center, the landmark, 2 x 2 and 1.6 to 2 times the tallest, with a needle
spire on a jumper; and the front row (y, 2 deep) with blocks 2 or 3 wide and 1 or 2 bricks tall in the front colors,
tiled on top. Each row is packed end to end, neighbors never the same height, and built far row first. seed picks the
widths, heights, colors, and crowns. Returns the z of the landmark's spire tip.
"""

import random

from emmet_dsl import Tile

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of set 21066. Not derived from any
model file."""

PARTS = [
    "2453b",
    "14716",
    "3005",
    "2454b",
    "3004",
    "30145",
    "3003",
    "3002",
    "49307",
    "37352",
    "3043",
    "3068b",
    "3069b",
    "87079",
    "63864",
    "3024",
    "3023b",
    "87580",
    "3062b",
    "30374",
]

# Bricks by footprint (along x, along y), tallest first: (part, height in bricks, quarter turn).
STACKS = {
    (1, 1): [("2453b", 5, 0), ("14716", 3, 0), ("3005", 1, 0)],
    (2, 1): [("2454b", 5, 1), ("3004", 1, 1)],
    (1, 2): [("2454b", 5, 0), ("3004", 1, 0)],
    (2, 2): [("30145", 3, 0), ("3003", 1, 0)],
    (3, 2): [("3002", 1, 1)],
}


def _widths(rng, length, choices):
    """Tower widths that add up to `length` exactly."""
    out, left = [], length
    while left:
        fits = [w for w in choices if w <= left]
        w = rng.choice(fits) if fits else left
        out.append(w)
        left -= w
    return out


def _tower(m, color, x, y, z, w, d, bricks):
    """A stack of the tallest bricks first; returns the z of its top."""
    left = bricks
    for part, h, rot in STACKS[(w, d)]:
        while left >= h:
            m.add(part, color, at=(x, y, z), rot=rot)
            z += 3 * h
            left -= h
    return z


def build(
    m,
    x,
    y,
    z,
    length=10,
    seed=1,
    front=("White", "Light Bluish Gray"),
    middle=("Light Bluish Gray", "Dark Bluish Gray"),
    back=("Dark Blue", "Dark Bluish Gray"),
    landmark="White",
    title="Skyline",
):
    """The back, middle, and front rows; returns the z of the landmark's tip."""
    if not 6 <= length <= 16 or not front or not middle or not back:
        raise ValueError(
            f"skyline_depth_rows takes a length of 6 to 16 and a color for each row, not {length}"
        )
    rng = random.Random(seed)

    def heights(n, lo, hi):
        out = []
        for _ in range(n):
            h = rng.randint(lo, hi)
            while out and h == out[-1]:
                h = rng.randint(lo, hi)
            out.append(h)
        return out

    # Back row: a wall of towers a stud thick.
    at = x
    widths = _widths(rng, length, (1, 2))
    for i, (w, h) in enumerate(zip(widths, heights(len(widths), 5, 9))):
        with m.step(f"{title}, back row: tower {i + 1}"):
            top = _tower(m, rng.choice(back), at, y + 4, z, w, 1, h)
            crown = rng.choice(("round", "round", "flat"))
            if crown == "round":
                m.add(
                    "49307" if w == 1 else "37352",
                    rng.choice(back),
                    at=(at, y + 4, top),
                    rot=1 if w == 2 else 0,
                )
        at += w
    # Middle row: towers 2 deep, the landmark near the middle.
    widths = _widths(rng, length - 2, (1, 2))
    k = len(widths) // 2
    widths.insert(k, "landmark")
    hs = heights(len(widths), 3, 7)
    at, tip = x, z
    for i, (w, h) in enumerate(zip(widths, hs)):
        if w == "landmark":
            tall = round(
                max(hs[j] for j in range(len(hs)) if j != i) * rng.uniform(1.6, 2.0)
            )
            with m.step(f"{title}, landmark: shaft"):
                top = _tower(m, landmark, at, y + 2, z, 2, 2, tall)
            with m.step(f"{title}, landmark: spire"):
                m.add("87580", landmark, at=(at, y + 2, top))
                m.add("3062b", landmark, at=(at + 0.5, y + 2.5, top + 1))
                m.add("30374", "Light Bluish Gray", at=(at + 0.8, y + 2.8, top + 2))
            tip = top + 12
            at += 2
            continue
        with m.step(f"{title}, middle row: tower {i + 1}"):
            top = _tower(m, rng.choice(middle), at, y + 2, z, w, 2, h)
            if w == 1:
                m.add("37352", rng.choice(middle), at=(at, y + 2, top))
            else:
                m.add(
                    "3043",
                    rng.choice(middle),
                    at=(at, y + 2, top),
                    rot=rng.choice((0, 1)),
                )
        at += w
    # Front row: low blocks with tiled roofs.
    at = x
    widths = _widths(rng, length, (2, 3))
    for i, (w, h) in enumerate(zip(widths, heights(len(widths), 1, 2))):
        color = rng.choice(front)
        with m.step(f"{title}, front row: block {i + 1}"):
            top = _tower(m, color, at, y, z, w, 2, h)
            m.add(
                Tile(min(w, 2), max(w, 2)),
                color,
                at=(at, y, top),
                rot=1 if w > 2 else 0,
            )
        at += w
    return tip


def example(m):
    """A skyline 10 studs long on a dark base, tiled in front."""
    from emmet_dsl import Plate

    with m.step("Base"):
        m.add(Plate(6, 10), "Black", at=(0, 0, 0), rot=1)
    with m.step("Front edge"):
        m.row(Tile, "Dark Bluish Gray", at=(0, 0, 1), length=10)
    m.use("skyline_depth_rows", x=0, y=1, z=1, length=10, seed=3)
