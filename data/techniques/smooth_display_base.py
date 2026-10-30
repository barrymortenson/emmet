# SPDX-License-Identifier: Apache-2.0
"""Smooth display base: a neat base for a model, two layers of plates with rounded corners, its top tiled like boards in
staggered lengths, open studs only where the model sits, and a nameplate tile set flush in the front edge.

m.use("smooth_display_base", x, y, z, length=12, depth=8, seat=(4, 4), color="Black", name_color="Pearl Gold",
name_length=4, seed=1, title="Base") builds, on the studs at height z (or on the table), a base `length` x `depth` studs
(each 6 to 24) from (x, y): two layers of plates, each with a round-cornered plate 2 x 2 at every corner, the first in
strips along x and the second along y so their seams cross; then a layer of tiles 1 wide running along x, each row's
lengths picked at random (1 to 6 studs) so the joints wander like boards, except where the model sits: `seat` (width,
depth) studs in the middle stay open (studs of the second layer), and the four corners, whose round-cornered plates keep
their studs, spots for a plant or a lamp. In the middle of the front row a tile 1 x `name_length` (2, 3, 4, or 6) in
`name_color` is the nameplate. seed picks the boards. It brings its own steps and returns the (x, y, z) of the seat's
front-left stud, where the model goes.
"""

import random

from emmet_dsl import Plate
from emmet_dsl.core import in_steps, row_actions

PROVENANCE = """Emmet's own (session 3D), from a method in the instruction books of sets 21368 and 21350 (the nameplate
from sets 21350 and 43014). Not derived from any model file."""

PARTS = [
    "79491",
    "3034",
    "3832",
    "3460",
    "3795",
    "3020",
    "3710",
    "3666",
    "3070b",
    "3069b",
    "63864",
    "2431",
    "6636",
]

TILES = {1: "3070b", 2: "3069b", 3: "63864", 4: "2431", 6: "6636"}  # Tile 1 x n
# The round-cornered plate's corner points to +x -y at rot 0 and a quarter turn around each turn: its turn at each
# corner of the base, by which end of x and y it's at.
CORNERS = {(1, 1): 1, (0, 1): 2, (0, 0): 3, (1, 0): 0}


def _bands(m, color, x0, y0, x1, y1, z, axis):
    """Actions covering x0..x1 by y0..y1 with plates 2 wide running along `axis`."""
    actions = []
    if x1 <= x0 or y1 <= y0:
        return actions
    lo, hi = (y0, y1) if axis == "x" else (x0, x1)
    a = lo
    while a < hi:
        w = min(2, hi - a)
        at = (x0, a, z) if axis == "x" else (a, y0, z)
        actions += row_actions(
            m,
            Plate,
            color,
            at,
            (x1 - x0) if axis == "x" else (y1 - y0),
            axis=axis,
            width=w,
        )
        a += w
    return actions


def _layer(m, color, x, y, length, depth, z, axis):
    """A layer of plates with a round-cornered plate at each corner, the cross between them in strips along `axis`."""
    actions = []
    for (i, j), rot in CORNERS.items():
        at = (x + i * (length - 2), y + j * (depth - 2), z)
        actions.append(
            (1, lambda at=at, rot=rot: m.add("79491", color, at=at, rot=rot))
        )
    x1, y1 = x + length, y + depth
    if axis == "x":
        arms = [
            (x, y + 2, x1, y1 - 2),
            (x + 2, y, x1 - 2, y + 2),
            (x + 2, y1 - 2, x1 - 2, y1),
        ]
    else:
        arms = [
            (x + 2, y, x1 - 2, y1),
            (x, y + 2, x + 2, y1 - 2),
            (x1 - 2, y + 2, x1, y1 - 2),
        ]
    for arm in arms:
        actions += _bands(m, color, *arm, z, axis)
    return actions


def _boards(rng, length, below):
    """Lengths of tiles filling a run, at random, avoiding a joint over one in the row below where it can."""
    for _ in range(20):
        out, a = [], 0
        while a < length:
            fits = [
                n
                for n in TILES
                if n <= length - a and (n == length - a or length - a - n != 5)
            ]
            n = rng.choice(fits)
            out.append(n)
            a += n
        joints, a = set(), 0
        for n in out[:-1]:
            a += n
            joints.add(a)
        if not joints & below:
            return out, joints
    return out, joints


def build(
    m,
    x,
    y,
    z,
    length=12,
    depth=8,
    seat=(4, 4),
    color="Black",
    name_color="Pearl Gold",
    name_length=4,
    seed=1,
    title="Base",
):
    """The two plate layers, then the boards around the seat and the nameplate; returns the seat's corner."""
    if not 6 <= length <= 24 or not 6 <= depth <= 24:
        raise ValueError(
            f"smooth_display_base takes a length and depth of 6 to 24, not {length}, {depth}"
        )
    sw, sd = seat
    if sw < 1 or sd < 1 or sw > length - 4 or sd > depth - 4:
        raise ValueError(
            f"smooth_display_base: the seat {sw} x {sd} has to leave 2 studs of tiles all around"
        )
    if name_length not in TILES or name_length == 1 or name_length > length - 4:
        raise ValueError(
            f"smooth_display_base takes a name_length of 2, 3, 4, or 6 that fits, not {name_length}"
        )
    rng = random.Random(seed)
    in_steps(m, f"{title}: plates", _layer(m, color, x, y, length, depth, z, "x"))
    in_steps(
        m, f"{title}: second plates", _layer(m, color, x, y, length, depth, z + 1, "y")
    )
    sx, sy = x + (length - sw) // 2, y + (depth - sd) // 2
    nx = x + (length - name_length) // 2
    zt = z + 2
    corners = {
        (i, j)
        for i in (0, 1, length - 2, length - 1)
        for j in (0, 1, depth - 2, depth - 1)
    }
    actions, below = [], set()
    for j in range(depth):
        joints = set()
        cut = {i for i in range(length) if (i, j) in corners}
        cut |= {
            i for i in range(length) if sx <= x + i < sx + sw and sy <= y + j < sy + sd
        }
        if j == 0:
            cut |= set(range(nx - x, nx - x + name_length))
            actions.append(
                (
                    1,
                    lambda: m.add(
                        TILES[name_length], name_color, at=(nx, y, zt), rot=1
                    ),
                )
            )
        i = 0
        while i < length:
            if i in cut:
                i += 1
                continue
            start = i
            while i < length and i not in cut:
                i += 1
            pieces, js = _boards(rng, i - start, {b - start for b in below})
            joints |= {start + b for b in js}
            a = start
            for n in pieces:
                at = (x + a, y + j, zt)
                actions.append(
                    (
                        1,
                        lambda at=at, n=n: m.add(
                            TILES[n], color, at=at, rot=1 if n > 1 else 0
                        ),
                    )
                )
                a += n
        below = joints
    in_steps(m, f"{title}: boards", actions)
    return (sx, sy, zt)


def example(m):
    """A base 12 x 8 with a small statue on its seat."""
    sx, sy, sz = m.use("smooth_display_base", x=0, y=0, z=0, seed=3)
    with m.step("Statue"):
        m.add("3031", "Light Bluish Gray", at=(sx, sy, sz))  # Plate 4 x 4
        m.add(
            "3941", "Light Bluish Gray", at=(sx + 1, sy + 1, sz + 1)
        )  # Round brick 2 x 2
        m.add("3941", "Light Bluish Gray", at=(sx + 1, sy + 1, sz + 4))
        m.add("30367a", "White", at=(sx + 1, sy + 1, sz + 7))  # Dome
